use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};

pub(super) struct TutorialWidget<'a> {
    can_exit: bool,
    state: &'a TutorialWidgetState,
}

impl<'a> TutorialWidget<'a> {
    pub(super) fn new(can_exit: bool, state: &'a TutorialWidgetState) -> Self {
        Self { can_exit, state }
    }
}

const HEIGHT_WITHOUT_EXIT: u16 = 8;
const HEIGHT_WITH_EXIT: u16 = 9;
const WIDTH: u16 = 23;

#[derive(Default)]
pub(super) struct TutorialWidgetState {
    pub(super) enabled: bool,
}

impl<'a> Widget for TutorialWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.enabled {
            return;
        }

        let height = if self.can_exit {
            HEIGHT_WITH_EXIT
        } else {
            HEIGHT_WITHOUT_EXIT
        };
        let [_, area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(height)]).areas(area);
        let [_, area] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(WIDTH)]).areas(area);

        let mut commands = [
            " /          close",
            " :          search",
            " ;'         variants",
            " df         tabs",
            " hjkl ←↓↑→  navigate",
            " .          versions",
        ]
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
        if self.can_exit {
            commands.push(Line::from(" Ctrl+C     exit"));
        }

        Clear.render(area, buf);
        Paragraph::new(commands)
            .block(Block::bordered().title("Keybinds").title_alignment(Alignment::Right))
            .render(area, buf);
    }
}
