use ratatui::{
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Tabs, Widget},
};

use crate::{model::ModelVersionGroup, projector::Section, widgets::WidgetExt};

pub(crate) struct MoveGenSelectorWidget {
    generations: Vec<ModelVersionGroup>,
    selected_gen: usize,
    focused: bool,
}

impl WidgetExt<(Vec<ModelVersionGroup>, usize, Section)> for MoveGenSelectorWidget {
    fn new((generations, selected_gen, section): (Vec<ModelVersionGroup>, usize, Section)) -> Self {
        Self {
            generations,
            selected_gen,
            focused: section == Section::MoveGenSelect,
        }
    }
}

impl Widget for MoveGenSelectorWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let highlight_style = if self.focused {
            Style::default().bold().underlined()
        } else {
            Style::default().bold()
        };

        let titles = self.generations.iter().enumerate().map(|(i, generation)| {
            let mut abbr = generation.abbreviation().to_string();
            if i == self.selected_gen {
                abbr = abbr.to_uppercase();
            }
            abbr
        });

        let tabs = Tabs::new(titles)
            .style(Color::Magenta)
            .highlight_style(highlight_style)
            .select(self.selected_gen)
            .divider(Span::styled(symbols::DOT, Color::DarkGray))
            .padding(" ", " ");

        tabs.render(area, buf);
    }
}
