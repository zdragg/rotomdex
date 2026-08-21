mod variant;

use std::{
    sync::Arc,
    task::{Context, Poll},
};

use color_eyre::eyre::Result;
use futures::{StreamExt, stream::FuturesUnordered};
use ratatui::style::Color;
use reqwest_middleware::ClientWithMiddleware;
use rustemon::{Follow, client::RustemonClient, model::pokemon::PokemonSpecies};

pub use variant::*;

use crate::offline::{LoadState, TaskSet};

#[derive(Debug)]
pub struct OfflineSpecies {
    futures: TaskSet<(usize, LoadState<OfflineVariant>)>,
    pkmn_client: Arc<RustemonClient>,
    req_client: ClientWithMiddleware,

    variants: Vec<LoadState<OfflineVariant>>,

    inner: PokemonSpecies,
}

impl OfflineSpecies {
    pub fn new(species: PokemonSpecies, pkmn_client: Arc<RustemonClient>, req_client: ClientWithMiddleware) -> Self {
        let mut pkmn = Self {
            pkmn_client,
            req_client,

            futures: FuturesUnordered::new(),

            variants: std::iter::repeat_with(|| LoadState::Loading)
                .take(species.varieties.len())
                .collect(),

            inner: species,
        };
        pkmn.spawn_variants_fetch();
        pkmn
    }

    pub(super) fn poll_load(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(Some(event)) = self.futures.poll_next_unpin(cx) {
            self.handle_event(event);
            return Poll::Ready(());
        }

        for maybe_variant in &mut self.variants {
            if let LoadState::Loaded(variant) = maybe_variant
                && variant.poll_load(cx).is_ready()
            {
                return Poll::Ready(());
            }
        }

        Poll::Pending
    }

    fn handle_event(&mut self, (idx, variant): (usize, LoadState<OfflineVariant>)) {
        self.variants[idx] = variant;
    }

    pub fn inner(&self) -> &PokemonSpecies {
        &self.inner
    }

    pub fn variants_cnt(&self) -> usize {
        self.variants.len()
    }

    pub fn variants(&self) -> &[LoadState<OfflineVariant>] {
        &self.variants
    }

    pub fn get_ratatui_color(&self) -> Color {
        match self.inner.color.name.as_str() {
            "black" => Color::Black,
            "blue" => Color::Blue,
            "brown" => Color::Yellow,
            "gray" => Color::Gray,
            "green" => Color::Green,
            "pink" => Color::LightMagenta,
            "purple" => Color::Magenta,
            "red" => Color::Red,
            "white" => Color::White,
            "yellow" => Color::Yellow,
            _ => unreachable!(),
        }
    }

    fn spawn_variants_fetch(&mut self) {
        let variants = self.inner.varieties.iter().map(|v| &v.pokemon).cloned().enumerate();
        for (idx, api) in variants {
            let pkmn_client = self.pkmn_client.clone();
            let req_client = self.req_client.clone();
            self.futures.push(Box::pin(async move {
                let variant: Result<OfflineVariant> = async {
                    let variant = api.follow(&pkmn_client).await?;
                    OfflineVariant::new(variant, pkmn_client, req_client)
                }
                .await;
                let variant = match variant {
                    Ok(variant) => LoadState::Loaded(variant),
                    Err(e) => LoadState::log_error(e.into()),
                };
                (idx, variant)
            }));
        }
    }

    pub fn is_fully_loaded(&self) -> bool {
        self.variants
            .iter()
            .all(|v| matches!(&v, &LoadState::Loaded(variant) if variant.is_fully_loaded()))
    }
}
