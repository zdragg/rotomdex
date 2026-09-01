mod name;
mod search;
mod sprite;
mod stats;
mod tabs;
mod tutorial;
mod variant;
mod versions;

use crate::widgets::dex::search::{SearchWidget, SearchWidgetState};
use crate::widgets::dex::tabs::TabsWidgetState;
use crate::widgets::dex::tutorial::{TutorialWidget, TutorialWidgetState};
use crate::widgets::dex::versions::{VersionState, VersionWidget};
use crate::{
    Action,
    model::ModelPokemon,
    widgets::dex::{
        name::NameWidget, sprite::SpriteWidget, stats::StatsWidget, tabs::TabsWidget, variant::VariantSelectorWidget,
    },
};
use crate::{ActionResult, Version};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, StatefulWidget, Widget},
};
use std::time::Duration;

pub(crate) struct DexWidget<'a> {
    pkmn: &'a ModelPokemon,
    elapsed: Duration,
    can_exit: bool,
    version: Version,

    state: &'a mut DexState,
}

impl<'a> DexWidget<'a> {
    pub(crate) fn new(
        pkmn: &'a ModelPokemon,
        state: &'a mut DexState,
        elapsed: Duration,
        can_exit: bool,
        version: Version,
    ) -> Self {
        Self {
            pkmn,
            elapsed,
            can_exit,
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
        SearchWidget::new(&self.state.search_state, self.can_exit).render(bottom_text_area, buf);

        let [left_area, right_area] = Layout::horizontal([Constraint::Percentage(35), Constraint::Fill(1)])
            .spacing(1)
            .areas(area);
        let [sprite_area, stats_area] = Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)])
            .spacing(1)
            .areas(left_area);
        let [name_area, variants_area, tab_area] =
            Layout::vertical([Constraint::Percentage(20), Constraint::Length(2), Constraint::Fill(1)])
                .areas(right_area);

        SpriteWidget::new(variant, self.elapsed).render(sprite_area, buf);
        StatsWidget::new(species, variant).render(stats_area, buf);
        NameWidget::new(species, variant).render(name_area, buf);
        VariantSelectorWidget::new(species, variant_idx).render(variants_area, buf);
        TabsWidget::new(species, variant, &self.state.tabs_state, self.elapsed).render(tab_area, buf);
        TutorialWidget::new(self.can_exit, &self.state.tutorial_state).render(area, buf);
        VersionWidget::new(self.version).render(stats_area, buf, &mut self.state.version_state);
    }
}

#[derive(Default)]
pub(crate) struct DexState {
    variant_cursor: Cursor,

    version_state: VersionState,
    search_state: SearchWidgetState,
    tabs_state: TabsWidgetState,
    tutorial_state: TutorialWidgetState,
}

impl DexState {
    pub(crate) fn handle_action(&mut self, action: Action, version: Version) -> ActionResult {
        match action {
            Action::Input('.') => {
                self.version_state.toggle(version);
                return ActionResult::Nothing;
            }
            Action::Input(':') => {
                self.search_state.start_search();
                return ActionResult::Nothing;
            }
            Action::Input('/') => {
                self.tutorial_state.enabled = !self.tutorial_state.enabled;
                return ActionResult::Nothing;
            }
            _ => {}
        }

        if self.version_state.enabled {
            return self.version_state.handle_action(action);
        }

        if self.search_state.searching {
            return self.search_state.handle_action(action);
        }

        match action {
            Action::Input('\'') => self.variant_cursor.next(),
            Action::Input(';') => self.variant_cursor.prev(),
            _ => self.tabs_state.handle_action(action),
        }
        ActionResult::Nothing
    }

    pub(crate) fn reset(&mut self) {
        self.variant_cursor.reset();
    }
}

#[derive(Default)]
struct Cursor {
    idx: isize,
}

impl Cursor {
    fn next(&mut self) {
        self.idx += 1;
    }

    fn prev(&mut self) {
        self.idx -= 1;
    }

    fn reset(&mut self) {
        self.idx = 0;
    }

    fn get(&self, total: usize) -> Option<usize> {
        self.idx.checked_rem_euclid(total as isize).map(|x| x as usize)
    }
}
