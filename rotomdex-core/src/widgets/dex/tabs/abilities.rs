use crate::model::ModelVariant;
use crate::widgets::dex::Cursor;
use crate::widgets::dex::tabs::TabAction;

pub(crate) struct AbilitiesTabWidget<'a> {
    variant: Option<&'a ModelVariant>,
}

#[derive(Default)]
pub(crate) struct AbilitiesTabWidgetState {
    cursor: Cursor,
}

impl AbilitiesTabWidgetState {
    pub(super) fn handle_action(&mut self, action: TabAction) {}
}
