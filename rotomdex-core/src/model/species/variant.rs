mod abilities;
pub(crate) use abilities::*;
mod moves;
pub(crate) use moves::*;
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
    pub(crate) types: ModelTypes,
    pub(crate) stats: ModelStats,
    pub(crate) moves: ModelMoves,
    pub(crate) sprite: Resource<ModelSprite>,
    pub(crate) abilities: ModelAbilities,

    pub(crate) inner: Pokemon,
}

impl Fetchable for ModelVariant {
    type Request = NamedApiResource<Pokemon>;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let variant = request.follow(&ctx.pkmn_client).await?;
        let result = Self {
            types: ModelTypes::new(&variant.types)?,
            stats: ModelStats::new(&variant.stats)?,
            moves: ModelMoves::new(&variant.moves, ctx.clone())?,
            sprite: Resource::<ModelSprite>::fetch(variant.sprites.clone(), ctx.clone(), false),
            abilities: ModelAbilities::new(&variant.abilities, ctx)?,

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
    pub(crate) fn get_variant_name(&self) -> &str {
        self.inner
            .name
            .strip_prefix(&format!("{}-", self.inner.species.name))
            .unwrap_or("base")
    }
}
