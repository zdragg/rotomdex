mod abilities;
pub use abilities::*;
mod sprite;
use futures::{StreamExt, stream::FuturesUnordered};
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
use rustemon::{client::RustemonClient, model::pokemon::Pokemon};

use crate::offline::{HttpClient, LoadState, TaskSet};

#[derive(Debug)]
pub struct OfflineVariant {
    futures: TaskSet<LoadState<OfflineSprite>>,
    req_client: HttpClient,

    types: OfflineTypes,
    stats: OfflineStats,
    sprite: LoadState<OfflineSprite>,
    abilities: OfflineAbilities,

    inner: Pokemon,
}

impl OfflineVariant {
    pub fn new(variant: Pokemon, pkmn_client: Arc<RustemonClient>, req_client: HttpClient) -> Result<Self> {
        let mut result = Self {
            futures: FuturesUnordered::new(),
            req_client,

            types: OfflineTypes::new(&variant.types)?,
            stats: OfflineStats::new(&variant.stats)?,
            sprite: LoadState::Loading,
            abilities: OfflineAbilities::new(&variant.abilities, pkmn_client)?,

            inner: variant,
        };
        result.spawn_sprite_fetch();
        Ok(result)
    }

    pub(super) fn poll_load(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(Some(event)) = self.futures.poll_next_unpin(cx) {
            self.handle_event(event);
            return Poll::Ready(());
        }
        if self.abilities.poll_load(cx).is_ready() {
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn handle_event(&mut self, sprite: LoadState<OfflineSprite>) {
        self.sprite = sprite;
    }

    pub fn inner(&self) -> &Pokemon {
        &self.inner
    }

    pub fn get_variant_name(&self) -> &str {
        &self
            .inner
            .name
            .strip_prefix(&format!("{}-", self.inner.species.name))
            .unwrap_or("base")
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

    pub fn abilities(&self) -> &OfflineAbilities {
        &self.abilities
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
            self.futures.push(Box::pin(
                async move { LoadState::Loaded(OfflineSprite { sprite: None }) },
            ));
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
        self.futures.push(Box::pin(async move {
            let result: Result<DynamicImage> = async {
                let image_bytes = client.get(link).send().await?.bytes().await?;
                let image = image::load_from_memory(&image_bytes)?;
                Ok(crop(image))
            }
            .await;

            match result {
                Ok(image) => LoadState::Loaded(OfflineSprite { sprite: Some(image) }),
                Err(e) => LoadState::log_error(e),
            }
        }));
    }

    pub fn is_fully_loaded(&self) -> bool {
        matches!(self.sprite, LoadState::Loaded(_)) && self.abilities.is_fully_loaded()
    }
}
