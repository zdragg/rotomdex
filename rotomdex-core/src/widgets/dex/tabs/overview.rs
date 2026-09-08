use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::Cursor;
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, Paragraph, StatefulWidget, Widget, Wrap};

pub(super) struct OverviewTabWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
    state: &'a OverviewTabWidgetState,
}

impl<'a> OverviewTabWidget<'a> {
    pub(super) fn new(
        species: Option<&'a ModelSpecies>,
        variant: Option<&'a ModelVariant>,
        state: &'a OverviewTabWidgetState,
    ) -> Self {
        Self {
            species,
            variant,
            state,
        }
    }
}

impl<'a> Widget for OverviewTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(species) = self.species else {
            return;
        };
        let Some(variant) = self.variant else {
            return;
        };

        let [first_line, area] = area.layout(&Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]));

        render_basic(species, variant, first_line, buf);
        let [flavor_area, evolution_area] =
            area.layout(&Layout::vertical([Constraint::Percentage(20), Constraint::Fill(1)]));
        render_flavor_text(species, flavor_area, buf);
        render_evolution(species, &self.state.evolution_cursor, evolution_area, buf);
    }
}

fn render_basic(species: &ModelSpecies, variant: &ModelVariant, area: Rect, buf: &mut Buffer) {
    Line::from_iter(itertools::chain!(
        name_span(species, variant),
        types_span(variant),
        [physique_span(variant)]
    ))
    .render(area, buf);
}

fn name_span<'a>(species: &'a ModelSpecies, variant: &'a ModelVariant) -> Vec<Span<'a>> {
    // charizard-mega-x#0006
    vec![
        Span::styled(&variant.inner.name, Color::White),
        Span::styled(format!("#{:04}  ", species.national_dex), Color::DarkGray),
    ]
}
fn types_span(variant: &ModelVariant) -> Vec<Span<'_>> {
    // Fire/Dragon
    let mut type_spans = vec![Span::styled(
        variant.types.primary.to_string(),
        variant.types.primary.tui_color(),
    )];
    if let Some(secondary) = &variant.types.secondary {
        type_spans.push(Span::styled("/", Color::White));
        type_spans.push(Span::styled(secondary.to_string(), secondary.tui_color()));
    }
    type_spans.push(Span::raw("  "));
    type_spans
}

fn physique_span(variant: &ModelVariant) -> Span<'_> {
    // 1.7m 110.5kg
    Span::styled(
        format!(
            "{:.1}m {:.1}kg ",
            (variant.inner.height as f64) / 10.0,
            (variant.inner.weight as f64) / 10.0,
        ),
        Color::White,
    )
}

fn render_flavor_text(species: &ModelSpecies, area: Rect, buf: &mut Buffer) {
    let flavor = if let Some(flavor_text) = &species.flavor_text {
        Paragraph::new(flavor_text.text.as_str()).style(Color::White)
    } else {
        Paragraph::new("Missing flavor text!").style(Color::Red)
    };
    flavor.wrap(Wrap { trim: true }).render(area, buf);
}

fn render_evolution(species: &ModelSpecies, cursor: &Cursor, area: Rect, buf: &mut Buffer) {
    let Some(chain) = &species.evolution_chain else {
        Line::styled("No evolutions!", Color::Red).render(area, buf);
        return;
    };
    let Some(chain) = chain.as_loaded() else {
        Line::styled("Loading...", Color::DarkGray).render(area, buf);
        return;
    };

    let views = chain.get_views();

    if views.len() == 1 {
        Line::styled("Only child!", Color::Yellow).render(area, buf);
        return;
    }

    let selected = cursor.get(views.len());
    let mut lines = Vec::with_capacity(views.len());
    let mut ancestor_prefix = Vec::new();

    for (index, view) in views.iter().enumerate() {
        ancestor_prefix.truncate(view.depth);
        let mut prefix = String::new();
        for continues in ancestor_prefix.iter().skip(1) {
            prefix.push_str(if *continues { "│  " } else { "   " });
        }
        if view.depth > 0 {
            prefix.push_str(if view.last_in_depth { "└─ " } else { "├─ " });
        }
        ancestor_prefix.push(!view.last_in_depth);

        let color = if view.species_name == species.name {
            Color::Yellow
        } else {
            Color::Reset
        };
        let mut spans = vec![Span::raw(prefix), Span::styled(view.species_name, color)];
        if selected == Some(index) {
            spans.push(Span::raw(" <"));
        }
        lines.push(Line::from(spans));
    }

    StatefulWidget::render(List::new(lines), area, buf, &mut cursor.list_state(views.len()));
}

#[derive(Default)]
pub(super) struct OverviewTabWidgetState {
    evolution_cursor: Cursor,
}

impl OverviewTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {
        match action {
            TabAction::Down => self.evolution_cursor.next(),
            TabAction::Up => self.evolution_cursor.prev(),
            _ => {}
        }
    }
}
