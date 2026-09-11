use crate::InnerActionResult;
use crate::model::{ModelAbility, ModelVariant, Resource};
use crate::widgets::dex::tabs::TabAction;
use crate::widgets::{Cursor, RenderBlockExt};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::macros::constraints;
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
        let Some(variant) = self.variant else {
            return;
        };
        let [first, second, hidden] = variant.abilities.get();

        let area = render_ability(first, false, area, buf);
        let area = render_ability(second, false, area, buf);
        let _ = render_ability(hidden, true, area, buf);
    }
}

fn render_ability(ability: Option<&Resource<ModelAbility>>, is_hidden: bool, area: Rect, buf: &mut Buffer) -> Rect {
    let Some(ability) = ability else {
        return area;
    };

    let Some(ability) = ability.as_loaded() else {
        let rest_area = Block::default()
            .borders(Borders::TOP)
            .title(Line::raw("── loading "))
            .render_inner(area, buf);
        return rest_area;
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
        let text = Paragraph::new(flavor_text.as_str()).wrap(Wrap { trim: true });
        let line_count = text.line_count(area.width);
        let [this_area, rest_area] = area.layout(&Layout::vertical(constraints![==line_count as u16, *=1]).spacing(1));
        text.render(this_area, buf);
        return rest_area;
    } else {
        return area;
    }
}

#[derive(Default)]
pub(super) struct AbilitiesTabWidgetState {
    cursor: Cursor,
}

impl AbilitiesTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) -> InnerActionResult {
        InnerActionResult::Nothing
    }
}
