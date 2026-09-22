//! # chafa-syms-rs
//!
//! A pure-Rust port of the **symbol-rendering core** of [chafa](https://hpjansson.org/chafa/):
//! turning a raster image into a grid of terminal character cells, where each cell picks the
//! Unicode symbol + foreground/background colors that best reconstruct that cell's pixels.
//!
//! The port targets *core numerical parity* with chafa 1.19.0: given an identical input pixel
//! grid, the chosen symbol and colors per cell match chafa bit-for-bit (sRGB color space only).
//!
//! See `devdocs/PLAN.md` for the full design and the C-source map.

#![no_std]

#[macro_use]
extern crate alloc;

pub mod canvas;
pub mod color;
pub mod geometry;
pub mod palette;
pub mod pixops;
pub mod printer;
pub mod select;
pub mod smolscale;
mod smolscale_luts;
pub mod symbol;
pub mod symbol_map;
pub mod work_cell;

pub use canvas::{Canvas, CanvasConfig};
pub use color::{COLOR_PAIR_BG, COLOR_PAIR_FG, Color, ColorPair, color_diff};
pub use pixops::PixelType;
pub use printer::{Optimizations, print_cells};
pub use select::{CanvasMode, CellOut, RenderConfig, render_cells};
pub use symbol::{Symbol, SymbolTags, WideSymbol};
pub use symbol_map::{Candidate, Selector, SymbolMap};
pub use work_cell::WorkCell;
