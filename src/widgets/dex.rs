use std::sync::Arc;

use image::{DynamicImage, GenericImageView, Pixel};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        Block, Widget,
        canvas::{Canvas, Painter, Shape},
    },
};
use rustemon::client::RustemonClient;

use crate::pokemon::OfflinePokemon;

#[derive(Default)]
pub struct RotomDexWidget {
    pub rustemon_client: Arc<RustemonClient>,
    pub pokemon: Option<OfflinePokemon>,
}

impl Widget for &RotomDexWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(pkmn) = &self.pokemon else {
            return;
        };

        let variant = pkmn.get_current_pkmn();
        let sprite = pkmn.get_current_sprite();

        let block = Block::new().title(variant.name.clone());

        let (w, h) = (area.width, area.height);

        let [sprite_area, text_area] =
            Layout::horizontal([Constraint::Length(h * 2), Constraint::Fill(1)])
                .areas(block.inner(area));

        // Renders the pokemon sprite. The side length needs to be equal to h * 2, or how many "pixels" the height can show.
        // Since the sprite is square, the pixel count of the height would also be the pixel count of the width.
        if let Some(sprite) = sprite {
            Canvas::default()
                .x_bounds([0.0, (h * 2) as f64])
                .y_bounds([0.0, (h * 2) as f64])
                .marker(Marker::HalfBlock)
                .paint(|ctx| {
                    ctx.draw(&RotomDexSprite {
                        sprite,
                        side_length: h * 2,
                    });
                })
                .render(sprite_area, buf);
        }

        block.render(area, buf);
    }
}

struct RotomDexSprite<'a> {
    sprite: &'a DynamicImage,
    side_length: u16,
}

impl<'a> Shape for RotomDexSprite<'a> {
    fn draw(&self, painter: &mut Painter) {
        let sprite = self.sprite.resize(
            self.side_length as u32,
            self.side_length as u32,
            image::imageops::FilterType::Nearest,
        );
        for (x, y, color) in sprite.pixels() {
            if color.alpha() == 0 {
                continue;
            }
            let (canvas_x, canvas_y) = (x, self.side_length as u32 - y); // If it works don't touch it. something about coordinates not being same
            if let Some((x, y)) = painter.get_point(canvas_x as f64, canvas_y as f64) {
                let color = Color::Rgb(color.0[0], color.0[1], color.0[2]);
                painter.paint(x as usize, y as usize, color);
            }
        }
    }
}
