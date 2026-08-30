use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::TabAction;

pub(crate) struct MovesetTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
}

#[derive(Default)]
pub(crate) struct MovesetTabWidgetState {
    cursor: Cursor,
}

impl MovesetTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
