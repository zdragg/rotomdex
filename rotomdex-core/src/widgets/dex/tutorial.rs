use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};

pub(super) struct TutorialWidget<'a> {
    state: &'a TutorialWidgetState,
}

impl<'a> TutorialWidget<'a> {
    pub(super) fn new(state: &'a TutorialWidgetState) -> Self {
        Self { state }
    }
}

const HEIGHT: u16 = 9;
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

        let [_, area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(HEIGHT)]).areas(area);
        let [_, area] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(WIDTH)]).areas(area);

        let commands = [
            " /          close",
            " :          search",
            " ;'         variants",
            " df         tabs",
            " hjkl ←↓↑→  navigate",
            " .          versions",
            " Ctrl+C     exit",
        ]
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();

        Clear.render(area, buf);
        Paragraph::new(commands)
            .block(Block::bordered().title("Keybinds").title_alignment(Alignment::Right))
            .render(area, buf);
    }
}
