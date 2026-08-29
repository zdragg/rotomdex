use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::TabAction;

pub(crate) struct LearnsetTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
}

#[derive(Default)]
pub(crate) struct LearnsetTabWidgetState {
    cursor: Cursor,
}

impl LearnsetTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
