use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use ratatui::widgets::ListState;

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
