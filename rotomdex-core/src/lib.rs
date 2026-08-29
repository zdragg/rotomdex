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
    widgets::{DexState, DexWidget, InputState, InputWidget},
};

pub struct RotomDexCore {
    fetch_ctx: ModelContext,

    pkmn: ModelPokemon,

    dex_state: DexState,
    input_state: InputState,
    bottom_text: String,
    timer: web_time::Instant,
}

impl RotomDexCore {
    pub fn new(settings: Settings, bottom_text: impl Into<String>) -> Self {
        let fetch_ctx = ModelContext::new(settings);
        Self {
            pkmn: ModelPokemon::new("rotom", fetch_ctx.clone()),

            fetch_ctx,

            dex_state: DexState::default(),
            input_state: InputState::default(),
            bottom_text: bottom_text.into(),
            timer: web_time::Instant::now(),
        }
    }

    #[cfg(feature = "cache")]
    pub fn new_with_cache(settings: Settings, bottom_text: impl Into<String>, cache_dir: PathBuf) -> Self {
        let fetch_ctx = ModelContext::new_with_cache(settings, cache_dir);
        Self {
            pkmn: ModelPokemon::new("rotom", fetch_ctx.clone()),

            fetch_ctx,

            dex_state: DexState::default(),
            input_state: InputState::default(),
            bottom_text: bottom_text.into(),
            timer: web_time::Instant::now(),
        }
    }

    fn new_pokemon(&mut self) {
        self.dex_state.reset();
        self.pkmn = ModelPokemon::new(self.input_state.take(), self.fetch_ctx.clone());
    }

    pub async fn poll_pkmn(&mut self) {
        self.pkmn.poll().await
    }
}

impl Widget for &mut RotomDexCore {
    fn render(self, area: Rect, buf: &mut Buffer) {
        DexWidget::new(&self.pkmn, &self.dex_state, self.timer.elapsed(), &self.bottom_text).render(area, buf);
        InputWidget.render(area, buf, &mut self.input_state);
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
}

pub trait ActionHandler {
    fn handle_action(&mut self, action: Action);
}

impl ActionHandler for RotomDexCore {
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Enter if !self.input_state.is_empty() => self.new_pokemon(),
            Action::Down | Action::Up | Action::Right | Action::Left => self.dex_state.handle_action(action),
            Action::Backspace => self.input_state.backspace(),
            Action::Input(ch) => self.input_state.handle_input(ch),
            _ => {}
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
