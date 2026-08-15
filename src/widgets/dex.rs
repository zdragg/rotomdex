pub mod name;
pub mod sprite;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    offline::OfflinePokemon,
    widgets::{dex::sprite::SpriteWidget, name::NameWidget},
};

pub struct RotomDexWidget<'a> {
    pub pkmn: &'a OfflinePokemon,
}

#[derive(Default)]
pub struct RotomDexState {
    pub variant_cursor: isize,
}

impl RotomDexState {
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
    type State = RotomDexState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
        let sprite_side_len = area.height * 2;
        let [sprite_area, area] =
            Layout::horizontal([Constraint::Length(sprite_side_len), Constraint::Fill(1)])
                .areas(area);
        if let Some(sprite) = &self.pkmn.sprites()[normalized_cursor] {
            SpriteWidget::new(sprite, sprite_side_len).render(sprite_area, buf);
        }

        // Pokemon Name
        let [name_area, area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(3)]).areas(area);

        NameWidget { variant }.render(name_area, buf);
    }
}
