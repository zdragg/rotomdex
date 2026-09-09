use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, ListState, Widget};

#[derive(Default, Clone)]
pub(crate) struct Cursor {
    selected: isize,
    offset: Cell<usize>,
}

impl Cursor {
    pub(crate) fn select(&mut self, value: isize) {
        self.selected = value;
    }

    pub(crate) fn next(&mut self) {
        self.selected += 1;
    }

    pub(crate) fn prev(&mut self) {
        self.selected -= 1;
    }

    pub(crate) fn reset(&mut self) {
        self.selected = 0;
    }

    pub(crate) fn get(&self, total: usize) -> Option<usize> {
        self.selected.checked_rem_euclid(total as isize).map(|x| x as usize)
    }

    pub(crate) fn list_state(&self, total: usize) -> impl DerefMut<Target = ListState> {
        CursorListStateGuard {
            offset: &self.offset,
            list_state: ListState::default()
                .with_selected(self.get(total))
                .with_offset(self.offset.get()),
        }
    }
}

struct CursorListStateGuard<'a> {
    offset: &'a Cell<usize>,
    list_state: ListState,
}

impl Deref for CursorListStateGuard<'_> {
    type Target = ListState;
    fn deref(&self) -> &Self::Target {
        &self.list_state
    }
}

impl DerefMut for CursorListStateGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list_state
    }
}

impl Drop for CursorListStateGuard<'_> {
    fn drop(&mut self) {
        self.offset.set(self.list_state.offset());
    }
}

pub(crate) trait RenderBlockExt {
    /// Renders provided block and returns the inner area
    #[must_use = "Must use the calculated inner Rect. Use Block::render() otherwise"]
    fn render_inner(self, area: Rect, buf: &mut Buffer) -> Rect;
}

impl RenderBlockExt for Block<'_> {
    fn render_inner(self, area: Rect, buf: &mut Buffer) -> Rect {
        let inner = self.inner(area);
        self.render(area, buf);
        inner
    }
}
