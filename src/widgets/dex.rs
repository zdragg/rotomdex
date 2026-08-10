mod sprite;
mod title;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

use crate::{
    pkmn::OfflinePokemon,
    widgets::dex::{sprite::SpriteWidget, title::TitleWidget},
};

pub struct RotomDexWidget {
    pub pkmn: OfflinePokemon,
}

impl Widget for &RotomDexWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let variant = self.pkmn.get_current_pkmn();

        // Title
        // let [title_area, area] =
        //     Layout::vertical([Constraint::Fill(1), Constraint::Fill(4)]).areas(area);
        TitleWidget::new(&variant.name).render(area, buf);

        // Sprites
        let sprite_side_len = area.height * 2;
        let [sprite_area, area] =
            Layout::horizontal([Constraint::Length(sprite_side_len), Constraint::Fill(1)])
                .areas(area);
        if let Some(sprite) = self.pkmn.get_current_sprite() {
            SpriteWidget::new(sprite, sprite_side_len).render(sprite_area, buf);
        }
    }
}
