use std::sync::Arc;

use color_eyre::eyre::Result;
use image::DynamicImage;
use rustemon::{
    Follow,
    client::RustemonClient,
    model::pokemon::{Pokemon, PokemonSpecies},
    pokemon,
};

#[derive(Debug)]
pub struct OfflinePokemon {
    species: PokemonSpecies,
    variants: Vec<Pokemon>,
    variant_sprites: Vec<Option<DynamicImage>>,
    current_idx: usize,
    default_idx: usize,
}

impl OfflinePokemon {
    /// Fetches species data and variant data, then combines into an `OfflinePokemon`.
    pub async fn fetch_data(
        name: &str,
        rustemon_client: Arc<RustemonClient>,
    ) -> Result<OfflinePokemon> {
        let species = pokemon::pokemon_species::get_by_name(name, &rustemon_client).await?;
        let mut variants = Vec::new();
        let mut default_idx = 0;
        for (i, v) in species.varieties.iter().enumerate() {
            let pkmn = v.pokemon.follow(&rustemon_client).await?;
            if pkmn.is_default {
                default_idx = i;
            }
            variants.push(pkmn);
        }
        let variant_sprites = vec![None; variants.len()];
        Ok(OfflinePokemon {
            species,
            variants,
            variant_sprites,
            current_idx: 0,
            default_idx,
        })
    }

    /// Inject a sprite into the pokemon at the specified index.
    /// Does nothing if the index is invalid,
    pub fn inject_sprite(&mut self, idx: usize, image: DynamicImage) {
        if idx >= self.variant_sprites.len() {
            return;
        } // Invalid index
        self.variant_sprites[idx] = Some(image);
    }

    /// Returns a vector of all existing sprite links in format `(idx: usize, link: String)`
    pub fn get_sprite_links(&self) -> Vec<(usize, String)> {
        self.variants
            .iter()
            .enumerate()
            .filter_map(|(i, pkmn)| {
                let link = pkmn
                    .sprites
                    .versions
                    .generation_v
                    .black_white
                    .front_default
                    .clone()?;
                Some((i, link))
            })
            .collect()
    }

    /// Get the pokemon variant specified by the index.
    pub fn get_current_pkmn(&self) -> &Pokemon {
        &self.variants[self.current_idx]
    }

    // Get the pokemon sprite associated with the variant specified by the index.
    // Some(&DynamicImage) if it exists, None if there is no image loaded.
    pub fn get_current_sprite(&self) -> Option<&DynamicImage> {
        if let Some(sprite) = &self.variant_sprites[self.current_idx] {
            return Some(sprite);
        }
        // if let Some(sprite) = &self.variant_sprites[self.default_idx] {
        //     return Some(sprite);
        // }
        None
    }

    /// Shifts variant index forward.
    pub fn next(&mut self) {
        self.current_idx = (self.current_idx + 1) % self.variants.len();
    }

    /// Shifts variant index backward.
    pub fn prev(&mut self) {
        self.current_idx = (self.current_idx + self.variants.len() - 1) % self.variants.len();
    }
}
