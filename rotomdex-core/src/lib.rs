#![forbid(unsafe_code)]
#![no_std]

#[macro_use]
extern crate alloc;

mod data;
mod settings;

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use embassy_time::Instant;
pub use rotomdex_api::client::Client;
pub use settings::*;
mod traits;
pub use traits::*;
mod widgets;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    data::DataStore,
    widgets::{DexWidget, DexWidgetState},
};

pub struct RotomDexCore {
    data: DataStore,
    dex_state: DexWidgetState,
    timer: Instant,

    session_rw: Box<dyn SessionRw>,
    pkmn_name: String,
    pub settings: Settings,
}

impl RotomDexCore {
    pub fn new(client: Client, session_rw: impl SessionRw + 'static) -> Self {
        let Session {
            pkmn_name,
            settings,
        } = session_rw.read();
        Self {
            data: DataStore::new(client, &pkmn_name, settings),
            dex_state: DexWidgetState::default(),
            timer: Instant::now(),

            session_rw: Box::new(session_rw),
            pkmn_name,
            settings,
        }
    }

    pub async fn poll(&mut self) {
        self.data.poll().await;
    }
}

impl Widget for &RotomDexCore {
    fn render(self, area: Rect, buf: &mut Buffer) {
        DexWidget::new(
            &self.data,
            &self.dex_state,
            self.settings,
            self.timer.elapsed(),
        )
        .render(area, buf);
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

pub enum MaybeExit {
    DontExit,
    Exit,
}

pub enum Command {
    NewPokemon(String),
    NewVersion(Version),
    AnimationMode(bool),
}

impl RotomDexCore {
    pub fn handle_key(&mut self, modifiers: DexKeyModifiers, key_code: DexKeyCode) -> MaybeExit {
        if modifiers.contains(DexKeyModifiers::CONTROL)
            && matches!(key_code, DexKeyCode::Char('c' | 'C'))
        {
            return self.exit();
        };

        if key_code == DexKeyCode::Char('g') {
            self.settings.animation_mode = !self.settings.animation_mode;
            return MaybeExit::DontExit;
        }

        let mut maybe_cmd = None;

        self.dex_state
            .handle_key(key_code, self.settings, &mut maybe_cmd);

        if let Some(cmd) = maybe_cmd {
            match cmd {
                Command::NewPokemon(name) => match name.as_str() {
                    "q" => return self.exit(),
                    "" => {}
                    _ => {
                        self.pkmn_name = name;
                        self.data.new_resource(&self.pkmn_name, self.settings);
                    }
                },
                Command::NewVersion(version) => {
                    self.settings.version = version;
                    self.data.new_resource(&self.pkmn_name, self.settings);
                }
                Command::AnimationMode(b) => self.settings.animation_mode = b,
            }
        }

        MaybeExit::DontExit
    }

    fn exit(&mut self) -> MaybeExit {
        self.session_rw.write(Session {
            pkmn_name: self.pkmn_name.clone(),
            settings: self.settings,
        });
        MaybeExit::Exit
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pkmn_name: String,
    settings: Settings,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            pkmn_name: "rotom".to_string(),
            settings: Settings::default(),
        }
    }
}
