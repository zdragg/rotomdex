use ratatui::{
    layout::{Constraint, Layout},
    text::Line,
    widgets::Widget,
};

use ratatui::prelude::*;

use crate::offline::{FetchProgress, Progress};

pub struct StatusBarWidget {
    pub progress: FetchProgress,
}

impl Widget for StatusBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [variant_area, instructions_area, sprite_area] = Layout::horizontal([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(area);

        if let Progress::Determinate { completed, total } = self.progress.variants {
            if completed == total {
                Line::from("✓")
            } else {
                Line::from(format!("{completed}/{total}"))
            }
            .style(Color::Red)
            .render(variant_area, buf);
        };

        if let Progress::Determinate { completed, total } = self.progress.sprites {
            if completed == total {
                Line::from("✓")
            } else {
                Line::from(format!("{completed}/{total}"))
            }
            .style(Color::Blue)
            .right_aligned()
            .render(sprite_area, buf);
        };

        Line::from("Type to search - ← / → to select variant - Esc / Ctrl-C to quit")
            .style(Color::DarkGray)
            .centered()
            .render(instructions_area, buf);
    }
}
