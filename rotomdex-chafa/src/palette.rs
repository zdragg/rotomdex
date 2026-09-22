//! Fixed terminal palettes and nearest-color lookup.
//!
//! Port of the fixed-palette parts of `chafa-palette.c` (sRGB only, per D1).
//! The 16-color table is chafa's *non-standard* ANSI set (e.g. green = `0x007000`),
//! not real xterm values — used verbatim for parity. Nearest-color ties resolve
//! to the lowest index (`update_candidates` uses strict `<`), and
//! `pick_color_fixed_256` deliberately scans the cube/grays *before* the 16 ANSI
//! colors so ties favour the higher-index universal colors.

use crate::color::{color_diff, Color};

/// Palette index for a transparent pixel.
pub const INDEX_TRANSPARENT: i32 = 256;
/// Palette index for the terminal default foreground.
pub const INDEX_FG: i32 = 257;
/// Palette index for the terminal default background.
pub const INDEX_BG: i32 = 258;

/// The 256 fixed terminal colors (`term_colors_256`, `chafa-palette.c:131`):
/// chafa's non-standard first 16, then the 216-color cube, then 24 grays.
#[rustfmt::skip]
const TERM_COLORS_256: [u32; 256] = [
    0x000000, 0x800000, 0x007000, 0x707000, 0x000070, 0x700070, 0x007070, 0xc0c0c0,
    0x404040, 0xff0000, 0x00ff00, 0xffff00, 0x0000ff, 0xff00ff, 0x00ffff, 0xffffff,
    0x000000, 0x00005f, 0x000087, 0x0000af, 0x0000d7, 0x0000ff, 0x005f00, 0x005f5f,
    0x005f87, 0x005faf, 0x005fd7, 0x005fff, 0x008700, 0x00875f, 0x008787, 0x0087af,
    0x0087d7, 0x0087ff, 0x00af00, 0x00af5f, 0x00af87, 0x00afaf, 0x00afd7, 0x00afff,
    0x00d700, 0x00d75f, 0x00d787, 0x00d7af, 0x00d7d7, 0x00d7ff, 0x00ff00, 0x00ff5f,
    0x00ff87, 0x00ffaf, 0x00ffd7, 0x00ffff, 0x5f0000, 0x5f005f, 0x5f0087, 0x5f00af,
    0x5f00d7, 0x5f00ff, 0x5f5f00, 0x5f5f5f, 0x5f5f87, 0x5f5faf, 0x5f5fd7, 0x5f5fff,
    0x5f8700, 0x5f875f, 0x5f8787, 0x5f87af, 0x5f87d7, 0x5f87ff, 0x5faf00, 0x5faf5f,
    0x5faf87, 0x5fafaf, 0x5fafd7, 0x5fafff, 0x5fd700, 0x5fd75f, 0x5fd787, 0x5fd7af,
    0x5fd7d7, 0x5fd7ff, 0x5fff00, 0x5fff5f, 0x5fff87, 0x5fffaf, 0x5fffd7, 0x5fffff,
    0x870000, 0x87005f, 0x870087, 0x8700af, 0x8700d7, 0x8700ff, 0x875f00, 0x875f5f,
    0x875f87, 0x875faf, 0x875fd7, 0x875fff, 0x878700, 0x87875f, 0x878787, 0x8787af,
    0x8787d7, 0x8787ff, 0x87af00, 0x87af5f, 0x87af87, 0x87afaf, 0x87afd7, 0x87afff,
    0x87d700, 0x87d75f, 0x87d787, 0x87d7af, 0x87d7d7, 0x87d7ff, 0x87ff00, 0x87ff5f,
    0x87ff87, 0x87ffaf, 0x87ffd7, 0x87ffff, 0xaf0000, 0xaf005f, 0xaf0087, 0xaf00af,
    0xaf00d7, 0xaf00ff, 0xaf5f00, 0xaf5f5f, 0xaf5f87, 0xaf5faf, 0xaf5fd7, 0xaf5fff,
    0xaf8700, 0xaf875f, 0xaf8787, 0xaf87af, 0xaf87d7, 0xaf87ff, 0xafaf00, 0xafaf5f,
    0xafaf87, 0xafafaf, 0xafafd7, 0xafafff, 0xafd700, 0xafd75f, 0xafd787, 0xafd7af,
    0xafd7d7, 0xafd7ff, 0xafff00, 0xafff5f, 0xafff87, 0xafffaf, 0xafffd7, 0xafffff,
    0xd70000, 0xd7005f, 0xd70087, 0xd700af, 0xd700d7, 0xd700ff, 0xd75f00, 0xd75f5f,
    0xd75f87, 0xd75faf, 0xd75fd7, 0xd75fff, 0xd78700, 0xd7875f, 0xd78787, 0xd787af,
    0xd787d7, 0xd787ff, 0xd7af00, 0xd7af5f, 0xd7af87, 0xd7afaf, 0xd7afd7, 0xd7afff,
    0xd7d700, 0xd7d75f, 0xd7d787, 0xd7d7af, 0xd7d7d7, 0xd7d7ff, 0xd7ff00, 0xd7ff5f,
    0xd7ff87, 0xd7ffaf, 0xd7ffd7, 0xd7ffff, 0xff0000, 0xff005f, 0xff0087, 0xff00af,
    0xff00d7, 0xff00ff, 0xff5f00, 0xff5f5f, 0xff5f87, 0xff5faf, 0xff5fd7, 0xff5fff,
    0xff8700, 0xff875f, 0xff8787, 0xff87af, 0xff87d7, 0xff87ff, 0xffaf00, 0xffaf5f,
    0xffaf87, 0xffafaf, 0xffafd7, 0xffafff, 0xffd700, 0xffd75f, 0xffd787, 0xffd7af,
    0xffd7d7, 0xffd7ff, 0xffff00, 0xffff5f, 0xffff87, 0xffffaf, 0xffffd7, 0xffffff,
    0x080808, 0x121212, 0x1c1c1c, 0x262626, 0x303030, 0x3a3a3a, 0x444444, 0x4e4e4e,
    0x585858, 0x626262, 0x6c6c6c, 0x767676, 0x808080, 0x8a8a8a, 0x949494, 0x9e9e9e,
    0xa8a8a8, 0xb2b2b2, 0xbcbcbc, 0xc6c6c6, 0xd0d0d0, 0xdadada, 0xe4e4e4, 0xeeeeee,
];

/// The fixed palette color at `index`. Indices 0..256 come from the table;
/// 256/257/258 are the transparent/fg/bg specials (`term_colors_256[256..259]`
/// = `0x808080`, `0xffffff`, `0x000000`). The gray walk can legitimately read
/// index 256 at the bright end before its error-increase break, so we support
/// the specials here (color_diff ignores alpha). Only channels 0..2 matter.
fn fixed_color(index: usize) -> Color {
    match index {
        0..=255 => Color::from_rgb_u32(TERM_COLORS_256[index]),
        256 => Color::from_rgb_u32(0x808080),
        257 => Color::from_rgb_u32(0xffffff),
        _ => Color::from_rgb_u32(0x000000),
    }
}

/// Map a channel value to its 216-cube level (0..5) via chafa's midpoint
/// cutoffs (`chafa-palette.c:224-235`).
fn cube_channel_index(v: u8) -> u32 {
    let v = v as i32;
    if v < 0x5f / 2 {
        0
    } else if v < (0x5f + 0x87) / 2 {
        1
    } else if v < (0x87 + 0xaf) / 2 {
        2
    } else if v < (0xaf + 0xd7) / 2 {
        3
    } else if v < (0xd7 + 0xff) / 2 {
        4
    } else {
        5
    }
}

/// Fixed-palette type. (Truecolor uses no palette.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteType {
    Fixed256,
    Fixed240,
    Fixed16,
    Fixed8,
    FixedFgbg,
}

/// Nearest + second-nearest palette candidates (`ChafaColorCandidates`).
#[derive(Clone, Copy, Debug)]
pub struct Candidates {
    pub index: [i32; 2],
    pub error: [i32; 2],
}

impl Candidates {
    fn init() -> Self {
        Candidates {
            index: [-1, -1],
            error: [i32::MAX, i32::MAX],
        }
    }

    /// Port of `update_candidates` (strict `<` → lowest-index / first-added wins).
    fn update(&mut self, index: i32, error: i32) {
        if error < self.error[0] {
            self.index[1] = self.index[0];
            self.index[0] = index;
            self.error[1] = self.error[0];
            self.error[0] = error;
        } else if error < self.error[1] {
            self.index[1] = index;
            self.error[1] = error;
        }
    }

    fn update_with_index(&mut self, color: Color, index: usize) -> i32 {
        let error = color_diff(color, fixed_color(index));
        self.update(index as i32, error);
        error
    }
}

/// A configured fixed palette.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub ptype: PaletteType,
    pub alpha_threshold: u8,
    pub transparent_index: i32,
    /// FG/BG colors (for `FixedFgbg`), opaque/transparent per chafa.
    pub fg: Color,
    pub bg: Color,
}

impl Palette {
    pub fn new(ptype: PaletteType, fg_rgb: u32, bg_rgb: u32, alpha_threshold: u8) -> Self {
        let mut fg = Color::from_rgb_u32(fg_rgb);
        let mut bg = Color::from_rgb_u32(bg_rgb);
        fg.ch[3] = 0xff;
        bg.ch[3] = 0x00;
        Palette {
            ptype,
            alpha_threshold,
            transparent_index: INDEX_TRANSPARENT,
            fg,
            bg,
        }
    }

    /// Nearest palette index for `color` (full candidate set). Port of
    /// `chafa_palette_lookup_nearest` (fixed-palette branch, sRGB).
    pub fn lookup(&self, color: Color) -> Candidates {
        let mut c = Candidates::init();

        if (color.ch[3] as i32) < self.alpha_threshold as i32 {
            c.index = [self.transparent_index, self.transparent_index];
            c.error = [0, 0];
        } else {
            match self.ptype {
                PaletteType::Fixed256 => self.pick_256(color, &mut c),
                PaletteType::Fixed240 => self.pick_240(color, &mut c),
                PaletteType::Fixed16 => pick_range(color, 0, 16, &mut c),
                PaletteType::Fixed8 => pick_range(color, 0, 8, &mut c),
                PaletteType::FixedFgbg => self.pick_fgbg(color, &mut c),
            }
        }

        // transparent_index < 256 remapping (chafa-palette.c:1017). With the
        // default transparent_index (256) this is a no-op.
        if self.transparent_index < 256 {
            if c.index[0] == self.transparent_index {
                c.index[0] = c.index[1];
                c.error[0] = c.error[1];
            } else {
                if c.index[0] == INDEX_TRANSPARENT {
                    c.index[0] = self.transparent_index;
                }
                if c.index[1] == INDEX_TRANSPARENT {
                    c.index[1] = self.transparent_index;
                }
            }
        }

        c
    }

    /// Convenience: nearest index only.
    pub fn lookup_nearest(&self, color: Color) -> i32 {
        self.lookup(color).index[0]
    }

    /// The color stored at palette `index` (`chafa_palette_get_color`). Fixed
    /// table for 0..=255; the configured fg/bg for the special indices.
    pub fn color_at(&self, index: i32) -> Color {
        match index {
            INDEX_FG => self.fg,
            INDEX_BG => self.bg,
            INDEX_TRANSPARENT => {
                let mut c = Color::from_rgb_u32(0x808080);
                c.ch[3] = 0x00;
                c
            }
            i if (0..=255).contains(&i) => fixed_color(i as usize),
            _ => Color::default(),
        }
    }

    fn pick_256(&self, color: Color, c: &mut Candidates) {
        pick_cube(color, c);
        pick_grays(color, c);
        // Last, so ties break in favour of the high-index cube/gray colors.
        pick_range(color, 0, 16, c);
    }

    fn pick_240(&self, color: Color, c: &mut Candidates) {
        pick_cube(color, c);
        pick_grays(color, c);
    }

    fn pick_fgbg(&self, color: Color, c: &mut Candidates) {
        c.update(INDEX_FG, color_diff(color, self.fg));
        c.update(INDEX_BG, color_diff(color, self.bg));
    }
}

fn pick_range(color: Color, first: usize, last: usize, c: &mut Candidates) {
    for i in first..last {
        c.update_with_index(color, i);
    }
}

/// `pick_color_fixed_216_cube`: index by per-channel cube level.
fn pick_cube(color: Color, c: &mut Candidates) {
    let i = 16
        + (cube_channel_index(color.ch[0]) * 36
            + cube_channel_index(color.ch[1]) * 6
            + cube_channel_index(color.ch[2])) as usize;
    c.update_with_index(color, i);
}

/// `pick_color_fixed_24_grays`: start at the middle gray, walk in the
/// improving direction until error stops decreasing.
fn pick_grays(color: Color, c: &mut Candidates) {
    let mut i: i32 = 232 + 12; // 244
    let mut last_error = c.update_with_index(color, i as usize);

    let error = color_diff(color, fixed_color((i + 1) as usize));
    let step: i32 = if error < last_error {
        c.update(i, error); // chafa updates index i (not i+1) here — faithful.
        last_error = error;
        i += 1;
        1
    } else {
        -1
    };

    loop {
        i += step;
        let error = color_diff(color, fixed_color(i as usize));
        if error > last_error {
            break;
        }
        c.update(i, error);
        last_error = error;
        if !(232..=255).contains(&i) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_cutoffs() {
        assert_eq!(cube_channel_index(0), 0);
        assert_eq!(cube_channel_index(0x2e), 0); // 46 < 47
        assert_eq!(cube_channel_index(0x2f), 1); // 47
        assert_eq!(cube_channel_index(0xff), 5);
    }

    #[test]
    fn pure_colors_map_to_themselves_256() {
        let p = Palette::new(PaletteType::Fixed256, 0xffffff, 0x000000, 127);
        // 0xff0000 is index 196 (cube 5,0,0) and also index 9 (ansi red). chafa
        // scans cube first; ties favour the cube entry. Just assert exactness.
        assert_eq!(p.lookup_nearest(Color::new(0, 0, 0, 255)), 16); // cube (0,0,0)
        assert_eq!(p.lookup_nearest(Color::new(0xff, 0xff, 0xff, 255)), 231); // cube white
    }

    #[test]
    fn transparent_pixel() {
        let p = Palette::new(PaletteType::Fixed256, 0xffffff, 0x000000, 127);
        assert_eq!(p.lookup_nearest(Color::new(0, 0, 0, 0)), INDEX_TRANSPARENT);
    }

    #[test]
    fn fixed_16_range() {
        let p = Palette::new(PaletteType::Fixed16, 0xffffff, 0x000000, 127);
        let idx = p.lookup_nearest(Color::new(0xff, 0, 0, 255));
        assert_eq!(idx, 9); // bright red
    }
}
