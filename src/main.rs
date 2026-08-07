mod pokemon;
mod widgets;

use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use crossterm::event::{EventStream, KeyModifiers};
use parking_lot::Mutex;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{Event, KeyCode},
    prelude::*,
    widgets::Block,
};
use smart_default::SmartDefault;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::StreamExt;

use crate::{
    pokemon::OfflinePokemon,
    widgets::{InputMode, InputWidget, RotomDexWidget},
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal).await;
    ratatui::restore();
    app_result
}

#[derive(SmartDefault)]
struct App {
    should_quit: bool,
    #[default = true]
    should_update: bool,
    pkmn_fetch_state: Arc<Mutex<FetchState>>,
    dex_widget: RotomDexWidget,
    input_widget: InputWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        let (pkmn_tx, mut pkmn_rx) = mpsc::channel(32);

        self.start_pokemon_fetch(pkmn_tx.clone());

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick(), if self.should_update => {
                    terminal.draw(|frame| self.render(frame))?;
                    self.should_update = false;
                },
                Some(Ok(event)) = events.next() => self.handle_event(&event, pkmn_tx.clone()),
                Some(result) = pkmn_rx.recv() => self.handle_pokemon_fetch_res(result)
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let block = Block::new().title_bottom(format!("{:?}", self.pkmn_fetch_state.lock()));
        let area_without_status_bar = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        frame.render_widget(&self.dex_widget, area_without_status_bar);
        if self.input_widget.input_mode == InputMode::Editing {
            frame.render_widget(&self.input_widget, area_without_status_bar); // Overlaid on top
        }
    }

    fn handle_event(&mut self, event: &Event, pkmn_tx: Sender<Result<OfflinePokemon>>) {
        match event {
            Event::Key(key) => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    self.should_quit = true;
                }
                (_, KeyCode::Enter) if self.input_widget.input_mode == InputMode::Editing => {
                    self.start_pokemon_fetch(pkmn_tx);
                    self.should_update = true;
                }
                (_, KeyCode::Right) => {
                    if let Some(pkmn) = &mut self.dex_widget.pokemon {
                        pkmn.next();
                        self.should_update = true;
                    }
                }
                (_, KeyCode::Left) => {
                    if let Some(pkmn) = &mut self.dex_widget.pokemon {
                        pkmn.prev();
                        self.should_update = true;
                    }
                }
                (_, KeyCode::Backspace) => {
                    self.input_widget.clear();
                    self.should_update = true;
                }
                (_, KeyCode::Char(ch)) => {
                    self.input_widget.handle_input(ch);
                    self.should_update = true;
                }
                _ => {}
            },
            Event::Resize(_, _) => self.should_update = true,
            _ => {}
        }
    }

    /// Spawns task to fetch pokemon. Returns `Receiver` for the task.
    fn start_pokemon_fetch(&mut self, tx: Sender<Result<OfflinePokemon>>) {
        if *self.pkmn_fetch_state.lock() == FetchState::Loading {
            return;
        }
        self.update_fetch_state(FetchState::Loading);
        let name = self.input_widget.take();
        let rustemon_client = self.dex_widget.rustemon_client.clone();

        tokio::spawn(async move {
            let result = OfflinePokemon::fetch(&name, rustemon_client).await;
            let _ = tx.send(result).await;
        });
    }

    /// Handles `Result` returned by `OfflinePokemon::fetch()`.
    fn handle_pokemon_fetch_res(&mut self, res: Result<OfflinePokemon>) {
        match res {
            Ok(pkmn) => {
                self.dex_widget.pokemon = Some(pkmn);
                self.update_fetch_state(FetchState::Loaded);
            }
            Err(e) => self.update_fetch_state(FetchState::Error(e.to_string())),
        }
        self.should_update = true;
    }

    /// Updates `FetchState` - how the construction of `OfflinePokemon` is going
    fn update_fetch_state(&mut self, state: FetchState) {
        *self.pkmn_fetch_state.lock() = state;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum FetchState {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error(String),
}
