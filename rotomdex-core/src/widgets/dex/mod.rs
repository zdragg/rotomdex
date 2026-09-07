mod name;
mod search;
mod sprite;
mod stats;
mod tabs;
mod tutorial;
mod variant;
mod versions;

use crate::widgets::Cursor;
use crate::widgets::dex::search::{SearchWidget, SearchWidgetState};
use crate::widgets::dex::sprite::SpriteWidgetState;
use crate::widgets::dex::tabs::TabsWidgetState;
use crate::widgets::dex::tutorial::{TutorialWidget, TutorialWidgetState};
use crate::widgets::dex::versions::{VersionState, VersionWidget};
use crate::{
    DexKeyCode,
    model::ModelPokemon,
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, stats::StatsWidget, tabs::TabsWidget, variant::VariantSelectorWidget,
    },
};
use crate::{InnerActionResult, Version};
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
    version: Version,

    state: &'a DexState,
}

impl<'a> DexWidget<'a> {
    pub(crate) fn new(pkmn: &'a ModelPokemon, state: &'a DexState, elapsed: Duration, version: Version) -> Self {
        Self {
            pkmn,
            elapsed,
            version,
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
        SearchWidget::new(&self.state.search_state).render(bottom_text_area, buf);

        let [left_area, right_area] = Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)])
            .spacing(1)
            .areas(area);
        let [sprite_area, stats_area] = Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)])
            .spacing(1)
            .areas(left_area);
        let [name_area, variants_area, tab_area] =
            Layout::vertical([Constraint::Percentage(20), Constraint::Length(2), Constraint::Fill(1)])
                .areas(right_area);

        SpriteWidget::new(variant, self.elapsed, &self.state.sprite_cache_state).render(sprite_area, buf);
        StatsWidget::new(species, variant).render(stats_area, buf);
        NameWidget::new(species, variant).render(name_area, buf);
        VariantSelectorWidget::new(species, variant_idx).render(variants_area, buf);
        TabsWidget::new(species, variant, &self.state.tabs_state).render(tab_area, buf);
        TutorialWidget::new(&self.state.tutorial_state).render(area, buf);
        VersionWidget::new(self.version, &self.state.version_state).render(stats_area, buf);
    }
}

#[derive(Default)]
pub(crate) struct DexState {
    variant_cursor: Cursor,

    version_state: VersionState,
    search_state: SearchWidgetState,
    tabs_state: TabsWidgetState,
    tutorial_state: TutorialWidgetState,
    sprite_cache_state: SpriteWidgetState,
}

impl DexState {
    pub(crate) fn handle_key(&mut self, key_code: DexKeyCode, version: Version) -> InnerActionResult {
        match key_code {
            DexKeyCode::Char('.') => {
                self.version_state.toggle(version);
                return InnerActionResult::Nothing;
            }
            DexKeyCode::Char(':') => {
                self.search_state.start_search();
                return InnerActionResult::Nothing;
            }
            DexKeyCode::Char('/') => {
                self.tutorial_state.enabled = !self.tutorial_state.enabled;
                return InnerActionResult::Nothing;
            }
            _ => {}
        }

        if self.version_state.enabled {
            return self.version_state.handle_key(key_code);
        }

        if self.search_state.searching {
            return self.search_state.handle_key(key_code);
        }

        match key_code {
            DexKeyCode::Char('\'') => self.variant_cursor.next(),
            DexKeyCode::Char(';') => self.variant_cursor.prev(),
            _ => self.tabs_state.handle_key(key_code),
        }
        InnerActionResult::Nothing
    }

    pub(crate) fn reset(&mut self) {}
}
