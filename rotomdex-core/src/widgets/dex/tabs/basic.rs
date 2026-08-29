use crate::model::{ModelSpecies, ModelVariant};
use crate::widgets::dex::tabs::TabAction;

pub(crate) struct BasicTabWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
}

#[derive(Default)]
pub(crate) struct BasicTabWidgetState {}

impl BasicTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
