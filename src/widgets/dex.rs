mod ability;
mod name;
mod sprite;
mod stats;
mod status_bar;
mod variant_selector;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    offline::OfflinePokemon,
    widgets::dex::{
        ability::AbilitiesWidget, name::NameWidget, sprite::SpriteWidget, stats::StatsWidget,
        status_bar::StatusBarWidget, variant_selector::VariantSelectorWidget,
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
        // Load all the shit
        let species = self.pkmn.species().as_loaded();
        let variant = match species {
            Some(species) => {
                // Normalize
                let normalized_cursor = state.variant_cursor.rem_euclid(species.variants_cnt() as isize) as usize;
                state.variant_cursor = normalized_cursor as isize;
                species.variants()[normalized_cursor].as_loaded()
            }
            None => None,
        };
        let sprite = variant.map(|v| v.sprite().as_loaded()).flatten();

        let block = Block::bordered().border_style(if let Some(species) = species {
            species.get_ratatui_color()
        } else {
            Color::DarkGray
        });
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Status bar
        let [area, _padding, status_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)]).areas(inner_area);
        StatusBarWidget.render(status_area, buf);

        let [left_area, _padding, right_area] =
            Layout::horizontal([Constraint::Percentage(35), Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        // Sprite
        let [sprite_area, _padding, stats_area] =
            Layout::vertical([Constraint::Percentage(70), Constraint::Length(1), Constraint::Fill(1)]).areas(left_area);
        sprite.map(SpriteWidget::new).render(sprite_area, buf);
        variant
            .map(|variant| StatsWidget::new(variant, species.unwrap())) // Species has to exist if variant exists
            .render(stats_area, buf);

        let [name_area, variants_area, area] =
            Layout::vertical([Constraint::Percentage(25), Constraint::Length(3), Constraint::Fill(1)])
                .areas(right_area);
        species
            .map(|species| NameWidget::new(species, variant))
            .render(name_area, buf);
        species
            .map(|species| VariantSelectorWidget::new(species, state.variant_cursor as usize))
            .render(variants_area, buf);
        variant.map(AbilitiesWidget::new).render(area, buf);
    }
}
