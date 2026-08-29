use color_eyre::eyre::{Result, eyre};
use futures::FutureExt;
use ratzilla::{
    SelectionMode, WebGl2Backend, WebRenderer,
    backend::webgl2::{FontAtlasConfig, FontAtlasData, WebGl2BackendOptions},
    event::KeyCode,
    ratatui::{Terminal, prelude::Widget, style::Color},
    web_sys::{
        KeyboardEvent,
        wasm_bindgen::{JsCast, closure::Closure},
        window,
    },
};
use rotomdex_core::{Action, ActionHandler, RotomDexCore, SettingsBuilder};
use std::{cell::RefCell, rc::Rc};
use tracing_web::MakeWebConsoleWriter;

fn main() -> Result<()> {
    setup_logs()?;
    run()
}

fn setup_logs() -> Result<()> {
    tracing_subscriber::fmt()
        .fmt_fields(tracing_subscriber::fmt::format::PrettyFields::new())
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(false)
        .without_time()
        .with_writer(MakeWebConsoleWriter::new())
        .try_init()
        .map_err(|err| eyre!(err))?;

    Ok(())
}

fn run() -> Result<()> {
    let core = Rc::new(RefCell::new(RotomDexCore::new(
        SettingsBuilder::default().build()?,
        format!(
            "← / → to select variant {} Type anything to search",
            ratzilla::ratatui::symbols::DOT
        ),
    )));
    let font_atlas = FontAtlasData::from_binary(include_bytes!("../assets/jetbrains-mono-30.atlas"))?;
    let backend = WebGl2Backend::new_with_options(
        WebGl2BackendOptions::new()
            .font_atlas_config(FontAtlasConfig::Static(font_atlas))
            .canvas_padding_color(Color::Black)
            .enable_mouse_selection_with_mode(SelectionMode::Block),
    )?;
    let terminal = Terminal::new(backend)?;

    install_key_handler(core.clone())?;

    terminal.draw_web({
        let core = core.clone();
        move |f| {
            let mut core = core.borrow_mut();
            let _ = core.poll_pkmn().now_or_never();
            core.render(f.area(), f.buffer_mut());
        }
    });

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
            KeyCode::Right => Action::Right,
            KeyCode::Up => Action::Up,
            KeyCode::Down => Action::Down,
            KeyCode::Left => Action::Left,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char(ch) => Action::Input(ch),
            KeyCode::Esc => Action::Escape,
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
