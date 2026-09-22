//! Unicode tag derivation — port of `get_default_tags_for_char`
//! (`chafa-symbols.c:520-560`) and its helper ranges.
//!
//! chafa uses GLib's Unicode classification (`g_unichar_iswide`, `isalpha`,
//! etc.). We approximate those with the `unicode-width` and `unicode-properties`
//! crates. For the **builtin** symbol set this derivation is validated
//! exhaustively against chafa's own dump (see the Phase 2 integration test), so
//! any classification divergence surfaces loudly rather than silently.

use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_width::UnicodeWidthChar;

use super::tags::SymbolTags;

/// Inclusive codepoint range.
struct Range {
    first: u32,
    last: u32,
}

const fn r(first: u32, last: u32) -> Range {
    Range { first, last }
}

/// Ranges chafa treats as ambiguous-width beyond GLib's set
/// (`chafa-symbols.c:93-117`).
const AMBIGUOUS_RANGES: &[Range] = &[
    r(0x00ad, 0x00ad),
    r(0x2196, 0x21ff),
    r(0x222c, 0x2237),
    r(0x2245, 0x2269),
    r(0x226d, 0x2279),
    r(0x2295, 0x22af),
    r(0x22bf, 0x22bf),
    r(0x22c8, 0x22ff),
    r(0x2300, 0x23ff),
    r(0x2460, 0x24ff),
    r(0x25a0, 0x25ff),
    r(0x2700, 0x27bf),
    r(0x27c0, 0x27e5),
    r(0x27f0, 0x27ff),
    r(0x2900, 0x297f),
    r(0x2980, 0x29ff),
    r(0x2b00, 0x2bff),
    r(0x1f100, 0x1f1ff),
];

/// Emoji / multicolored ranges chafa tags `UGLY` (`chafa-symbols.c:122-134`).
const EMOJI_RANGES: &[Range] = &[
    r(0x2600, 0x26ff),
    r(0x1f000, 0x1fb3b),
    r(0x1fbcb, 0x1ffff),
    r(0x534d, 0x534d),
];

/// Meta ranges chafa tags `UGLY` (`chafa-symbols.c:136-148`).
const META_RANGES: &[Range] = &[r(0x0640, 0x0640), r(0x2ff0, 0x2fff)];

fn in_ranges(c: u32, ranges: &[Range]) -> bool {
    ranges.iter().any(|r| c >= r.first && c <= r.last)
}

/// `is_private_use` (`chafa-symbols.c:512-518`).
fn is_private_use(c: u32) -> bool {
    (0xe000..=0xf8ff).contains(&c)
        || (0xf0000..=0xfffff).contains(&c)
        || (0x100000..=0x10ffff).contains(&c)
}

/// `g_unichar_iswide` approximation: East Asian Width Wide or Fullwidth.
fn is_wide(c: char) -> bool {
    c.width() == Some(2)
}

/// `g_unichar_iswide_cjk` approximation: wide *or* ambiguous (CJK context).
fn is_wide_cjk(c: char) -> bool {
    c.width_cjk() == Some(2)
}

/// `g_unichar_isalpha`: general category is one of the letter categories.
fn is_alpha(c: char) -> bool {
    matches!(
        c.general_category(),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}

/// `g_unichar_isdigit`: decimal digit (Nd).
fn is_digit(c: char) -> bool {
    c.general_category() == GeneralCategory::DecimalNumber
}

/// `g_unichar_ismark`: a combining mark (Mn/Mc/Me).
fn is_mark(c: char) -> bool {
    matches!(
        c.general_category(),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    )
}

/// `g_unichar_iszerowidth` approximation: combining marks and format chars
/// (excluding a few that GLib special-cases). Only feeds the `AMBIGUOUS` tag,
/// which is masked off for builtin symbols, so precision here matters only for
/// user-supplied ranges.
fn is_zero_width(c: char) -> bool {
    // GLib: zero-width if the char is a non-spacing mark, an enclosing mark, or
    // a format character (Cf) other than a handful of exceptions; plus the
    // explicit ZWSP/ZWNJ/ZWJ. unicode-width returns Some(0) for these.
    c.width() == Some(0) && c != '\u{00ad}'
}

/// Port of `get_default_tags_for_char` (`chafa-symbols.c:520-560`).
pub fn get_default_tags_for_char(c: char) -> SymbolTags {
    let cp = c as u32;
    let mut tags = SymbolTags::empty();

    if is_wide(c) {
        tags |= SymbolTags::WIDE;
    } else if is_wide_cjk(c) && !is_private_use(cp) {
        tags |= SymbolTags::AMBIGUOUS;
    }

    if is_mark(c) || is_zero_width(c) || in_ranges(cp, AMBIGUOUS_RANGES) {
        tags |= SymbolTags::AMBIGUOUS;
    }

    if in_ranges(cp, EMOJI_RANGES) || in_ranges(cp, META_RANGES) {
        tags |= SymbolTags::UGLY;
    }

    if cp <= 0x7f {
        tags |= SymbolTags::ASCII;
    } else if (0x2300..=0x23ff).contains(&cp) {
        tags |= SymbolTags::TECHNICAL;
    } else if (0x25a0..=0x25ff).contains(&cp) {
        tags |= SymbolTags::GEOMETRIC;
    } else if (0x2800..=0x28ff).contains(&cp) {
        tags |= SymbolTags::BRAILLE;
    } else if (0x1fb00..=0x1fb3b).contains(&cp) {
        tags |= SymbolTags::SEXTANT;
    }

    if is_alpha(c) {
        tags |= SymbolTags::ALPHA;
    }
    if is_digit(c) {
        tags |= SymbolTags::DIGIT;
    }

    if !tags.contains(SymbolTags::WIDE) {
        tags |= SymbolTags::NARROW;
    }

    tags
}

/// Final tags for a **parsed builtin** symbol: author tags OR'd with the derived
/// tags, but with `AMBIGUOUS` masked off — exactly `def_to_symbol`'s
/// `def->sc | (get_default_tags_for_char (c) & ~AMBIGUOUS)`
/// (`chafa-symbols.c:569`).
pub fn builtin_symbol_tags(author: SymbolTags, c: char) -> SymbolTags {
    author | (get_default_tags_for_char(c) & !SymbolTags::AMBIGUOUS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_letter_digit() {
        assert!(get_default_tags_for_char('A').contains(SymbolTags::ALPHA));
        assert!(get_default_tags_for_char('A').contains(SymbolTags::ASCII));
        assert!(get_default_tags_for_char('A').contains(SymbolTags::NARROW));
        assert!(get_default_tags_for_char('0').contains(SymbolTags::DIGIT));
        assert!(!get_default_tags_for_char('0').contains(SymbolTags::ALPHA));
    }

    #[test]
    fn ranges() {
        assert!(get_default_tags_for_char('\u{25a0}').contains(SymbolTags::GEOMETRIC));
        assert!(get_default_tags_for_char('\u{2300}').contains(SymbolTags::TECHNICAL));
    }
}
