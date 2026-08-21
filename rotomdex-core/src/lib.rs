mod offline;
mod widgets;

#[cfg(feature = "cache")]
use std::path::PathBuf;
use std::sync::Arc;

use ratatui::prelude::*;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::RustemonClient;

use crate::{
    offline::OfflinePokemon,
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

pub struct RotomDexCore {
    req_client: ClientWithMiddleware,
    pkmn_client: Arc<RustemonClient>,

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

        let new_cache_manager = http_cache_reqwest::CACacheManager::new(cache_dir.clone(), false);
        let req_client = ClientBuilder::new(reqwest::Client::new())
            .with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: new_cache_manager,
                options: HttpCacheOptions::default(),
            }))
            .build();

        let old_cache_manager = rustemon::client::CACacheManager::new(cache_dir, false);
        let pkmn_client = Arc::new(
            RustemonClientBuilder::default()
                .with_manager(old_cache_manager)
                .try_build()
                .unwrap(),
        );
        Self {
            pkmn: OfflinePokemon::new(pkmn_name, pkmn_client.clone(), req_client.clone()),
            req_client,
            pkmn_client,
            input_state: InputState::default(),
            dex_state: VariantState::default(),
            bottom_text: bottom_text.into(),
        }
    }

    #[cfg(not(feature = "cache"))]
    pub fn new(bottom_text: impl Into<String>) -> Self {
        let pkmn_name = "rotom".to_string();
        let req_client = ClientBuilder::new(reqwest::Client::new()).build();
        let pkmn_client = Arc::new(RustemonClient::default());
        Self {
            pkmn: OfflinePokemon::new(pkmn_name, pkmn_client.clone(), req_client.clone()),
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
        self.pkmn = OfflinePokemon::new(
            self.input_state.take(),
            self.pkmn_client.clone(),
            self.req_client.clone(),
        );
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        RotomDexWidget::new(&self.pkmn, &self.bottom_text).render(area, buf, &mut self.dex_state);
        InputWidget.render(area, buf, &mut self.input_state);
    }

    pub async fn ping(&mut self) {
        self.pkmn.ping().await
    }
}
