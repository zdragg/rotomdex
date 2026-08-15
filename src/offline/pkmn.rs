use std::sync::Arc;

use color_eyre::eyre::Result;
use image::{DynamicImage, GenericImageView, Pixel};
use rustemon::{Follow, client::RustemonClient, model::pokemon::PokemonSpecies, pokemon};
use tokio::task::JoinSet;

use crate::offline::{OfflineSprite, OfflineVariant};

#[derive(Debug)]
pub struct OfflinePokemon {
    name: String,

    rustemon_client: Arc<RustemonClient>,
    reqwest_client: reqwest::Client,

    joinset: JoinSet<FetchEvent>, // all async tasks spawn from this. If the struct drops, all async tasks drop along with this JoinSet

    species: Option<PokemonSpecies>,

    variant_cnt: usize,
    variants: Vec<Option<OfflineVariant>>,
    sprites: Vec<Option<OfflineSprite>>,
}

impl OfflinePokemon {
    pub fn new(name: String, reqwest_client: reqwest::Client, rustemon_client: Arc<RustemonClient>) -> Self {
        let joinset = JoinSet::new();
        let mut pkmn = Self {
            name,

            reqwest_client,
            rustemon_client,

            joinset,

            species: None,

            variant_cnt: 0,
            variants: Vec::new(),
            sprites: Vec::new(),
        };
        pkmn.spawn_pokemon_fetch(); // Starts fetching data as soon as this is initialized
        pkmn
    }

    pub async fn ping(&mut self) -> Option<FetchEvent> {
        self.joinset.join_next().await.transpose().unwrap() // join error -> just panic entire program because it shouldnt happen
    }

    pub fn handle_fetch_event(&mut self, event: FetchEvent) {
        match event {
            FetchEvent::Error { err } => {
                log::error!("error occurred during fetching: {err}");
            }
            // PROBABLY SHOULD ADJUST COUNT BEFORE PUTTING SPECIES OR ELSE FETCH PROGRESS MAY BREAK
            // because fetch progress decides whether or not to fetch actual (loaded/total variant cnt) only when species exist
            // maybe make it atomic later if i figure out what atomic means
            FetchEvent::Species { species } => {
                self.adjust_variant_cnt(species.varieties.len());
                self.species = Some(*species);
                self.spawn_variants_fetch();
            }
            FetchEvent::Variant { idx, variant } => {
                self.variants[idx] = Some(*variant);
                self.spawn_sprite_fetch(idx);
            }
            FetchEvent::Sprite { idx, sprite } => {
                self.sprites[idx] = Some(sprite);
            }
        }
    }

    fn adjust_variant_cnt(&mut self, variant_cnt: usize) {
        self.variants.resize(variant_cnt, None);
        self.sprites.resize(variant_cnt, None);
        self.variant_cnt = variant_cnt;
    }

    pub fn variant_cnt(&self) -> usize {
        self.variant_cnt
    }

    pub fn variants(&self) -> &[Option<OfflineVariant>] {
        &self.variants
    }

    pub fn sprites(&self) -> &[Option<OfflineSprite>] {
        &self.sprites
    }

    fn spawn_pokemon_fetch(&mut self) {
        let name = self.name.clone();
        let client = self.rustemon_client.clone();
        self.joinset.spawn(async move {
            match pokemon::pokemon_species::get_by_name(&name, &client).await {
                Ok(species) => FetchEvent::Species {
                    species: Box::new(species),
                },
                Err(e) => FetchEvent::Error { err: e.into() },
            }
        });
    }

    fn spawn_variants_fetch(&mut self) {
        let Some(species) = &self.species else {
            log::error!("attempted to fetch variant when no species to fetch variant from was found");
            return;
        };
        let client = self.rustemon_client.clone();
        let variants: Vec<_> = species.varieties.iter().map(|v| &v.pokemon).cloned().collect();
        for (idx, v) in variants.into_iter().enumerate() {
            let client = client.clone();
            self.joinset.spawn(async move {
                let variant: Result<OfflineVariant> = async {
                    let variant = v.follow(&client).await?;
                    OfflineVariant::try_from(variant)
                }
                .await;
                match variant {
                    Ok(variant) => FetchEvent::Variant {
                        idx,
                        variant: Box::new(variant),
                    },
                    Err(e) => FetchEvent::Error { err: e },
                }
            });
        }
    }

    fn spawn_sprite_fetch(&mut self, idx: usize) {
        let Some(variant) = &self.variants[idx] else {
            log::error!("attempted to fetch sprite when the corresponding variant was not found");
            return;
        };
        let sprite_link = variant
            .pkmn
            .sprites
            .versions
            .generation_v
            .black_white
            .front_default
            .clone();

        let crop = |image: DynamicImage| -> DynamicImage {
            let (mut min_x, mut max_x, mut min_y, mut max_y) = (image.width(), 0, image.height(), 0);
            for (x, y, color) in image.pixels() {
                if color.alpha() == 0 {
                    continue;
                }
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
            let (mid_x, mid_y) = ((min_x + max_x) / 2, (min_y + max_y) / 2);
            let side_len = (max_x - min_x).max(max_y - min_y) + 2; // leave some space
            let (corner_x, corner_y) = (mid_x.saturating_sub(side_len / 2), mid_y.saturating_sub(side_len / 2));
            image.crop_imm(corner_x, corner_y, side_len, side_len)
        };

        let client = self.reqwest_client.clone();
        self.joinset.spawn(async move {
            let Some(link) = sprite_link else {
                return FetchEvent::Sprite {
                    idx,
                    sprite: OfflineSprite { sprite: None },
                };
            };

            let result: Result<DynamicImage> = async {
                let image_bytes = client.get(link).send().await?.bytes().await?;
                let image = image::load_from_memory(&image_bytes)?;
                Ok(crop(image))
            }
            .await;

            match result {
                Ok(image) => FetchEvent::Sprite {
                    idx,
                    sprite: OfflineSprite { sprite: Some(image) },
                },
                Err(e) => FetchEvent::Error { err: e },
            }
        });
    }

    pub fn fetch_progress(&self) -> FetchProgress {
        let species_loaded = self.species.is_some();
        if !species_loaded {
            return FetchProgress {
                species_loaded,
                variants: Progress::Indeterminate,
                sprites: Progress::Indeterminate,
            };
        }
        FetchProgress {
            species_loaded: self.species.is_some(),
            variants: Progress::Determinate {
                completed: self.variants.iter().filter(|v| v.is_some()).count(),
                total: self.variant_cnt,
            },
            sprites: Progress::Determinate {
                completed: self.sprites.iter().filter(|s| s.is_some()).count(),
                total: self.variant_cnt,
            },
        }
    }
}

pub struct FetchProgress {
    pub species_loaded: bool,
    pub variants: Progress,
    pub sprites: Progress,
}

pub enum Progress {
    Indeterminate,
    Determinate { completed: usize, total: usize },
}

pub enum FetchEvent {
    Species { species: Box<PokemonSpecies> },
    Variant { idx: usize, variant: Box<OfflineVariant> },
    Sprite { idx: usize, sprite: OfflineSprite },
    Error { err: color_eyre::eyre::Report },
}
