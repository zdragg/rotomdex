use ratatui::prelude::*;

pub struct NameWidget {
    text: String,
}

impl NameWidget {
    pub fn new(text: &str) -> Self {
        let text = text.to_uppercase();
        Self { text }
    }
}

impl Widget for &NameWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}
