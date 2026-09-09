use ratatui::prelude::{Color, Line};
use ratatui::text::Span;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::DexKeyCode;
use crate::widgets::dex::InnerActionResult;

pub(crate) struct SearchWidget<'a> {
    state: &'a SearchWidgetState,
}

impl<'a> SearchWidget<'a> {
    pub(crate) fn new(state: &'a SearchWidgetState) -> Self {
        Self { state }
    }
}

#[derive(Default)]
pub(crate) struct SearchWidgetState {
    pub(crate) searching: bool,
    input: String,
}

impl SearchWidgetState {
    pub(crate) fn handle_key(&mut self, key_code: DexKeyCode) -> InnerActionResult {
        if !self.searching {
            return InnerActionResult::Nothing;
        }
        match key_code {
            DexKeyCode::Char(ch) => self.handle_input(ch),
            DexKeyCode::Backspace => self.backspace(),
            DexKeyCode::Escape | DexKeyCode::CapsLock => self.abort_search(),
            DexKeyCode::Enter => return InnerActionResult::NewPokemon(self.take()),
            _ => (),
        }
        InnerActionResult::Nothing
    }

    pub(crate) fn start_search(&mut self) {
        self.searching = true;
    }

    pub(crate) fn abort_search(&mut self) {
        self.searching = false;
        self.input.clear();
    }

    /// Extracts the stored String and resets.
    pub(crate) fn take(&mut self) -> String {
        self.searching = false;
        std::mem::take(&mut self.input)
    }

    /// Remove one character.
    pub(crate) fn backspace(&mut self) {
        if let None = self.input.pop() {
            self.searching = false;
        };
    }

    /// Input one character.
    pub(crate) fn handle_input(&mut self, ch: char) {
        self.input.push(ch);
    }
}

impl<'a> Widget for SearchWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = const_format::concatcp!("Type / to see keybinds • Exit with Ctrl-C");

        let span = if self.state.searching {
            Span::raw(format!(" :{} ", self.state.input.as_str()))
        } else {
            Span::raw(format!(" {} ", text)).style(Color::DarkGray)
        };

        Line::from(span).centered().render(area, buf);
    }
}
