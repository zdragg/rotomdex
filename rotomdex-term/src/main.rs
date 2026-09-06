use std::{fs, path::PathBuf, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use etcetera::{AppStrategy, AppStrategyArgs};
use ratatui::prelude::Widget;
use rotomdex_core::{Action, RotomDexCore};
use tokio_stream::StreamExt;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;

mod resources;

#[derive(Debug, Parser)]
struct Cli {
    /// Use the locally downloaded PokéAPI data and sprites.
    #[arg(long)]
    offline: bool,

    /// Download or update the offline PokéAPI data and sprites.
    #[arg(long)]
    download: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: "dev".to_string(),
        author: "zerodrag".to_string(),
        app_name: "rotomdex".to_string(),
    })?;

    let data_dir = strategy.data_dir();
    let resources = resources::ResourcePaths::new(data_dir);
    if cli.download {
        resources.download()?;
        return Ok(());
    }

    let _log_guard = setup_logs(strategy.data_dir())?;
    tracing::info!("──────── session started ────────");

    if cli.offline {
        resources.validate()?;
    }

    let result = run(strategy.cache_dir(), cli.offline.then_some(resources)).await;

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
async fn run(cache_dir: PathBuf, resources: Option<resources::ResourcePaths>) -> Result<()> {
    let mut core = if let Some(resources) = resources {
        RotomDexCore::new_offline(resources.resource_path())
    } else {
        RotomDexCore::new_cached(cache_dir)
    };
    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND));
    let mut terminal = ratatui::init();
    let mut events = EventStream::new();
    loop {
        tokio::select! {
            Some(Ok(event)) = events.next() => {
                if let Some(event) =  map_event(event) {
                    match event {
                        AppEvent::Quit => break,
                        AppEvent::Action(action) => core.handle_action(action),
                    }
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
    Quit,
    Action(Action),
}

fn map_event(event: Event) -> Option<AppEvent> {
    let action = if let Event::Key(key) = event {
        if matches!((key.modifiers, key.code), (KeyModifiers::CONTROL, KeyCode::Char('c'))) {
            return Some(AppEvent::Quit);
        }
        match key.code {
            KeyCode::Esc => Action::Escape,
            KeyCode::Enter => Action::Enter,
            KeyCode::Down => Action::Down,
            KeyCode::Up => Action::Up,
            KeyCode::Right => Action::Right,
            KeyCode::Left => Action::Left,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char(ch) => Action::Input(ch),
            KeyCode::CapsLock => Action::CapsLock,
            _ => return None,
        }
    } else {
        return None;
    };
    Some(AppEvent::Action(action))
}
