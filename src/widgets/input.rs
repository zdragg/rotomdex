use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

pub struct InputWidget;

#[derive(Default)]
pub struct InputState {
    input: String,
}

impl InputState {
    /// Extracts the stored String and resets.
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.input)
    }

    /// Remove one character.
    pub fn backspace(&mut self) {
        self.input.pop();
    }

    /// Input one character.
    pub fn handle_input(&mut self, ch: char) {
        self.input.push(ch);
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }
}

impl StatefulWidget for InputWidget {
    type State = InputState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.is_empty() {
            return;
        }
        let area = Rect {
            x: (area.width.max(28) - 28) / 2,
            y: (area.height.max(3) - 3) / 2,
            width: 28,
            height: 3,
        }; // A fixed, centered rectangle of size 14x3. 14 = 12 (pokemon name) + borders, 3 = 1 + borders

        Clear.render(area, buf);
        let input = Paragraph::new(state.input.as_str()).block(Block::bordered().title("Input Pokémon name:"));
        input.render(area, buf);
    }
}
