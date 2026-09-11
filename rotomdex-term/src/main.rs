#![forbid(unsafe_code)]

use std::{fs, path::PathBuf, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use crossterm::event::{Event, EventStream, KeyCode};
use etcetera::{AppStrategy, AppStrategyArgs};
use ratatui::prelude::Widget;
use rotomdex_core::{ActionResult, DexKeyCode, DexKeyModifiers, RotomDexCore};
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

const FRAMES_PER_SECOND: f32 = 33.3;
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
                if let Some((modifiers, key_code)) = map_event(event)
                    && matches!(core.handle_key(modifiers, key_code), ActionResult::Exit)
                {
                    break;
                }
            }
            _ = core.poll_pkmn() => {}
            _ = interval.tick(), if core.needs_continuous_render() => {}
        }

        terminal.draw(|frame| core.render(frame.area(), frame.buffer_mut()))?;
    }
    Ok(())
}

fn map_event(event: Event) -> Option<(DexKeyModifiers, DexKeyCode)> {
    let Event::Key(event) = event else {
        return None;
    };

    let key_code = match event.code {
        KeyCode::Char(ch) => DexKeyCode::Char(ch),
        KeyCode::Backspace => DexKeyCode::Backspace,
        KeyCode::Enter => DexKeyCode::Enter,
        KeyCode::Right => DexKeyCode::Right,
        KeyCode::Down => DexKeyCode::Down,
        KeyCode::Left => DexKeyCode::Left,
        KeyCode::Up => DexKeyCode::Up,
        KeyCode::Esc => DexKeyCode::Escape,
        KeyCode::CapsLock => DexKeyCode::CapsLock,
        _ => return None,
    };
    Some((DexKeyModifiers::from_bits_retain(event.modifiers.bits()), key_code))
}
