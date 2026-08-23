use std::time::Duration;

use image::{
    Pixel, RgbaImage,
    imageops::{self, FilterType},
};
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
    elapsed: Duration,
}

impl<'a> SpriteWidget<'a> {
    pub(crate) fn new(sprite: &'a ModelSprite, elapsed: Duration) -> Self {
        Self { sprite, elapsed }
    }
}

impl<'a> Widget for SpriteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(sprite) = self
            .sprite
            .animated()
            .map(|anim| anim.frame_at(self.elapsed))
            .or(self.sprite.image())
        else {
            return; // TODO: handle missing sprite case. maybe render some kind of image that shows that there is no sprite
        };
        let side_len = area.width.min(area.height.saturating_mul(2));
        let side_len = side_len - side_len % 2;
        if side_len < 2 {
            return;
        }

        let area = area.centered(Constraint::Length(side_len), Constraint::Length(side_len / 2));
        let sprite: RgbaImage = imageops::resize(sprite, side_len.into(), side_len.into(), FilterType::Nearest);
        let max = (side_len - 1) as f64;

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

struct RotomDexSprite<'a>(&'a RgbaImage);

impl<'a> Shape for RotomDexSprite<'a> {
    fn draw(&self, painter: &mut Painter) {
        let height = self.0.height();

        for (x, y, color) in self.0.enumerate_pixels() {
            if color.alpha() == 0 {
                continue;
            }
            if let Some((x, y)) = painter.get_point(x.into(), (height - y - 1).into()) {
                let color = Color::Rgb(color.0[0], color.0[1], color.0[2]);
                painter.paint(x, y, color);
            }
        }
    }
}
