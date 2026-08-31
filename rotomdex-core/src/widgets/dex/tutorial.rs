use crate::widgets::dex::tabs::DexTab;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};
use strum::EnumCount;

pub(super) struct TutorialWidget<'a> {
    can_exit: bool,
    state: &'a TutorialWidgetState,
}

impl<'a> TutorialWidget<'a> {
    pub(super) fn new(can_exit: bool, state: &'a TutorialWidgetState) -> Self {
        Self { can_exit, state }
    }
}

#[derive(Default)]
pub(super) struct TutorialWidgetState {
    pub(super) enabled: bool,
}

impl<'a> Widget for TutorialWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.enabled {
            return;
        }

        let height = if self.can_exit { 8 } else { 7 };
        let [_, area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(height)]).areas(area);
        let [_, area] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(23)]).areas(area);

        let mut commands = [
            " /          close",
            " :          search",
            " d/f        variants",
            " hjkl/←↓↑→  navigate",
            const_format::concatcp!(" 1-", DexTab::COUNT, "        tabs"),
        ]
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
        if self.can_exit {
            commands.push(Line::from(" Ctrl+C     exit"));
        }

        Clear.render(area, buf);
        Paragraph::new(commands)
            .block(Block::bordered().title("Keybinds"))
            .render(area, buf);
    }
}
