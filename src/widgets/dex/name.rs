use ratatui::prelude::*;
use tui_big_text::{BigText, PixelSize};

use crate::offline::OfflineVariant;

pub struct NameWidget<'a> {
    pub variant: &'a OfflineVariant,
}

impl Widget for &NameWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().blue())
            .lines(vec![self.variant.pkmn.name.clone().into()])
            .build()
            .render(area, buf);
    }
}
