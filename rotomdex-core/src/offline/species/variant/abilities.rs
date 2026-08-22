use std::task::{Context, Poll};

use color_eyre::eyre::{OptionExt, Result, eyre};
use rustemon::{
    Follow,
    model::{
        pokemon::{Ability, PokemonAbility},
        resource::NamedApiResource,
    },
};

use crate::offline::{FetchContext, Fetchable, Resource};

#[derive(Debug)]
pub enum OfflineAbilities {
    None, // Pokemon like Zygarde-Mega has no abilities in Z-A. Maybe it will have one later after introduced in Champions
    P {
        primary: Resource<OfflineAbility>,
    },
    PS {
        primary: Resource<OfflineAbility>,
        secondary: Resource<OfflineAbility>,
    },
    PH {
        primary: Resource<OfflineAbility>,
        hidden: Resource<OfflineAbility>,
    },
    PSH {
        primary: Resource<OfflineAbility>,
        secondary: Resource<OfflineAbility>,
        hidden: Resource<OfflineAbility>,
    },
}

impl OfflineAbilities {
    pub fn new(abilities: Vec<PokemonAbility>, ctx: FetchContext) -> Result<Self> {
        let mut slots: [Option<Resource<OfflineAbility>>; 3] = [const { None }; 3];
        for ability in abilities {
            let idx = if let 1..=3 = ability.slot {
                (ability.slot - 1) as usize
            } else {
                return Err(eyre!("invalid ability slot"));
            };

            let api = ability.ability.ok_or_eyre("ability not found")?;

            let resource = Resource::<OfflineAbility>::fetch(api, ctx.clone());

            if slots[idx].replace(resource).is_some() {
                return Err(eyre!("duplicate ability slot"));
            }
        }

        let res = match slots {
            [None, None, None] => Self::None,
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

    pub(super) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.iter_mut().any(|a| a.poll(cx).is_ready()) {
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Resource<OfflineAbility>> {
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

    pub fn is_loaded(&self) -> bool {
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
pub struct OfflineAbility {
    name: String,
    desc: String,
}

impl OfflineAbility {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn desc(&self) -> &str {
        &self.desc
    }
}

impl Fetchable for OfflineAbility {
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

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn is_loaded(&self) -> bool {
        true
    }
}
