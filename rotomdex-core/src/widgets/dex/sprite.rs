use std::{char, time::Duration};

use chafa_syms_rs::{Canvas, CanvasConfig, CanvasMode, PixelType};
use ratatui::{prelude::*, widgets::Widget};

use crate::{model::ModelSprite, widgets::WidgetExt};

pub(crate) struct SpriteWidget<'a> {
    sprite: &'a ModelSprite,
    elapsed: Duration,
}

impl<'a> WidgetExt<(&'a ModelSprite, Duration)> for SpriteWidget<'a> {
    fn new((sprite, elapsed): (&'a ModelSprite, Duration)) -> Self {
        Self { sprite, elapsed }
    }
}

impl<'a> Widget for SpriteWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let side_width = area.width.min(area.height * 2);
        let side_height = side_width / 2;
        let area = area.inner(Margin::new(
            (area.width - side_width) / 2,
            (area.height - side_height) / 2,
        ));

        let Some(sprite) = self
            .sprite
            .animated()
            .map(|anim| anim.frame_at(self.elapsed))
            .or(self.sprite.image())
        else {
            return; // TODO: handle missing sprite case. maybe render some kind of image that shows that there is no sprite
        };

        let cfg = CanvasConfig::new(area.width as usize, area.height as usize).mode(CanvasMode::Truecolor);
        let mut canvas = Canvas::new(cfg);
        canvas.draw_all_pixels(
            PixelType::Rgba8,
            sprite.as_raw(),
            sprite.width() as usize,
            sprite.height() as usize,
            sprite.width() as usize * 4,
        );

        for (idx, source) in canvas.cells().iter().enumerate() {
            let (fg_alpha, fg_color) = aarrggbb_to_color(source.fg);
            let (bg_alpha, bg_color) = aarrggbb_to_color(source.bg);
            let ch = char::from_u32(source.c).unwrap_or(' ');
            let x = area.x + idx as u16 % area.width;
            let y = area.y + idx as u16 / area.width;
            if let Some(target) = buf.cell_mut((x, y)) {
                match (fg_alpha < 128, bg_alpha < 128) {
                    (true, true) => target.set_char(' '),
                    (true, false) => target.set_char(ch).set_fg(bg_color).set_style(Modifier::REVERSED),
                    (false, true) => target.set_char(ch).set_fg(fg_color),
                    (false, false) => target.set_char(ch).set_fg(fg_color).set_bg(bg_color),
                };
            }
        }
    }
}

fn aarrggbb_to_color(color: u32) -> (u8, Color) {
    let a = (color >> 24) as u8;
    let r = (color >> 16) as u8;
    let g = (color >> 8) as u8;
    let b = color as u8;
    (a, Color::Rgb(r, g, b))
}
