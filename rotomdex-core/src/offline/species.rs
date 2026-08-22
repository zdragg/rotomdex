mod variant;

use std::task::{Context, Poll};

use color_eyre::eyre::Result;
use ratatui::style::Color;
use rustemon::model::pokemon::PokemonSpecies;

pub use variant::*;

use crate::offline::{FetchContext, Fetchable, Resource};

#[derive(Debug)]
pub struct OfflineSpecies {
    variants: Vec<Resource<OfflineVariant>>,
    inner: PokemonSpecies,
}

impl Fetchable for OfflineSpecies {
    type Request = String;
    async fn fetch(request: Self::Request, ctx: FetchContext) -> Result<Self> {
        let species = rustemon::pokemon::pokemon_species::get_by_name(&request, &ctx.pkmn_client).await?;
        let variants: Vec<_> = species
            .varieties
            .iter()
            .map(|v| Resource::<OfflineVariant>::fetch(v.pokemon.clone(), ctx.clone()))
            .collect();
        Ok(Self {
            variants,
            inner: species,
        })
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.variants.iter_mut().any(|variant| variant.poll(cx).is_ready()) {
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn is_loaded(&self) -> bool {
        self.variants.iter().all(|variant| variant.is_loaded())
    }
}

impl OfflineSpecies {
    pub fn inner(&self) -> &PokemonSpecies {
        &self.inner
    }

    pub fn variants_cnt(&self) -> usize {
        self.variants.len()
    }

    pub fn variants(&self) -> &[Resource<OfflineVariant>] {
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
}
