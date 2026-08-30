use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::tabs::TabAction;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub(super) struct BasicTabWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
}

impl<'a> BasicTabWidget<'a> {
    pub(super) fn new(species: Option<&'a ModelSpecies>, variant: Option<&'a ModelVariant>) -> Self {
        Self { species, variant }
    }
}

impl<'a> Widget for BasicTabWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

#[derive(Default)]
pub(super) struct BasicTabWidgetState {}

impl BasicTabWidgetState {
    pub(super) fn handle_action(&mut self, _action: TabAction) {
        // Nothing to handle.
    }
}
