mod name;
mod sprite;
mod stats;
mod status_bar;
mod variant_selector;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    offline::{LoadState, OfflinePokemon},
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, stats::StatsWidget, status_bar::StatusBarWidget,
        variant_selector::VariantSelectorWidget,
    },
};

pub struct RotomDexWidget<'a> {
    pub pkmn: &'a OfflinePokemon,
}

impl<'a> RotomDexWidget<'a> {
    pub fn new(pkmn: &'a OfflinePokemon) -> Self {
        Self { pkmn }
    }
}

#[derive(Default)]
pub struct VariantState {
    pub variant_cursor: isize,
}

impl VariantState {
    pub fn reset(&mut self) {
        self.variant_cursor = 0;
    }
    pub fn next(&mut self) {
        self.variant_cursor = self.variant_cursor.wrapping_add(1);
    }
    pub fn prev(&mut self) {
        self.variant_cursor = self.variant_cursor.wrapping_sub(1);
    }
}

impl<'a> StatefulWidget for RotomDexWidget<'a> {
    type State = VariantState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Fetch species
        let (species, variant) = if let LoadState::Loaded(species) = self.pkmn.species() {
            let normalized_cursor = state.variant_cursor.rem_euclid(species.variants_cnt() as isize) as usize;
            state.variant_cursor = normalized_cursor as isize;
            let variant = if let LoadState::Loaded(variant) = &species.variants()[normalized_cursor] {
                Some(variant)
            } else {
                None
            };
            (Some(species), variant)
        } else {
            (None, None)
        };

        let area = area.inner(Margin::new(2, 0)); // leave some padding on the sides

        // Status bar
        let [area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        StatusBarWidget.render(status_area, buf);

        // Sprite
        let [sprite_area, _padding, area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(2), Constraint::Fill(1)]).areas(area);
        if let Some(variant) = variant {
            if let LoadState::Loaded(sprite) = variant.sprite() {
                SpriteWidget::new(sprite).render(sprite_area, buf);
            }
        }

        let [name_area, variants_area, stats_area, area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        if let Some(species) = species {
            NameWidget::new(&species.inner().name, variant.map(|v| v.types())).render(name_area, buf);
            VariantSelectorWidget::new(species.variants()).render(variants_area, buf, state);
            if let Some(variant) = variant {
                StatsWidget::new(&variant.stats()).render(stats_area, buf);
            }
        }
    }
}
