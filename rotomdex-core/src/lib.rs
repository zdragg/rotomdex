mod model;
mod settings;
mod widgets;

use std::sync::Arc;

use ratatui::prelude::*;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::RustemonClient;

pub use settings::*;

#[cfg(feature = "cache")]
use std::path::PathBuf;

use crate::{
    model::ModelPokemon,
    widgets::{DexState, DexWidget},
};

pub struct RotomDexCore {
    fetch_ctx: ModelContext,

    pkmn: ModelPokemon,

    dex_state: DexState,
    can_exit: bool,
    timer: web_time::Instant,
}

impl RotomDexCore {
    pub fn new(settings: Settings, can_exit: bool) -> Self {
        let fetch_ctx = ModelContext::new(settings);
        Self {
            pkmn: ModelPokemon::new("rotom", fetch_ctx.clone()),

            fetch_ctx,

            dex_state: DexState::default(),
            can_exit,
            timer: web_time::Instant::now(),
        }
    }

    #[cfg(feature = "cache")]
    pub fn new_with_cache(settings: Settings, can_exit: bool, cache_dir: PathBuf) -> Self {
        let fetch_ctx = ModelContext::new_with_cache(settings, cache_dir);
        Self {
            pkmn: ModelPokemon::new("rotom", fetch_ctx.clone()),

            fetch_ctx,

            dex_state: DexState::default(),
            can_exit,
            timer: web_time::Instant::now(),
        }
    }

    fn new_pokemon(&mut self, name: String) {
        self.dex_state.reset();
        self.pkmn = ModelPokemon::new(name, self.fetch_ctx.clone());
    }

    pub async fn poll_pkmn(&mut self) {
        self.pkmn.poll().await
    }
}

impl Widget for &mut RotomDexCore {
    fn render(self, area: Rect, buf: &mut Buffer) {
        DexWidget::new(&self.pkmn, &self.dex_state, self.timer.elapsed(), self.can_exit).render(area, buf);
    }
}

pub enum Action {
    Input(char),
    Backspace,
    Enter,
    Right,
    Down,
    Left,
    Up,
    Escape,
    CapsLock,
}

impl RotomDexCore {
    pub fn handle_action(&mut self, action: Action) {
        let pkmn_name = self.dex_state.handle_action(action);

        if let Some(name) = pkmn_name {
            self.new_pokemon(name);
        }
    }
}

#[derive(Clone)]
pub(crate) struct ModelContext {
    pub(crate) pkmn_client: Arc<RustemonClient>,
    pub(crate) req_client: ClientWithMiddleware,
    pub(crate) settings: Settings,
}

impl ModelContext {
    fn new(settings: Settings) -> Self {
        Self {
            pkmn_client: Arc::new(RustemonClient::default()),
            req_client: ClientBuilder::new(Client::new()).build(),
            settings,
        }
    }

    #[cfg(feature = "cache")]
    fn new_with_cache(settings: Settings, cache_dir: PathBuf) -> Self {
        use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
        use rustemon::client::RustemonClientBuilder;

        let non_pokeapi_cache_manager = http_cache_reqwest::CACacheManager::new(cache_dir.join("non-pokeapi"), false);
        let req_client = ClientBuilder::new(reqwest::Client::new())
            .with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: non_pokeapi_cache_manager,
                options: HttpCacheOptions::default(),
            }))
            .build();

        let pokeapi_cache_manager = rustemon::client::CACacheManager::new(cache_dir.join("pokeapi"), false);
        let pkmn_client = Arc::new(
            RustemonClientBuilder::default()
                .with_manager(pokeapi_cache_manager)
                .try_build()
                .unwrap(),
        );

        Self {
            pkmn_client,
            req_client,
            settings,
        }
    }
}
