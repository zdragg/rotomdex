mod abilities;
mod basic;
mod moveset;

use crate::Action;
use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::tabs::abilities::{AbilitiesTabWidget, AbilitiesTabWidgetState};
use crate::widgets::dex::tabs::basic::{BasicTabWidget, BasicTabWidgetState};
use crate::widgets::dex::tabs::moveset::{MovesetTabWidget, MovesetTabWidgetState};
use ratatui::layout::{Constraint, Layout};
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
        let [tab_area, content_area] =
            area.layout(&Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1));

        Tabs::new(DexTab::iter().map(|e| e.to_string()))
            .style(Color::White)
            .highlight_style(Style::default().black().on_white().bold())
            .select(self.state.selected_tab)
            .render(tab_area, buf);

        match DexTab::VARIANTS[self.state.selected_tab] {
            DexTab::Basic => BasicTabWidget::new(self.species, self.variant).render(content_area, buf),
            DexTab::Abilities => AbilitiesTabWidget::new(self.variant).render(content_area, buf),
            DexTab::Moveset => MovesetTabWidget::new(self.variant).render(content_area, buf),
        };
    }
}

#[derive(EnumCount, VariantArray, Display, EnumIter)]
enum DexTab {
    Basic,
    #[strum(to_string = "Abil.")]
    Abilities,
    #[strum(to_string = "Moves")]
    Moveset,
}

#[derive(Default)]
pub(crate) struct TabsWidgetState {
    selected_tab: usize,

    basic_state: BasicTabWidgetState,
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
            DexTab::Moveset => &mut self.moveset_state.handle_action(tab_action),
        };
    }
}
