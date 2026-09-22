mod evolution_chain;
mod flavor_text;
mod variant;
use alloc::string::String;
use alloc::vec::Vec;
pub(crate) use evolution_chain::*;
pub(crate) use flavor_text::*;
use rotomdex_api::Client;
pub(crate) use variant::*;

use core::task::{Context, Poll};

use ratatui::style::Color;
use tracing::Span;

use crate::Settings;
use crate::data::resource::{AsyncResource, Fetchable, ResourceResult, SyncResource};
use crate::data::species::evolution_chain::ModelEvolutionChain;

#[derive(Debug)]
pub(crate) struct ModelSpecies {
    pub(crate) national_dex: u32,
    pub(crate) name: String,
    pub(crate) variants: Vec<AsyncResource<ModelVariant>>,
    pub(crate) color: Color,
    pub(crate) flavor_text: SyncResource<ModelFlavorText>,
    pub(crate) evolution_chain: Option<AsyncResource<ModelEvolutionChain>>,
}

impl Fetchable for ModelSpecies {
    type Request = String;
    async fn fetch(
        request: Self::Request,
        client: Client,
        settings: Settings,
    ) -> ResourceResult<Self> {
        let species =
            rotomdex_api::pokemon::pokemon_species::get_by_name(&request, &client).await?;

        let national_dex = species.id as u32;

        let variants: Vec<_> = species
            .varieties
            .into_iter()
            .map(|v| AsyncResource::<ModelVariant>::fetch(v.pokemon, &client, settings))
            .collect();

        let flavor_text =
            SyncResource::<ModelFlavorText>::derive(species.flavor_text_entries, &client, settings);

        let color = color(&species.color.name);

        let name = species.name;

        let evolution_chain = species
            .evolution_chain
            .map(|api| AsyncResource::<ModelEvolutionChain>::fetch(api, &client, settings));

        Ok(Self {
            national_dex,
            variants,
            name,
            color,
            flavor_text,
            evolution_chain,
        })
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let mut is_ready = false;

        is_ready |= self.variants.iter_mut().fold(false, |is_ready, variant| {
            is_ready | variant.poll(cx).is_ready()
        });

        if let Some(evolution_chain) = &mut self.evolution_chain
            && evolution_chain.poll(cx).is_ready()
        {
            is_ready |= true;
        }

        is_ready |= self.flavor_text.poll(cx).is_ready();

        if is_ready {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn fetch_span(request: &Self::Request) -> Span {
        tracing::info_span!("fetch_species", species = %request)
    }
}

fn color(name: &str) -> Color {
    match name {
        "black" => Color::Gray,
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
