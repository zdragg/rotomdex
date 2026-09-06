mod context;
mod model;
mod versions;
mod widgets;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
pub use versions::*;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::{
    context::ModelContext,
    model::ModelPokemon,
    widgets::{DexState, DexWidget},
};

pub struct RotomDexCore {
    ctx: ModelContext,

    pkmn_name: String,
    pkmn: ModelPokemon,

    dex_state: DexState,
    local: bool,
    timer: web_time::Instant,
}

impl RotomDexCore {
    fn from_ctx(ctx: ModelContext, local: bool) -> Self {
        Self {
            pkmn_name: "rotom".into(),
            pkmn: ModelPokemon::new("rotom", ctx.clone()),

            ctx,

            dex_state: DexState::default(),
            local,
            timer: web_time::Instant::now(),
        }
    }

    pub fn new(local: bool) -> Self {
        let ctx = ModelContext::new();
        Self::from_ctx(ctx, local)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_cached(cache_dir: PathBuf) -> Self {
        let ctx = ModelContext::new_cache(cache_dir);
        Self::from_ctx(ctx, true)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// What should be under path:
    /// api/v2/pokemon-species/index.html
    /// sprites/pokemon/132.png
    pub fn new_offline(resource_path: PathBuf) -> Self {
        let ctx = ModelContext::new_offline(resource_path);
        Self::from_ctx(ctx, true)
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
            self.local,
            self.ctx.version,
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
        let action_result = self.dex_state.handle_action(action, self.ctx.version);

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
                self.ctx.version = version;
                self.refresh();
            }
        }
    }
}
