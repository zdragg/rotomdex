mod model;
mod projector;
mod widgets;

use std::sync::Arc;

use ratatui::prelude::*;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::RustemonClient;

#[cfg(feature = "cache")]
use std::path::PathBuf;

use crate::{
    model::ModelPokemon,
    projector::Model2WidgetProjector,
    widgets::{DexWidget, InputState, InputWidget},
};

pub struct RotomDexCore {
    fetch_ctx: FetchContext,

    pkmn: ModelPokemon,

    projector: Model2WidgetProjector,
    input_state: InputState,
    bottom_text: String,
}

impl RotomDexCore {
    pub fn new(bottom_text: impl Into<String>) -> Self {
        let fetch_ctx = FetchContext::new();
        Self {
            pkmn: ModelPokemon::new("rotom", fetch_ctx.clone()),

            fetch_ctx,

            input_state: InputState::default(),
            projector: Model2WidgetProjector::new(),
            bottom_text: bottom_text.into(),
        }
    }

    #[cfg(feature = "cache")]
    pub fn new_with_cache(cache_dir: PathBuf, bottom_text: impl Into<String>) -> Self {
        let fetch_ctx = FetchContext::new_with_cache(cache_dir);
        Self {
            pkmn: ModelPokemon::new("rotom", fetch_ctx.clone()),

            fetch_ctx,

            input_state: InputState::default(),
            projector: Model2WidgetProjector::new(),
            bottom_text: bottom_text.into(),
        }
    }

    fn new_pokemon(&mut self) {
        self.projector.reset();
        self.pkmn = ModelPokemon::new(self.input_state.take(), self.fetch_ctx.clone());
    }

    pub async fn poll_pkmn(&mut self) {
        self.pkmn.poll().await
    }
}

impl Widget for &mut RotomDexCore {
    fn render(self, area: Rect, buf: &mut Buffer) {
        DexWidget::new(self.projector.project(&self.pkmn), &self.bottom_text).render(area, buf);
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
    Ignore,
}

pub trait ActionHandler {
    fn handle_action(&mut self, action: Action);
}

impl ActionHandler for RotomDexCore {
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Enter if !self.input_state.is_empty() => self.new_pokemon(),
            Action::Down | Action::Up | Action::Right | Action::Left => self.projector.handle_action(action),
            Action::Backspace => self.input_state.backspace(),
            Action::Input(ch) => self.input_state.handle_input(ch),
            _ => {}
        }
    }
}

#[derive(Clone)]
pub(crate) struct FetchContext {
    pub(crate) pkmn_client: Arc<RustemonClient>,
    pub(crate) req_client: ClientWithMiddleware,
}

impl FetchContext {
    fn new() -> Self {
        Self {
            pkmn_client: Arc::new(RustemonClient::default()),
            req_client: ClientBuilder::new(Client::new()).build(),
        }
    }

    #[cfg(feature = "cache")]
    fn new_with_cache(cache_dir: PathBuf) -> Self {
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
        }
    }
}
