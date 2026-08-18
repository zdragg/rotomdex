use ratatui::{text::Line, widgets::Widget};

use ratatui::prelude::*;

pub struct StatusBarWidget;

impl Widget for StatusBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::raw("← / → to select variant ・ Type anything to search ・ Esc / Ctrl-C to quit")
            .style(Color::DarkGray)
            .centered()
            .render(area, buf);
    }
}
