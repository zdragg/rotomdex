//! Symbol map: selector grammar, selection (`char_is_selected`), compilation
//! (dedup + popcount sort + packed bitmaps), and candidate search.
//!
//! Port of `chafa-symbol-map.c`. One deliberate, documented divergence: equal
//! popcount ties in the compiled sort are broken by **codepoint** (a total
//! order), making the symbol array fully deterministic. Stock chafa leaves this
//! order to an unstable `qsort` over GLib hashtable iteration; the oracle is
//! patched to match (env `CHAFA_SYMS_RS_TIEBREAK`). See `devdocs/oracle`.

use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

use crate::geometry::N_CANDIDATES_MAX;
use crate::symbol::{Symbol, SymbolTags, WideSymbol, builtin_narrow, builtin_wide};

/// A single selector, applied in order (later overrides earlier).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selector {
    /// Match symbols whose tags intersect `tags`.
    Tag { additive: bool, tags: SymbolTags },
    /// Match codepoints in `[first, last]`.
    Range {
        additive: bool,
        first: u32,
        last: u32,
    },
}

/// A shape candidate (`ChafaCandidate`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub symbol_index: usize,
    pub hamming_distance: u8,
    pub is_inverted: bool,
}

/// Errors from selector-string parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorParseError(pub String);

impl core::fmt::Display for SelectorParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl core::error::Error for SelectorParseError {}

/// A set of symbols, selected from the builtin glyph set by an ordered list of
/// selectors and compiled into popcount-sorted arrays for matching.
#[derive(Clone, Debug)]
pub struct SymbolMap {
    selectors: Vec<Selector>,
    use_builtin_glyphs: bool,
    dirty: bool,

    // Compiled (sorted ascending by (popcount, codepoint)).
    symbols: Vec<Symbol>,
    packed_bitmaps: Vec<u64>,
    symbols_wide: Vec<WideSymbol>,
    /// Interleaved `[l0, r0, l1, r1, ...]`.
    packed_bitmaps_wide: Vec<u64>,
}

impl Default for SymbolMap {
    fn default() -> Self {
        SymbolMap::new()
    }
}

impl SymbolMap {
    /// An empty map (selects nothing until selectors are added).
    pub fn new() -> Self {
        SymbolMap {
            selectors: Vec::new(),
            use_builtin_glyphs: true,
            dirty: true,
            symbols: Vec::new(),
            packed_bitmaps: Vec::new(),
            symbols_wide: Vec::new(),
            packed_bitmaps_wide: Vec::new(),
        }
    }

    /// chafa's default canvas symbol set: `+block +border +space -wide`
    /// (`chafa-canvas-config.c:72-76`).
    pub fn chafa_default() -> Self {
        let mut m = SymbolMap::new();
        m.add_by_tags(SymbolTags::BLOCK);
        m.add_by_tags(SymbolTags::BORDER);
        m.add_by_tags(SymbolTags::SPACE);
        m.remove_by_tags(SymbolTags::WIDE);
        m
    }

    pub fn add_by_tags(&mut self, tags: SymbolTags) {
        self.selectors.push(Selector::Tag {
            additive: true,
            tags,
        });
        self.dirty = true;
    }

    pub fn remove_by_tags(&mut self, tags: SymbolTags) {
        self.selectors.push(Selector::Tag {
            additive: false,
            tags,
        });
        self.dirty = true;
    }

    pub fn add_by_range(&mut self, first: u32, last: u32) {
        self.selectors.push(Selector::Range {
            additive: true,
            first,
            last,
        });
        self.dirty = true;
    }

    pub fn remove_by_range(&mut self, first: u32, last: u32) {
        self.selectors.push(Selector::Range {
            additive: false,
            first,
            last,
        });
        self.dirty = true;
    }

    /// Apply a `--symbols`-style selector string. Port of `parse_selectors`.
    pub fn apply_selectors(&mut self, s: &str) -> Result<(), SelectorParseError> {
        let parsed = parse_selectors(s)?;
        if parsed.do_clear {
            self.selectors.clear();
        }
        self.selectors.extend(parsed.selectors);
        self.dirty = true;
        Ok(())
    }

    /// Compiled narrow symbols (popcount-sorted). Triggers a rebuild if dirty.
    pub fn symbols(&mut self) -> &[Symbol] {
        self.ensure_built();
        &self.symbols
    }

    /// Packed narrow bitmaps, parallel to [`SymbolMap::symbols`].
    pub fn packed_bitmaps(&mut self) -> &[u64] {
        self.ensure_built();
        &self.packed_bitmaps
    }

    pub fn wide_symbols(&mut self) -> &[WideSymbol] {
        self.ensure_built();
        &self.symbols_wide
    }

    pub fn n_symbols(&mut self) -> usize {
        self.ensure_built();
        self.symbols.len()
    }

    pub fn n_wide_symbols(&mut self) -> usize {
        self.ensure_built();
        self.symbols_wide.len()
    }

    /// Force compilation now.
    pub fn prepare(&mut self) {
        self.ensure_built();
    }

    /// Immutable view of the compiled narrow symbols. Caller must have
    /// [`prepare`](Self::prepare)d first.
    pub fn symbols_ref(&self) -> &[Symbol] {
        debug_assert!(!self.dirty, "SymbolMap not prepared");
        &self.symbols
    }

    /// Immutable view of the compiled wide symbols (must be prepared).
    pub fn wide_symbols_ref(&self) -> &[WideSymbol] {
        debug_assert!(!self.dirty, "SymbolMap not prepared");
        &self.symbols_wide
    }

    /// Whether codepoint `c` is present in the compiled map (narrow or wide).
    /// Port of `chafa_symbol_map_has_symbol`.
    pub fn has_symbol(&self, c: char) -> bool {
        self.symbols.iter().any(|s| s.c == c) || self.symbols_wide.iter().any(|s| s.sym[0].c == c)
    }

    fn ensure_built(&mut self) {
        if self.dirty {
            self.rebuild();
            self.dirty = false;
        }
    }

    /// Port of `rebuild_symbols` + `compile_symbols`. Builtin glyphs only (user
    /// glyph import is not in scope for this port).
    fn rebuild(&mut self) {
        // Narrow: filter, dedup by codepoint (last wins, matching
        // g_hash_table_replace over chafa_symbols order).
        let mut narrow_by_cp: HashMap<u32, Symbol> = HashMap::new();
        let mut narrow_order: Vec<u32> = Vec::new();
        if self.use_builtin_glyphs {
            for sym in builtin_narrow() {
                if char_is_selected(&self.selectors, sym.tags, sym.c) {
                    let cp = sym.c as u32;
                    if narrow_by_cp.insert(cp, sym).is_none() {
                        narrow_order.push(cp);
                    }
                }
            }
        }
        let mut symbols: Vec<Symbol> = narrow_order
            .into_iter()
            .map(|cp| narrow_by_cp.remove(&cp).unwrap())
            .collect();
        // Total order: (popcount, codepoint). Deterministic; see module docs.
        symbols.sort_by(|a, b| {
            a.popcount
                .cmp(&b.popcount)
                .then((a.c as u32).cmp(&(b.c as u32)))
        });
        self.packed_bitmaps = symbols.iter().map(|s| s.bitmap).collect();
        self.symbols = symbols;

        // Wide: same treatment, ranked by combined popcount then codepoint.
        let mut wide_by_cp: HashMap<u32, WideSymbol> = HashMap::new();
        let mut wide_order: Vec<u32> = Vec::new();
        if self.use_builtin_glyphs {
            for sym in builtin_wide() {
                if char_is_selected(&self.selectors, sym.sym[0].tags, sym.sym[0].c) {
                    let cp = sym.sym[0].c as u32;
                    if wide_by_cp.insert(cp, sym).is_none() {
                        wide_order.push(cp);
                    }
                }
            }
        }
        let mut wide: Vec<WideSymbol> = wide_order
            .into_iter()
            .map(|cp| wide_by_cp.remove(&cp).unwrap())
            .collect();
        wide.sort_by(|a, b| {
            let pa = a.sym[0].popcount + a.sym[1].popcount;
            let pb = b.sym[0].popcount + b.sym[1].popcount;
            pa.cmp(&pb)
                .then((a.sym[0].c as u32).cmp(&(b.sym[0].c as u32)))
        });
        self.packed_bitmaps_wide = wide
            .iter()
            .flat_map(|w| [w.sym[0].bitmap, w.sym[1].bitmap])
            .collect();
        self.symbols_wide = wide;
    }

    /// Find up to `n_candidates_inout` narrow shape candidates for `bitmap`.
    /// Port of `chafa_symbol_map_find_candidates`.
    pub fn find_candidates(
        &self,
        bitmap: u64,
        do_inverse: bool,
        out: &mut [Candidate],
        n_candidates_inout: &mut usize,
    ) {
        let mut candidates = [Candidate {
            symbol_index: 0,
            hamming_distance: 65,
            is_inverted: false,
        }; N_CANDIDATES_MAX];

        for (i, &packed) in self.packed_bitmaps.iter().enumerate() {
            let hd = (bitmap ^ packed).count_ones() as u8;
            if hd < candidates[N_CANDIDATES_MAX - 1].hamming_distance {
                insert_candidate(
                    &mut candidates,
                    Candidate {
                        symbol_index: i,
                        hamming_distance: hd,
                        is_inverted: false,
                    },
                );
            }
            if do_inverse {
                let hd = 64 - hd;
                if hd < candidates[N_CANDIDATES_MAX - 1].hamming_distance {
                    insert_candidate(
                        &mut candidates,
                        Candidate {
                            symbol_index: i,
                            hamming_distance: hd,
                            is_inverted: true,
                        },
                    );
                }
            }
        }

        finish_candidates(&candidates, 64, out, n_candidates_inout);
    }

    /// Find up to `n_candidates_inout` wide shape candidates for the two-cell
    /// `bitmaps`. Port of `chafa_symbol_map_find_wide_candidates`.
    pub fn find_wide_candidates(
        &self,
        bitmaps: [u64; 2],
        do_inverse: bool,
        out: &mut [Candidate],
        n_candidates_inout: &mut usize,
    ) {
        let mut candidates = [Candidate {
            symbol_index: 0,
            hamming_distance: 129,
            is_inverted: false,
        }; N_CANDIDATES_MAX];

        for i in 0..self.symbols_wide.len() {
            let s0 = self.packed_bitmaps_wide[i * 2];
            let s1 = self.packed_bitmaps_wide[i * 2 + 1];
            let hd = ((bitmaps[0] ^ s0).count_ones() + (bitmaps[1] ^ s1).count_ones()) as u8;
            if hd < candidates[N_CANDIDATES_MAX - 1].hamming_distance {
                insert_candidate(
                    &mut candidates,
                    Candidate {
                        symbol_index: i,
                        hamming_distance: hd,
                        is_inverted: false,
                    },
                );
            }
            if do_inverse {
                let hd = 128 - hd;
                if hd < candidates[N_CANDIDATES_MAX - 1].hamming_distance {
                    insert_candidate(
                        &mut candidates,
                        Candidate {
                            symbol_index: i,
                            hamming_distance: hd,
                            is_inverted: true,
                        },
                    );
                }
            }
        }

        finish_candidates(&candidates, 128, out, n_candidates_inout);
    }
}

/// Truncate the sorted candidate buffer at the first sentinel and copy out.
fn finish_candidates(
    candidates: &[Candidate; N_CANDIDATES_MAX],
    max_hd: u8,
    out: &mut [Candidate],
    n_candidates_inout: &mut usize,
) {
    let mut i = 0;
    while i < N_CANDIDATES_MAX {
        if candidates[i].hamming_distance > max_hd {
            break;
        }
        i += 1;
    }
    let n = i.min(*n_candidates_inout);
    out[..n].copy_from_slice(&candidates[..n]);
    *n_candidates_inout = n;
}

/// Port of `insert_candidate` (`chafa-symbol-map.c:1153-1174`): insert into the
/// hamming-sorted fixed array, shifting worse entries right. Among equal
/// distances, earlier-inserted entries stay ahead (the new one goes after).
fn insert_candidate(candidates: &mut [Candidate; N_CANDIDATES_MAX], new_cand: Candidate) {
    let mut i = N_CANDIDATES_MAX - 1;
    while i != 0 {
        i -= 1;
        if new_cand.hamming_distance >= candidates[i].hamming_distance {
            // Shift [i+1 .. N-1) right by one, insert at i+1.
            candidates.copy_within(i + 1..N_CANDIDATES_MAX - 1, i + 2);
            candidates[i + 1] = new_cand;
            return;
        }
    }
    candidates.copy_within(0..N_CANDIDATES_MAX - 1, 1);
    candidates[0] = new_cand;
}

/// Port of `char_is_selected` (`chafa-symbol-map.c:477-536`).
pub fn char_is_selected(selectors: &[Selector], tags: SymbolTags, c: char) -> bool {
    // Always exclude characters that would mangle the output.
    if !is_printable(c) || is_zero_width(c) || c == '\t' {
        return false;
    }
    if is_rtl(c) {
        return false;
    }

    let mut auto_exclude_tags = SymbolTags::BAD;
    let mut is_selected = false;

    for sel in selectors {
        match *sel {
            Selector::Tag { additive, tags: st } => {
                if tags.intersects(st) {
                    is_selected = additive;
                    auto_exclude_tags &= !st;
                }
            }
            Selector::Range {
                additive,
                first,
                last,
            } => {
                let cp = c as u32;
                if cp >= first && cp <= last {
                    is_selected = additive;
                }
            }
        }
    }

    if tags.intersects(auto_exclude_tags) {
        is_selected = false;
    }

    is_selected
}

/// `g_unichar_isprint` approximation: everything except control, format,
/// surrogate, unassigned, and line/paragraph separators.
fn is_printable(c: char) -> bool {
    !matches!(
        c.general_category(),
        GeneralCategory::Control
            | GeneralCategory::Format
            | GeneralCategory::Surrogate
            | GeneralCategory::Unassigned
            | GeneralCategory::LineSeparator
            | GeneralCategory::ParagraphSeparator
    )
}

/// `g_unichar_iszerowidth` approximation: combining marks, Jamo medial/final,
/// and ZWSP; soft hyphen excluded. Matches GLib's special cases closely enough
/// for symbol selection.
fn is_zero_width(c: char) -> bool {
    let cp = c as u32;
    if cp == 0x00ad {
        return false;
    }
    matches!(
        c.general_category(),
        GeneralCategory::NonspacingMark | GeneralCategory::EnclosingMark
    ) || (0x1160..0x1200).contains(&cp)
        || cp == 0x200b
}

/// RTL exclusion: chafa drops Arabic/Hebrew/Thaana/Syriac scripts. We use block
/// ranges (no script DB dependency); covers the relevant builtin-free space.
fn is_rtl(c: char) -> bool {
    let cp = c as u32;
    (0x0590..=0x05ff).contains(&cp) // Hebrew
        || (0x0600..=0x06ff).contains(&cp) // Arabic
        || (0x0700..=0x074f).contains(&cp) // Syriac
        || (0x0750..=0x077f).contains(&cp) // Arabic Supplement
        || (0x0780..=0x07bf).contains(&cp) // Thaana
        || (0x08a0..=0x08ff).contains(&cp) // Arabic Extended-A
        || (0xfb1d..=0xfb4f).contains(&cp) // Hebrew presentation forms
        || (0xfb50..=0xfdff).contains(&cp) // Arabic presentation forms-A
        || (0xfe70..=0xfeff).contains(&cp) // Arabic presentation forms-B
}

// --- Selector string parser (port of parse_selectors / parse_symbol_tag) ---

struct ParsedSelectors {
    selectors: Vec<Selector>,
    do_clear: bool,
}

fn tag_for_name(name: &str) -> Option<SymbolTags> {
    let t = match name.to_ascii_lowercase().as_str() {
        "all" => SymbolTags::ALL,
        "none" => SymbolTags::empty(),
        "space" => SymbolTags::SPACE,
        "solid" => SymbolTags::SOLID,
        "stipple" => SymbolTags::STIPPLE,
        "block" => SymbolTags::BLOCK,
        "border" => SymbolTags::BORDER,
        "diagonal" => SymbolTags::DIAGONAL,
        "dot" => SymbolTags::DOT,
        "quad" => SymbolTags::QUAD,
        "half" => SymbolTags::HALF,
        "hhalf" => SymbolTags::HHALF,
        "vhalf" => SymbolTags::VHALF,
        "inverted" => SymbolTags::INVERTED,
        "braille" => SymbolTags::BRAILLE,
        "sextant" => SymbolTags::SEXTANT,
        "wedge" => SymbolTags::WEDGE,
        "technical" => SymbolTags::TECHNICAL,
        "geometric" => SymbolTags::GEOMETRIC,
        "ascii" => SymbolTags::ASCII,
        "alpha" => SymbolTags::ALPHA,
        "digit" => SymbolTags::DIGIT,
        "narrow" => SymbolTags::NARROW,
        "wide" => SymbolTags::WIDE,
        "ambiguous" => SymbolTags::AMBIGUOUS,
        "ugly" => SymbolTags::UGLY,
        "extra" => SymbolTags::EXTRA,
        "alnum" => SymbolTags::ALNUM,
        "bad" => SymbolTags::BAD,
        "legacy" => SymbolTags::LEGACY,
        "latin" => SymbolTags::LATIN,
        "import" | "imported" => SymbolTags::IMPORTED,
        "octant" => SymbolTags::OCTANT,
        _ => return None,
    };
    Some(t)
}

/// Parse a single codepoint token: optional `u`/`U` then optional `0x`, then
/// hex digits. Port of `parse_code_point`. Returns `(codepoint, consumed_len)`.
fn parse_code_point(s: &str) -> Option<(u32, usize)> {
    let b = s.as_bytes();
    let mut i = 0;
    if !b.is_empty() && (b[0] == b'u' || b[0] == b'U') {
        i += 1;
    }
    if b.len() >= 2 && b[0] == b'0' && b[1] == b'x' {
        i += 2;
    }
    let mut code: u32 = 0;
    let mut any = false;
    while i < b.len() {
        let c = b[i];
        let d = match c {
            b'0'..=b'9' => (c - b'0') as u32,
            b'a'..=b'f' => (c - b'a' + 10) as u32,
            b'A'..=b'F' => (c - b'A' + 10) as u32,
            _ => break,
        };
        code = code * 16 + d;
        any = true;
        i += 1;
    }
    if any { Some((code, i)) } else { None }
}

enum TokenKind {
    Tag(SymbolTags),
    Range(u32, u32),
}

fn parse_symbol_tag(token: &str) -> Result<TokenKind, SelectorParseError> {
    if let Some(t) = tag_for_name(token) {
        return Ok(TokenKind::Tag(t));
    }
    // Range or single code point.
    let (first, consumed) = parse_code_point(token)
        .ok_or_else(|| SelectorParseError(format!("Unrecognized symbol tag '{token}'.")))?;
    if consumed == token.len() {
        return Ok(TokenKind::Range(first, first));
    }
    // Expect "..last".
    let rest = &token[consumed..];
    if let Some(after) = rest.strip_prefix("..")
        && let Some((last, c2)) = parse_code_point(after)
            && c2 == after.len() {
                return Ok(TokenKind::Range(first, last));
            }
    Err(SelectorParseError(format!(
        "Unrecognized symbol tag '{token}'."
    )))
}

fn parse_selectors(s: &str) -> Result<ParsedSelectors, SelectorParseError> {
    let mut selectors = Vec::new();
    let mut do_clear = false;
    let mut is_add = false;
    let mut is_remove = false;

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Skip separators (space/comma).
        while i < chars.len() && (chars[i] == ' ' || chars[i] == ',') {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        if chars[i] == '-' {
            is_add = false;
            is_remove = true;
            i += 1;
        } else if chars[i] == '+' {
            is_add = true;
            is_remove = false;
            i += 1;
        }
        while i < chars.len() && chars[i] == ' ' {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        if !is_add && !is_remove {
            do_clear = true;
            is_add = true;
        }

        if chars[i] == '[' {
            // Literal set: each char added/removed as a (c,c) range.
            i += 1;
            let mut escape = false;
            let mut closed = false;
            while i < chars.len() {
                let c = chars[i];
                if c == '\\' && !escape {
                    escape = true;
                    i += 1;
                    continue;
                }
                if c == ']' && !escape {
                    closed = true;
                    i += 1;
                    break;
                }
                selectors.push(Selector::Range {
                    additive: is_add,
                    first: c as u32,
                    last: c as u32,
                });
                escape = false;
                i += 1;
            }
            if !closed {
                return Err(SelectorParseError(
                    "Syntax error in symbol selector set.".into(),
                ));
            }
        } else {
            // Token of [a-zA-Z0-9.].
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '.') {
                i += 1;
            }
            if i == start {
                return Err(SelectorParseError(
                    "Syntax error in symbol tag selectors.".into(),
                ));
            }
            let token: String = chars[start..i].iter().collect();
            match parse_symbol_tag(&token)? {
                TokenKind::Tag(t) => selectors.push(Selector::Tag {
                    additive: is_add,
                    tags: t,
                }),
                TokenKind::Range(first, last) => selectors.push(Selector::Range {
                    additive: is_add,
                    first,
                    last,
                }),
            }
        }
    }

    Ok(ParsedSelectors {
        selectors,
        do_clear,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_map_is_block_border_space_minus_wide() {
        let mut m = SymbolMap::chafa_default();
        m.prepare();
        // Space is selected.
        assert!(m.symbols().iter().any(|s| s.c == ' '));
        // A wide kana must NOT be in the narrow set, and wide set is empty
        // because -wide removed all wide symbols.
        assert_eq!(m.n_wide_symbols(), 0);
        // Sorted ascending by popcount.
        let pcs: Vec<u32> = m.symbols().iter().map(|s| s.popcount).collect();
        assert!(pcs.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn selector_string_clear_and_add() {
        let mut m = SymbolMap::new();
        m.apply_selectors("ascii").unwrap();
        m.prepare();
        // All selected symbols carry the ASCII tag.
        assert!(
            m.symbols()
                .iter()
                .all(|s| s.tags.contains(SymbolTags::ASCII))
        );
        assert!(!m.symbols().is_empty());
    }

    #[test]
    fn selector_range_and_literal_set() {
        let mut m = SymbolMap::new();
        m.apply_selectors("0x20..0x7e").unwrap();
        m.prepare();
        assert!(m.symbols().iter().any(|s| s.c == 'A'));

        let mut m2 = SymbolMap::new();
        m2.apply_selectors("[ABC]").unwrap();
        m2.prepare();
        let cps: Vec<char> = m2.symbols().iter().map(|s| s.c).collect();
        assert!(cps.contains(&'A') && cps.contains(&'B') && cps.contains(&'C'));
    }

    #[test]
    fn insert_candidate_orders_by_hamming() {
        let mut m = SymbolMap::chafa_default();
        m.prepare();
        let mut out = [Candidate {
            symbol_index: 0,
            hamming_distance: 0,
            is_inverted: false,
        }; N_CANDIDATES_MAX];
        let mut n = 5;
        m.find_candidates(0u64, false, &mut out, &mut n);
        // Distances must be non-decreasing.
        let hds: Vec<u8> = out[..n].iter().map(|c| c.hamming_distance).collect();
        assert!(hds.windows(2).all(|w| w[0] <= w[1]));
        assert!(n <= 5);
    }
}
