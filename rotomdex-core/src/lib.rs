mod offline;
mod widgets;

#[cfg(feature = "cache")]
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "cache")]
use std::path::PathBuf;
use std::sync::Arc;

use ratatui::prelude::*;

use crate::{
    offline::{FetchContext, OfflinePokemon},
    widgets::{InputState, InputWidget, RotomDexWidget, VariantState},
};

pub enum Action {
    Input(char),
    Backspace,
    Enter,
    RightArrow,
    LeftArrow,
    Ignore,
}
#[cfg(feature = "cache")]
type HttpClient = reqwest_middleware::ClientWithMiddleware;
#[cfg(not(feature = "cache"))]
type HttpClient = reqwest::Client;

pub struct RotomDexCore {
    fetch_ctx: FetchContext,

    pkmn: OfflinePokemon,

    dex_state: VariantState,
    input_state: InputState,
    bottom_text: String,
}

impl RotomDexCore {
    #[cfg(feature = "cache")]
    pub fn new(cache_dir: PathBuf, bottom_text: impl Into<String>) -> Self {
        use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
        use rustemon::client::RustemonClientBuilder;

        let pkmn_name = "rotom".to_string();

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

        let fetch_ctx = FetchContext {
            pkmn_client,
            req_client,
        };

        Self {
            pkmn: OfflinePokemon::new(pkmn_name, fetch_ctx.clone()),
            fetch_ctx: fetch_ctx,
            input_state: InputState::default(),
            dex_state: VariantState::default(),
            bottom_text: bottom_text.into(),
        }
    }

    #[cfg(not(feature = "cache"))]
    pub fn new(bottom_text: impl Into<String>) -> Self {
        use rustemon::client::RustemonClient;

        let pkmn_name = "rotom".to_string();
        let req_client = reqwest::Client::new();
        let pkmn_client = Arc::new(RustemonClient::default());

        let fetch_ctx = FetchContext {
            pkmn_client,
            req_client,
        };

        Self {
            pkmn: OfflinePokemon::new(pkmn_name, fetch_ctx.clone()),
            fetch_ctx,
            req_client,
            pkmn_client,
            input_state: InputState::default(),
            dex_state: VariantState::default(),
            bottom_text: bottom_text.into(),
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Enter if !self.input_state.is_empty() => self.new_pokemon(),
            Action::RightArrow => self.dex_state.next(),
            Action::LeftArrow => self.dex_state.prev(),
            Action::Backspace => self.input_state.backspace(),
            Action::Input(ch) => self.input_state.handle_input(ch),
            _ => {}
        }
    }

    fn new_pokemon(&mut self) {
        self.dex_state.reset();
        self.pkmn = OfflinePokemon::new(self.input_state.take(), self.fetch_ctx.clone());
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        RotomDexWidget::new(&self.pkmn, &self.bottom_text).render(area, buf, &mut self.dex_state);
        InputWidget.render(area, buf, &mut self.input_state);
    }

    pub async fn poll_pkmn(&mut self) {
        self.pkmn.poll().await
    }
}
