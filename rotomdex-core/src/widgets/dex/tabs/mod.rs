use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::model::{ModelSpecies, ModelVariant};

pub(crate) struct TabWidget<'a> {
    _species: Option<&'a ModelSpecies>,
    _variant: Option<&'a ModelVariant>,
}

impl<'a> TabWidget<'a> {
    pub(crate) fn new(species: Option<&'a ModelSpecies>, variant: Option<&'a ModelVariant>) -> Self {
        Self {
            _species: species,
            _variant: variant,
        }
    }
}

impl Widget for TabWidget<'_> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
