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

const COMMANDS: [&'static str; 9] = [
    " /          close",
    " :          search",
    " ;'         variants",
    " df         tabs",
    " g          animate",
    " hjkl ←↓↑→  navigate",
    " .          versions",
    " enter      select",
    " ctrl+c     exit",
];
const HEIGHT: u16 = COMMANDS.len() as u16 + 2;
const WIDTH: u16 = {
    let mut max_len = 0;
    let mut i = 0;

    while i < COMMANDS.len() {
        let cmd_len = COMMANDS[i].len() as u16;
        if cmd_len > max_len {
            max_len = cmd_len;
        }
        i += 1;
    }

    max_len
};

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

        let commands = COMMANDS.into_iter().map(Line::from).collect::<Vec<_>>();

        Clear.render(area, buf);
        Paragraph::new(commands)
            .block(Block::bordered().title("Keybinds").title_alignment(Alignment::Right))
            .render(area, buf);
    }
}
