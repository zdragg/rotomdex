//! Symbols: the builtin glyph set (parsed outlines + procedural generators)
//! with chafa-exact tags, coverage bitmaps, popcounts and weights.

mod data;
pub mod derive;
mod generators;
pub mod tags;

use crate::geometry::{SYMBOL_N_PIXELS, bitmap_to_coverage};
use alloc::vec::Vec;
pub use derive::get_default_tags_for_char;
pub use tags::SymbolTags;

/// A narrow (8x8) symbol. Field semantics mirror chafa's `ChafaSymbol`.
#[derive(Clone, Debug)]
pub struct Symbol {
    /// Classification tags (final, post-derivation).
    pub tags: SymbolTags,
    /// The Unicode scalar.
    pub c: char,
    /// Packed coverage bitmap, MSB = top-left pixel.
    pub bitmap: u64,
    /// Number of set (foreground) pixels = `bitmap.count_ones()`.
    pub popcount: u32,
    /// Foreground weight (== `popcount`).
    pub fg_weight: u32,
    /// Background weight (== `64 - popcount`).
    pub bg_weight: u32,
}

impl Symbol {
    fn new(tags: SymbolTags, c: char, bitmap: u64) -> Self {
        let popcount = bitmap.count_ones();
        Symbol {
            tags,
            c,
            bitmap,
            popcount,
            fg_weight: popcount,
            bg_weight: SYMBOL_N_PIXELS as u32 - popcount,
        }
    }

    /// The row-major coverage array (`coverage[i] ∈ {0,1}`), derived from the
    /// bitmap. chafa stores this explicitly; we reconstruct it on demand.
    pub fn coverage(&self) -> [u8; SYMBOL_N_PIXELS] {
        bitmap_to_coverage(self.bitmap)
    }
}

/// A wide (16x8) symbol: two side-by-side 8x8 cells `[left, right]`.
/// Mirrors chafa's `ChafaSymbol2`.
#[derive(Clone, Debug)]
pub struct WideSymbol {
    pub sym: [Symbol; 2],
}

/// Build the full builtin **narrow** symbol set, in chafa's `chafa_symbols`
/// order: parsed defs (file order) then Braille, Sextant, Octant.
pub fn builtin_narrow() -> Vec<Symbol> {
    let mut out = Vec::with_capacity(1300);

    for &(author, cp, bitmap) in data::NARROW_DEFS {
        let c = char::from_u32(cp).unwrap();
        let author = SymbolTags::from_bits_retain(author);
        out.push(Symbol::new(
            derive::builtin_symbol_tags(author, c),
            c,
            bitmap,
        ));
    }

    // Generated families carry fixed tags (no derived merge, no NARROW bit).
    for g in generators::generate_braille() {
        out.push(Symbol::new(SymbolTags::BRAILLE, g.c, g.bitmap));
    }
    for g in generators::generate_sextant() {
        out.push(Symbol::new(
            SymbolTags::LEGACY | SymbolTags::SEXTANT,
            g.c,
            g.bitmap,
        ));
    }
    for g in generators::generate_octant() {
        out.push(Symbol::new(
            SymbolTags::LEGACY | SymbolTags::OCTANT,
            g.c,
            g.bitmap,
        ));
    }

    out
}

/// Build the full builtin **wide** symbol set, in chafa's `chafa_symbols2`
/// order. Both halves share the def's codepoint and (final) tags, matching
/// `def_to_symbol` being called twice with the same def.
pub fn builtin_wide() -> Vec<WideSymbol> {
    let mut out = Vec::with_capacity(data::WIDE_DEFS.len());
    for &(author, cp, left, right) in data::WIDE_DEFS {
        let c = char::from_u32(cp).unwrap();
        let author = SymbolTags::from_bits_retain(author);
        let tags = derive::builtin_symbol_tags(author, c);
        out.push(WideSymbol {
            sym: [Symbol::new(tags, c, left), Symbol::new(tags, c, right)],
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_count_matches_chafa() {
        // 712 parsed + 256 braille + 59 sextant + 234 octant = 1261.
        assert_eq!(builtin_narrow().len(), 1261);
    }

    #[test]
    fn wide_count_matches_chafa() {
        assert_eq!(builtin_wide().len(), 181);
    }

    #[test]
    fn space_symbol_tags() {
        let n = builtin_narrow();
        let space = n.iter().find(|s| s.c == ' ').unwrap();
        assert_eq!(space.tags.bits(), 88227977);
        assert_eq!(space.popcount, 0);
    }
}
