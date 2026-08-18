use itertools::Itertools;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::Widget,
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
                    let mut spans = v.types().spans_iter(&v.inner().name);
                    if i == self.selected_idx {
                        for span in &mut spans {
                            span.style = span.style.underlined().bold().italic();
                        }
                    }
                    spans
                }
                LoadState::Failed(_) => vec![Span::raw(format!("failed to load")).style(Color::Red)],
                LoadState::Loading => vec![Span::raw("loading").style(Color::DarkGray)],
            })
            .intersperse(vec![Span::raw(" | ").style(Color::DarkGray)])
            .flatten()
            .collect();
        let line = Line::from(variants);
        if line.width() >= area.width as usize {
            line.left_aligned()
        } else {
            line.centered()
        }
        .render(area, buf);
    }
}
