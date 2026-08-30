use crate::model::{ModelSpecies, ModelVariant, Resource};
use ratatui::layout::{Constraint, Layout};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::Widget,
};

pub(crate) struct VariantSelectorWidget<'a> {
    species: Option<&'a ModelSpecies>,
    cursor: Option<usize>,
}

impl<'a> VariantSelectorWidget<'a> {
    pub(crate) fn new(species: Option<&'a ModelSpecies>, cursor: Option<usize>) -> Self {
        Self { species, cursor }
    }
}

impl Widget for VariantSelectorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some((species, cursor)) = self.species.zip(self.cursor) else {
            return;
        };
        let variants = species.variants();
        let Some(selected) = variants.get(cursor) else {
            return;
        };

        if variants.len() == 1 {
            Line::from(get_variant_spans(selected, true))
                .centered()
                .render(area, buf);
            return;
        }

        let prev_idx = (cursor + variants.len() - 1) % variants.len();
        let next_idx = (cursor + 1) % variants.len();

        let mut prev_span = get_variant_spans(&variants[prev_idx], false);
        prev_span.push(Span::raw(" <- ").style(Color::DarkGray));
        let prev_line = Line::from(prev_span);

        let selected_line = Line::from(get_variant_spans(selected, true));

        let mut next_span = vec![Span::raw(" -> ").style(Color::DarkGray)];
        next_span.append(&mut get_variant_spans(&variants[next_idx], false));
        let next_line = Line::from(next_span);

        let [prev_area, selected_area, next_area] = area.layout(&Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(selected_line.width() as u16),
            Constraint::Fill(1),
        ]));

        prev_line.right_aligned().render(prev_area, buf);
        selected_line.centered().render(selected_area, buf);
        next_line.left_aligned().render(next_area, buf);
    }
}

fn get_variant_spans(variant: &Resource<ModelVariant>, selected: bool) -> Vec<Span<'_>> {
    let mut spans = match variant {
        Resource::Loaded(variant) => {
            let name = if selected {
                variant.get_variant_name().to_ascii_uppercase()
            } else {
                variant.get_variant_name().to_owned()
            };
            variant.types.spans_iter(&name)
        }
        Resource::Loading { deferred, .. } if deferred.get() => {
            vec![Span::raw("deferred").style(Color::DarkGray)]
        }
        Resource::Loading { .. } => vec![Span::raw("loading").style(Color::DarkGray)],
        Resource::Failed(_) => vec![Span::raw("failed: check log").style(Color::Red)],
    };

    if selected {
        for span in &mut spans {
            span.style = span.style.bold().underlined();
        }
    }

    spans
}
