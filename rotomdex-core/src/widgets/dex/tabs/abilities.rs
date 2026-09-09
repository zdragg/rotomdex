use crate::model::{ModelAbility, ModelVariant, Resource};
use crate::widgets::dex::tabs::TabAction;
use crate::widgets::{Cursor, RenderBlockExt};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

pub(super) struct AbilitiesTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
}

impl<'a> AbilitiesTabWidget<'a> {
    pub(super) fn new(variant: Option<&'a ModelVariant>) -> Self {
        Self { variant }
    }
}

impl<'a> Widget for AbilitiesTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas: [Rect; 3] = area.layout(&Layout::vertical([Constraint::Fill(1); 3]));
        let Some(variant) = self.variant else {
            areas.into_iter().for_each(|area| Block::bordered().render(area, buf));
            return;
        };
        let [first, second, hidden] = variant.abilities.get();

        render_ability(first, false, areas[0], buf);
        render_ability(second, false, areas[1], buf);
        render_ability(hidden, true, areas[2], buf);
    }
}

fn render_ability(ability: Option<&Resource<ModelAbility>>, is_hidden: bool, area: Rect, buf: &mut Buffer) {
    let Some(ability) = ability else {
        Block::default()
            .borders(Borders::TOP)
            .title(Line::from(vec![
                Span::raw("── "),
                Span::styled("no ability in slot ", Color::Red),
            ]))
            .render(area, buf);
        return;
    };

    let Some(ability) = ability.as_loaded() else {
        Block::default()
            .borders(Borders::TOP)
            .title(Line::raw("── loading "))
            .render(area, buf);
        return;
    };

    let block_title = if is_hidden {
        Line::from(vec![
            Span::raw(format!("── {}", &ability.name)),
            Span::styled(" (hidden) ", Color::DarkGray),
        ])
    } else {
        Line::raw(format!("── {} ", &ability.name))
    };

    let area = Block::default()
        .borders(Borders::TOP)
        .title(block_title)
        .render_inner(area, buf);

    if let Some(flavor_text) = &ability.flavor_text {
        Paragraph::new(flavor_text.as_str())
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

#[derive(Default)]
pub(super) struct AbilitiesTabWidgetState {
    cursor: Cursor,
}

impl AbilitiesTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
