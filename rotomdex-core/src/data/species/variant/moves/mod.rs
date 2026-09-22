mod machine;
use alloc::{string::String, vec::Vec};
use itertools::Itertools;
pub(crate) use machine::*;
use snafu::{ResultExt, Snafu};
use tracing::info_span;

use core::{
    cmp::Ordering,
    str::FromStr,
    task::{Context, Poll},
};

use rotomdex_api::{
    Client, Follow,
    model::{moves::Move, pokemon::PokemonMove, resource::NamedApiResource},
};
use strum::{Display, EnumCount, EnumString, VariantArray};

use crate::{
    Settings, VersionGroup,
    data::{
        AsyncResource, ModelType,
        resource::{ArbitraryResourceError, Derivable, Fetchable, ResourceResult},
    },
    misc::strum::StrumParseError,
};

#[derive(Debug)]
pub(crate) struct ModelMoves {
    moves: [Vec<ModelVersionMove>; ModelMoveLearnMethod::COUNT],
}

impl Derivable for ModelMoves {
    type Request = Vec<PokemonMove>;
    fn derive(
        request: Self::Request,
        client: &Client,
        settings: crate::Settings,
    ) -> ResourceResult<Self> {
        let mut move_baskets = [const { vec![] }; ModelMoveLearnMethod::COUNT];
        for m in request {
            for move_version in &m.version_group_details {
                let Ok(version) = move_version.version_group.name.parse::<VersionGroup>() else {
                    continue;
                };

                if version != settings.version.version_group() {
                    continue;
                }

                let method_name = move_version.move_learn_method.name.as_str();
                let Ok(learn_method) = ModelMoveLearnMethod::from_str(method_name) else {
                    tracing::warn!("unsupported move learn method {}", method_name);
                    continue;
                };

                let is_machine = learn_method == ModelMoveLearnMethod::Machine;

                let resource = AsyncResource::<ModelMove>::fetch(
                    (m.move_.clone(), is_machine),
                    client,
                    settings,
                );
                let move_model = ModelVersionMove {
                    name: m.move_.name.clone(),
                    level_learned_at: move_version.level_learned_at as u32,
                    resource,
                };

                move_baskets[learn_method as usize].push(move_model);
            }
        }
        for move_basket in move_baskets.iter_mut() {
            move_basket.sort_unstable_by_key(|move_| move_.level_learned_at);
        }
        Ok(Self {
            moves: move_baskets,
        })
    }

    fn fetch_span(_request: &Self::Request) -> tracing::Span {
        info_span!("deriving_moves")
    }

    fn poll(&mut self, cx: &mut core::task::Context<'_>) -> Poll<()> {
        let has_ready = self
            .moves
            .iter_mut()
            .flatten()
            .fold(false, |has_ready, move_| {
                let move_is_ready = move_.resource.poll(cx).is_ready();
                has_ready | move_is_ready
            });
        if has_ready {
            for bucket in &mut self.moves {
                bucket.sort_unstable();
            }
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl ModelMoves {
    pub(crate) fn get_all_nonempty(&self) -> Vec<(ModelMoveLearnMethod, &[ModelVersionMove])> {
        self.moves
            .iter()
            .enumerate()
            .filter_map(|(i, moves)| {
                if moves.is_empty() {
                    None
                } else {
                    Some((ModelMoveLearnMethod::VARIANTS[i], moves.as_slice()))
                }
            })
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct ModelVersionMove {
    pub(crate) name: String,
    pub(crate) level_learned_at: u32,
    pub(crate) resource: AsyncResource<ModelMove>,
}

impl PartialEq for ModelVersionMove {
    fn eq(&self, other: &Self) -> bool {
        self.level_learned_at == other.level_learned_at && self.resource == other.resource
    }
}

impl Eq for ModelVersionMove {}

impl PartialOrd for ModelVersionMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModelVersionMove {
    fn cmp(&self, other: &Self) -> Ordering {
        let level_order = self.level_learned_at.cmp(&other.level_learned_at);
        if level_order != Ordering::Equal {
            return level_order;
        }

        let resource_order = self.resource.cmp(&other.resource);
        if resource_order != Ordering::Equal {
            return resource_order;
        }

        Ordering::Equal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumCount, EnumString, VariantArray)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum ModelMoveLearnMethod {
    LevelUp,
    Egg,
    Tutor,
    Machine,
    StadiumSurfingPikachu,
    LightBallEgg,
    ColosseumPurification,
    XdShadow,
    XdPurification,
    FormChange,
    ZygardeCube,
    Train,
}

#[derive(Debug)]
pub(crate) struct ModelMove {
    pub(crate) name: String,

    pub(crate) power: Option<u32>,
    pub(crate) accuracy: Option<u32>,
    pub(crate) effect_chance: Option<u32>,

    pub(crate) type_: ModelType,

    pub(crate) damage_class: ModelDamageClass,

    pub(crate) effect: Option<String>,
    pub(crate) short_effect: Option<String>,

    pub(crate) machine: Option<AsyncResource<ModelMachine>>,
}

#[derive(Debug, Snafu)]
enum MoveError {
    #[snafu(display("{name} is an invalid type"))]
    InvalidType {
        name: String,
        #[snafu(source(from(strum::ParseError, StrumParseError)))]
        source: StrumParseError,
    },
    #[snafu(display("{name} is an invalid damage class"))]
    InvalidDamageClass {
        name: String,
        #[snafu(source(from(strum::ParseError, StrumParseError)))]
        source: StrumParseError,
    },
}

impl ArbitraryResourceError for MoveError {}

impl Fetchable for ModelMove {
    type Request = (NamedApiResource<Move>, bool); // bool -> is_machine
    async fn fetch(
        request: Self::Request,
        client: Client,
        settings: Settings,
    ) -> ResourceResult<Self> {
        let (api, is_machine) = request;

        let move_ = api.follow(&client).await?;

        let name = move_.name;
        let power = move_.power.map(|x| x as u32);
        let accuracy = move_.accuracy.map(|x| x as u32);
        let effect_chance = move_.effect_chance.map(|x| x as u32);

        let type_name = move_.type_.name;
        let type_ = type_name
            .parse::<ModelType>()
            .context(InvalidTypeSnafu { name: type_name })?;

        let damage_class_name = move_.damage_class.name;
        let damage_class =
            damage_class_name
                .parse::<ModelDamageClass>()
                .context(InvalidDamageClassSnafu {
                    name: damage_class_name,
                })?;

        let (effect, short_effect) = move_
            .effect_entries
            .into_iter()
            .find(|entry| entry.language.name == "en")
            .map(|e| {
                let long = e.effect.split_whitespace().join(" ");
                let short = e.short_effect.split_whitespace().join(" ");
                (long, short)
            })
            .unzip();

        let machine = if is_machine {
            move_
                .machines
                .into_iter()
                .filter_map(|machine| {
                    if machine.version_group.name.parse::<VersionGroup>().ok()?
                        == settings.version.version_group()
                    {
                        Some(machine.machine)
                    } else {
                        None
                    }
                })
                .next()
                .map(|api| AsyncResource::<ModelMachine>::fetch(api, &client, settings))
        } else {
            None
        };

        Ok(Self {
            name,
            power,
            accuracy,
            effect_chance,
            type_,
            damage_class,
            effect,
            short_effect,
            machine,
        })
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(machine) = &mut self.machine
            && machine.poll(cx).is_ready()
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
    fn fetch_span(request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_move", move_ = %request.0.name)
    }
}

impl PartialEq for ModelMove {
    fn eq(&self, other: &Self) -> bool {
        self.machine == other.machine
            && self.type_ == other.type_
            && self.damage_class == other.damage_class
    }
}

impl Eq for ModelMove {}

impl PartialOrd for ModelMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModelMove {
    fn cmp(&self, other: &Self) -> Ordering {
        let machine_order = self.machine.cmp(&other.machine);
        if machine_order != Ordering::Equal {
            return machine_order;
        }

        let type_order = self.type_.cmp(&other.type_);
        if type_order != Ordering::Equal {
            return type_order;
        }

        let class_order = self.damage_class.cmp(&other.damage_class);
        if class_order != Ordering::Equal {
            return class_order;
        }

        Ordering::Equal
    }
}

#[derive(EnumString, Debug, PartialEq, PartialOrd, Eq, Ord, Display)]
#[strum(ascii_case_insensitive)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum ModelDamageClass {
    Physical,
    Special,
    Status,
}
