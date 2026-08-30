mod name;
mod search;
mod sprite;
mod stats;
mod tabs;
mod variant;

use crate::widgets::dex::search::{SearchWidget, SearchWidgetState};
use crate::widgets::dex::tabs::TabsWidgetState;
use crate::{
    Action,
    model::ModelPokemon,
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, stats::StatsWidget, tabs::TabsWidget, variant::VariantSelectorWidget,
    },
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, Widget},
};
use std::time::Duration;

pub(crate) struct DexWidget<'a> {
    pkmn: &'a ModelPokemon,
    elapsed: Duration,
    bottom_text: &'static str,

    state: &'a DexState,
}

impl<'a> DexWidget<'a> {
    pub(crate) fn new(
        pkmn: &'a ModelPokemon,
        state: &'a DexState,
        elapsed: Duration,
        bottom_text: &'static str,
    ) -> Self {
        Self {
            pkmn,
            elapsed,
            bottom_text,

            state,
        }
    }
}

impl Widget for DexWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let species = self.pkmn.species.as_loaded();
        let variant_idx = species.and_then(|species| self.state.variant_cursor.get(species.variants_cnt()));
        let variant = species
            .zip(variant_idx)
            .and_then(|(species, idx)| species.variants().get(idx))
            .and_then(|variant| variant.as_loaded());

        // Block + bottom text / search widget render
        let block =
            Block::bordered().border_style(species.map_or(Color::DarkGray, |species| species.get_ratatui_color()));
        let [_area, bottom_text_area] = area.layout(&Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]));
        let outer = area;
        let area = block.inner(outer);
        block.render(outer, buf);
        SearchWidget::new(&self.state.search_state, self.bottom_text).render(bottom_text_area, buf);

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
        TabsWidget::new(species, variant, &self.state.tabs_state).render(tab_area, buf);
    }
}

#[derive(Default)]
pub(crate) struct DexState {
    variant_cursor: Cursor,

    search_state: SearchWidgetState,
    tabs_state: TabsWidgetState,
}

impl DexState {
    pub(crate) fn handle_action(&mut self, action: Action) -> Option<String> {
        match action {
            Action::Input(ch) if self.search_state.searching => self.search_state.handle_input(ch),
            Action::Backspace if self.search_state.searching => self.search_state.backspace(),
            Action::Escape | Action::CapsLock if self.search_state.searching => self.search_state.abort_search(),
            Action::Enter if self.search_state.searching => return Some(self.search_state.take()),
            Action::Input('f') => self.variant_cursor.next(),
            Action::Input('d') => self.variant_cursor.prev(),
            Action::Input(':') => self.search_state.start_search(),
            _ => self.tabs_state.handle_action(action),
        }
        None
    }

    pub(crate) fn reset(&mut self) {
        self.variant_cursor.reset();
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
