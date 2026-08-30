use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub(super) struct MovesetTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
}

impl<'a> MovesetTabWidget<'a> {
    pub(super) fn new(variant: Option<&'a ModelVariant>) -> Self {
        Self { variant }
    }
}

impl<'a> Widget for MovesetTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

#[derive(Default)]
pub(super) struct MovesetTabWidgetState {
    cursor: Cursor,
}

impl MovesetTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
