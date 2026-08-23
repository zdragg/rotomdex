use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

pub(crate) struct InputWidget;

#[derive(Default)]
pub(crate) struct InputState {
    input: String,
}

impl InputState {
    /// Extracts the stored String and resets.
    pub(crate) fn take(&mut self) -> String {
        std::mem::take(&mut self.input)
    }

    /// Remove one character.
    pub(crate) fn backspace(&mut self) {
        self.input.pop();
    }

    /// Input one character.
    pub(crate) fn handle_input(&mut self, ch: char) {
        self.input.push(ch);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.input.is_empty()
    }
}

impl StatefulWidget for InputWidget {
    type State = InputState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.is_empty() {
            return;
        }
        let area = area.centered(Constraint::Length(28), Constraint::Length(3));

        Clear.render(area, buf);
        let input = Paragraph::new(state.input.as_str()).block(Block::bordered().title("Enter Pokémon name:"));
        input.render(area, buf);
    }
}
