#![forbid(unsafe_code)]

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

    pub(crate) dex_state: DexState,
    timer: web_time::Instant,
}

impl RotomDexCore {
    fn from_ctx(ctx: ModelContext) -> Self {
        Self {
            pkmn_name: "rotom".into(),
            pkmn: ModelPokemon::new("rotom", ctx.clone()),

            ctx,

            dex_state: DexState::default(),
            timer: web_time::Instant::now(),
        }
    }

    pub fn new() -> Self {
        let ctx = ModelContext::new();
        Self::from_ctx(ctx)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_cached(cache_dir: PathBuf) -> Self {
        let ctx = ModelContext::new_cache(cache_dir);
        Self::from_ctx(ctx)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// What should be under path:
    /// api/v2/pokemon-species/index.html
    /// sprites/pokemon/132.png
    pub fn new_offline(resource_path: PathBuf) -> Self {
        let ctx = ModelContext::new_offline(resource_path);
        Self::from_ctx(ctx)
    }

    fn refresh(&mut self) {
        self.dex_state.reset();
        self.pkmn = ModelPokemon::new(self.pkmn_name.clone(), self.ctx.clone());
    }

    pub async fn poll_pkmn(&mut self) {
        self.pkmn.poll().await;
    }

    pub fn needs_continuous_render(&self) -> bool {
        self.dex_state.sprite_state.prefer_animation
    }
}

impl Widget for &RotomDexCore {
    fn render(self, area: Rect, buf: &mut Buffer) {
        DexWidget::new(&self.pkmn, &self.dex_state, self.timer.elapsed(), self.ctx.version).render(area, buf);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexKeyCode {
    Char(char),
    Backspace,
    Enter,
    Right,
    Down,
    Left,
    Up,
    Escape,
    CapsLock,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct DexKeyModifiers: u8 {
        const SHIFT = 1 << 0;
        const CONTROL = 1 << 1;
        const ALT = 1 << 2;
        const SUPER = 1 << 3;
        const HYPER = 1 << 4;
        const META = 1 << 5;
    }
}

pub enum ActionResult {
    Nothing,
    Exit,
}

enum InnerActionResult {
    Nothing,
    NewPokemon(String),
    NewVersion(Version),
}

impl RotomDexCore {
    pub fn handle_key(&mut self, modifiers: DexKeyModifiers, key_code: DexKeyCode) -> ActionResult {
        if modifiers.contains(DexKeyModifiers::CONTROL) {
            return if matches!(key_code, DexKeyCode::Char('c' | 'C')) {
                ActionResult::Exit
            } else {
                ActionResult::Nothing
            };
        }

        let action_result = self.dex_state.handle_key(key_code, self.ctx.version);

        match action_result {
            InnerActionResult::Nothing => (),
            InnerActionResult::NewPokemon(name) => {
                if name == "q" {
                    return ActionResult::Exit;
                }
                self.pkmn_name = name;
                self.refresh();
            }
            InnerActionResult::NewVersion(version) => {
                self.ctx.version = version;
                self.refresh();
            }
        }

        ActionResult::Nothing
    }
}
