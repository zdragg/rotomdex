use std::task::{Context, Poll};

use crate::model::{Fetchable, Resource};
use crate::{Generation, ModelContext};
use color_eyre::eyre::{Result, eyre};
use itertools::Itertools;
use rustemon::model::pokemon::PokemonAbilityPast;
use rustemon::{
    Follow,
    model::{
        pokemon::{Ability, PokemonAbility},
        resource::NamedApiResource,
    },
};
use tracing::Span;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum ModelAbilities {
    None,
    P {
        primary: Resource<ModelAbility>,
    },
    PS {
        primary: Resource<ModelAbility>,
        secondary: Resource<ModelAbility>,
    },
    PH {
        primary: Resource<ModelAbility>,
        hidden: Resource<ModelAbility>,
    },
    PSH {
        primary: Resource<ModelAbility>,
        secondary: Resource<ModelAbility>,
        hidden: Resource<ModelAbility>,
    },
}

impl ModelAbilities {
    pub(crate) fn new(current: &[PokemonAbility], past: &[PokemonAbilityPast], ctx: ModelContext) -> Result<Self> {
        let mut slots: [Option<NamedApiResource<Ability>>; 3] = [const { None }; 3];

        let mut apply_ability = |ability: &PokemonAbility| {
            let idx = if let 1..=3 = ability.slot {
                (ability.slot - 1) as usize
            } else {
                return Err(eyre!("invalid ability slot"));
            };

            slots[idx] = ability.ability.clone();
            Ok(())
        };

        for ability in current {
            apply_ability(ability)?;
        }

        let target_generation = ctx.version.generation();
        let patches: Vec<_> = past
            .iter()
            .filter_map(|patch| {
                let generation = patch.generation.name.parse::<Generation>().ok()?;
                (generation >= target_generation).then_some((generation, patch))
            })
            .sorted_unstable_by_key(|(generation, _)| std::cmp::Reverse(*generation))
            .map(|(_, patch)| &patch.abilities[..])
            .collect();

        for patch in patches {
            for ability in patch {
                apply_ability(ability)?;
            }
        }

        let slots =
            slots.map(|maybe_api| maybe_api.map(|api| Resource::<ModelAbility>::fetch(api, ctx.clone(), false)));

        let res = match slots {
            [None, None, None] => {
                tracing::warn!("no ability found");
                Self::None
            }
            [Some(primary), None, None] => Self::P { primary },
            [Some(primary), Some(secondary), None] => Self::PS { primary, secondary },
            [Some(primary), None, Some(hidden)] => Self::PH { primary, hidden },
            [Some(primary), Some(secondary), Some(hidden)] => Self::PSH {
                primary,
                secondary,
                hidden,
            },
            _ => return Err(eyre!("invalid ability set found")),
        };

        Ok(res)
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self
            .iter_mut()
            .fold(false, |is_ready, ability| is_ready | ability.poll(cx).is_ready())
        {
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Resource<ModelAbility>> {
        match self {
            Self::None => [None, None, None],
            Self::P { primary } => [Some(primary), None, None],
            Self::PS { primary, secondary } => [Some(primary), Some(secondary), None],
            Self::PH { primary, hidden } => [Some(primary), None, Some(hidden)],
            Self::PSH {
                primary,
                secondary,
                hidden,
            } => [Some(primary), Some(secondary), Some(hidden)],
        }
        .into_iter()
        .flatten()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &Resource<ModelAbility>)> {
        match self {
            Self::None => [None, None, None],
            Self::P { primary } => [Some((0, primary)), None, None],
            Self::PS { primary, secondary } => [Some((0, primary)), Some((1, secondary)), None],
            Self::PH { primary, hidden } => [Some((0, primary)), None, Some((2, hidden))],
            Self::PSH {
                primary,
                secondary,
                hidden,
            } => [Some((0, primary)), Some((1, secondary)), Some((2, hidden))],
        }
        .into_iter()
        .flatten()
    }

    pub(crate) fn is_loaded(&self) -> bool {
        match self {
            Self::None => true,
            Self::P { primary } => primary.is_loaded(),
            Self::PH { primary, hidden } => primary.is_loaded() && hidden.is_loaded(),
            Self::PS { primary, secondary } => primary.is_loaded() && secondary.is_loaded(),
            Self::PSH {
                primary,
                secondary,
                hidden,
            } => primary.is_loaded() && secondary.is_loaded() && hidden.is_loaded(),
        }
    }

    pub(crate) fn ability_cnt(&self) -> usize {
        match self {
            Self::None => 0,
            Self::P { .. } => 1,
            Self::PH { .. } | Self::PS { .. } => 2,
            Self::PSH { .. } => 3,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ModelAbility {
    name: String,
    desc: String,
}

impl ModelAbility {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn desc(&self) -> &str {
        &self.desc
    }
}

impl Fetchable for ModelAbility {
    type Request = NamedApiResource<Ability>;
    async fn fetch(request: Self::Request, ctx: ModelContext) -> Result<Self> {
        let ability = request.follow(&ctx.pkmn_client).await?;
        let name = ability.name;

        let (_rank, desc) = ability
            .flavor_text_entries
            .into_iter()
            .filter_map(|entry| {
                if entry.language.name != "en" {
                    return None;
                }
                let rank = match entry.version_group.name.as_str() {
                    "scarlet-violet" => 9,
                    "sword-shield" => 8,
                    "sun-moon" => 7,
                    "x-y" | "omega-ruby-alpha-sapphire" => 7,
                    "black-white" | "black-2-white-2" => 5,
                    "diamond-pearl" | "platinum" | "heartgold-soulsilver" => 4,
                    "ruby-sapphire" | "emerald" | "firered-leafgreen" => 3,
                    _ => 0,
                };
                Some((rank, entry.flavor_text))
            })
            .max_by_key(|(rank, _)| *rank)
            .unwrap();

        Ok(Self { name, desc })
    }

    fn is_loaded(&self) -> bool {
        true
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn fetch_span(request: &Self::Request) -> Span {
        tracing::info_span!("fetch_ability", ability = %request.name)
    }
}
