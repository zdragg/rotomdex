//! High-level [`Canvas`] / [`CanvasConfig`] API tying together the pixel
//! pipeline, selection core, and printer.
//!
//! Builder-style configuration (idiomatic Rust) over chafa-tracking semantics.
//! Geometry is in **cells**; the pixel grid is `width*8 × height*8`. Input is
//! resampled to that grid (best-effort, see [`crate::pixops`]); pass a
//! pre-sized buffer for bit-exact selection.

use alloc::string::String;
use alloc::vec::Vec;

use crate::color::Color;
use crate::geometry::{SYMBOL_HEIGHT_PIXELS, SYMBOL_WIDTH_PIXELS};
use crate::pixops::{self, PixelType};
use crate::printer::{Optimizations, print_cells};
use crate::select::{CanvasMode, CellOut, RenderConfig, render_cells};
use crate::smolscale;
use crate::symbol_map::SymbolMap;

/// Canvas configuration. Construct with [`CanvasConfig::new`] then use the
/// builder setters.
#[derive(Clone, Debug)]
pub struct CanvasConfig {
    /// Width in cells.
    pub width: usize,
    /// Height in cells.
    pub height: usize,
    pub mode: CanvasMode,
    /// Foreground color, packed `0x00RRGGBB`.
    pub fg_rgb: u32,
    /// Background color, packed `0x00RRGGBB`.
    pub bg_rgb: u32,
    /// Work factor `0.0..=1.0` (higher = more thorough, slower).
    pub work_factor: f32,
    pub fg_only: bool,
    pub optimizations: Optimizations,
    pub symbol_map: SymbolMap,
}

impl CanvasConfig {
    /// A config with chafa's library defaults (truecolor, white-on-black, work
    /// 0.5, all optimizations, `block+border+space-wide` symbols).
    pub fn new(width: usize, height: usize) -> Self {
        CanvasConfig {
            width,
            height,
            mode: CanvasMode::Truecolor,
            fg_rgb: 0xffffff,
            bg_rgb: 0x000000,
            work_factor: 0.5,
            fg_only: false,
            optimizations: Optimizations::all(),
            symbol_map: SymbolMap::chafa_default(),
        }
    }

    pub fn geometry(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn mode(mut self, mode: CanvasMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn fg_color(mut self, rgb: u32) -> Self {
        self.fg_rgb = rgb;
        self
    }
    pub fn bg_color(mut self, rgb: u32) -> Self {
        self.bg_rgb = rgb;
        self
    }
    pub fn work_factor(mut self, w: f32) -> Self {
        self.work_factor = w;
        self
    }
    pub fn fg_only(mut self, on: bool) -> Self {
        self.fg_only = on;
        self
    }
    pub fn optimizations(mut self, o: Optimizations) -> Self {
        self.optimizations = o;
        self
    }
    pub fn symbol_map(mut self, m: SymbolMap) -> Self {
        self.symbol_map = m;
        self
    }
}

/// A drawable canvas: holds the resolved render config and the rendered cells.
pub struct Canvas {
    cfg: CanvasConfig,
    render_cfg: RenderConfig,
    cells: Vec<CellOut>,
}

impl Canvas {
    /// Create a canvas, compiling the symbol map and resolving the render
    /// configuration (mode gates, palettes, blank/solid chars).
    pub fn new(mut cfg: CanvasConfig) -> Self {
        cfg.symbol_map.prepare();
        let render_cfg = RenderConfig::new(
            cfg.mode,
            cfg.fg_only,
            cfg.fg_rgb,
            cfg.bg_rgb,
            cfg.work_factor,
            &cfg.symbol_map,
            None,
        );
        let cells = vec![
            CellOut {
                c: 0x20,
                fg: 0,
                bg: 0
            };
            cfg.width * cfg.height
        ];
        Canvas {
            cfg,
            render_cfg,
            cells,
        }
    }

    fn width_pixels(&self) -> usize {
        self.cfg.width * SYMBOL_WIDTH_PIXELS
    }
    fn height_pixels(&self) -> usize {
        self.cfg.height * SYMBOL_HEIGHT_PIXELS
    }

    /// Draw raw pixels onto the canvas: convert format → resample to the cell
    /// grid (a bit-exact port of chafa's smolscale, gamma-correct in
    /// premultiplied linear light) → composite alpha over the background → run
    /// the selection core.
    ///
    /// The resample is a pure stretch to `width*8 × height*8`, matching chafa
    /// run with `--stretch`. Placement/tuck/align (chafa's CLI default
    /// centering) is not modeled here.
    pub fn draw_all_pixels(
        &mut self,
        ptype: PixelType,
        data: &[u8],
        w: usize,
        h: usize,
        rowstride: usize,
    ) {
        // Decode the input format to tightly-packed RGBA8 bytes, then resample
        // with the smolscale port (which does its own unpack/premul/gamma).
        let src = pixops::to_rgba(ptype, data, w, h, rowstride);
        let src_bytes: Vec<u8> = src.iter().flat_map(|c| c.ch).collect();
        let (dw, dh) = (self.width_pixels(), self.height_pixels());
        let scaled = smolscale::scale_rgba8(&src_bytes, w, h, dw, dh);
        let mut grid: Vec<Color> = scaled
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| Color::new(p[0], p[1], p[2], p[3]))
            .collect();
        if pixops::has_alpha(&grid) {
            composite_grid(&mut grid, self.cfg.bg_rgb);
        }
        self.cells = render_cells(&self.render_cfg, &self.cfg.symbol_map, None, &grid, dw, dh);
    }

    /// Draw a pre-sized RGBA grid (`width*8 × height*8`) directly — no
    /// resampling, so selection is bit-exact for the given pixels.
    pub fn draw_rgba_presized(&mut self, grid: &[Color]) {
        let (dw, dh) = (self.width_pixels(), self.height_pixels());
        assert_eq!(
            grid.len(),
            dw * dh,
            "pre-sized grid must be width*8 x height*8"
        );
        let mut grid = grid.to_vec();
        if pixops::has_alpha(&grid) {
            composite_grid(&mut grid, self.cfg.bg_rgb);
        }
        self.cells = render_cells(&self.render_cfg, &self.cfg.symbol_map, None, &grid, dw, dh);
    }

    /// The rendered cells.
    pub fn cells(&self) -> &[CellOut] {
        &self.cells
    }

    /// Serialize to an ANSI/UTF-8 string (rows joined by newlines).
    pub fn print(&self) -> String {
        print_cells(
            &self.render_cfg,
            &self.cfg.symbol_map,
            &self.cells,
            self.cfg.width,
            self.cfg.height,
            self.cfg.optimizations,
        )
    }
}

fn composite_grid(grid: &mut [Color], bg_rgb: u32) {
    let mut bg = Color::from_rgb_u32(bg_rgb);
    bg.ch[3] = 0xff;
    pixops::composite_over_bg(grid, bg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_truecolor_runs() {
        let cfg = CanvasConfig::new(10, 5);
        let mut canvas = Canvas::new(cfg);
        // A 40x20 RGBA image (will be downscaled to 80x40).
        let data: Vec<u8> = (0..40 * 20 * 4).map(|i| (i % 256) as u8).collect();
        canvas.draw_all_pixels(PixelType::Rgba8, &data, 40, 20, 40 * 4);
        let out = canvas.print();
        assert!(!out.is_empty());
        assert_eq!(canvas.cells().len(), 50);
    }
}
