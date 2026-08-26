mod dex;
pub(crate) use dex::*;
mod input;
pub(crate) use input::*;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

// I think this allows rendering from projectview -> widget directly

trait WidgetExt<T>: Widget {
    fn new(args: T) -> Self;
}

trait OptionWidgetExt<T> {
    fn render_option<W>(self, area: Rect, buf: &mut Buffer)
    where
        W: WidgetExt<T>;
}

impl<T> OptionWidgetExt<T> for Option<T> {
    fn render_option<W>(self, area: Rect, buf: &mut Buffer)
    where
        W: WidgetExt<T>,
    {
        self.map(W::new).render(area, buf);
    }
}
