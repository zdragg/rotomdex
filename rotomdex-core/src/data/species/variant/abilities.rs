use core::cmp::Reverse;
use core::task::{Context, Poll};

use crate::data::resource::{AsyncResource, Derivable, Fetchable};
use crate::{Generation, Settings, VersionGroup};
use alloc::string::String;
use alloc::vec::Vec;
use color_eyre::eyre::{Result, eyre};
use itertools::Itertools;
use rotomdex_api::client::Client;
use rotomdex_api::model::pokemon::PokemonAbilityPast;
use rotomdex_api::{
    Follow,
    model::{
        pokemon::{Ability, PokemonAbility},
        resource::NamedApiResource,
    },
};
use tracing::{Span, info_span};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum ModelAbilities {
    None,
    P {
        primary: AsyncResource<ModelAbility>,
    },
    PS {
        primary: AsyncResource<ModelAbility>,
        secondary: AsyncResource<ModelAbility>,
    },
    PH {
        primary: AsyncResource<ModelAbility>,
        hidden: AsyncResource<ModelAbility>,
    },
    PSH {
        primary: AsyncResource<ModelAbility>,
        secondary: AsyncResource<ModelAbility>,
        hidden: AsyncResource<ModelAbility>,
    },
}

impl Derivable for ModelAbilities {
    type Request = (Vec<PokemonAbility>, Vec<PokemonAbilityPast>);
    fn derive(request: Self::Request, client: &Client, settings: Settings) -> Result<Self> {
        let (current, past) = request;

        let mut slots: [Option<NamedApiResource<Ability>>; 3] = [const { None }; 3];

        let mut apply_ability = |ability: PokemonAbility| {
            let idx = if let 1..=3 = ability.slot {
                (ability.slot - 1) as usize
            } else {
                return Err(eyre!("invalid ability slot"));
            };

            slots[idx] = ability.ability;
            Ok(())
        };

        for ability in current {
            apply_ability(ability)?;
        }

        let target_generation = settings.version.generation();
        let patches: Vec<_> = past
            .into_iter()
            .filter_map(|patch| {
                let generation = patch.generation.name.parse::<Generation>().ok()?;
                (generation >= target_generation).then_some((generation, patch))
            })
            .sorted_unstable_by_key(|(generation, _)| Reverse(*generation))
            .map(|(_, patch)| patch.abilities)
            .collect();

        for patch in patches {
            for ability in patch {
                apply_ability(ability)?;
            }
        }

        let slots = slots.map(|maybe_api| {
            maybe_api.map(|api| AsyncResource::<ModelAbility>::fetch(api, client, settings))
        });

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

    fn fetch_span(_request: &Self::Request) -> Span {
        info_span!("derive_abilities")
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.iter_mut().fold(false, |is_ready, ability| {
            is_ready | ability.poll(cx).is_ready()
        }) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl ModelAbilities {
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut AsyncResource<ModelAbility>> {
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

    pub(crate) fn get(&self) -> [Option<&AsyncResource<ModelAbility>>; 3] {
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
    }
}

#[derive(Debug)]
pub(crate) struct ModelAbility {
    pub(crate) name: String,
    pub(crate) flavor_text: Option<String>,
}

impl Fetchable for ModelAbility {
    type Request = NamedApiResource<Ability>;
    async fn fetch(request: Self::Request, client: Client, settings: Settings) -> Result<Self> {
        let ability = request.follow(&client).await?;
        let name = ability.name;

        let flavor_text = ability
            .flavor_text_entries
            .into_iter()
            .filter_map(|text| {
                if text.language.name != "en" {
                    return None;
                }
                if text.version_group.name.parse::<VersionGroup>().ok()?
                    != settings.version.version_group()
                {
                    return None;
                }
                let flavor_text = text.flavor_text.split_whitespace().join(" ");
                Some(flavor_text)
            })
            .next();

        Ok(Self { name, flavor_text })
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn fetch_span(request: &Self::Request) -> Span {
        tracing::info_span!("fetch_ability", ability = %request.name)
    }
}
