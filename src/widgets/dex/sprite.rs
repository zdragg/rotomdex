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

pub struct SpriteWidget<'a> {
    pub sprite: &'a DynamicImage,
    pub side_len: u16,
}

impl<'a> SpriteWidget<'a> {
    pub fn new(sprite: &'a DynamicImage, side_len: u16) -> Self {
        Self { sprite, side_len }
    }
}

impl<'a> Widget for &SpriteWidget<'a> {
    // Renders the pokemon sprite. The side length needs to be equal to h * 2, or how many "pixels" the height can show.
    // Since the sprite is square, the pixel count of the height would also be the pixel count of the width.
    fn render(self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .x_bounds([0.0, self.side_len as f64])
            .y_bounds([0.0, self.side_len as f64])
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&RotomDexSprite {
                    sprite: self.sprite,
                    side_len: self.side_len,
                });
            })
            .render(area, buf);
    }
}

struct RotomDexSprite<'a> {
    sprite: &'a DynamicImage,
    side_len: u16,
}

impl<'a> Shape for RotomDexSprite<'a> {
    fn draw(&self, painter: &mut Painter) {
        let sprite = self.sprite.resize(
            self.side_len as u32,
            self.side_len as u32,
            FilterType::Nearest,
        );
        for (x, y, color) in sprite.pixels() {
            if color.alpha() < 128 {
                continue;
            }
            let (canvas_x, canvas_y) = (x, self.side_len as u32 - y - 1); // If it works don't touch it. something about coordinates not being same
            if let Some((x, y)) = painter.get_point(canvas_x as f64, canvas_y as f64) {
                let color = Color::Rgb(color.0[0], color.0[1], color.0[2]);
                painter.paint(x, y, color);
            }
        }
    }
}
