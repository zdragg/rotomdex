mod pokemon;
mod widgets;

use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use crossterm::event::{EventStream, KeyModifiers};
use image::{DynamicImage, GenericImageView, Pixel};
use parking_lot::Mutex;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{Event, KeyCode},
    prelude::*,
    widgets::Block,
};
use rustemon::client::RustemonClient;
use smart_default::SmartDefault;
use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinSet,
};
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

    fetch_state: Arc<Mutex<FetchState>>,

    reqwest_client: reqwest::Client,
    rustemon_client: Arc<RustemonClient>,

    dex_widget: Option<RotomDexWidget>,
    input_widget: InputWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        let (pkmn_data_tx, mut pkmn_data_rx) = mpsc::channel(32);
        let (pkmn_sprite_tx, mut pkmn_sprite_rx) = mpsc::channel(32);

        self.start_pkmn_data_fetch(pkmn_data_tx.clone());

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick(), if self.should_update => {
                    terminal.draw(|frame| self.render(frame))?;
                    self.should_update = false;
                },
                Some(Ok(event)) = events.next() => self.handle_event(&event, pkmn_data_tx.clone()),
                Some(result) = pkmn_data_rx.recv(), if *self.fetch_state.lock() == FetchState::LoadingData => {
                    self.handle_pkmn_data(result, pkmn_sprite_tx.clone());
                }
                Some(result) = pkmn_sprite_rx.recv(), if *self.fetch_state.lock() == FetchState::LoadingSprite => {
                    self.handle_pkmn_sprite(result);
                }
            }
        }
        Ok(())
    }

    /// Handles crossterm events like keybinds and terminal resizes
    fn handle_event(&mut self, event: &Event, pkmn_tx: Sender<Result<OfflinePokemon>>) {
        match event {
            Event::Key(key) => match (key.modifiers, key.code) {
                (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    self.should_quit = true;
                }
                (_, KeyCode::Enter) if self.input_widget.input_mode == InputMode::Editing => {
                    self.start_pkmn_data_fetch(pkmn_tx);
                    self.should_update = true;
                }
                (_, KeyCode::Right) => {
                    if let Some(widget) = &mut self.dex_widget {
                        widget.pkmn.next();
                        self.should_update = true;
                    }
                }
                (_, KeyCode::Left) => {
                    if let Some(widget) = &mut self.dex_widget {
                        widget.pkmn.prev();
                        self.should_update = true;
                    }
                }
                (_, KeyCode::Backspace) => {
                    self.input_widget.backspace();
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
    fn start_pkmn_data_fetch(&mut self, tx: Sender<Result<OfflinePokemon>>) {
        if let FetchState::LoadingData | FetchState::LoadingSprite = *self.fetch_state.lock() {
            return;
        }
        self.update_fetch_state(FetchState::LoadingData);
        let name = self.input_widget.take();
        let rustemon_client = self.rustemon_client.clone();
        tokio::spawn(async move {
            let result = OfflinePokemon::fetch_data(&name, rustemon_client).await;
            let _ = tx.send(result).await;
        });
    }

    /// Handles `Result` returned by `OfflinePokemon::fetch()`.
    fn handle_pkmn_data(&mut self, res: Result<OfflinePokemon>, tx: Sender<SpriteFetchEvent>) {
        match res {
            Ok(pkmn) => {
                self.start_pkmn_sprite_fetch(tx, &pkmn);
                self.dex_widget = Some(RotomDexWidget { pkmn });
            }
            Err(e) => {
                self.update_fetch_state(FetchState::Error(e.to_string()));
            }
        };
        self.should_update = true;
    }

    /// Spawns (as) many tasks (as the number of variants with existing sprite links) to fetch pokemon sprites
    fn start_pkmn_sprite_fetch(&mut self, tx: Sender<SpriteFetchEvent>, pkmn: &OfflinePokemon) {
        if *self.fetch_state.lock() != FetchState::LoadingData {
            return;
        }; // Must happen right after fetching pkmn data

        self.update_fetch_state(FetchState::LoadingSprite);

        let trim_image = |image: &DynamicImage| -> DynamicImage {
            let (mut min_x, mut max_x, mut min_y, mut max_y) =
                (image.width(), 0, image.height(), 0);
            for (x, y, color) in image.pixels() {
                if color.alpha() == 0 {
                    continue;
                }
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
            let (mid_x, mid_y) = ((min_x + max_x) / 2, (min_y + max_y) / 2);
            let side_len = (max_x - min_x).max(max_y - min_y) + 2; // leave some space
            let (corner_x, corner_y) = (
                mid_x.saturating_sub(side_len / 2),
                mid_y.saturating_sub(side_len / 2),
            );
            image.crop_imm(corner_x, corner_y, side_len, side_len)
        };

        let client = self.reqwest_client.clone();
        let links = pkmn.get_sprite_links();
        tokio::spawn(async move {
            let mut tasks = JoinSet::new();

            for (variant_idx, link) in links {
                let client = client.clone();
                tasks.spawn(async move {
                    let image_result: Result<(usize, DynamicImage)> = async {
                        let image_bytes = client.get(link).send().await?.bytes().await?;
                        let image = image::load_from_memory(&image_bytes)?;
                        Ok((variant_idx, trim_image(&image)))
                    }
                    .await;
                    match image_result {
                        Ok((variant_idx, image)) => SpriteFetchEvent::Sprite { variant_idx, image },
                        Err(err) => SpriteFetchEvent::Error { err },
                    }
                });
            }

            while let Some(res) = tasks.join_next().await {
                match res {
                    Ok(event) => {
                        let _ = tx.send(event).await;
                    }
                    Err(e) => {
                        let _ = tx.send(SpriteFetchEvent::Error { err: e.into() }).await;
                    }
                }
            }
            let _ = tx.send(SpriteFetchEvent::Finished).await;
        });
    }

    /// Handles the fetched pokemon sprite events from `start_pkmn_sprite_fetch`
    fn handle_pkmn_sprite(&mut self, event: SpriteFetchEvent) {
        match event {
            SpriteFetchEvent::Sprite { variant_idx, image } => {
                let Some(widget) = &mut self.dex_widget else {
                    return; // Probably not reachable.
                };
                widget.pkmn.inject_sprite(variant_idx, image);
            }
            SpriteFetchEvent::Error { err } => {
                self.update_fetch_state(FetchState::Error(err.to_string()))
            }
            SpriteFetchEvent::Finished => *self.fetch_state.lock() = FetchState::Loaded,
        }
        self.should_update = true;
    }

    /// Updates `FetchState` - how the construction of `OfflinePokemon` is going
    fn update_fetch_state(&mut self, state: FetchState) {
        *self.fetch_state.lock() = state;
    }

    fn render(&self, frame: &mut Frame) {
        let block = Block::new().title_bottom(format!("{:?}", self.fetch_state.lock()));
        let area_without_status_bar = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        if let Some(dex_widget) = &self.dex_widget {
            frame.render_widget(dex_widget, area_without_status_bar);
        }
        if self.input_widget.input_mode == InputMode::Editing {
            frame.render_widget(&self.input_widget, area_without_status_bar); // Overlaid on top
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum FetchState {
    #[default]
    Unloaded,
    LoadingData,
    LoadingSprite,
    Loaded,
    Error(String),
}

enum SpriteFetchEvent {
    Sprite {
        variant_idx: usize,
        image: DynamicImage,
    },
    Error {
        err: color_eyre::eyre::Report,
    },
    Finished,
}
