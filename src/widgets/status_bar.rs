use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    symbols,
    text::Line,
    widgets::{LineGauge, Widget},
};

use crate::offline::{FetchProgress, Progress};

pub struct StatusBarWidget {
    pub progress: FetchProgress,
}

impl Widget for &StatusBarWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if !self.progress.species_loaded {
            Line::from("Loading pokemon...").render(area, buf);
            return;
        }
        let [variant_area, _space, sprite_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        if let Progress::Determinate { completed, total } = self.progress.variants {
            LineGauge::default()
                .label(format!("variants: {}/{}", completed, total))
                .filled_style(Style::new().red().bold())
                .filled_symbol(symbols::line::THICK_HORIZONTAL)
                .ratio(completed as f64 / total as f64)
                .render(variant_area, buf);
        };

        if let Progress::Determinate { completed, total } = self.progress.sprites {
            LineGauge::default()
                .label(format!("sprites: {}/{}", completed, total))
                .filled_style(Style::new().blue().bold())
                .filled_symbol(symbols::line::THICK_HORIZONTAL)
                .ratio(completed as f64 / total as f64)
                .render(sprite_area, buf);
        };
    }
}
