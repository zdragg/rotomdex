mod species;
use std::{
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

use rustemon::client::RustemonClient;
pub use species::*;
use tokio::task::JoinSet;

#[derive(Debug, Default)]
pub enum LoadState<T> {
    #[default]
    Loading,
    Loaded(T),
    Failed(color_eyre::eyre::Report),
}

impl<T> LoadState<T> {
    pub fn as_loaded(&self) -> Option<&T> {
        if let Self::Loaded(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    pub fn log_error(err: color_eyre::eyre::Report) -> Self {
        log::error!("{err}");
        Self::Failed(err)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, LoadState::Loaded(_))
    }
}

enum FetchEvent {
    Species { species: LoadState<OfflineSpecies> },
}

pub struct OfflinePokemon {
    name: String,

    joinset: JoinSet<FetchEvent>,
    pkmn_client: Arc<RustemonClient>,
    req_client: reqwest::Client,

    benchmark: Instant,

    species: LoadState<OfflineSpecies>,

    fully_loaded: bool,
}

impl OfflinePokemon {
    pub fn new(name: String, pkmn_client: Arc<RustemonClient>, req_client: reqwest::Client) -> Self {
        let mut result = Self {
            name,

            joinset: JoinSet::new(),
            pkmn_client,
            req_client,

            benchmark: Instant::now(),

            species: LoadState::Loading,

            fully_loaded: false,
        };
        result.spawn_species_fetch();
        result
    }

    pub async fn ping(&mut self) {
        std::future::poll_fn(|cx| self.poll_load(cx)).await;
        if !self.fully_loaded && self.is_fully_loaded() {
            self.fully_loaded = true;
            log::info!(
                "{} fully loaded in {}ms",
                self.name,
                self.benchmark.elapsed().as_millis()
            );
        }
    }

    fn poll_load(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(Some(event)) = self.joinset.poll_join_next(cx) {
            self.handle_event(event.unwrap());
            return Poll::Ready(());
        }

        if let LoadState::Loaded(species) = &mut self.species
            && species.poll_load(cx).is_ready()
        {
            return Poll::Ready(());
        }

        Poll::Pending
    }

    fn handle_event(&mut self, event: FetchEvent) {
        match event {
            FetchEvent::Species { species } => {
                self.species = species;
            }
        }
    }

    pub fn species(&self) -> &LoadState<OfflineSpecies> {
        &self.species
    }

    fn spawn_species_fetch(&mut self) {
        let name = self.name.clone();
        log::info!("fetching species: {name}");
        let pkmn_client = self.pkmn_client.clone();
        let req_client = self.req_client.clone();

        self.joinset.spawn(async move {
            let species = match rustemon::pokemon::pokemon_species::get_by_name(&name, &pkmn_client).await {
                Ok(species) => {
                    let species = OfflineSpecies::new(species, pkmn_client, req_client);
                    LoadState::Loaded(species)
                }
                Err(e) => LoadState::log_error(e.into()),
            };
            FetchEvent::Species { species }
        });
    }

    pub fn is_fully_loaded(&self) -> bool {
        if let LoadState::Loaded(species) = &self.species {
            species.is_fully_loaded()
        } else {
            false
        }
    }
}
