mod abilities;
pub use abilities::*;
mod sprite;
use image::{DynamicImage, GenericImageView, Pixel};
pub use sprite::*;
mod stats;
pub use stats::*;
mod types;
pub use types::*;

use std::{
    sync::Arc,
    task::{Context, Poll},
};

use color_eyre::eyre::Result;
use rustemon::{Follow, client::RustemonClient, model::pokemon::Pokemon};

use tokio::task::JoinSet;

use crate::offline::LoadState;

enum VariantFetchEvent {
    Ability {
        idx: usize,
        ability: LoadState<OfflineAbility>,
    },
    Sprite {
        sprite: LoadState<OfflineSprite>,
    },
}

#[derive(Debug)]
pub struct OfflineVariant {
    joinset: JoinSet<VariantFetchEvent>,
    pkmn_client: Arc<RustemonClient>,
    req_client: reqwest::Client,

    types: OfflineTypes,
    stats: OfflineStats,
    sprite: LoadState<OfflineSprite>,
    abilities: [LoadState<OfflineAbility>; 3],

    inner: Pokemon,
}

impl OfflineVariant {
    pub fn new(variant: Pokemon, pkmn_client: Arc<RustemonClient>, req_client: reqwest::Client) -> Result<Self> {
        let mut result = Self {
            joinset: JoinSet::new(),
            pkmn_client,
            req_client,

            types: OfflineTypes::new(&variant.types[..])?,
            stats: OfflineStats::new(&variant.stats[..])?,
            sprite: LoadState::Loading,
            abilities: std::array::from_fn(|_| LoadState::Loading),

            inner: variant,
        };
        result.spawn_abilities_fetch();
        result.spawn_sprite_fetch();
        Ok(result)
    }

    pub(super) fn poll_load(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(Some(event)) = self.joinset.poll_join_next(cx) {
            self.handle_event(event.unwrap());
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn handle_event(&mut self, event: VariantFetchEvent) {
        match event {
            VariantFetchEvent::Ability { idx, ability } => {
                self.abilities[idx] = ability;
            }
            VariantFetchEvent::Sprite { sprite } => {
                self.sprite = sprite;
            }
        }
    }

    pub fn inner(&self) -> &Pokemon {
        &self.inner
    }

    pub fn types(&self) -> &OfflineTypes {
        &self.types
    }

    pub fn stats(&self) -> &OfflineStats {
        &self.stats
    }

    pub fn sprite(&self) -> &LoadState<OfflineSprite> {
        &self.sprite
    }

    pub fn abilities(&self) -> &[LoadState<OfflineAbility>] {
        &self.abilities
    }

    fn spawn_abilities_fetch(&mut self) {
        for ability in self.inner.abilities.clone() {
            let client = self.pkmn_client.clone();
            self.joinset.spawn(async move {
                let idx = ability.slot as usize - 1;
                let ability = {
                    if let Some(api) = ability.ability {
                        match api.follow(&client).await {
                            Ok(a) => LoadState::Loaded(OfflineAbility::new(Some(a))),
                            Err(e) => LoadState::Failed(e.into()),
                        }
                    } else {
                        LoadState::Loaded(OfflineAbility::new(None))
                    }
                };
                VariantFetchEvent::Ability { idx, ability }
            });
        }
    }

    fn spawn_sprite_fetch(&mut self) {
        let sprite_link = self
            .inner()
            .sprites
            .versions
            .generation_v
            .black_white
            .front_default
            .clone();

        let client = self.req_client.clone();
        let Some(link) = sprite_link else {
            log::warn!("sprite not found: {}", self.inner().name);
            self.joinset.spawn(async move {
                VariantFetchEvent::Sprite {
                    sprite: LoadState::Loaded(OfflineSprite { sprite: None }),
                }
            });
            return;
        };

        let crop = |image: DynamicImage| -> DynamicImage {
            let (mut min_x, mut max_x, mut min_y, mut max_y) = (image.width(), 0, image.height(), 0);
            for (x, y, color) in image.pixels() {
                if color.alpha() == 0 {
                    continue;
                }
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
            let (mid_x, mid_y) = ((min_x + max_x) / 2, (min_y + max_y) / 2);
            let side_len = (max_x - min_x).max(max_y - min_y) + 2; // leave some space
            let (corner_x, corner_y) = (mid_x.saturating_sub(side_len / 2), mid_y.saturating_sub(side_len / 2));
            image.crop_imm(corner_x, corner_y, side_len, side_len)
        };
        self.joinset.spawn(async move {
            let result: Result<DynamicImage> = async {
                let image_bytes = client.get(link).send().await?.bytes().await?;
                let image = image::load_from_memory(&image_bytes)?;
                Ok(crop(image))
            }
            .await;

            match result {
                Ok(image) => VariantFetchEvent::Sprite {
                    sprite: LoadState::Loaded(OfflineSprite { sprite: Some(image) }),
                },
                Err(e) => VariantFetchEvent::Sprite {
                    sprite: LoadState::Failed(e),
                },
            }
        });
    }

    pub fn is_fully_loaded(&self) -> bool {
        matches!(self.sprite, LoadState::Loaded(_))
            && self.abilities().iter().all(|a| matches!(a, LoadState::Loaded(_)))
    }
}
