//! Pixel ingestion pipeline: input-format conversion and
//! alpha-over-background compositing.
//!
//! The **alpha composite** and **format conversion** are faithful ports of
//! `chafa-pixops.c` (`composite_alpha_on_bg`, the `ChafaPixelType` byte orders).
//! **Resampling** is handled separately by [`crate::smolscale`], a bit-exact
//! port of chafa's smolscale resampler.
//!
//! Per-channel `for c in 0..4` loops mirror chafa's fixed RGBA iteration.
#![allow(clippy::needless_range_loop)]

use alloc::vec::Vec;

use crate::color::Color;

/// Supported raw input pixel formats (the six in-scope `ChafaPixelType`s).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelType {
    Rgba8,
    Bgra8,
    Argb8,
    Abgr8,
    Rgb8,
    Bgr8,
}

impl PixelType {
    /// Bytes per pixel.
    pub fn bpp(self) -> usize {
        match self {
            PixelType::Rgba8 | PixelType::Bgra8 | PixelType::Argb8 | PixelType::Abgr8 => 4,
            PixelType::Rgb8 | PixelType::Bgr8 => 3,
        }
    }

    /// Decode one pixel at `p` into RGBA.
    #[inline]
    fn decode(self, p: &[u8]) -> Color {
        match self {
            PixelType::Rgba8 => Color::new(p[0], p[1], p[2], p[3]),
            PixelType::Bgra8 => Color::new(p[2], p[1], p[0], p[3]),
            PixelType::Argb8 => Color::new(p[1], p[2], p[3], p[0]),
            PixelType::Abgr8 => Color::new(p[3], p[2], p[1], p[0]),
            PixelType::Rgb8 => Color::new(p[0], p[1], p[2], 0xff),
            PixelType::Bgr8 => Color::new(p[2], p[1], p[0], 0xff),
        }
    }
}

/// Convert a raw buffer (`w`×`h`, `rowstride` bytes/row) to a tight RGBA `Color`
/// grid. Faithful to `ChafaPixelType` channel orders.
pub fn to_rgba(ptype: PixelType, data: &[u8], w: usize, h: usize, rowstride: usize) -> Vec<Color> {
    let bpp = ptype.bpp();
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        let row = &data[y * rowstride..];
        for x in 0..w {
            out.push(ptype.decode(&row[x * bpp..x * bpp + bpp]));
        }
    }
    out
}

/// Whether any pixel is non-opaque (`have_alpha`).
pub fn has_alpha(pixels: &[Color]) -> bool {
    pixels.iter().any(|p| p.ch[3] != 0xff)
}

/// Composite straight-alpha pixels over a background: `(c*a + bg*(255-a)) / 255`
/// per RGB channel. Faithful port of `composite_alpha_on_bg`
/// (`chafa-pixops.c`): the **alpha channel is left untouched**, so downstream
/// selection/palette/printer can still apply chafa's `alpha_threshold`
/// (sub-threshold cells render transparent). chafa only runs this when the
/// canvas `have_alpha` (any pixel non-opaque) — see [`has_alpha`].
pub fn composite_over_bg(pixels: &mut [Color], bg: Color) {
    for p in pixels.iter_mut() {
        let a = p.ch[3] as u32;
        for c in 0..3 {
            p.ch[c] = ((p.ch[c] as u32 * a + bg.ch[c] as u32 * (255 - a)) / 255) as u8;
        }
    }
}

// Resampling lives in [`crate::smolscale`] — a bit-exact port of chafa's
// smolscale (gamma-correct, premultiplied linear light). The previous
// best-effort box/bilinear resampler here has been retired.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_orders() {
        let p = [0x11u8, 0x22, 0x33, 0x44];
        assert_eq!(
            PixelType::Rgba8.decode(&p),
            Color::new(0x11, 0x22, 0x33, 0x44)
        );
        assert_eq!(
            PixelType::Bgra8.decode(&p),
            Color::new(0x33, 0x22, 0x11, 0x44)
        );
        assert_eq!(
            PixelType::Argb8.decode(&p),
            Color::new(0x22, 0x33, 0x44, 0x11)
        );
        assert_eq!(
            PixelType::Abgr8.decode(&p),
            Color::new(0x44, 0x33, 0x22, 0x11)
        );
        assert_eq!(
            PixelType::Rgb8.decode(&p[..3]),
            Color::new(0x11, 0x22, 0x33, 0xff)
        );
        assert_eq!(
            PixelType::Bgr8.decode(&p[..3]),
            Color::new(0x33, 0x22, 0x11, 0xff)
        );
    }

    #[test]
    fn composite_opaque_is_identity() {
        let mut px = [Color::new(10, 20, 30, 255)];
        composite_over_bg(&mut px, Color::new(0, 0, 0, 255));
        assert_eq!(px[0], Color::new(10, 20, 30, 255));
    }

    #[test]
    fn composite_transparent_is_bg() {
        // Color becomes the background; alpha is retained (chafa leaves ch[3]),
        // so the selector can still classify the pixel as transparent.
        let mut px = [Color::new(10, 20, 30, 0)];
        composite_over_bg(&mut px, Color::new(7, 8, 9, 255));
        assert_eq!(px[0], Color::new(7, 8, 9, 0));
    }
}
