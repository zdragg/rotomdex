use std::{cell::RefCell, rc::Rc};

use color_eyre::eyre::{Result, eyre};
use futures::FutureExt;
use log::Level;
use ratzilla::{
    WebRenderer,
    event::KeyCode,
    ratatui::Terminal,
    web_sys::{
        KeyboardEvent,
        wasm_bindgen::{JsCast, closure::Closure},
        window,
    },
};
use rotomdex_core::{Action, RotomDexCore};
use wasm_bindgen_futures::{JsFuture, spawn_local};

use crate::backend::MeasuredDomBackend;

mod backend;

fn main() -> Result<()> {
    console_log::init_with_level(Level::Info)?;

    spawn_local(async {
        if let Err(error) = run().await {
            log::error!("failed to initialize RotomDex: {error:?}");
        }
    });

    Ok(())
}

async fn run() -> Result<()> {
    load_font().await?;

    let core = Rc::new(RefCell::new(RotomDexCore::new(
        "← / → to select variant ・ Type anything to search",
    )));
    let backend = MeasuredDomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    install_key_handler(core.clone())?;

    terminal.draw_web({
        let core = core.clone();
        move |f| {
            let mut core = core.borrow_mut();
            let _ = core.ping().now_or_never();
            core.render(f);
        }
    });

    Ok(())
}

async fn load_font() -> Result<()> {
    let document = window()
        .and_then(|window| window.document())
        .ok_or_else(|| eyre!("unable to access the browser document"))?;

    JsFuture::from(document.fonts().load_with_text("16px 'Fira Code'", "█"))
        .await
        .map_err(|error| eyre!("unable to load Fira Code: {error:?}"))?;

    Ok(())
}

fn install_key_handler(core: Rc<RefCell<RotomDexCore>>) -> Result<()> {
    let window = window().ok_or_else(|| eyre!("unable to access the browser window"))?;
    let callback = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
        if event.ctrl_key() || event.alt_key() || event.meta_key() {
            return;
        }

        let action = match KeyCode::from(event.clone()) {
            KeyCode::Enter => Action::Enter,
            KeyCode::Right => Action::RightArrow,
            KeyCode::Left => Action::LeftArrow,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char(ch) => Action::Input(ch),
            _ => return,
        };

        event.prevent_default();
        core.borrow_mut().handle_action(action);
    });

    window
        .add_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref())
        .map_err(|error| eyre!("unable to install keyboard handler: {error:?}"))?;
    callback.forget();

    Ok(())
}
