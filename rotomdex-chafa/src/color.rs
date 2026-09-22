//! Colors and color difference.
//!
//! Port of the color-space-agnostic parts of `chafa/internal/chafa-color.{c,h}`.
//! Per decision D1 (see `devdocs/PLAN.md`), only sRGB is supported: there is no
//! DIN99d transform, so the matching path is all-integer.

/// Index of the background color within a [`ColorPair`].
///
/// BG/FG indices must be 0 and 1 respectively, matching coverage-bitmap values
/// (`chafa-color.h:41-44`): a coverage bit of 0 selects BG, 1 selects FG.
pub const COLOR_PAIR_BG: usize = 0;
/// Index of the foreground color within a [`ColorPair`].
pub const COLOR_PAIR_FG: usize = 1;

/// A color-space-agnostic RGBA color: four `u8` channels `[R, G, B, A]`.
///
/// Mirrors `ChafaColor` (`chafa-color.h:35-39`). Only channels 0..2 participate
/// in [`color_diff`]; the alpha channel is carried through but ignored by the
/// difference metric.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    /// `[R, G, B, A]`.
    pub ch: [u8; 4],
}

impl Color {
    /// Construct a color from explicit channels.
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { ch: [r, g, b, a] }
    }

    /// Build a color from a packed `0x00RRGGBB` (or `0xAARRGGBB`) value the way
    /// chafa stores fg/bg config colors. The low 24 bits are R,G,B; alpha is set
    /// to `0xff` (opaque) which matches chafa's treatment of config colors.
    ///
    /// Note: this is the *config-color* convention (R in the high byte of the
    /// 24-bit value), distinct from the in-memory RGBA byte order used by
    /// [`Color::from_rgba_u32`].
    #[inline]
    pub const fn from_rgb_u32(rgb: u32) -> Self {
        Color {
            ch: [
                ((rgb >> 16) & 0xff) as u8,
                ((rgb >> 8) & 0xff) as u8,
                (rgb & 0xff) as u8,
                0xff,
            ],
        }
    }

    /// Reinterpret a little-endian `u32` as four bytes `[ch0, ch1, ch2, ch3]`,
    /// matching `chafa_color8_from_u32` (`chafa-color.h:80-86`) on a
    /// little-endian host.
    #[inline]
    pub const fn from_rgba_u32(u: u32) -> Self {
        Color {
            ch: [
                (u & 0xff) as u8,
                ((u >> 8) & 0xff) as u8,
                ((u >> 16) & 0xff) as u8,
                ((u >> 24) & 0xff) as u8,
            ],
        }
    }

    /// Unpack chafa's packed `0xAARRGGBB` color (`chafa_unpack_color`):
    /// R = bits 16..24, G = 8..16, B = 0..8, A = 24..32.
    #[inline]
    pub const fn unpack(packed: u32) -> Self {
        Color {
            ch: [
                ((packed >> 16) & 0xff) as u8,
                ((packed >> 8) & 0xff) as u8,
                (packed & 0xff) as u8,
                ((packed >> 24) & 0xff) as u8,
            ],
        }
    }

    /// Inverse of [`Color::from_rgba_u32`] (`chafa_color8_to_u32`).
    #[inline]
    pub const fn to_rgba_u32(self) -> u32 {
        (self.ch[0] as u32)
            | ((self.ch[1] as u32) << 8)
            | ((self.ch[2] as u32) << 16)
            | ((self.ch[3] as u32) << 24)
    }
}

/// Per-channel average of two colors, matching `chafa_color_average_2`
/// (`chafa-color.h:96-106`) bit-for-bit (the `>>1 & 0x7f7f7f7f` SWAR trick,
/// which truncates rather than rounds).
#[inline]
pub fn color_average_2(a: Color, b: Color) -> Color {
    let ua = a.to_rgba_u32();
    let ub = b.to_rgba_u32();
    Color::from_rgba_u32(((ua >> 1) & 0x7f7f_7f7f).wrapping_add((ub >> 1) & 0x7f7f_7f7f))
}

/// A `[background, foreground]` color pair.
///
/// Mirrors `ChafaColorPair` (`chafa-color.h:46-50`). Index with
/// [`COLOR_PAIR_BG`] / [`COLOR_PAIR_FG`], or directly with a coverage bit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorPair {
    /// `[BG, FG]`.
    pub colors: [Color; 2],
}

/// Signed per-channel accumulator used while averaging covered/uncovered pixels.
///
/// Mirrors `ChafaColorAccum` (`chafa-color.h:114-118`): `i16` channels. chafa
/// uses 16-bit accumulators; with at most 64 pixels of 8-bit values the sum fits
/// (`64 * 255 = 16320 < 32767`), so the width is faithful and safe.
#[derive(Clone, Copy, Debug, Default)]
pub struct ColorAccum {
    /// Signed per-channel running sum `[R, G, B, A]`.
    pub ch: [i16; 4],
}

/// Squared-Euclidean color difference over RGB channels 0..2 (alpha ignored).
///
/// Exact port of the `chafa_color_diff_fast` macro (`chafa-color.h:145-148`),
/// computed in `i32` arithmetic to match C's `gint`.
#[inline]
pub fn color_diff(a: Color, b: Color) -> i32 {
    let d0 = b.ch[0] as i32 - a.ch[0] as i32;
    let d1 = b.ch[1] as i32 - a.ch[1] as i32;
    let d2 = b.ch[2] as i32 - a.ch[2] as i32;
    d0 * d0 + d1 * d1 + d2 * d2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_is_squared_euclidean_rgb_only() {
        let a = Color::new(0, 0, 0, 0);
        let b = Color::new(3, 4, 0, 255);
        // 9 + 16 + 0 = 25; alpha difference ignored.
        assert_eq!(color_diff(a, b), 25);
        assert_eq!(color_diff(b, a), 25);
    }

    #[test]
    fn diff_max() {
        let a = Color::new(0, 0, 0, 0);
        let b = Color::new(255, 255, 255, 255);
        assert_eq!(color_diff(a, b), 255 * 255 * 3);
    }

    #[test]
    fn u32_roundtrip_is_little_endian() {
        let c = Color::new(0x11, 0x22, 0x33, 0x44);
        assert_eq!(c.to_rgba_u32(), 0x4433_2211);
        assert_eq!(Color::from_rgba_u32(0x4433_2211), c);
    }

    #[test]
    fn from_rgb_u32_packs_rgb_high_to_low() {
        let c = Color::from_rgb_u32(0x00aabbcc);
        assert_eq!(c.ch, [0xaa, 0xbb, 0xcc, 0xff]);
    }

    #[test]
    fn average_2_truncates_like_swar() {
        // 0xff and 0x00 -> (0x7f + 0x00) = 0x7f per channel (truncating).
        let a = Color::new(0xff, 0xff, 0xff, 0xff);
        let b = Color::new(0x00, 0x00, 0x00, 0x00);
        assert_eq!(color_average_2(a, b).ch, [0x7f, 0x7f, 0x7f, 0x7f]);
        // 0xff and 0x03 -> (0x7f + 0x01) = 0x80.
        let c = Color::new(0xff, 0x03, 0x01, 0x00);
        let d = Color::new(0x03, 0xff, 0x01, 0x00);
        assert_eq!(color_average_2(c, d).ch, [0x80, 0x80, 0x00, 0x00]);
    }
}
