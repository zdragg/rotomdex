mod abilities;
mod moveset;
mod overview;

use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::abilities::{AbilitiesTabWidget, AbilitiesTabWidgetState};
use crate::widgets::dex::tabs::moveset::{MovesetTabWidget, MovesetTabWidgetState};
use crate::widgets::dex::tabs::overview::{OverviewTabWidget, OverviewTabWidgetState};
use crate::{DexKeyCode, InnerActionResult};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::Tabs;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator, VariantArray};

pub(super) struct TabsWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
    state: &'a TabsWidgetState,
}

impl<'a> TabsWidget<'a> {
    pub(super) fn new(
        species: Option<&'a ModelSpecies>,
        variant: Option<&'a ModelVariant>,
        state: &'a TabsWidgetState,
    ) -> Self {
        Self {
            species,
            variant,
            state,
        }
    }
}

impl Widget for TabsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if matches!(self.variant, None) {
            return;
        }
        let [tab_area, content_area] =
            area.layout(&Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1));

        Tabs::new(DexTab::iter().map(|e| e.to_string()))
            .highlight_style(Style::default().black().on_white().bold())
            .select(self.state.selected_tab.get(DexTab::COUNT))
            .render(tab_area, buf);

        match DexTab::VARIANTS[self.state.selected_tab.get(DexTab::COUNT).unwrap()] {
            DexTab::Overview => {
                OverviewTabWidget::new(self.species, self.variant, &self.state.overview_state).render(content_area, buf)
            }
            DexTab::Abilities => AbilitiesTabWidget::new(self.variant).render(content_area, buf),
            DexTab::Moveset => MovesetTabWidget::new(self.variant, &self.state.moveset_state).render(content_area, buf),
        };
    }
}

#[derive(EnumCount, VariantArray, Display, EnumIter)]
pub(crate) enum DexTab {
    #[strum(to_string = "Ovw.")]
    Overview,
    #[strum(to_string = "Abil.")]
    Abilities,
    #[strum(to_string = "Moves")]
    Moveset,
}

#[derive(Default)]
pub(crate) struct TabsWidgetState {
    selected_tab: Cursor,

    overview_state: OverviewTabWidgetState,
    abilities_state: AbilitiesTabWidgetState,
    moveset_state: MovesetTabWidgetState,
}

enum TabAction {
    Left,
    Down,
    Up,
    Right,
    Enter,
    Escape,
}

impl TabsWidgetState {
    pub(super) fn handle_key(&mut self, key_code: DexKeyCode) -> InnerActionResult {
        let tab_action = match key_code {
            DexKeyCode::Char('h') | DexKeyCode::Left => TabAction::Left,
            DexKeyCode::Char('j') | DexKeyCode::Down => TabAction::Down,
            DexKeyCode::Char('k') | DexKeyCode::Up => TabAction::Up,
            DexKeyCode::Char('l') | DexKeyCode::Right => TabAction::Right,
            DexKeyCode::Enter => TabAction::Enter,
            DexKeyCode::Escape | DexKeyCode::CapsLock => TabAction::Escape,
            DexKeyCode::Char('d') => {
                self.selected_tab.prev();
                return InnerActionResult::Nothing;
            }
            DexKeyCode::Char('f') => {
                self.selected_tab.next();
                return InnerActionResult::Nothing;
            }
            _ => return InnerActionResult::Nothing,
        };

        let result = match DexTab::VARIANTS[self.selected_tab.get(DexTab::COUNT).unwrap()] {
            DexTab::Overview => self.overview_state.handle_action(tab_action),
            DexTab::Abilities => self.abilities_state.handle_action(tab_action),
            DexTab::Moveset => self.moveset_state.handle_action(tab_action),
        };
        result
    }
}
