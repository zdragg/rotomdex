use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};
use tui_big_text::BigText;

pub struct TitleWidget<'a> {
    text: &'a str,
}

impl<'a> TitleWidget<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Widget for &TitleWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = BigText::builder()
            .pixel_size(tui_big_text::PixelSize::Full)
            .style(Style::new())
            .lines(vec![self.text.into()])
            .centered()
            .build();
        text.render(area, buf);
    }
}
