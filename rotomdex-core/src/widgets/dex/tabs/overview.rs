use std::cell::Cell;

use crate::InnerActionResult;
use crate::model::{ModelEvolutionDetail, ModelSpecies, ModelVariant};
use crate::widgets::dex::tabs::TabAction;
use crate::widgets::{Cursor, RenderBlockExt};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, Paragraph, StatefulWidget, Widget, Wrap};

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

        let evo_area = render_flavor_text(species, area, buf);

        render_evolutions(species, &self.state.evolution_cursor, self.state, evo_area, buf);
    }
}

fn render_basic(species: &ModelSpecies, variant: &ModelVariant, area: Rect, buf: &mut Buffer) {
    Line::from_iter(itertools::chain!(
        name_span(species, variant),
        types_span(variant),
        effectiveness_span(variant)
    ))
    .render(area, buf);
}

fn name_span<'a>(species: &'a ModelSpecies, variant: &'a ModelVariant) -> Vec<Span<'a>> {
    // charizard-mega-x#0006
    vec![
        Span::raw(&variant.name),
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
        type_spans.push(Span::raw("/"));
        type_spans.push(Span::styled(secondary.to_string(), secondary.tui_color()));
    }
    type_spans.push(Span::raw("  "));
    type_spans
}

fn effectiveness_span(variant: &ModelVariant) -> Vec<Span<'_>> {
    variant.types.def_effectiveness().into_spans()
}

fn render_flavor_text(species: &ModelSpecies, area: Rect, buf: &mut Buffer) -> Rect {
    let flavor = if let Some(flavor_text) = &species.flavor_text {
        Paragraph::new(flavor_text.text.as_str()).style(Color::Gray)
    } else {
        Paragraph::new("Missing flavor text!").style(Color::Red)
    }
    .wrap(Wrap { trim: true });

    let line_count = flavor.line_count(area.width - 1);

    let [this_area, rest_area] =
        area.layout(&Layout::vertical([Constraint::Length(line_count as u16), Constraint::Fill(1)]).spacing(1));

    let this_area = Block::default().borders(Borders::LEFT).render_inner(this_area, buf);
    flavor.render(this_area, buf);

    rest_area
}

fn render_evolutions(
    species: &ModelSpecies,
    cursor: &Cursor,
    state: &OverviewTabWidgetState,
    area: Rect,
    buf: &mut Buffer,
) {
    let Some(chain) = &species.evolution_chain else {
        Line::styled("No evolutions!", Color::Red).render(area, buf);
        return;
    };

    let Some(chain) = chain.as_loaded() else {
        Line::styled("Loading...", Color::DarkGray).render(area, buf);
        return;
    };

    let views = chain.get_views();

    if views.len() <= 1 {
        Line::styled("Only child!", Color::Yellow).render(area, buf);
        return;
    }

    let [tree_area, detail_area] = area.layout(&Layout::horizontal([Constraint::Percentage(40), Constraint::Fill(1)]));

    let selected = cursor.get(views.len()).unwrap();
    let mut lines = Vec::with_capacity(views.len());
    let mut ancestor_prefix = Vec::new();

    for (index, view) in views.iter().enumerate() {
        if selected == index && state.needs_new_name.get() {
            state.selected_pkmn_name.set(view.species_name.to_owned());
            state.needs_new_name.set(false);
        }

        ancestor_prefix.truncate(view.depth);
        let mut prefix = String::new();
        for continues in ancestor_prefix.iter().skip(1) {
            prefix.push_str(if *continues { "│  " } else { "   " });
        }
        if view.depth > 0 {
            prefix.push_str(if view.last_in_depth { "└─ " } else { "├─ " });
        }
        ancestor_prefix.push(!view.last_in_depth);

        let name = if selected == index {
            Span::raw(view.species_name.to_uppercase())
        } else {
            Span::raw(view.species_name)
        };
        let name = if view.species_name == species.name {
            name.style(Color::Yellow)
        } else {
            name
        };
        let line = Line::from(if selected == index {
            vec![Span::raw(prefix), name, Span::raw(" <")]
        } else {
            vec![Span::raw(prefix), name]
        });
        lines.push(line);
    }

    let list = List::new(lines).scroll_padding(1);

    StatefulWidget::render(list, tree_area, buf, &mut cursor.list_state(views.len()));

    render_evo_details(views[selected].evolution_detail, detail_area, buf);
}

fn render_evo_details(detail: &ModelEvolutionDetail, area: Rect, buf: &mut Buffer) {
    let reqs = detail.to_strings();
    if reqs.is_empty() {
        Span::styled("No evolution found", Color::Red).render(area, buf);
        return;
    };

    let iter = reqs.iter().map(|str| Line::from(format!("- {}", str)));

    let mut lines = vec![Line::raw("Evolution requires:")];
    lines.extend(iter);

    let paragraph = Paragraph::new(lines);
    paragraph.render(area, buf);
}

#[derive(Default)]
pub(super) struct OverviewTabWidgetState {
    evolution_cursor: Cursor,
    needs_new_name: Cell<bool>,
    selected_pkmn_name: Cell<String>,
}

impl OverviewTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) -> InnerActionResult {
        match action {
            TabAction::Down => {
                self.evolution_cursor.next();
                self.needs_new_name.set(true);
            }
            TabAction::Up => {
                self.evolution_cursor.prev();
                self.needs_new_name.set(true);
            }
            TabAction::Enter => {
                let name = self.selected_pkmn_name.take();
                if !name.is_empty() {
                    return InnerActionResult::NewPokemon(name);
                }
            }
            _ => {}
        }
        InnerActionResult::Nothing
    }
}
