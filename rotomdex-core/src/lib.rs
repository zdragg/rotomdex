mod model;
mod versions;
mod widgets;

use std::sync::Arc;

use ratatui::prelude::*;
use reqwest::{Client, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::{Environment, RustemonClient};

pub use versions::*;

#[cfg(feature = "cache")]
use std::path::PathBuf;

use crate::{
    model::ModelPokemon,
    widgets::{DexState, DexWidget},
};

pub struct RotomDexCore {
    ctx: ModelContext,
    version: Version,

    pkmn_name: String,
    pkmn: ModelPokemon,

    dex_state: DexState,
    can_exit: bool,
    timer: web_time::Instant,
}

impl RotomDexCore {
    pub fn new(can_exit: bool) -> Self {
        let version = Version::default();
        let ctx = ModelContext::new_without_cache(version);
        Self {
            version: Version::default(),

            pkmn_name: "rotom".into(),
            pkmn: ModelPokemon::new("rotom", ctx.clone()),

            ctx,

            dex_state: DexState::default(),
            can_exit,
            timer: web_time::Instant::now(),
        }
    }

    #[cfg(feature = "cache")]
    pub fn new_with_cache(can_exit: bool, cache_dir: PathBuf) -> Self {
        let version = Version::default();
        let ctx = ModelContext::new_with_cache(version, cache_dir);
        Self {
            version: Version::default(),
            pkmn_name: "rotom".into(),
            pkmn: ModelPokemon::new("rotom", ctx.clone()),

            ctx,

            dex_state: DexState::default(),
            can_exit,
            timer: web_time::Instant::now(),
        }
    }

    fn refresh(&mut self) {
        self.dex_state.reset();
        self.pkmn = ModelPokemon::new(self.pkmn_name.clone(), self.ctx.clone());
    }

    pub async fn poll_pkmn(&mut self) {
        self.pkmn.poll().await;
    }
}

impl Widget for &RotomDexCore {
    fn render(self, area: Rect, buf: &mut Buffer) {
        DexWidget::new(
            &self.pkmn,
            &self.dex_state,
            self.timer.elapsed(),
            self.can_exit,
            self.version,
        )
        .render(area, buf);
    }
}

#[derive(Clone, Copy)]
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

enum ActionResult {
    Nothing,
    NewPokemon(String),
    NewVersion(Version),
}

impl RotomDexCore {
    pub fn handle_action(&mut self, action: Action) {
        let action_result = self.dex_state.handle_action(action, self.version);

        match action_result {
            ActionResult::Nothing => (),
            ActionResult::NewPokemon(name) => {
                if name == "q" {
                    panic!()
                }
                self.pkmn_name = name;
                self.refresh();
            }
            ActionResult::NewVersion(version) => {
                self.version = version;
                self.ctx.version = version;
                self.refresh();
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct ModelContext {
    pub(crate) pkmn_client: Arc<RustemonClient>,
    pub(crate) req_client: ClientWithMiddleware,
    pub(crate) version: Version,
}

impl ModelContext {
    fn new(version: Version, builder: ClientBuilder) -> Self {
        let client = builder.build();
        let rustemon_wrapper = Arc::new(RustemonClient {
            base: Url::try_from(Environment::default()).unwrap(),
            client: client.clone(),
        });

        Self {
            pkmn_client: rustemon_wrapper,
            req_client: client,
            version,
        }
    }
    fn new_without_cache(version: Version) -> Self {
        let builder = ClientBuilder::new(Client::new());

        ModelContext::new(version, builder)
    }

    #[cfg(feature = "cache")]
    fn new_with_cache(version: Version, cache_dir: PathBuf) -> Self {
        use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};

        let cache_manager = http_cache_reqwest::CACacheManager::new(cache_dir, false);
        let cache_middleware = HttpCache {
            mode: CacheMode::Default,
            manager: cache_manager,
            options: HttpCacheOptions::default(),
        };

        let builder = ClientBuilder::new(reqwest::Client::new()).with(Cache(cache_middleware));

        ModelContext::new(version, builder)
    }
}
