#![forbid(unsafe_code)]

mod client;
mod session_rw;
mod sync;

use std::{fs, path::PathBuf, sync::Mutex, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use crossterm::event::{Event, EventStream, KeyCode};
use etcetera::{AppStrategy, AppStrategyArgs};
use ratatui::prelude::Widget;
use rotomdex_api::Client;
use rotomdex_core::{DexKeyCode, DexKeyModifiers, MaybeExit, RotomDexCore};
use tokio_stream::StreamExt;

use crate::client::{offline::OfflineClient, online::CachedClient};

#[derive(Debug, Parser)]
struct Cli {
    /// Use the locally downloaded PokéAPI data and sprites.
    #[arg(long, short)]
    offline: bool,

    /// Download or update the offline PokéAPI data and sprites.
    #[arg(long, short)]
    download: bool,

    /// Folder to store resources portable in
    #[arg(long, short)]
    portable: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    let (data, cache) = match cli.portable {
        Some(dir) => (dir.join("data"), dir.join("cache")),
        None => {
            let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
                top_level_domain: "dev".to_string(),
                author: "zerodrag".to_string(),
                app_name: "rotomdex".to_string(),
            })?;
            (strategy.data_dir(), strategy.cache_dir())
        }
    };

    fs::create_dir_all(&data)?;
    fs::create_dir_all(&cache)?;

    let resource_git_repo_dir = data.join("resource");

    if cli.download {
        sync::download_repo(&resource_git_repo_dir)?;
        return Ok(());
    }

    let config = if cli.offline {
        PathConfig::Offline(resource_git_repo_dir)
    } else {
        PathConfig::Cache(cache.join("http-cache.redb"))
    };

    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(data.join("app.log"))?;
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(Mutex::new(log))
        .try_init()
        .map_err(|err| eyre!(err))?;

    let result = run(config).await;

    ratatui::restore();
    result
}

enum PathConfig {
    /// Path to `redb` file
    Cache(PathBuf),
    /// Path to repository containing `zdragg/rotomdex-data` files
    Offline(PathBuf),
}

const FRAMES_PER_SECOND: f32 = 33.3;
async fn run(config: PathConfig) -> Result<()> {
    let client = match config {
        PathConfig::Cache(cache_dir) => Client::new(CachedClient::new(cache_dir)?),
        PathConfig::Offline(resource_path) => Client::new(OfflineClient::new(resource_path)),
    };
    let session_rw = crate::session_rw::SessionRw::new();

    let mut core = RotomDexCore::new(client, session_rw);

    let mut interval = tokio::time::interval(Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND));
    let mut terminal = ratatui::init();
    let mut events = EventStream::new();
    loop {
        tokio::select! {
            Some(Ok(event)) = events.next() => {
                if let Some((modifiers, key_code)) = map_event(event)
                    && matches!(core.handle_key(modifiers, key_code), MaybeExit::Exit)
                {
                    break;
                }
            }
            _ = core.poll() => {}
            _ = interval.tick(), if core.settings.animation_mode => {}
        }

        terminal.draw(|frame| core.render(frame.area(), frame.buffer_mut()))?;
    }
    Ok(())
}

fn map_event(event: Event) -> Option<(DexKeyModifiers, DexKeyCode)> {
    let Event::Key(event) = event else {
        return None;
    };

    if event.is_release() {
        return None;
    }

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
    Some((
        DexKeyModifiers::from_bits_retain(event.modifiers.bits()),
        key_code,
    ))
}
