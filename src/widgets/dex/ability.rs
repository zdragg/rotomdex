use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Span,
    widgets::{Paragraph, Widget, Wrap},
};

use crate::offline::{AbilitiesLayout, LoadState, OfflineAbilities, OfflineAbility, OfflineVariant};

pub struct AbilitiesWidget<'a> {
    abilities: &'a OfflineAbilities,
}

impl<'a> AbilitiesWidget<'a> {
    pub fn new(variant: &'a OfflineVariant) -> Self {
        Self {
            abilities: variant.abilities(),
        }
    }
}

impl<'a> Widget for AbilitiesWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let lines = match self.abilities.layout() {
            AbilitiesLayout::None => {
                vec![]
            }
            AbilitiesLayout::P { primary } => {
                vec![get_paragraph(1, primary)]
            }
            AbilitiesLayout::PS { primary, secondary } => {
                vec![get_paragraph(1, primary), get_paragraph(2, secondary)]
            }
            AbilitiesLayout::PH { primary, hidden } => {
                vec![get_paragraph(1, primary), get_paragraph(3, hidden)]
            }
            AbilitiesLayout::PSH {
                primary,
                secondary,
                hidden,
            } => {
                vec![
                    get_paragraph(1, primary),
                    get_paragraph(2, secondary),
                    get_paragraph(3, hidden),
                ]
            }
        };
        let rects = Layout::vertical(vec![Constraint::Fill(1); lines.len()]).split(area);

        for (i, line) in lines.into_iter().enumerate() {
            line.render(rects[i], buf);
        }
    }
}

fn get_paragraph(slot: usize, ability: &LoadState<OfflineAbility>) -> Option<Paragraph<'_>> {
    let number = Span::raw(match slot {
        1 => "  1.",
        2 => "  2.",
        3 => "hid.",
        _ => unreachable!(),
    })
    .dark_gray();
    ability.as_loaded().map(|ability| {
        let name = ability.name();
        let desc = ability.desc();
        Paragraph::new(format!("{number} {name}\n{desc}")).wrap(Wrap { trim: false })
    })
}
