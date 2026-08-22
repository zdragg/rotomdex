use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use etcetera::{AppStrategy, AppStrategyArgs};
use rotomdex_core::{Action, RotomDexCore};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: "dev".to_string(),
        author: "zerodrag".to_string(),
        app_name: "rotomdex".to_string(),
    })?;
    color_eyre::install()?;
    setup_logs(strategy.data_dir())?;
    let result = run(strategy.cache_dir()).await;
    ratatui::restore();
    result
}

fn setup_logs(mut log_dir: PathBuf) -> Result<()> {
    fs::create_dir_all(&log_dir)?;
    log_dir.push("app.log");
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {:<5} {}",
                chrono::Local::now().format("%H:%M:%S%.3f"),
                record.level(),
                message,
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_dir)?)
        .apply()?;
    log::info!("──────── session started ────────");
    Ok(())
}

async fn run(cache_dir: PathBuf) -> Result<()> {
    let mut core = RotomDexCore::new(
        cache_dir,
        "← / → to select variant ・ Type anything to search ・ Esc / Ctrl-C to quit",
    );
    let mut terminal = ratatui::init();
    let mut events = EventStream::new();
    loop {
        tokio::select! {
            Some(Ok(event)) = events.next() => {
                match map_event(event) {
                    AppEvent::Quit => break,
                    AppEvent::Action(action) => core.handle_action(action),
                }
            }
            () = core.poll_pkmn() => {}
        }

        terminal.draw(|frame| core.render(frame))?;
    }
    Ok(())
}

enum AppEvent {
    Action(Action),
    Quit,
}

fn map_event(event: Event) -> AppEvent {
    if let Event::Key(key) = event {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => AppEvent::Quit,
            (_, KeyCode::Enter) => AppEvent::Action(Action::Enter),
            (_, KeyCode::Right) => AppEvent::Action(Action::RightArrow),
            (_, KeyCode::Left) => AppEvent::Action(Action::LeftArrow),
            (_, KeyCode::Backspace) => AppEvent::Action(Action::Backspace),
            (_, KeyCode::Char(ch)) => AppEvent::Action(Action::Input(ch)),
            _ => AppEvent::Action(Action::Ignore),
        }
    } else {
        AppEvent::Action(Action::Ignore)
    }
}
