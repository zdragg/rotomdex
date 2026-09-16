#![forbid(unsafe_code)]

use std::{
    fs, io,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};

use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use crossterm::event::{Event, EventStream, KeyCode};
use etcetera::{AppStrategy, AppStrategyArgs};
use gix::remote::fetch::Shallow;
use ratatui::prelude::Widget;
use rotomdex_core::{ActionResult, DexKeyCode, DexKeyModifiers, RotomDexCore};
use tokio_stream::StreamExt;

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

    let resource_git_repo_dir = strategy.in_data_dir("resource");

    if cli.download {
        download_repo(&resource_git_repo_dir)?;
        return Ok(());
    }

    fs::create_dir_all(strategy.data_dir())?;
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(strategy.in_data_dir("app.log"))?;
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(Mutex::new(log))
        .try_init()
        .map_err(|err| eyre!(err))?;

    let config = if cli.offline {
        PathConfig::Offline(resource_git_repo_dir)
    } else {
        PathConfig::Cache(strategy.cache_dir())
    };

    let result = run(config).await;

    ratatui::restore();
    result
}

enum PathConfig {
    Cache(PathBuf),
    Offline(PathBuf),
}

const URL: &str = "https://github.com/zdragg/rotomdex-data.git";

fn download_repo(repo_path: &Path) -> Result<()> {
    let should_interrupt = AtomicBool::new(false);
    let root = prodash::tree::Root::new();
    let mut progress = root.add_child("downloading offline resources");

    let renderer = prodash::render::line::render(
        io::stdout(),
        Arc::downgrade(&root),
        prodash::render::line::Options {
            throughput: true,
            ..Default::default()
        }
        .auto_configure(prodash::render::line::StreamKind::Stderr),
    );

    let (mut checkout, _outcome) = gix::prepare_clone(URL, repo_path)?
        .with_shallow(Shallow::DepthAtRemote(NonZeroU32::new(1).unwrap()))
        .fetch_then_checkout(&mut progress, &should_interrupt)?;

    checkout.main_worktree(&mut progress, &should_interrupt)?;

    renderer.shutdown_and_wait();

    Ok(())
}

const FRAMES_PER_SECOND: f32 = 33.3;
async fn run(config: PathConfig) -> Result<()> {
    let mut core = match config {
        PathConfig::Cache(cache_dir) => RotomDexCore::new_cached(cache_dir),
        PathConfig::Offline(resource_path) => RotomDexCore::new_offline(resource_path),
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
    Some((DexKeyModifiers::from_bits_retain(event.modifiers.bits()), key_code))
}
