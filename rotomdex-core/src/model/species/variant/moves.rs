use std::{fmt::Display, str::FromStr, task::Poll};

use color_eyre::eyre::Result;
use rustemon::{
    Follow,
    model::{moves::Move, pokemon::PokemonMove, resource::NamedApiResource},
};
use strum::{Display, EnumCount, EnumIter, EnumString, VariantArray};

use crate::{
    ModelContext, VersionGroup,
    model::{Fetchable, Resource},
};

#[derive(Debug)]
pub(crate) struct ModelMoves {
    moves: [Vec<ModelVersionMove>; ModelMoveLearnMethod::COUNT],
}

impl ModelMoves {
    pub(crate) fn new(moves: &[PokemonMove], ctx: ModelContext) -> Result<Self> {
        let mut move_baskets = std::array::from_fn(|_| Vec::new());
        for m in moves {
            for move_version in &m.version_group_details {
                let Ok(version) = move_version.version_group.name.parse::<VersionGroup>() else {
                    continue;
                };

                if version != ctx.version.version_group() {
                    continue;
                }

                let method_name = move_version.move_learn_method.name.as_str();
                let Ok(learn_method) = ModelMoveLearnMethod::from_str(method_name) else {
                    tracing::warn!("unsupported move learn method {}", method_name);
                    continue;
                };

                let resource = Resource::<ModelMove>::fetch(m.move_.clone(), ctx.clone(), true);
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
        Ok(Self { moves: move_baskets })
    }

    pub(crate) fn poll(&mut self, cx: &mut std::task::Context<'_>) -> Poll<()> {
        let has_ready = self.moves.iter_mut().flatten().fold(false, |has_ready, move_| {
            let move_is_ready = move_.resource.poll(cx).is_ready();
            has_ready | move_is_ready
        });
        if has_ready { Poll::Ready(()) } else { Poll::Pending }
    }

    pub(crate) fn get(&self, basket: ModelMoveLearnMethod) -> &[ModelVersionMove] {
        self.moves[basket as usize].as_slice()
    }

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
    pub(crate) resource: Resource<ModelMove>,
}

impl Display for ModelVersionMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.level_learned_at == 0 {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{} Lv.{}", self.name, self.level_learned_at)
        }
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
    inner: Move,
}

impl Fetchable for ModelMove {
    type Request = NamedApiResource<Move>;
    async fn fetch(request: Self::Request, ctx: ModelContext) -> Result<Self> {
        let move_ = request.follow(&ctx.pkmn_client).await?;
        Ok(Self { inner: move_ })
    }
    fn is_loaded(&self) -> bool {
        true
    }
    fn poll(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<()> {
        Poll::Pending
    }
    fn fetch_span(request: &Self::Request) -> tracing::Span {
        tracing::info_span!("fetch_move", move_ = %request.name)
    }
}
