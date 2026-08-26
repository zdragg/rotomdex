use itertools::Itertools;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::{
    model::{ModelSpecies, ModelVariant, Resource},
    projector::Section,
    widgets::WidgetExt,
};

pub(crate) struct VariantSelectorWidget<'a> {
    variants: &'a [Resource<ModelVariant>],
    selected_idx: usize,
    focused: bool,
}

impl<'a> WidgetExt<(&'a ModelSpecies, usize, Section)> for VariantSelectorWidget<'a> {
    fn new((pkmn, selected_idx, section): (&'a ModelSpecies, usize, Section)) -> Self {
        Self {
            variants: pkmn.variants(),
            selected_idx,
            focused: section == Section::VariantSelect,
        }
    }
}

impl<'a> Widget for VariantSelectorWidget<'a> {
    #[allow(unstable_name_collisions)] // Should do something about this
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let variants: Vec<_> = self
            .variants
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Resource::Loaded(v) => {
                    if i == self.selected_idx {
                        let mut spans = v.types().spans_iter(&v.get_variant_name().to_ascii_uppercase());
                        for span in &mut spans {
                            span.style = span.style.bold();
                            if self.focused {
                                span.style = span.style.underlined()
                            }
                        }
                        spans
                    } else {
                        v.types().spans_iter(v.get_variant_name())
                    }
                }
                Resource::Loading(_) => vec![Span::raw("loading").style(Color::DarkGray)],
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

        // If line width overflows: sophisticated thing
        // not that good to be honest, maybe rework it later
        let line_width = line.width();
        let selected_start = variant_widths[..self.selected_idx].iter().sum::<usize>() + self.selected_idx * 3;
        let first_center = variant_widths[0];
        let selected_center = 2 * selected_start + variant_widths[self.selected_idx];
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
