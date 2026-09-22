//! ANSI/UTF-8 serialization. Port of `chafa-canvas-printer.c` using the
//! **fallback** terminal sequences from `chafa-term-db.c` (`fallback_list`).
//!
//! Produces the canonical canvas output (`chafa_canvas_print`): rows joined by
//! `\n`, last row without a trailing newline, no cursor framing. Validated by
//! byte-exact comparison against the patched oracle's `CHAFA_DUMP_ANSI`.
//!
//! The `for i in i0..i_max` cell loops mirror chafa and several need the index
//! for next-cell lookahead, so explicit indexing reads clearest here.
#![allow(clippy::needless_range_loop)]

use alloc::string::String;
use bitflags::bitflags;

use crate::color::Color;
use crate::palette::{INDEX_FG, INDEX_TRANSPARENT};
use crate::select::{CanvasMode, CellOut, RenderConfig};
use crate::symbol_map::SymbolMap;

bitflags! {
    /// Output-compression optimizations (`ChafaOptimizations`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Optimizations: u32 {
        /// Reuse SGR attributes across cells instead of resetting each cell.
        const REUSE_ATTRIBUTES = 1 << 0;
        /// Collapse runs of identical chars with the REP control sequence.
        const REPEAT_CELLS = 1 << 1;
    }
}

const TRANSPARENT: u32 = INDEX_TRANSPARENT as u32;

// --- Fallback terminal sequences (chafa-term-db.c) ---

const SEQ_RESET_ATTRIBUTES: &str = "\x1b[0m";
const SEQ_INVERT_COLORS: &str = "\x1b[7m";
const SEQ_ENABLE_BOLD: &str = "\x1b[1m";
const SEQ_RESET_COLOR_FG: &str = "\x1b[39m";

struct Printer<'a> {
    cfg: &'a RenderConfig,
    smap: &'a SymbolMap,
    opts: Optimizations,
    out: String,

    cur_char: u32,
    n_reps: i32,
    cur_inverted: bool,
    cur_bold: bool,
    cur_fg: u32,
    cur_bg: u32,
    cur_fg_direct: Color,
    cur_bg_direct: Color,
}

impl<'a> Printer<'a> {
    fn new(cfg: &'a RenderConfig, smap: &'a SymbolMap, opts: Optimizations) -> Self {
        Printer {
            cfg,
            smap,
            opts,
            out: String::new(),
            cur_char: 0,
            n_reps: 0,
            cur_inverted: false,
            cur_bold: false,
            cur_fg: TRANSPARENT,
            cur_bg: TRANSPARENT,
            cur_fg_direct: Color::new(0, 0, 0, 0),
            cur_bg_direct: Color::new(0, 0, 0, 0),
        }
    }

    fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn push_char(&mut self, c: u32) {
        if let Some(ch) = char::from_u32(c) {
            self.out.push(ch);
        }
    }

    /// Port of `flush_chars`.
    ///
    /// REPEAT_CELLS additionally requires the terminal to advertise the
    /// `REPEAT_CHAR` sequence (`have_seq` in chafa). The **fallback** term-info
    /// — which chafa uses by default and which we model for parity — does *not*
    /// include `REPEAT_CHAR` (it lives in `rep_seqs`, absent from
    /// `fallback_list`). So REPEAT_CELLS is inert here and chars always emit
    /// literally, matching chafa's default output.
    fn flush_chars(&mut self) {
        if self.cur_char == 0 {
            return;
        }
        let c = self.cur_char;
        while self.n_reps != 0 {
            self.push_char(c);
            self.n_reps -= 1;
        }
        self.cur_char = 0;
    }

    /// Port of `queue_char`.
    fn queue_char(&mut self, c: u32) {
        if self.cur_char == c {
            self.n_reps += 1;
        } else {
            if self.cur_char != 0 {
                self.flush_chars();
            }
            self.cur_char = c;
            self.n_reps = 1;
        }
    }

    fn reset_fg(&mut self) {
        self.push(SEQ_RESET_COLOR_FG);
        self.cur_fg = TRANSPARENT;
        self.cur_fg_direct.ch[3] = 0;
    }

    fn reset_attributes(&mut self) {
        self.push(SEQ_RESET_ATTRIBUTES);
        self.cur_inverted = false;
        self.cur_bold = false;
        self.cur_fg = TRANSPARENT;
        self.cur_bg = TRANSPARENT;
        self.cur_fg_direct.ch[3] = 0;
        self.cur_bg_direct.ch[3] = 0;
    }

    // --- truecolor ---

    fn emit_fg_direct(&mut self, c: Color) {
        self.push(&format!("\x1b[38;2;{};{};{}m", c.ch[0], c.ch[1], c.ch[2]));
    }
    fn emit_bg_direct(&mut self, c: Color) {
        self.push(&format!("\x1b[48;2;{};{};{}m", c.ch[0], c.ch[1], c.ch[2]));
    }
    fn emit_fgbg_direct(&mut self, f: Color, b: Color) {
        self.push(&format!(
            "\x1b[38;2;{};{};{};48;2;{};{};{}m",
            f.ch[0], f.ch[1], f.ch[2], b.ch[0], b.ch[1], b.ch[2]
        ));
    }

    fn emit_attributes_truecolor(&mut self, fg: Color, bg: Color, inverted: bool) {
        if self.opts.contains(Optimizations::REUSE_ATTRIBUTES) {
            if !self.cfg.fg_only
                && ((self.cur_inverted && !inverted)
                    || (self.cur_fg_direct.ch[3] != 0 && fg.ch[3] == 0)
                    || (self.cur_bg_direct.ch[3] != 0 && bg.ch[3] == 0))
            {
                self.flush_chars();
                self.reset_attributes();
            }
            if !self.cur_inverted && inverted {
                self.flush_chars();
                self.push(SEQ_INVERT_COLORS);
            }
            if fg != self.cur_fg_direct {
                if bg != self.cur_bg_direct && bg.ch[3] != 0 {
                    self.flush_chars();
                    self.emit_fgbg_direct(fg, bg);
                } else if fg.ch[3] != 0 {
                    self.flush_chars();
                    self.emit_fg_direct(fg);
                }
            } else if bg != self.cur_bg_direct && bg.ch[3] != 0 {
                self.flush_chars();
                self.emit_bg_direct(bg);
            }
        } else {
            self.flush_chars();
            self.reset_attributes();
            if inverted {
                self.push(SEQ_INVERT_COLORS);
            }
            if fg.ch[3] != 0 {
                if bg.ch[3] != 0 {
                    self.emit_fgbg_direct(fg, bg);
                } else {
                    self.emit_fg_direct(fg);
                }
            } else if bg.ch[3] != 0 {
                self.emit_bg_direct(bg);
            }
        }
        self.cur_fg_direct = fg;
        self.cur_bg_direct = bg;
        self.cur_inverted = inverted;
    }

    fn threshold_alpha(&self, mut c: Color) -> Color {
        c.ch[3] = if (c.ch[3] as i32) < self.cfg.alpha_threshold as i32 {
            0
        } else {
            255
        };
        c
    }

    fn emit_ansi_truecolor(&mut self, cells: &[CellOut], i0: usize, i_max: usize) {
        for i in i0..i_max {
            let cell = cells[i];
            if cell.c == 0 {
                continue;
            }
            let fg = self.threshold_alpha(Color::unpack(cell.fg));
            let bg = self.threshold_alpha(Color::unpack(cell.bg));
            if fg.ch[3] == 0 && bg.ch[3] != 0 {
                self.emit_attributes_truecolor(bg, fg, true);
            } else {
                self.emit_attributes_truecolor(fg, bg, false);
            }
            if fg.ch[3] == 0 && bg.ch[3] == 0 {
                self.queue_char(b' ' as u32);
                if i < i_max - 1 && cells[i + 1].c == 0 {
                    self.queue_char(b' ' as u32);
                }
            } else {
                self.queue_char(cell.c);
            }
        }
    }

    // --- indexed (256/240, 16, 16/8) ---

    fn handle_attrs_with_reuse(&mut self, fg: u32, bg: u32, inverted: bool, bold: bool) {
        if self.cfg.fg_only {
            return;
        }
        if (self.cur_inverted && !inverted)
            || (self.cur_bold && !bold)
            || (self.cur_fg != TRANSPARENT && fg == TRANSPARENT)
            || (self.cur_bg != TRANSPARENT && bg == TRANSPARENT)
        {
            self.flush_chars();
            self.reset_attributes();
        }
        if !self.cur_inverted && inverted {
            self.flush_chars();
            self.push(SEQ_INVERT_COLORS);
        }
        if !self.cur_bold && bold {
            self.flush_chars();
            self.push(SEQ_ENABLE_BOLD);
        }
    }

    fn emit_attributes_256(&mut self, fg: u32, bg: u32, inverted: bool) {
        // chafa's 256-color sequences take a `guint8` pen, so a special index
        // that leaks into emission (e.g. `INDEX_FG` 257 on a transparent blank
        // cell) is truncated to a byte: 257 -> 1, 258 -> 2, 256 -> 0. Normal
        // indices (0..=255) are unaffected. We replicate that exactly.
        let (fgb, bgb) = (fg as u8, bg as u8);
        if self.opts.contains(Optimizations::REUSE_ATTRIBUTES) {
            self.handle_attrs_with_reuse(fg, bg, inverted, false);
            if fg != self.cur_fg {
                if bg != self.cur_bg && bg != TRANSPARENT {
                    self.flush_chars();
                    self.push(&format!("\x1b[38;5;{fgb};48;5;{bgb}m"));
                } else if fg != TRANSPARENT {
                    self.flush_chars();
                    self.push(&format!("\x1b[38;5;{fgb}m"));
                }
            } else if bg != self.cur_bg && bg != TRANSPARENT {
                self.flush_chars();
                self.push(&format!("\x1b[48;5;{bgb}m"));
            }
        } else {
            self.flush_chars();
            self.reset_attributes();
            if inverted {
                self.push(SEQ_INVERT_COLORS);
            }
            if fg != TRANSPARENT {
                if bg != TRANSPARENT {
                    self.push(&format!("\x1b[38;5;{fgb};48;5;{bgb}m"));
                } else {
                    self.push(&format!("\x1b[38;5;{fgb}m"));
                }
            } else if bg != TRANSPARENT {
                self.push(&format!("\x1b[48;5;{bgb}m"));
            }
        }
        self.cur_fg = fg;
        self.cur_bg = bg;
        self.cur_inverted = inverted;
    }

    fn emit_attributes_16(&mut self, fg: u32, bg: u32, inverted: bool) {
        // As with 256-color, chafa's 16-color sequences take a `guint8` pen, so
        // a leaked special index (e.g. `INDEX_FG` 257) is truncated to a byte
        // before the aixterm mapping: 257 -> 1 -> `\e[31m`. Truncate first.
        let (fgb, bgb) = (fg as u8 as u32, bg as u8 as u32);
        if self.opts.contains(Optimizations::REUSE_ATTRIBUTES) {
            self.handle_attrs_with_reuse(fg, bg, inverted, false);
            if fg != self.cur_fg {
                if bg != self.cur_bg && bg != TRANSPARENT {
                    self.flush_chars();
                    self.push(&format!("\x1b[{};{}m", aix_fg(fgb), aix_bg(bgb)));
                } else if fg != TRANSPARENT {
                    self.flush_chars();
                    self.push(&format!("\x1b[{}m", aix_fg(fgb)));
                }
            } else if bg != self.cur_bg && bg != TRANSPARENT {
                self.flush_chars();
                self.push(&format!("\x1b[{}m", aix_bg(bgb)));
            }
        } else {
            self.flush_chars();
            self.reset_attributes();
            if inverted {
                self.push(SEQ_INVERT_COLORS);
            }
            if fg != TRANSPARENT {
                if bg != TRANSPARENT {
                    self.push(&format!("\x1b[{};{}m", aix_fg(fgb), aix_bg(bgb)));
                } else {
                    self.push(&format!("\x1b[{}m", aix_fg(fgb)));
                }
            } else if bg != TRANSPARENT {
                self.push(&format!("\x1b[{}m", aix_bg(bgb)));
            }
        }
        self.cur_fg = fg;
        self.cur_bg = bg;
        self.cur_inverted = inverted;
    }

    fn emit_attributes_16_8(&mut self, fg: u32, bg: u32, inverted: bool) {
        let bold = fg > 7 && fg < 256;
        if self.opts.contains(Optimizations::REUSE_ATTRIBUTES) {
            self.handle_attrs_with_reuse(fg, bg, inverted, bold);
            if fg != self.cur_fg {
                if bg != self.cur_bg && bg != TRANSPARENT {
                    self.flush_chars();
                    self.push(&format!("\x1b[{};{}m", (fg & 7) + 30, bg + 40));
                } else if fg != TRANSPARENT {
                    self.flush_chars();
                    self.push(&format!("\x1b[{}m", (fg & 7) + 30));
                }
            } else if bg != self.cur_bg && bg != TRANSPARENT {
                self.flush_chars();
                self.push(&format!("\x1b[{}m", bg + 40));
            }
        } else {
            self.flush_chars();
            self.reset_attributes();
            if inverted {
                self.push(SEQ_INVERT_COLORS);
            }
            if fg > 7 {
                self.push(SEQ_ENABLE_BOLD);
            }
            if fg != TRANSPARENT {
                if bg != TRANSPARENT {
                    self.push(&format!("\x1b[{};{}m", (fg & 7) + 30, bg + 40));
                } else {
                    self.push(&format!("\x1b[{}m", (fg & 7) + 30));
                }
            } else if bg != TRANSPARENT {
                self.push(&format!("\x1b[{}m", bg + 40));
            }
        }
        self.cur_fg = fg;
        self.cur_bg = bg;
        self.cur_inverted = inverted;
        self.cur_bold = bold;
    }

    /// Shared indexed body: choose emitter by mode.
    fn emit_ansi_indexed(&mut self, cells: &[CellOut], i0: usize, i_max: usize) {
        for i in i0..i_max {
            let cell = cells[i];
            if cell.c == 0 {
                continue;
            }
            let (fg, bg) = (cell.fg, cell.bg);
            let inverted = fg == TRANSPARENT && bg != TRANSPARENT;
            let (a, b) = if inverted { (bg, fg) } else { (fg, bg) };
            match self.cfg.mode {
                CanvasMode::Indexed256 | CanvasMode::Indexed240 => {
                    self.emit_attributes_256(a, b, inverted)
                }
                CanvasMode::Indexed16 | CanvasMode::Indexed8 => {
                    self.emit_attributes_16(a, b, inverted)
                }
                CanvasMode::Indexed16_8 => self.emit_attributes_16_8(a, b, inverted),
                _ => unreachable!(),
            }
            if fg == TRANSPARENT && bg == TRANSPARENT {
                self.queue_char(b' ' as u32);
                if i < i_max - 1 && cells[i + 1].c == 0 {
                    self.queue_char(b' ' as u32);
                }
            } else {
                self.queue_char(cell.c);
            }
        }
    }

    // --- fgbg / fgbg-bgfg ---

    fn emit_ansi_fgbg_bgfg(&mut self, cells: &[CellOut], i0: usize, i_max: usize) {
        let blank_symbol = if self.smap.has_symbol(' ') {
            b' ' as u32
        } else if self.smap.has_symbol('\u{2588}') {
            0x2588
        } else {
            0
        };
        for i in i0..i_max {
            let cell = cells[i];
            let mut c = cell.c;
            if c == 0 {
                continue;
            }
            let mut invert = false;
            if cell.fg == cell.bg && blank_symbol != 0 && (i == i_max - 1 || cells[i + 1].c != 0) {
                c = blank_symbol;
                if blank_symbol == 0x2588 {
                    invert = true;
                }
            }
            if cell.bg == INDEX_FG as u32 {
                invert ^= true;
            }
            if self.opts.contains(Optimizations::REUSE_ATTRIBUTES) {
                if !self.cur_inverted && invert {
                    self.flush_chars();
                    self.push(SEQ_INVERT_COLORS);
                } else if self.cur_inverted && !invert {
                    self.flush_chars();
                    self.reset_attributes();
                }
                self.cur_inverted = invert;
            } else {
                self.flush_chars();
                if invert {
                    self.push(SEQ_INVERT_COLORS);
                } else {
                    self.reset_attributes();
                }
            }
            self.queue_char(c);
        }
    }

    fn emit_ansi_fgbg(&mut self, cells: &[CellOut], i0: usize, i_max: usize) {
        for i in i0..i_max {
            let cell = cells[i];
            if cell.c == 0 {
                continue;
            }
            self.queue_char(cell.c);
        }
    }

    fn build_row(&mut self, cells: &[CellOut], cols: usize, row: usize) {
        let i = row * cols;
        let i_max = i + cols;

        if row == 0 && self.cfg.mode != CanvasMode::Fgbg {
            if self.cfg.fg_only {
                self.reset_fg();
            } else {
                self.reset_attributes();
            }
        }

        match self.cfg.mode {
            CanvasMode::Truecolor => self.emit_ansi_truecolor(cells, i, i_max),
            CanvasMode::Indexed256
            | CanvasMode::Indexed240
            | CanvasMode::Indexed16
            | CanvasMode::Indexed8
            | CanvasMode::Indexed16_8 => self.emit_ansi_indexed(cells, i, i_max),
            CanvasMode::FgbgBgfg => self.emit_ansi_fgbg_bgfg(cells, i, i_max),
            CanvasMode::Fgbg => self.emit_ansi_fgbg(cells, i, i_max),
        }

        self.flush_chars();

        if self.cfg.mode != CanvasMode::Fgbg {
            if self.cfg.fg_only {
                self.reset_fg();
            } else {
                self.reset_attributes();
            }
        }
    }
}

/// aixterm SGR code for a 16-color fg index (`DEFINE_EMIT_SEQ_1_aix16fg`).
fn aix_fg(pen: u32) -> u32 {
    if pen < 8 { pen + 30 } else { pen + 82 }
}
/// aixterm SGR code for a 16-color bg index.
fn aix_bg(pen: u32) -> u32 {
    if pen < 8 { pen + 40 } else { pen + 92 }
}

/// Serialize a cell grid to the canonical canvas ANSI string (rows joined by
/// `\n`, last without). Port of `build_ansi_gstring`.
pub fn print_cells(
    cfg: &RenderConfig,
    smap: &SymbolMap,
    cells: &[CellOut],
    cols: usize,
    rows: usize,
    opts: Optimizations,
) -> String {
    let mut p = Printer::new(cfg, smap, opts);
    for row in 0..rows {
        p.build_row(cells, cols, row);
        if row < rows - 1 {
            p.out.push('\n');
        }
    }
    p.out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aix_codes() {
        assert_eq!(aix_fg(1), 31);
        assert_eq!(aix_fg(9), 91);
        assert_eq!(aix_bg(1), 41);
        assert_eq!(aix_bg(9), 101);
    }
}
