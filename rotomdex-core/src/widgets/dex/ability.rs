use ratatui::{
    layout::{HorizontalAlignment, Offset},
    style::{Color, Style},
    symbols,
    widgets::{Block, Paragraph, Tabs, Widget, Wrap},
};

use crate::model::{ModelAbilities, ModelVariant};

pub(crate) struct AbilitiesWidget<'a> {
    abilities: &'a ModelAbilities,
    selected_tab: usize,
}

impl<'a> AbilitiesWidget<'a> {
    pub(crate) fn new(variant: &'a ModelVariant, selected_tab: usize) -> Self {
        Self {
            abilities: variant.abilities(),
            selected_tab,
        }
    }
}

impl<'a> Widget for AbilitiesWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut desc = "...";
        let names = self
            .abilities
            .iter()
            .enumerate()
            .map(|(i, (slot, ability))| match ability.as_loaded() {
                Some(ability) => {
                    let mut result = format!("{} {}", prefix(slot), ability.name());
                    if self.selected_tab == i {
                        desc = ability.desc();
                        result = result.to_uppercase()
                    }
                    result
                }
                None => "loading".to_string(),
            });

        let tabs = Tabs::new(names)
            .style(Color::White)
            .highlight_style(Style::default().magenta().bold())
            .select(self.selected_tab)
            .divider(symbols::DOT)
            .padding(" ", " ");

        Paragraph::new(desc)
            .alignment(HorizontalAlignment::Center)
            .block(Block::bordered())
            .wrap(Wrap { trim: false })
            .render(area, buf);

        tabs.render(area + Offset::new(1, 0), buf);
    }
}

fn prefix(slot: usize) -> &'static str {
    match slot {
        0 => "1.",
        1 => "2.",
        2 => "hid.",
        _ => panic!("invalid prefix found"),
    }
}
