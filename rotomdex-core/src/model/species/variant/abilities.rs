use std::task::{Context, Poll};

use crate::FetchContext;
use crate::model::{Fetchable, Resource};
use color_eyre::eyre::{OptionExt, Result, eyre};
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
    None, // Pokemon like Zygarde-Mega has no abilities in Z-A. Maybe it will have one later after introduced in Champions
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
    pub(crate) fn new(abilities: Vec<PokemonAbility>, ctx: FetchContext) -> Result<Self> {
        let mut slots: [Option<Resource<ModelAbility>>; 3] = [const { None }; 3];
        for ability in abilities {
            let idx = if let 1..=3 = ability.slot {
                (ability.slot - 1) as usize
            } else {
                return Err(eyre!("invalid ability slot"));
            };

            let api = ability.ability.ok_or_eyre("ability not found")?;

            let resource = Resource::<ModelAbility>::fetch(api, ctx.clone());

            if slots[idx].replace(resource).is_some() {
                return Err(eyre!("duplicate ability slot"));
            }
        }

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
        // bitwise OR for no short circuit
        if self
            .iter_mut()
            .fold(false, |is_ready, a| is_ready | a.poll(cx).is_ready())
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
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let ability = request.follow(&ctx.pkmn_client).await?;
        let name = ability.name;

        let (_rank, desc) = ability
            .flavor_text_entries
            .into_iter()
            .filter_map(|e| {
                if e.language.name != "en" {
                    return None;
                }
                let rank = match e.version_group.name.as_str() {
                    "scarlet-violet" => 9,
                    "sword-shield" => 8,
                    "sun-moon" => 7,
                    "x-y" | "omega-ruby-alpha-sapphire" => 7,
                    "black-white" | "black-2-white-2" => 5,
                    "diamond-pearl" | "platinum" | "heartgold-soulsilver" => 4,
                    "ruby-sapphire" | "emerald" | "firered-leafgreen" => 3,
                    _ => 0,
                };
                Some((rank, e.flavor_text))
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
