mod abilities;
mod moveset;
mod overview;

use crate::data::{ModelSpecies, ModelVariant};
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::abilities::{AbilitiesTabWidget, AbilitiesTabWidgetState};
use crate::widgets::dex::tabs::moveset::{MovesetTabWidget, MovesetTabWidgetState};
use crate::widgets::dex::tabs::overview::{OverviewTabWidget, OverviewTabWidgetState};
use crate::{Command, DexKeyCode};
use alloc::string::ToString;
use ratatui::layout::Layout;
use ratatui::macros::constraints;
use ratatui::style::{Color, Style};
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
        if self.variant.is_none() {
            return;
        }

        let [tab_area, content_area] =
            area.layout(&Layout::vertical(constraints![==1, *=1]).spacing(1));

        let selected_color = self.species.map_or(Color::Reset, |species| species.color);
        Tabs::new(DexTab::iter().map(|e| e.to_string()))
            .divider(" ")
            .style(Color::DarkGray)
            .highlight_style(Style::default().bold().patch(selected_color))
            .select(self.state.selected_tab.get(DexTab::COUNT))
            .render(tab_area, buf);

        match DexTab::VARIANTS[self.state.selected_tab.get(DexTab::COUNT).unwrap()] {
            DexTab::Overview => {
                OverviewTabWidget::new(self.species, self.variant, &self.state.overview_state)
                    .render(content_area, buf)
            }
            DexTab::Abilities => {
                AbilitiesTabWidget::new(self.species, self.variant).render(content_area, buf)
            }
            DexTab::Moveset => {
                MovesetTabWidget::new(self.variant, self.species, &self.state.moveset_state)
                    .render(content_area, buf)
            }
        };
    }
}

#[derive(EnumCount, VariantArray, Display, EnumIter)]
pub(crate) enum DexTab {
    Overview,
    Abilities,
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
    pub(super) fn handle_key(&mut self, key_code: DexKeyCode, cmd: &mut Option<Command>) {
        let tab_action = match key_code {
            DexKeyCode::Char('h') | DexKeyCode::Left => TabAction::Left,
            DexKeyCode::Char('j') | DexKeyCode::Down => TabAction::Down,
            DexKeyCode::Char('k') | DexKeyCode::Up => TabAction::Up,
            DexKeyCode::Char('l') | DexKeyCode::Right => TabAction::Right,
            DexKeyCode::Enter => TabAction::Enter,
            DexKeyCode::Escape | DexKeyCode::CapsLock => TabAction::Escape,
            DexKeyCode::Char('c') => {
                self.selected_tab.prev();
                return;
            }
            DexKeyCode::Char('v') => {
                self.selected_tab.next();
                return;
            }
            _ => {
                return;
            }
        };

        match DexTab::VARIANTS[self.selected_tab.get(DexTab::COUNT).unwrap()] {
            DexTab::Overview => self.overview_state.handle_action(tab_action, cmd),
            DexTab::Abilities => self.abilities_state.handle_action(tab_action),
            DexTab::Moveset => self.moveset_state.handle_action(tab_action),
        }
    }
}
