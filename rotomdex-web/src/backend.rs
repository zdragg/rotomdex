// hacky override to fix size problems

use std::io::{Error as IoError, Result as IoResult};

use ratzilla::{
    CellSized, DomBackend, WebEventHandler,
    error::Error,
    event::{KeyEvent, MouseEvent},
    ratatui::{
        backend::{Backend, ClearType, WindowSize},
        buffer::Cell,
        layout::{Position, Size},
    },
    web_sys::window,
};

pub struct MeasuredDomBackend(DomBackend);

impl MeasuredDomBackend {
    pub fn new() -> Result<Self, Error> {
        DomBackend::new().map(Self)
    }

    fn measured_size(&self) -> IoResult<Size> {
        let body = window()
            .and_then(|window| window.document())
            .and_then(|document| document.body())
            .ok_or_else(|| IoError::other("unable to measure the document body"))?;
        let bounds = body.get_bounding_client_rect();
        let (cell_width, cell_height) = self.0.cell_size_css_px();

        Ok(Size::new(
            ((bounds.width() / f64::from(cell_width)) as u16).saturating_sub(1),
            ((bounds.height() / f64::from(cell_height)) as u16).saturating_sub(1),
        ))
    }
}

impl Backend for MeasuredDomBackend {
    type Error = IoError;

    fn draw<'a, I>(&mut self, content: I) -> IoResult<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.0.draw(content)
    }

    fn append_lines(&mut self, n: u16) -> IoResult<()> {
        self.0.append_lines(n)
    }

    fn hide_cursor(&mut self) -> IoResult<()> {
        self.0.hide_cursor()
    }

    fn show_cursor(&mut self) -> IoResult<()> {
        self.0.show_cursor()
    }

    fn get_cursor_position(&mut self) -> IoResult<Position> {
        self.0.get_cursor_position()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> IoResult<()> {
        self.0.set_cursor_position(position)
    }

    fn clear(&mut self) -> IoResult<()> {
        self.0.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> IoResult<()> {
        self.0.clear_region(clear_type)
    }

    fn size(&self) -> IoResult<Size> {
        self.measured_size()
    }

    fn window_size(&mut self) -> IoResult<WindowSize> {
        let mut size = self.0.window_size()?;
        size.columns_rows = self.measured_size()?;
        Ok(size)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.0.flush()
    }
}

impl WebEventHandler for MeasuredDomBackend {
    fn on_mouse_event<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(MouseEvent) + 'static,
    {
        self.0.on_mouse_event(callback)
    }

    fn clear_mouse_events(&mut self) {
        self.0.clear_mouse_events();
    }

    fn on_key_event<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(KeyEvent) + 'static,
    {
        self.0.on_key_event(callback)
    }

    fn clear_key_events(&mut self) {
        self.0.clear_key_events();
    }
}
