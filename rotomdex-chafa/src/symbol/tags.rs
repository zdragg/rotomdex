//! Symbol tag bitflags.
//!
//! Exact port of `ChafaSymbolTags` (`chafa-symbol-map.h:32-69`). Bit positions
//! are load-bearing: they must match chafa so that the `--symbols` selector
//! grammar and the per-symbol tag values agree bit-for-bit.

use bitflags::bitflags;

bitflags! {
    /// Classification tags attached to each symbol.
    ///
    /// Single-bit flags plus the composite masks `HALF`, `ALNUM`, `BAD`, `ALL`.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct SymbolTags: u32 {
        const SPACE      = 1 << 0;
        const SOLID      = 1 << 1;
        const STIPPLE    = 1 << 2;
        const BLOCK      = 1 << 3;
        const BORDER     = 1 << 4;
        const DIAGONAL   = 1 << 5;
        const DOT        = 1 << 6;
        const QUAD       = 1 << 7;
        const HHALF      = 1 << 8;
        const VHALF      = 1 << 9;
        const INVERTED   = 1 << 10;
        const BRAILLE    = 1 << 11;
        const TECHNICAL  = 1 << 12;
        const GEOMETRIC  = 1 << 13;
        const ASCII      = 1 << 14;
        const ALPHA      = 1 << 15;
        const DIGIT      = 1 << 16;
        const NARROW     = 1 << 17;
        const WIDE       = 1 << 18;
        const AMBIGUOUS  = 1 << 19;
        const UGLY       = 1 << 20;
        const LEGACY     = 1 << 21;
        const SEXTANT    = 1 << 22;
        const WEDGE      = 1 << 23;
        const LATIN      = 1 << 24;
        const IMPORTED   = 1 << 25;
        const OCTANT     = 1 << 26;
        const EXTRA      = 1 << 30;

        // Composites.
        const HALF  = (1 << 8) | (1 << 9);                 // HHALF | VHALF
        const ALNUM = (1 << 15) | (1 << 16);               // ALPHA | DIGIT
        const BAD   = (1 << 19) | (1 << 20);               // AMBIGUOUS | UGLY
        /// Everything except `EXTRA` and the `BAD` (ambiguous/ugly) bits.
        const ALL   = !((1 << 30) | (1 << 19) | (1 << 20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_positions_match_chafa() {
        assert_eq!(SymbolTags::SPACE.bits(), 1);
        assert_eq!(SymbolTags::OCTANT.bits(), 1 << 26);
        assert_eq!(SymbolTags::EXTRA.bits(), 1 << 30);
        assert_eq!(SymbolTags::HALF, SymbolTags::HHALF | SymbolTags::VHALF);
        assert_eq!(SymbolTags::ALNUM, SymbolTags::ALPHA | SymbolTags::DIGIT);
        assert_eq!(SymbolTags::BAD, SymbolTags::AMBIGUOUS | SymbolTags::UGLY);
    }

    #[test]
    fn all_excludes_extra_and_bad() {
        assert!(!SymbolTags::ALL.contains(SymbolTags::EXTRA));
        assert!(!SymbolTags::ALL.contains(SymbolTags::AMBIGUOUS));
        assert!(!SymbolTags::ALL.contains(SymbolTags::UGLY));
        assert!(SymbolTags::ALL.contains(SymbolTags::BLOCK));
    }

    #[test]
    fn space_symbol_sc_matches_dump() {
        // chafa dump: U+0020 sc = 88227977 =
        // ASCII|LATIN|SPACE|BLOCK|QUAD|SEXTANT|OCTANT|NARROW.
        let sc = SymbolTags::ASCII
            | SymbolTags::LATIN
            | SymbolTags::SPACE
            | SymbolTags::BLOCK
            | SymbolTags::QUAD
            | SymbolTags::SEXTANT
            | SymbolTags::OCTANT
            | SymbolTags::NARROW;
        assert_eq!(sc.bits(), 88227977);
    }
}
