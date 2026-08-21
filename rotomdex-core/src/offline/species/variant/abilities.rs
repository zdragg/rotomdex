use std::{
    sync::Arc,
    task::{Context, Poll},
};

use color_eyre::eyre::{OptionExt, Result, eyre};
use futures::{StreamExt, stream::FuturesUnordered};
use rustemon::{
    Follow,
    client::RustemonClient,
    model::{
        pokemon::{Ability, PokemonAbility},
        resource::NamedApiResource,
    },
};

use crate::offline::{LoadState, TaskSet};

#[derive(Debug)]
pub struct OfflineAbilities {
    futures: TaskSet<(usize, LoadState<OfflineAbility>)>,
    pkmn_client: Arc<RustemonClient>,

    layout: AbilitiesLayout,
}

impl OfflineAbilities {
    pub fn new(abilities: &[PokemonAbility], pkmn_client: Arc<RustemonClient>) -> Result<Self> {
        let mut mask = 0b000;
        let apis = abilities
            .iter()
            .map(|a| {
                mask = mask | (0b1 << a.slot - 1);
                a.ability
                    .clone()
                    .ok_or_eyre("ability not found")
                    .map(|api| (a.slot as usize - 1, api))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut result = Self {
            futures: FuturesUnordered::new(),
            pkmn_client,

            layout: AbilitiesLayout::new(mask)?,
        };
        result.spawn_abilities_fetch(apis);
        Ok(result)
    }

    pub(super) fn poll_load(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(Some(event)) = self.futures.poll_next_unpin(cx) {
            self.handle_event(event);
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn handle_event(&mut self, (idx, ability): (usize, LoadState<OfflineAbility>)) {
        *self
            .layout
            .get_mut(idx)
            .expect("fetched an ability for a variant without a slot for it") = ability;
    }

    pub fn layout(&self) -> &AbilitiesLayout {
        &self.layout
    }

    fn spawn_abilities_fetch(&mut self, apis: Vec<(usize, NamedApiResource<Ability>)>) {
        for (idx, api) in apis {
            let pkmn_client = self.pkmn_client.clone();
            self.futures.push(Box::pin(async move {
                match api.follow(&pkmn_client).await {
                    Ok(ability) => (idx, LoadState::Loaded(OfflineAbility::new(ability))),
                    Err(e) => (idx, LoadState::log_error(e.into())),
                }
            }));
        }
    }

    pub fn is_fully_loaded(&self) -> bool {
        self.layout.is_fully_loaded()
    }
}

#[derive(Debug)]
pub enum AbilitiesLayout {
    None, // Pokemon like Zygarde-Mega has no abilities in Z-A. Maybe it will have one later after introduced in Champions
    P {
        primary: LoadState<OfflineAbility>,
    },
    PS {
        primary: LoadState<OfflineAbility>,
        secondary: LoadState<OfflineAbility>,
    },
    PH {
        primary: LoadState<OfflineAbility>,
        hidden: LoadState<OfflineAbility>,
    },
    PSH {
        primary: LoadState<OfflineAbility>,
        secondary: LoadState<OfflineAbility>,
        hidden: LoadState<OfflineAbility>,
    },
}

impl AbilitiesLayout {
    // 0b1 for primary, 0b10 for secondary, 0b100 for hidden
    fn new(mask: u8) -> Result<Self> {
        Ok(match mask {
            0b000 => Self::None,
            0b001 => Self::P {
                primary: LoadState::Loading,
            },
            0b011 => Self::PS {
                primary: LoadState::Loading,
                secondary: LoadState::Loading,
            },
            0b101 => Self::PH {
                primary: LoadState::Loading,
                hidden: LoadState::Loading,
            },
            0b111 => Self::PSH {
                primary: LoadState::Loading,
                secondary: LoadState::Loading,
                hidden: LoadState::Loading,
            },
            _ => return Err(eyre!("invalid ability set found")),
        })
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut LoadState<OfflineAbility>> {
        match (self, idx) {
            (Self::P { primary }, 0)
            | (Self::PS { primary, .. }, 0)
            | (Self::PH { primary, .. }, 0)
            | (Self::PSH { primary, .. }, 0) => Some(primary),

            (Self::PS { secondary, .. }, 1) | (Self::PSH { secondary, .. }, 1) => Some(secondary),

            (Self::PH { hidden, .. }, 2) | (Self::PSH { hidden, .. }, 2) => Some(hidden),

            _ => None,
        }
    }

    fn is_fully_loaded(&self) -> bool {
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
    pub fn new(ability: Ability) -> Self {
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
        Self { name, desc }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn desc(&self) -> &str {
        &self.desc
    }
}
