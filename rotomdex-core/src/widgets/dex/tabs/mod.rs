mod abilities;
mod basic;
mod learnset;

use crate::Action;
use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::tabs::abilities::AbilitiesTabWidgetState;
use crate::widgets::dex::tabs::basic::BasicTabWidgetState;
use crate::widgets::dex::tabs::learnset::LearnsetTabWidgetState;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use strum::{EnumCount, VariantArray};

pub(crate) struct TabsWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
    state: &'a TabsWidgetState,
}

impl<'a> TabsWidget<'a> {
    pub(crate) fn new(
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
        todo!()
    }
}

#[derive(EnumCount, VariantArray)]
enum DexTab {
    Basic,
    Abilities,
    Learnset,
}

#[derive(Default)]
pub(crate) struct TabsWidgetState {
    selected_tab: usize,

    basic_state: BasicTabWidgetState,
    abilities_state: AbilitiesTabWidgetState,
    learnset_state: LearnsetTabWidgetState,
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
    pub(super) fn handle_action(&mut self, action: Action) {
        let tab_action = match action {
            Action::Input('h') | Action::Left => TabAction::Left,
            Action::Input('j') | Action::Down => TabAction::Down,
            Action::Input('k') | Action::Up => TabAction::Up,
            Action::Input('l') | Action::Right => TabAction::Right,
            Action::Enter => TabAction::Enter,
            Action::Escape | Action::CapsLock => TabAction::Escape,
            Action::Input(digit @ '1'..='9') => {
                let index = (digit as u8 - b'1') as usize; // 0..=8
                if index < DexTab::COUNT {
                    self.selected_tab = index;
                }
                return;
            }
            _ => return,
        };

        match DexTab::VARIANTS[self.selected_tab] {
            DexTab::Basic => &mut self.basic_state.handle_action(tab_action),
            DexTab::Abilities => &mut self.abilities_state.handle_action(tab_action),
            DexTab::Learnset => &mut self.learnset_state.handle_action(tab_action),
        };
    }
}
