use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Layout, Rect},
    macros::constraints,
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
    " /          close ",
    " :          search ",
    " df         variants ",
    " vc         tabs ",
    " g          animate ",
    " hjkl ←↓↑→  navigate ",
    " .          versions ",
    " enter      select ",
    " ctrl+c     exit ",
];
const HEIGHT: u16 = COMMANDS.len() as u16 + 2;
const WIDTH: u16 = {
    let mut max_len = 0;

    konst::iter::for_each! {string in COMMANDS =>
        let chars = konst::string::chars(string);
        let len = konst::iter::eval!(chars, count());
        if len > max_len { max_len = len }
    }

    max_len as u16 + 2
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

        let [_, area] = Layout::vertical(constraints![*=1, ==HEIGHT]).areas(area);
        let [_, area] = Layout::horizontal(constraints![*=1, ==WIDTH]).areas(area);

        let commands = COMMANDS.into_iter().map(Line::from).collect::<Vec<_>>();

        Clear.render(area, buf);
        Paragraph::new(commands)
            .block(Block::bordered().title("Keybinds").title_alignment(Alignment::Right))
            .render(area, buf);
    }
}
