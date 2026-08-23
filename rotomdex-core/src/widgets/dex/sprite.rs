use image::{DynamicImage, GenericImageView, Pixel, imageops::FilterType};
use ratatui::{
    prelude::*,
    style::Color,
    symbols::Marker,
    widgets::{
        Widget,
        canvas::{Canvas, Painter, Shape},
    },
};

use crate::model::ModelSprite;

pub(crate) struct SpriteWidget<'a> {
    sprite: &'a ModelSprite,
}

impl<'a> SpriteWidget<'a> {
    pub(crate) fn new(sprite: &'a ModelSprite) -> Self {
        Self { sprite }
    }
}

impl<'a> Widget for SpriteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(sprite) = &self.sprite.sprite else {
            return; // TODO: handle missing sprite case. maybe render some kind of image that shows that there is no sprite
        };
        let side_len = area.width.min(area.height.saturating_mul(2));
        let side_len = side_len - side_len % 2;
        if side_len < 2 {
            return;
        }

        let area = area.centered(Constraint::Length(side_len), Constraint::Length(side_len / 2));
        let sprite = sprite.resize_exact(side_len.into(), side_len.into(), FilterType::Nearest);
        let max = f64::from(side_len - 1);

        Canvas::default()
            .x_bounds([0.0, max])
            .y_bounds([0.0, max])
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&RotomDexSprite(&sprite));
            })
            .render(area, buf);
    }
}

struct RotomDexSprite<'a>(&'a DynamicImage);

impl<'a> Shape for RotomDexSprite<'a> {
    fn draw(&self, painter: &mut Painter) {
        let height = self.0.height();

        for (x, y, color) in self.0.pixels() {
            if color.alpha() < 128 {
                continue;
            }
            if let Some((x, y)) = painter.get_point(x.into(), (height - y - 1).into()) {
                let color = Color::Rgb(color.0[0], color.0[1], color.0[2]);
                painter.paint(x, y, color);
            }
        }
    }
}
