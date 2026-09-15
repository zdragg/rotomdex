use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Layout, Rect},
    macros::constraints,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::model::ModelSpecies;

pub(super) struct TutorialWidget<'a> {
    species: Option<&'a ModelSpecies>,
    state: &'a TutorialWidgetState,
}

impl<'a> TutorialWidget<'a> {
    pub(super) fn new(species: Option<&'a ModelSpecies>, state: &'a TutorialWidgetState) -> Self {
        Self { species, state }
    }
}

const fn new_span(str: &'static str) -> Span<'static> {
    Span {
        style: Style::new(),
        content: Cow::Borrowed(str),
    }
}

const ITEM_COUNT: usize = 9;

const KEYBINDS: [&str; ITEM_COUNT] = [
    " /         ",
    " :         ",
    " df        ",
    " vc        ",
    " g         ",
    " hjkl ←↓↑→ ",
    " .         ",
    " Enter     ",
    " Ctrl+C    ",
];

const DESCRIPTIONS: [&str; ITEM_COUNT] = [
    "close    ",
    "search   ",
    "variants ",
    "tabs     ",
    "animate  ",
    "navigate ",
    "versions ",
    "select   ",
    "exit     ",
];

const HEIGHT: u16 = ITEM_COUNT as u16 + 2;

const KEY_WIDTH: u16 = KEYBINDS[0].len() as u16;
const DESC_WIDTH: u16 = DESCRIPTIONS[0].len() as u16;
const BLOCK_WIDTH: u16 = KEY_WIDTH + DESC_WIDTH + 2;

const KEYBIND_SPANS: [Span; ITEM_COUNT] = konst::array::map!(KEYBINDS, new_span);
const DESCRIPTION_SPANS: [Span; ITEM_COUNT] = konst::array::map!(DESCRIPTIONS, new_span);

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
        let [_, area] = Layout::horizontal(constraints![*=1, ==BLOCK_WIDTH]).areas(area);

        let color = self.species.map_or(Color::Reset, |species| species.color);

        let commands = KEYBIND_SPANS
            .into_iter()
            .zip(DESCRIPTION_SPANS.into_iter())
            .map(|(key, desc)| Line::from(vec![key.style(color), desc]))
            .collect::<Vec<_>>();

        Clear.render(area, buf);
        Paragraph::new(commands)
            .block(Block::bordered().title("Keybinds").title_alignment(Alignment::Right))
            .render(area, buf);
    }
}
