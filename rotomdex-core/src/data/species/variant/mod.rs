mod abilities;
pub(crate) use abilities::*;
mod moves;
use alloc::string::String;
pub(crate) use moves::*;
mod sprite;
pub(crate) use sprite::*;
mod stats;
pub(crate) use stats::*;
mod types;
pub(crate) use types::*;

use core::task::{Context, Poll};

use color_eyre::eyre::Result;
use rotomdex_api::{
    Follow,
    client::Client,
    model::{pokemon::Pokemon, resource::NamedApiResource},
};
use tracing::Span;

use crate::{
    Settings,
    data::resource::{AsyncResource, Fetchable, SyncResource},
};

#[derive(Debug)]
pub(crate) struct ModelVariant {
    pub(crate) name: String,
    pub(crate) species_name: String,

    pub(crate) types: SyncResource<ModelTypes>,
    pub(crate) stats: SyncResource<ModelStats>,
    pub(crate) abilities: SyncResource<ModelAbilities>,

    pub(crate) moves: SyncResource<ModelMoves>,

    pub(crate) sprite: AsyncResource<ModelSprite>,
}

impl Fetchable for ModelVariant {
    type Request = NamedApiResource<Pokemon>;
    async fn fetch(request: Self::Request, client: Client, settings: Settings) -> Result<Self> {
        let variant = request.follow(&client).await?;

        Ok(Self {
            name: variant.name,
            species_name: variant.species.name,
            types: SyncResource::<ModelTypes>::derive(
                (variant.types, variant.past_types),
                &client,
                settings,
            ),
            stats: SyncResource::<ModelStats>::derive(
                (variant.stats, variant.past_stats),
                &client,
                settings,
            ),
            abilities: SyncResource::<ModelAbilities>::derive(
                (variant.abilities, variant.past_abilities),
                &client,
                settings,
            ),
            moves: SyncResource::<ModelMoves>::derive(variant.moves, &client, settings),
            sprite: AsyncResource::<ModelSprite>::fetch(variant.sprites, &client, settings),
        })
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.types.poll(cx).is_ready()
            | self.stats.poll(cx).is_ready()
            | self.abilities.poll(cx).is_ready()
            | self.moves.poll(cx).is_ready()
            | self.sprite.poll(cx).is_ready()
        {
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
        self.name
            .strip_prefix(&format!("{}-", self.species_name))
            .unwrap_or("base")
    }
}
