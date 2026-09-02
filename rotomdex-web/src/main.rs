use color_eyre::eyre::{Result, eyre};
use futures::FutureExt;
use ratzilla::{
    SelectionMode, WebGl2Backend, WebRenderer,
    backend::webgl2::{FontAtlasConfig, FontAtlasData, WebGl2BackendOptions},
    event::KeyCode,
    ratatui::{Terminal, prelude::Widget, style::Color},
    web_sys::{
        HtmlCanvasElement, KeyboardEvent,
        wasm_bindgen::{JsCast, closure::Closure},
        window,
    },
};
use rotomdex_core::{Action, RotomDexCore};
use std::{cell::RefCell, rc::Rc};
use tracing_web::MakeWebConsoleWriter;

const MIN_TERMINAL_COLS: u32 = 81;
const MIN_TERMINAL_ROWS: u32 = 25;

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
    let core = Rc::new(RefCell::new(RotomDexCore::new(false)));
    let font_atlas = FontAtlasData::from_binary(include_bytes!("../assets/jetbrains-mono-30.atlas"))?;
    let padded_cell_size = font_atlas.cell_size();
    let cell_size = (
        padded_cell_size.width - 2 * FontAtlasData::PADDING,
        padded_cell_size.height - 2 * FontAtlasData::PADDING,
    );
    let window = window().ok_or_else(|| eyre!("unable to access the browser window"))?;
    let canvas_size = terminal_canvas_size(cell_size, window.device_pixel_ratio(), viewport_size(&window)?);
    let backend = WebGl2Backend::new_with_options(
        WebGl2BackendOptions::new()
            .size(canvas_size)
            .font_atlas_config(FontAtlasConfig::Static(font_atlas))
            .canvas_padding_color(Color::Black)
            .enable_mouse_selection_with_mode(SelectionMode::Block),
    )?;
    let terminal = Terminal::new(backend)?;

    install_canvas_fit_handler(cell_size)?;
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

fn terminal_canvas_size(cell_size: (i32, i32), device_pixel_ratio: f64, viewport_size: (f64, f64)) -> (u32, u32) {
    let device_pixel_ratio = device_pixel_ratio.max(f64::EPSILON);
    let atlas_scale = if device_pixel_ratio <= 0.5 {
        0.5
    } else {
        device_pixel_ratio.round().max(1.0)
    };
    let cell_width = f64::from(cell_size.0) * atlas_scale / device_pixel_ratio;
    let cell_height = f64::from(cell_size.1) * atlas_scale / device_pixel_ratio;
    let display_scale = (viewport_size.0 / (cell_width * f64::from(MIN_TERMINAL_COLS)))
        .min(viewport_size.1 / (cell_height * f64::from(MIN_TERMINAL_ROWS)));

    (
        (viewport_size.0 / display_scale).ceil() as u32,
        (viewport_size.1 / display_scale).ceil() as u32,
    )
}

fn viewport_size(window: &ratzilla::web_sys::Window) -> Result<(f64, f64)> {
    let width = window
        .inner_width()
        .map_err(|error| eyre!("unable to read viewport width: {error:?}"))?
        .as_f64()
        .ok_or_else(|| eyre!("viewport width is not numeric"))?;
    let height = window
        .inner_height()
        .map_err(|error| eyre!("unable to read viewport height: {error:?}"))?
        .as_f64()
        .ok_or_else(|| eyre!("viewport height is not numeric"))?;

    Ok((width, height))
}

fn install_canvas_fit_handler(cell_size: (i32, i32)) -> Result<()> {
    let window = window().ok_or_else(|| eyre!("unable to access the browser window"))?;
    let canvas = window
        .document()
        .ok_or_else(|| eyre!("unable to access the browser document"))?
        .query_selector("canvas")
        .map_err(|error| eyre!("unable to query the terminal canvas: {error:?}"))?
        .ok_or_else(|| eyre!("unable to find the terminal canvas"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| eyre!("terminal canvas has an unexpected element type"))?;

    fit_canvas_to_viewport(&canvas, cell_size)?;

    let callback = Closure::<dyn FnMut()>::new(move || {
        if let Err(error) = fit_canvas_to_viewport(&canvas, cell_size) {
            tracing::error!(?error, "unable to fit terminal canvas to viewport");
        }
    });
    window
        .add_event_listener_with_callback("resize", callback.as_ref().unchecked_ref())
        .map_err(|error| eyre!("unable to install canvas resize handler: {error:?}"))?;
    callback.forget();

    Ok(())
}

fn fit_canvas_to_viewport(canvas: &HtmlCanvasElement, cell_size: (i32, i32)) -> Result<()> {
    let window = window().ok_or_else(|| eyre!("unable to access the browser window"))?;
    let viewport_size = viewport_size(&window)?;
    let (canvas_width, canvas_height) = terminal_canvas_size(cell_size, window.device_pixel_ratio(), viewport_size);
    let display_scale_x = viewport_size.0 / f64::from(canvas_width);
    let display_scale_y = viewport_size.1 / f64::from(canvas_height);
    let style = canvas.style();

    // Match the viewport with the smallest grid above the minimum dimensions. The two scales
    // differ only by integer-pixel rounding, avoiding letterboxing without visible distortion.
    style
        .set_property("width", &format!("{canvas_width}px"))
        .map_err(|error| eyre!("unable to set terminal canvas width: {error:?}"))?;
    style
        .set_property("height", &format!("{canvas_height}px"))
        .map_err(|error| eyre!("unable to set terminal canvas height: {error:?}"))?;
    style
        .set_property("transform", &format!("scale({display_scale_x}, {display_scale_y})"))
        .map_err(|error| eyre!("unable to scale terminal canvas: {error:?}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_canvas_size_uses_smallest_grid_above_minimum() {
        let cell_size = (18, 40);

        for viewport_size in [(1920.0, 1080.0), (1440.0, 900.0), (390.0, 844.0), (3440.0, 1440.0)] {
            for device_pixel_ratio in [1.0_f64, 1.25, 1.5, 2.0, 3.0] {
                let (width, height) = terminal_canvas_size(cell_size, device_pixel_ratio, viewport_size);
                let atlas_scale = device_pixel_ratio.round().max(1.0);
                let physical_cell_width = f64::from(cell_size.0) * atlas_scale;
                let physical_cell_height = f64::from(cell_size.1) * atlas_scale;
                let cols = (f64::from(width) * device_pixel_ratio / physical_cell_width).floor() as u32;
                let rows = (f64::from(height) * device_pixel_ratio / physical_cell_height).floor() as u32;

                assert!(cols >= MIN_TERMINAL_COLS);
                assert!(rows >= MIN_TERMINAL_ROWS);
                assert!(cols == MIN_TERMINAL_COLS || rows == MIN_TERMINAL_ROWS);
            }
        }
    }
}

fn install_key_handler(core: Rc<RefCell<RotomDexCore>>) -> Result<()> {
    let window = window().ok_or_else(|| eyre!("unable to access the browser window"))?;
    let mut caps_lock_state = None;
    let callback = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
        if event.ctrl_key() || event.alt_key() || event.meta_key() {
            return;
        }

        let action = if event.key() == "CapsLock" {
            let state = event.get_modifier_state("CapsLock");

            // macOS may report enabling Caps Lock as keydown and disabling it as keyup,
            // while other platforms emit both events for each press.
            if caps_lock_state.replace(state) == Some(state) {
                return;
            }

            Action::CapsLock
        } else {
            if event.type_() != "keydown" {
                return;
            }

            match KeyCode::from(event.clone()) {
                KeyCode::Enter => Action::Enter,
                KeyCode::Right => Action::Right,
                KeyCode::Up => Action::Up,
                KeyCode::Down => Action::Down,
                KeyCode::Left => Action::Left,
                KeyCode::Backspace => Action::Backspace,
                KeyCode::Char(ch) => Action::Input(ch),
                KeyCode::Esc => Action::Escape,
                _ => return,
            }
        };

        event.prevent_default();
        core.borrow_mut().handle_action(action);
    });

    for event_type in ["keydown", "keyup"] {
        window
            .add_event_listener_with_callback(event_type, callback.as_ref().unchecked_ref())
            .map_err(|error| eyre!("unable to install {event_type} handler: {error:?}"))?;
    }
    callback.forget();

    Ok(())
}
