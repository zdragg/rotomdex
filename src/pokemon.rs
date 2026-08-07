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
    /// Fetches species data, variant data, and 96x96 DS sprites, then combines into an `OfflinePokemon`.
    pub async fn fetch(name: &str, rustemon_client: Arc<RustemonClient>) -> Result<OfflinePokemon> {
        let species = pokemon::pokemon_species::get_by_name(name, &rustemon_client).await?;
        let mut variants = Vec::new();
        let mut variant_sprites = Vec::new();
        let mut default_idx = 0;
        for (i, v) in species.varieties.iter().enumerate() {
            let pkmn = v.pokemon.follow(&rustemon_client).await?;
            if pkmn.is_default {
                default_idx = i;
            }
            let maybe_sprite_link = pkmn
                .sprites
                .versions
                .generation_v
                .black_white
                .front_default
                .clone();
            let image = match maybe_sprite_link {
                Some(sprite_link) => {
                    let image_bytes = reqwest::get(sprite_link).await?.bytes().await?;
                    let sprite = image::load_from_memory(&image_bytes)?;
                    Some(sprite)
                }
                None => None,
            };
            variants.push(pkmn);
            variant_sprites.push(image);
        }
        Ok(OfflinePokemon {
            species,
            variants,
            variant_sprites,
            current_idx: 0,
            default_idx,
        })
    }

    pub fn get_current_pkmn(&self) -> &Pokemon {
        &self.variants[self.current_idx]
    }

    pub fn get_current_sprite(&self) -> Option<&DynamicImage> {
        if let Some(sprite) = &self.variant_sprites[self.current_idx] {
            return Some(sprite);
        } else if let Some(sprite) = &self.variant_sprites[self.default_idx] {
            return Some(sprite);
        } else {
            None
        }
    }

    pub fn next(&mut self) {
        self.current_idx = (self.current_idx + 1) % self.variants.len();
    }

    pub fn prev(&mut self) {
        self.current_idx = (self.current_idx + self.variants.len() - 1) % self.variants.len();
    }
}
