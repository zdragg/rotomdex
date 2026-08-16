mod name;
mod sprite;
mod status_bar;
mod variant_selector;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    offline::OfflinePokemon,
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, status_bar::StatusBarWidget, variant_selector::VariantSelectorWidget,
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
        // Status bar
        let [area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        StatusBarWidget::new(self.pkmn.fetch_progress()).render(status_area, buf);

        // Fetch variants. Without variants, none of the other widgets can be rendered
        let variant_cnt = self.pkmn.variant_cnt();
        if variant_cnt == 0 {
            return; // No variants to render
        }
        let normalized_cursor = state.variant_cursor.rem_euclid(variant_cnt as isize) as usize;
        state.variant_cursor = normalized_cursor as isize;
        let Some(variant) = &self.pkmn.variants()[normalized_cursor] else {
            return; // Selected variant not loaded
        };

        // Title
        // let [title_area, area] =
        //     Layout::vertical([Constraint::Fill(1), Constraint::Fill(4)]).areas(area);

        // Sprites
        let [sprite_area, area] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);
        if let Some(sprite) = &self.pkmn.sprites()[normalized_cursor] {
            SpriteWidget::new(sprite).render(sprite_area, buf);
        }

        // Pokemon Name
        let [name_area, area] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(5)]).areas(area);
        NameWidget::new(self.pkmn.name(), &variant.types).render(name_area, buf);

        // Variant select (visualized)
        let [variants_area, area] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        VariantSelectorWidget::new(self.pkmn.variants()).render(variants_area, buf, state);
    }
}
