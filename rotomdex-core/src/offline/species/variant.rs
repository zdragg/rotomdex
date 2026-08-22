mod abilities;
pub use abilities::*;
mod sprite;
pub use sprite::*;
mod stats;
pub use stats::*;
mod types;
pub use types::*;

use std::task::{Context, Poll};

use color_eyre::eyre::Result;
use rustemon::{
    Follow,
    model::{pokemon::Pokemon, resource::NamedApiResource},
};

use crate::offline::{FetchContext, Fetchable, Resource};

#[derive(Debug)]
pub struct OfflineVariant {
    types: OfflineTypes,
    stats: OfflineStats,
    sprite: Resource<OfflineSprite>,
    abilities: OfflineAbilities,

    inner: Pokemon,
}

impl Fetchable for OfflineVariant {
    type Request = NamedApiResource<Pokemon>;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let variant = request.follow(&ctx.pkmn_client).await?;
        let result = Self {
            types: OfflineTypes::new(&variant.types)?,
            stats: OfflineStats::new(&variant.stats)?,
            sprite: Resource::<OfflineSprite>::fetch(variant.sprites.clone(), ctx.clone()),
            abilities: OfflineAbilities::new(variant.abilities.clone(), ctx)?,

            inner: variant,
        };
        Ok(result)
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.sprite.poll(cx).is_ready() || self.abilities.poll(cx).is_ready() {
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn is_loaded(&self) -> bool {
        self.sprite.is_loaded() && self.abilities.is_loaded()
    }
}

impl OfflineVariant {
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

    pub fn sprite(&self) -> &Resource<OfflineSprite> {
        &self.sprite
    }

    pub fn abilities(&self) -> &OfflineAbilities {
        &self.abilities
    }
}
