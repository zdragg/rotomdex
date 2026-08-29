mod name;
mod sprite;
mod stats;
mod tabs;
mod variant;

use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Line,
    widgets::{Block, Widget},
};

use crate::{
    Action,
    model::ModelPokemon,
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, stats::StatsWidget, tabs::TabWidget, variant::VariantSelectorWidget,
    },
};

pub(crate) struct DexWidget<'a> {
    pkmn: &'a ModelPokemon,
    state: &'a DexState,
    elapsed: Duration,
    bottom_text: &'a str,
}

impl<'a> DexWidget<'a> {
    pub(crate) fn new(pkmn: &'a ModelPokemon, state: &'a DexState, elapsed: Duration, bottom_text: &'a str) -> Self {
        Self {
            pkmn,
            state,
            elapsed,
            bottom_text,
        }
    }
}

impl Widget for DexWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let species = self.pkmn.species.as_loaded();
        let variant_idx = species.and_then(|species| self.state.variant_idx(species.variants_cnt()));
        let variant = species
            .zip(variant_idx)
            .and_then(|(species, idx)| species.variants().get(idx))
            .and_then(|variant| variant.as_loaded());

        let block = Block::bordered()
            .border_style(species.map_or(Color::DarkGray, |species| species.get_ratatui_color()))
            .title_bottom(
                Line::raw(format!(" {} ", self.bottom_text))
                    .style(Color::DarkGray)
                    .centered(),
            );
        let outer = area;
        let area = block.inner(outer);
        block.render(outer, buf);

        let [left_area, right_area] = Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)])
            .spacing(1)
            .areas(area);
        let [sprite_area, stats_area] = Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)])
            .spacing(1)
            .areas(left_area);
        let [name_area, variants_area, tab_area] =
            Layout::vertical([Constraint::Percentage(20), Constraint::Length(1), Constraint::Fill(1)])
                .areas(right_area);

        SpriteWidget::new(variant, self.elapsed).render(sprite_area, buf);
        StatsWidget::new(variant, species).render(stats_area, buf);
        NameWidget::new(species, variant).render(name_area, buf);
        VariantSelectorWidget::new(species, variant_idx).render(variants_area, buf);
        TabWidget::new(species, variant).render(tab_area, buf);
    }
}

#[derive(Default)]
pub(crate) struct DexState {
    variant_cursor: Cursor,
}

impl DexState {
    pub(crate) fn handle_action(&mut self, action: Action) {
        match action {
            Action::Right => self.variant_cursor.next(),
            Action::Left => self.variant_cursor.prev(),
            _ => {}
        }
    }

    pub(crate) fn reset(&mut self) {
        self.variant_cursor.reset();
    }

    fn variant_idx(&self, total: usize) -> Option<usize> {
        self.variant_cursor.get(total)
    }
}

#[derive(Default)]
struct Cursor {
    idx: usize,
}

impl Cursor {
    fn next(&mut self) {
        self.idx = self.idx.wrapping_add(1);
    }

    fn prev(&mut self) {
        self.idx = self.idx.wrapping_sub(1);
    }

    fn reset(&mut self) {
        self.idx = 0;
    }

    fn get(&self, total: usize) -> Option<usize> {
        self.idx.checked_rem(total)
    }
}
