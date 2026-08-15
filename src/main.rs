mod offline;
mod widgets;

use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::{EventStream, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{Event, KeyCode},
    prelude::*,
};
use rustemon::client::RustemonClient;
use tokio_stream::StreamExt;

use crate::{
    offline::OfflinePokemon,
    widgets::{InputState, InputWidget, RotomDexState, RotomDexWidget, StatusBarWidget},
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tui_logger::init_logger(log::LevelFilter::Warn)?;
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal).await;
    ratatui::restore();
    app_result
}

struct App {
    should_quit: bool,

    reqwest_client: reqwest::Client,
    rustemon_client: Arc<RustemonClient>,

    pkmn: OfflinePokemon,

    dex_state: RotomDexState,
    input_state: InputState,
}

impl Default for App {
    fn default() -> Self {
        let pkmn_name = "rotom".to_string();
        let reqwest_client = reqwest::Client::new();
        let rustemon_client = Arc::new(RustemonClient::default());
        Self {
            should_quit: false,
            pkmn: OfflinePokemon::new(pkmn_name, reqwest_client.clone(), rustemon_client.clone()),
            reqwest_client,
            rustemon_client,
            input_state: InputState::default(),
            dex_state: RotomDexState::default(),
        }
    }
}

impl App {
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                Some(Ok(event)) = events.next() => self.handle_event(&event),
                Some(event) = self.pkmn.ping() => self.pkmn.handle_fetch_event(event),
            }
            terminal.draw(|frame| self.render(frame.area(), frame.buffer_mut()))?;
        }
        Ok(())
    }

    /// Handles crossterm events like keybinds and terminal resizes
    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    self.should_quit = true;
                }
                (_, KeyCode::Enter) if !self.input_state.is_empty() => {
                    self.new_pokemon();
                }
                (_, KeyCode::Right) => {
                    self.dex_state.next();
                }
                (_, KeyCode::Left) => {
                    self.dex_state.prev();
                }
                (_, KeyCode::Backspace) => {
                    self.input_state.backspace();
                }
                (_, KeyCode::Char(ch)) => {
                    self.input_state.handle_input(ch);
                }
                _ => {}
            }
        }
    }

    fn new_pokemon(&mut self) {
        self.dex_state.reset();
        self.pkmn = OfflinePokemon::new(
            self.input_state.take(),
            self.reqwest_client.clone(),
            self.rustemon_client.clone(),
        );
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        StatusBarWidget {
            progress: self.pkmn.fetch_progress(),
        }
        .render(status_area, buf);

        RotomDexWidget { pkmn: &self.pkmn }.render(area, buf, &mut self.dex_state);
        InputWidget.render(area, buf, &mut self.input_state);
    }
}
