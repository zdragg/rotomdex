use colorgrad::{Gradient, LinearGradient};
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use ratatui::widgets::BlockExt;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::Widget,
};
use tui_big_text::{BigText, PixelSize};

use crate::model::{ModelSpecies, ModelVariant};

pub(crate) struct NameWidget<'a> {
    species: Option<&'a ModelSpecies>,
    variant: Option<&'a ModelVariant>,
}

impl<'a> NameWidget<'a> {
    pub(crate) fn new(species: Option<&'a ModelSpecies>, variant: Option<&'a ModelVariant>) -> Self {
        Self { species, variant }
    }
}

impl Widget for NameWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(species) = self.species else {
            return;
        };
        let Some(variant) = self.variant else {
            return;
        };
        let name = species.inner().name.to_uppercase();
        let line = Line::from(name.as_str());

        let Some((area, pixel_size)) = calculate_size(area, name.len()) else {
            let [area] = Layout::vertical([Constraint::Length(1)]).flex(Flex::Center).areas(area);
            line.centered().render(area, buf);
            return;
        };

        let big_text = BigText::builder().pixel_size(pixel_size).lines(&[line]).build();

        let text_area = big_text.block.inner_if_some(area);

        big_text.render(area, buf);

        paint_area(variant.types.gradient(), text_area, buf);
    }
}

fn paint_area(gradient: LinearGradient, area: Rect, buf: &mut Buffer) {
    let width = area.width;
    for (i, area) in area.columns().enumerate() {
        let fraction = i as f32 / width as f32;
        let [r, g, b, _a] = gradient.at(fraction).to_rgba8();
        buf.set_style(area, Color::Rgb(r, g, b))
    }
}

fn calculate_size(area: Rect, char_count: usize) -> Option<(Rect, PixelSize)> {
    let char_count = u16::try_from(char_count).ok()?;
    let full_width = char_count.checked_mul(8)?;
    let half_width = char_count.checked_mul(4)?;

    let glyph_width = if full_width <= area.width {
        8
    } else if half_width <= area.width {
        4
    } else {
        return None;
    };

    let glyph_height = match area.height {
        4.. => 4,
        3 => 3,
        2 => 2,
        _ => return None,
    };

    let pixel_size = match (glyph_width, glyph_height) {
        (8, 4) => PixelSize::HalfHeight,
        (8, 3) => PixelSize::ThirdHeight,
        (8, 2) => PixelSize::QuarterHeight,
        (4, 4) => PixelSize::Quadrant,
        (4, 3) => PixelSize::Sextant,
        (4, 2) => PixelSize::Octant,
        _ => unreachable!("thats all the possibilities"),
    };

    let text_width = glyph_width * char_count;
    let [area] = Layout::horizontal([Constraint::Length(text_width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(glyph_height)])
        .flex(Flex::Center)
        .areas(area);

    Some((area, pixel_size))
}
