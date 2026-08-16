use itertools::Itertools;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::{offline::OfflineVariant, widgets::VariantState};

pub struct VariantSelectorWidget<'a> {
    pkmn: &'a [Option<OfflineVariant>],
}

impl<'a> VariantSelectorWidget<'a> {
    pub fn new(pkmn: &'a [Option<OfflineVariant>]) -> Self {
        Self { pkmn }
    }
}

impl<'a> StatefulWidget for VariantSelectorWidget<'a> {
    type State = VariantState;
    #[allow(unstable_name_collisions)] // Should do something about this
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let variant_cnt = self.pkmn.len();
        state.variant_cursor = state.variant_cursor.rem_euclid(variant_cnt as isize); // Just in case dex.rs didn't do this
        let variants: Vec<_> = self
            .pkmn
            .iter()
            .enumerate()
            .map(|(i, v)| match v.as_ref() {
                Some(v) => {
                    let mut spans = v.types.spans_iter(&v.pkmn.name);
                    if state.variant_cursor == i as isize {
                        for span in &mut spans {
                            span.style = span.style.underlined().bold().italic();
                        }
                    }
                    spans
                }
                None => vec![Span::raw("loading").style(Color::DarkGray)],
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
