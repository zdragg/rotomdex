use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Clear, Paragraph, Widget},
};
use smart_default::SmartDefault;

#[derive(SmartDefault)]
pub struct InputWidget {
    #[default = "rotom"]
    input: String,
    pub input_mode: InputMode,
}

impl InputWidget {
    /// Extracts the stored String and resets.
    pub fn take(&mut self) -> String {
        let str = std::mem::take(&mut self.input);
        self.input_mode = InputMode::Idle;
        str
    }

    /// Remove one character.
    pub fn backspace(&mut self) {
        self.input.pop();
        if self.input.is_empty() {
            self.input_mode = InputMode::Idle;
        };
    }

    /// Input one character.
    pub fn handle_input(&mut self, ch: char) {
        self.input.push(ch);
        self.input_mode = InputMode::Editing;
    }
}

/// Idle = no words, Editing = has words
#[derive(Default, PartialEq)]
pub enum InputMode {
    #[default]
    Idle,
    Editing,
}

impl Widget for &InputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.input_mode == InputMode::Idle {
            return;
        }
        let area = Rect {
            x: (area.width.max(28) - 28) / 2,
            y: (area.height.max(3) - 3) / 2,
            width: 28,
            height: 3,
        }; // A fixed, centered rectangle of size 14x3. 14 = 12 (pokemon name) + borders, 3 = 1 + borders

        Clear.render(area, buf);
        let input = Paragraph::new(self.input.clone())
            .block(Block::bordered().title("Input Pokémon name:"));
        input.render(area, buf);
    }
}
