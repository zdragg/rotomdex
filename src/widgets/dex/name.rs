use ratatui::prelude::*;
use smart_default::SmartDefault;
use swash::{
    CacheKey, FontRef,
    scale::{Render, ScaleContext, Source},
    shape::ShapeContext,
};

pub struct NameWidget<'a> {
    text: String,
    state: &'a mut TitleState,
}

impl<'a> NameWidget<'a> {
    pub fn new(text: &'a str, state: &'a mut TitleState) -> Self {
        let text = text.to_uppercase();
        Self { text, state }
    }
}

#[derive(SmartDefault)]
pub struct TitleState {
    shape_context: ShapeContext,
    scale_context: ScaleContext,
    #[default(_code = "Font::new()")]
    font: Font,
}

pub struct Font {
    data: Vec<u8>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    fn new() -> Self {
        let data = std::fs::read("assets/Anton-Regular.ttf").ok().unwrap();
        let font = FontRef::from_index(&data, 0).unwrap();
        let (offset, key) = (font.offset, font.key);
        Self { data, offset, key }
    }
    fn as_ref(&self) -> FontRef<'_> {
        FontRef {
            data: &self.data,
            offset: self.offset,
            key: self.key,
        }
    }
}

impl<'a> Widget for &mut NameWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let font = self.state.font.as_ref();
        let mut shaper = self.state.shape_context.builder(font).size(50.).build();
        shaper.add_str(&self.text);

        let mut pen_x = 0u16;
        let baseline_row = area.height;
        shaper.shape_with(|c| {
            for g in c.glyphs {
                let pen_x_clone = pen_x;
                pen_x += g.advance as u16;
                let mut scaler = self.state.scale_context.builder(font).size(50.).build();
                let Some(image) = Render::new(&[Source::Outline]).render(&mut scaler, g.id) else {
                    continue;
                };

                let (Some(top_left_x), Some(top_left_y)) = (
                    pen_x_clone.checked_add_signed(image.placement.left as i16),
                    baseline_row.checked_sub_signed(image.placement.top as i16),
                ) else {
                    continue;
                };

                let (mut delta_x, mut delta_y) = (0, 0);
                for alpha in image.data {
                    let block = match alpha {
                        0..=50 => ' ',
                        51..=101 => '░',
                        102..=152 => '▒',
                        153..=203 => '▓',
                        204..=255 => '█',
                    };
                    if let Some(cell) = buf.cell_mut((top_left_x + delta_x, top_left_y + delta_y)) {
                        cell.set_char(block);
                    };
                    delta_x += 1;
                    if delta_x == image.placement.width as u16 {
                        delta_x = 0;
                        delta_y += 1;
                    }
                }
            }
        });
    }
}
