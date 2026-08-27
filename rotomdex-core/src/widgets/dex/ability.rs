use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Paragraph, Tabs, Widget, Wrap},
};

use crate::{model::ModelAbilities, projector::Section, widgets::WidgetExt};

pub(crate) struct AbilitiesWidget<'a> {
    abilities: &'a ModelAbilities,
    selected_tab: usize,
    focused: bool,
}

impl<'a> WidgetExt<(&'a ModelAbilities, usize, Section)> for AbilitiesWidget<'a> {
    fn new((abilities, selected_tab, section): (&'a ModelAbilities, usize, Section)) -> Self {
        Self {
            abilities,
            selected_tab,
            focused: section == Section::Abilities,
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
                    let mut name = format!("{} {}", prefix(slot), ability.name());
                    if self.selected_tab == i {
                        desc = ability.desc();
                        name = name.to_uppercase()
                    }
                    name
                }
                None => "loading".to_string(),
            });

        let highlight_style = if self.focused {
            Style::default().bold().underlined()
        } else {
            Style::default().bold()
        };

        let tabs = Tabs::new(names)
            .style(Color::Magenta)
            .highlight_style(highlight_style)
            .select(self.selected_tab)
            .divider(Span::styled(symbols::DOT, Color::DarkGray))
            .padding(" ", " ");

        tabs.render(area, buf);

        Paragraph::new(desc)
            .wrap(Wrap { trim: false })
            .style(Color::White)
            .render(
                Rect {
                    x: area.x,
                    y: area.y + 1,
                    width: area.width,
                    height: area.height.saturating_sub(1),
                },
                buf,
            );
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
