use crate::data::resource::AsyncResource;
use crate::data::{ModelAbility, ModelSpecies, ModelVariant};
use crate::widgets::RenderBlockExt;
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Rect};
use ratatui::macros::constraints;
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

pub(super) struct AbilitiesTabWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
}

impl<'a> AbilitiesTabWidget<'a> {
    pub(super) fn new(
        species: Option<&'a ModelSpecies>,
        variant: Option<&'a ModelVariant>,
    ) -> Self {
        Self { species, variant }
    }
}

impl<'a> Widget for AbilitiesTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(variant) = self.variant else {
            return;
        };

        let Some(abilities) = variant.abilities.as_loaded() else {
            return;
        };

        let [first, second, hidden] = abilities.get();

        let area = render_ability(self.species, first, false, area, buf);
        let area = render_ability(self.species, second, false, area, buf);
        let _ = render_ability(self.species, hidden, true, area, buf);
    }
}

fn render_ability(
    species: Option<&ModelSpecies>,
    ability: Option<&AsyncResource<ModelAbility>>,
    is_hidden: bool,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    let color = species.map_or(Color::Reset, |species| species.color);

    let Some(ability) = ability else {
        return area;
    };

    let Some(ability) = ability.as_loaded() else {
        let rest_area = Block::default()
            .borders(Borders::TOP)
            .title(Line::from(vec![Span::raw("── loading ")]))
            .render_inner(area, buf);
        return rest_area;
    };

    let block_title = if is_hidden {
        Line::from(vec![
            Span::raw("── "),
            Span::styled(&ability.name, color),
            Span::styled(" (hidden) ", Color::DarkGray),
        ])
    } else {
        Line::from(vec![
            Span::raw("── "),
            Span::styled(&ability.name, color),
            Span::raw(" "),
        ])
    };

    let area = Block::default()
        .borders(Borders::TOP)
        .title(block_title)
        .render_inner(area, buf);

    if let Some(flavor_text) = &ability.flavor_text {
        let text = Paragraph::new(flavor_text.as_str())
            .style(Color::White)
            .wrap(Wrap { trim: true });
        let line_count = text.line_count(area.width);
        let [this_area, rest_area] =
            area.layout(&Layout::vertical(constraints![==line_count as u16, *=1]).spacing(1));
        text.render(this_area, buf);
        rest_area
    } else {
        area
    }
}

#[derive(Default)]
pub(super) struct AbilitiesTabWidgetState {}

impl AbilitiesTabWidgetState {
    pub(super) fn handle_action(&mut self, _action: TabAction) {}
}
