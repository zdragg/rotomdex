mod variant;
pub(crate) use variant::*;

use std::task::{Context, Poll};

use color_eyre::eyre::Result;
use ratatui::style::Color;
use rustemon::model::pokemon::PokemonSpecies;
use tracing::Span;

use crate::FetchContext;
use crate::model::{Fetchable, Resource};

#[derive(Debug)]
pub(crate) struct ModelSpecies {
    pub(crate) variants: Vec<Resource<ModelVariant>>,
    pub(crate) inner: PokemonSpecies,
}

impl Fetchable for ModelSpecies {
    type Request = String;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let species = rustemon::pokemon::pokemon_species::get_by_name(&request, &ctx.pkmn_client).await?;
        let variants: Vec<_> = species
            .varieties
            .iter()
            .map(|v| Resource::<ModelVariant>::fetch(v.pokemon.clone(), ctx.clone(), false))
            .collect();
        Ok(Self {
            variants,
            inner: species,
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
        Poll::Pending
    }

    fn fetch_span(request: &Self::Request) -> Span {
        tracing::info_span!("fetch_species", species = %request)
    }
}

impl ModelSpecies {
    pub(crate) fn inner(&self) -> &PokemonSpecies {
        &self.inner
    }

    pub(crate) fn variants_cnt(&self) -> usize {
        self.variants.len()
    }

    pub(crate) fn variants(&self) -> &[Resource<ModelVariant>] {
        &self.variants
    }

    pub(crate) fn get_ratatui_color(&self) -> Color {
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
}
