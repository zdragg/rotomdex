use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub(super) struct OverviewTabWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
}

impl<'a> OverviewTabWidget<'a> {
    pub(super) fn new(species: Option<&'a ModelSpecies>, variant: Option<&'a ModelVariant>) -> Self {
        Self { species, variant }
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

        let [first_line, area] = area.layout(&Layout::vertical([Constraint::Percentage(5), Constraint::Fill(1)]));

        Line::from_iter(itertools::chain!(
            name_span(species, variant),
            types_span(variant),
            [physique_span(variant)]
        ))
        .render(first_line, buf);
    }
}

fn name_span<'a>(species: &'a ModelSpecies, variant: &'a ModelVariant) -> Vec<Span<'a>> {
    // charizard-mega-x#0006
    vec![
        Span::styled(&variant.inner.name, Color::White),
        Span::styled(format!("#{:04}  ", species.inner.id), Color::DarkGray),
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

#[derive(Default)]
pub(super) struct BasicTabWidgetState {}

impl BasicTabWidgetState {
    pub(super) fn handle_action(&mut self, _action: TabAction) {
        // Nothing to handle.
    }
}
