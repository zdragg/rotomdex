mod evolution_chain;
mod flavor_text;
mod variant;
pub(crate) use evolution_chain::*;
pub(crate) use flavor_text::*;
pub(crate) use variant::*;

use std::task::{Context, Poll};

use color_eyre::eyre::Result;
use ratatui::style::Color;
use tracing::Span;

use crate::ModelContext;
use crate::model::species::evolution_chain::ModelEvolutionChain;
use crate::model::{Fetchable, Resource};

#[derive(Debug)]
pub(crate) struct ModelSpecies {
    pub(crate) national_dex: u32,
    pub(crate) name: String,
    pub(crate) variants: Vec<Resource<ModelVariant>>,
    pub(crate) color: Color,
    pub(crate) flavor_text: Option<ModelFlavorText>,
    pub(crate) evolution_chain: Option<Resource<ModelEvolutionChain>>,
}

impl Fetchable for ModelSpecies {
    type Request = String;
    async fn fetch(request: Self::Request, ctx: ModelContext) -> Result<Self> {
        let species = rustemon::pokemon::pokemon_species::get_by_name(&request, &ctx.pkmn_client).await?;

        let national_dex = species.id as u32;

        let variants: Vec<_> = species
            .varieties
            .into_iter()
            .map(|v| Resource::<ModelVariant>::fetch(v.pokemon, &ctx))
            .collect();

        let flavor_text = ModelFlavorText::new(species.flavor_text_entries, ctx.version);

        let color = color(&species.color.name);

        let name = species.name;

        let evolution_chain = species
            .evolution_chain
            .map(|api| Resource::<ModelEvolutionChain>::fetch(api, &ctx));

        Ok(Self {
            national_dex,
            variants,
            name,
            color,
            flavor_text,
            evolution_chain,
        })
    }

    fn is_loaded(&self) -> bool {
        self.variants.iter().all(|variant| variant.is_loaded())
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        // bitwise OR for no short circuit
        if self
            .variants
            .iter_mut()
            .fold(false, |is_ready, variant| is_ready | variant.poll(cx).is_ready())
        {
            return Poll::Ready(());
        }
        if let Some(evolution_chain) = &mut self.evolution_chain {
            if evolution_chain.poll(cx).is_ready() {
                return Poll::Ready(());
            }
        }
        Poll::Pending
    }

    fn fetch_span(request: &Self::Request) -> Span {
        tracing::info_span!("fetch_species", species = %request)
    }
}

fn color<'a>(name: &'a str) -> Color {
    match name {
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

impl ModelSpecies {
    pub(crate) fn variants_cnt(&self) -> usize {
        self.variants.len()
    }

    pub(crate) fn variants(&self) -> &[Resource<ModelVariant>] {
        &self.variants
    }
}
