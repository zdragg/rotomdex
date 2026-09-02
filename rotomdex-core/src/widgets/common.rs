use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{ListState, Paragraph, Widget};

pub(crate) struct HangingParagraph<'a> {
    text: &'a str,
    style: Style,
}

impl<'a> HangingParagraph<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            style: Style::default(),
        }
    }

    pub(crate) fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for HangingParagraph<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 {
            return;
        }

        let mut wrapped = String::new();
        let mut line_length = 0;

        for word in self.text.split_whitespace() {
            let word_length = word.chars().count();
            if line_length != 0 && line_length + 1 + word_length > area.width as usize {
                wrapped.push_str("\n│");
                line_length = 1;
            } else if line_length != 0 {
                wrapped.push(' ');
                line_length += 1;
            }
            wrapped.push_str(word);
            line_length += word_length;
        }

        Paragraph::new(wrapped).style(self.style).render(area, buf);
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct Cursor {
    idx: isize,
}

impl Cursor {
    pub(crate) fn select(&mut self, value: isize) {
        self.idx = value;
    }

    pub(crate) fn next(&mut self) {
        self.idx += 1;
    }

    pub(crate) fn prev(&mut self) {
        self.idx -= 1;
    }

    pub(crate) fn reset(&mut self) {
        self.idx = 0;
    }

    pub(crate) fn get(&self, total: usize) -> Option<usize> {
        self.idx.checked_rem_euclid(total as isize).map(|x| x as usize)
    }

    pub(crate) fn into_list_state(&self, total: usize) -> ListState {
        ListState::default().with_selected(self.get(total))
    }
}
