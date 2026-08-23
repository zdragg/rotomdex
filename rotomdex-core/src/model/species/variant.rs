mod abilities;
pub(crate) use abilities::*;
mod sprite;
pub(crate) use sprite::*;
mod stats;
pub(crate) use stats::*;
mod types;
pub(crate) use types::*;

use std::task::{Context, Poll};

use crate::FetchContext;
use crate::model::{Fetchable, Resource};
use color_eyre::eyre::Result;
use rustemon::{
    Follow,
    model::{pokemon::Pokemon, resource::NamedApiResource},
};
use tracing::Span;

#[derive(Debug)]
pub(crate) struct ModelVariant {
    types: ModelTypes,
    stats: ModelStats,
    sprite: Resource<ModelSprite>,
    abilities: ModelAbilities,

    inner: Pokemon,
}

impl Fetchable for ModelVariant {
    type Request = NamedApiResource<Pokemon>;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let variant = request.follow(&ctx.pkmn_client).await?;
        let result = Self {
            types: ModelTypes::new(&variant.types)?,
            stats: ModelStats::new(&variant.stats)?,
            sprite: Resource::<ModelSprite>::fetch(variant.sprites.clone(), ctx.clone()),
            abilities: ModelAbilities::new(variant.abilities.clone(), ctx)?,

            inner: variant,
        };
        Ok(result)
    }

    fn is_loaded(&self) -> bool {
        self.sprite.is_loaded() && self.abilities.is_loaded()
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        // bitwise OR for no short circuit
        if self.sprite.poll(cx).is_ready() | self.abilities.poll(cx).is_ready() {
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn fetch_span(request: &Self::Request) -> Span {
        tracing::info_span!("fetch_variant", variant = %request.name)
    }
}

impl ModelVariant {
    pub(crate) fn inner(&self) -> &Pokemon {
        &self.inner
    }

    pub(crate) fn get_variant_name(&self) -> &str {
        self.inner
            .name
            .strip_prefix(&format!("{}-", self.inner.species.name))
            .unwrap_or("base")
    }

    pub(crate) fn types(&self) -> &ModelTypes {
        &self.types
    }

    pub(crate) fn stats(&self) -> &ModelStats {
        &self.stats
    }

    pub(crate) fn sprite(&self) -> &Resource<ModelSprite> {
        &self.sprite
    }

    pub(crate) fn abilities(&self) -> &ModelAbilities {
        &self.abilities
    }
}
