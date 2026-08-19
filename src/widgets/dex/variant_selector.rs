use itertools::Itertools;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::offline::{LoadState, OfflineSpecies, OfflineVariant};

pub struct VariantSelectorWidget<'a> {
    variants: &'a [LoadState<OfflineVariant>],
    selected_idx: usize,
}

impl<'a> VariantSelectorWidget<'a> {
    pub fn new(pkmn: &'a OfflineSpecies, selected_idx: usize) -> Self {
        Self {
            variants: pkmn.variants(),
            selected_idx,
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
                LoadState::Loaded(v) => {
                    if i == self.selected_idx {
                        let mut spans = v.types().spans_iter(&v.get_variant_name().to_ascii_uppercase());
                        for span in &mut spans {
                            span.style = span.style.underlined().bold().italic();
                        }
                        spans
                    } else {
                        v.types().spans_iter(&v.get_variant_name())
                    }
                }
                LoadState::Failed(_) => vec![Span::raw(format!("failed: check log")).style(Color::Red)],
                LoadState::Loading => vec![Span::raw("loading").style(Color::DarkGray)],
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
        let scroll = if center_range == 0 {
            0
        } else {
            (scroll_range * (selected_center - first_center) + center_range / 2) / center_range
        };

        Paragraph::new(line)
            .scroll((0, scroll.min(usize::from(u16::MAX)) as u16))
            .render(area, buf);
    }
}
