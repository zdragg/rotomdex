mod context;
mod model;
mod versions;
mod widgets;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
pub use versions::*;

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
    can_exit: bool,
    timer: web_time::Instant,
}

impl RotomDexCore {
    pub fn new(can_exit: bool) -> Self {
        let ctx = ModelContext::new(None);
        Self {
            pkmn_name: "rotom".into(),
            pkmn: ModelPokemon::new("rotom", ctx.clone()),

            ctx,

            dex_state: DexState::default(),
            can_exit,
            timer: web_time::Instant::now(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_with_cache(can_exit: bool, cache_dir: PathBuf) -> Self {
        let ctx = ModelContext::new(Some(cache_dir));
        Self {
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
