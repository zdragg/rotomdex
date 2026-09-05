use std::{cell::RefCell, char, num::NonZeroUsize, rc::Rc, time::Duration};

use chafa_syms_rs::{Canvas, CanvasConfig, CanvasMode, CellOut, PixelType};
use image::RgbaImage;
use lru::LruCache;
use rapidhash::v3::rapidhash_v3;
use ratatui::{prelude::*, widgets::Widget};

use crate::model::ModelVariant;

pub(crate) struct SpriteWidget<'a> {
    variant: Option<&'a ModelVariant>,
    elapsed: Duration,

    state: &'a SpriteWidgetState,
}

impl<'a> SpriteWidget<'a> {
    pub(crate) fn new(variant: Option<&'a ModelVariant>, elapsed: Duration, state: &'a SpriteWidgetState) -> Self {
        Self {
            variant,
            elapsed,
            state,
        }
    }
}

impl Widget for SpriteWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(sprite) = self.variant.and_then(|variant| variant.sprite.as_loaded()) else {
            return;
        };

        let side_width = area.width.min(area.height * 2);
        let side_height = side_width / 2;
        let area = area.inner(Margin::new(
            (area.width - side_width) / 2,
            (area.height - side_height) / 2,
        ));

        let Some(sprite) = sprite
            .animated()
            .map(|anim| anim.frame_at(self.elapsed))
            .or(sprite.image())
        else {
            return;
        };

        let cells = self.state.render_with_cache(sprite, area.width, area.height);

        for (idx, source) in cells.iter().enumerate() {
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

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    image_hash: u64,
    target_width: u16,
    target_height: u16,
}

pub(super) struct SpriteWidgetState {
    cache: RefCell<LruCache<CacheKey, Rc<[CellOut]>>>,
}

impl Default for SpriteWidgetState {
    fn default() -> Self {
        Self {
            cache: RefCell::new(LruCache::new(NonZeroUsize::new(128).unwrap())),
        }
    }
}

impl SpriteWidgetState {
    fn render_with_cache(&self, image: &RgbaImage, target_width: u16, target_height: u16) -> Rc<[CellOut]> {
        let key = CacheKey {
            image_hash: rapidhash_v3(image.as_raw()),
            target_width,
            target_height,
        };

        if let Some(cells) = self.cache.borrow_mut().get(&key).cloned() {
            return cells;
        }

        let cfg = CanvasConfig::new(target_width as usize, target_height as usize).mode(CanvasMode::Truecolor);
        let mut canvas = Canvas::new(cfg);
        canvas.draw_all_pixels(
            PixelType::Rgba8,
            image.as_raw(),
            image.width() as usize,
            image.height() as usize,
            image.width() as usize * 4,
        );

        let cells: Rc<[CellOut]> = Rc::from(canvas.cells());

        self.cache.borrow_mut().put(key, cells.clone());

        cells
    }
}
