use crate::model::ModelVariant;
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub(super) struct AbilitiesTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
}

impl<'a> AbilitiesTabWidget<'a> {
    pub(super) fn new(variant: Option<&'a ModelVariant>) -> Self {
        Self { variant }
    }
}

impl<'a> Widget for AbilitiesTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

#[derive(Default)]
pub(super) struct AbilitiesTabWidgetState {
    cursor: Cursor,
}

impl AbilitiesTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
