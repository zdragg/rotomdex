use itertools::Itertools;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::model::{ModelSpecies, Resource};

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
    #[allow(unstable_name_collisions)]
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let Some((species, cursor)) = self.species.zip(self.cursor) else {
            return;
        };
        let variants: Vec<_> = species
            .variants()
            .iter()
            .enumerate()
            .map(|(idx, variant)| match variant {
                Resource::Loaded(variant) => {
                    let name = if idx == cursor {
                        variant.get_variant_name().to_ascii_uppercase()
                    } else {
                        variant.get_variant_name().to_owned()
                    };
                    let mut spans = variant.types.spans_iter(&name);
                    if idx == cursor {
                        for span in &mut spans {
                            span.style = span.style.bold();
                        }
                    }
                    spans
                }
                Resource::Loading { deferred, .. } if deferred.get() => {
                    vec![Span::raw("deferred").style(Color::DarkGray)]
                }
                Resource::Loading { .. } => vec![Span::raw("loading").style(Color::DarkGray)],
                Resource::Failed(_) => vec![Span::raw("failed: check log").style(Color::Red)],
            })
            .collect();
        let variant_widths: Vec<_> = variants
            .iter()
            .map(|spans| spans.iter().map(Span::width).sum::<usize>())
            .collect();
        let variants: Vec<_> = variants
            .into_iter()
            .intersperse(vec![Span::raw(" | ").style(Color::DarkGray)])
            .flatten()
            .collect();
        let line = Line::from(variants);
        if line.width() <= area.width as usize {
            line.centered().render(area, buf);
            return;
        }

        let line_width = line.width();
        let selected_start = variant_widths[..cursor].iter().sum::<usize>() + cursor * 3;
        let first_center = variant_widths[0];
        let selected_center = 2 * selected_start + variant_widths[cursor];
        let last_center = 2 * line_width - variant_widths[variant_widths.len() - 1];
        let center_range = last_center - first_center;
        let scroll_range = line_width - usize::from(area.width);
        let scroll = (scroll_range * (selected_center - first_center) + center_range / 2)
            .checked_div(center_range)
            .unwrap_or(0);

        Paragraph::new(line)
            .scroll((0, scroll.min(usize::from(u16::MAX)) as u16))
            .render(area, buf);
    }
}
