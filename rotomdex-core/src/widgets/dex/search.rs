use ratatui::prelude::{Color, Line};
use ratatui::text::Span;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::Action;
use crate::widgets::dex::ActionResult;

pub(crate) struct SearchWidget<'a> {
    state: &'a SearchWidgetState,
    can_exit: bool,
}

impl<'a> SearchWidget<'a> {
    pub(crate) fn new(state: &'a SearchWidgetState, can_exit: bool) -> Self {
        Self { state, can_exit }
    }
}

#[derive(Default)]
pub(crate) struct SearchWidgetState {
    pub(crate) searching: bool,
    input: String,
}

impl SearchWidgetState {
    pub(crate) fn handle_action(&mut self, action: Action) -> ActionResult {
        if !self.searching {
            return ActionResult::Nothing;
        }
        match action {
            Action::Input(ch) => self.handle_input(ch),
            Action::Backspace => self.backspace(),
            Action::Escape | Action::CapsLock => self.abort_search(),
            Action::Enter => return ActionResult::NewPokemon(self.take()),
            _ => (),
        }
        ActionResult::Nothing
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
        let text = if self.can_exit {
            const_format::concatcp!("Type / to see keybinds • Exit with Ctrl-C",)
        } else {
            "Type / to see keybinds"
        };

        let span = if self.state.searching {
            Span::raw(format!(" :{} ", self.state.input.as_str())).style(Color::White)
        } else {
            Span::raw(format!(" {} ", text)).style(Color::DarkGray)
        };

        Line::from(span).centered().render(area, buf);
    }
}
