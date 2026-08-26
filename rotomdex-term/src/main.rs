use std::{fs, path::PathBuf, time::Duration};

use color_eyre::eyre::{Result, eyre};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use etcetera::{AppStrategy, AppStrategyArgs};
use ratatui::prelude::Widget;
use rotomdex_core::{Action, ActionHandler, RotomDexCore};
use tokio_stream::StreamExt;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: "dev".to_string(),
        author: "zerodrag".to_string(),
        app_name: "rotomdex".to_string(),
    })?;

    let _log_guard = setup_logs(strategy.data_dir())?;
    tracing::info!("──────── session started ────────");

    let result = run(strategy.cache_dir()).await;

    ratatui::restore();
    result
}

fn setup_logs(log_dir: PathBuf) -> Result<WorkerGuard> {
    fs::create_dir_all(&log_dir)?;

    let appender = tracing_appender::rolling::never(log_dir, "app.log");

    let (writer, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .fmt_fields(tracing_subscriber::fmt::format::PrettyFields::new())
        .with_ansi(false)
        .with_env_filter(filter)
        .with_writer(writer)
        .with_target(false)
        .with_timer(ChronoLocal::new("[%H:%M:%S%.3f]".to_owned()))
        .try_init()
        .map_err(|err| eyre!(err))?;

    Ok(guard)
}

const FRAMES_PER_SECOND: f32 = 30.0;
async fn run(cache_dir: PathBuf) -> Result<()> {
    let mut core = RotomDexCore::new_with_cache(
        cache_dir,
        format!(
            "← / → to select variant {} Type anything to search {} Esc / Ctrl-C to quit",
            ratatui::symbols::DOT,
            ratatui::symbols::DOT
        ),
    );
    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND));
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
            _ = core.poll_pkmn() => {}
            _ = interval.tick() => {}
        }

        terminal.draw(|frame| core.render(frame.area(), frame.buffer_mut()))?;
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
            (_, KeyCode::Down) => AppEvent::Action(Action::Down),
            (_, KeyCode::Up) => AppEvent::Action(Action::Up),
            (_, KeyCode::Right) => AppEvent::Action(Action::Right),
            (_, KeyCode::Left) => AppEvent::Action(Action::Left),
            (_, KeyCode::Backspace) => AppEvent::Action(Action::Backspace),
            (_, KeyCode::Char(ch)) => AppEvent::Action(Action::Input(ch)),
            _ => AppEvent::Action(Action::Ignore),
        }
    } else {
        AppEvent::Action(Action::Ignore)
    }
}
