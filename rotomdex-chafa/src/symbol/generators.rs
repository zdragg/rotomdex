//! Procedurally generated symbol families: Braille, Sextant, Octant.
//!
//! Exact ports of the generator functions in `chafa-symbols.c`. Generated
//! symbols carry **fixed** tags (no derived merge, no `NARROW` bit), matching
//! chafa: braille = `BRAILLE`, sextant = `LEGACY|SEXTANT`, octant =
//! `LEGACY|OCTANT`.

use alloc::vec::Vec;

use crate::geometry::{SYMBOL_N_PIXELS, SYMBOL_WIDTH_PIXELS, coverage_to_bitmap};

/// A generated symbol: codepoint + packed coverage bitmap.
pub struct GenSymbol {
    pub c: char,
    pub bitmap: u64,
}

/// `gen_braille_sym` (`chafa-symbols.c:247-266`): map a Braille byte `val` to a
/// 2x4 dot pattern in columns {1,2} and {5,6} over row-pairs.
fn gen_braille_coverage(val: u8) -> [u8; SYMBOL_N_PIXELS] {
    let mut cov = [0u8; SYMBOL_N_PIXELS];
    let w = SYMBOL_WIDTH_PIXELS;
    let set = |cov: &mut [u8; SYMBOL_N_PIXELS], base: usize, bit: u8| {
        let v = (val >> bit) & 1;
        cov[base + 1] = v;
        cov[base + 2] = v;
    };
    let set2 = |cov: &mut [u8; SYMBOL_N_PIXELS], base: usize, bit: u8| {
        let v = (val >> bit) & 1;
        cov[base + 5] = v;
        cov[base + 6] = v;
    };
    set(&mut cov, 0, 0);
    set2(&mut cov, 0, 3);
    set(&mut cov, w * 2, 1);
    set2(&mut cov, w * 2, 4);
    set(&mut cov, w * 4, 2);
    set2(&mut cov, w * 4, 5);
    set(&mut cov, w * 6, 6);
    set2(&mut cov, w * 6, 7);
    cov
}

/// Generate the 256 Braille symbols (U+2800..=U+28FF), in codepoint order.
pub fn generate_braille() -> Vec<GenSymbol> {
    let mut out = Vec::with_capacity(256);
    for cp in 0x2800u32..0x2900 {
        let cov = gen_braille_coverage((cp - 0x2800) as u8);
        out.push(GenSymbol {
            c: char::from_u32(cp).unwrap(),
            bitmap: coverage_to_bitmap(&cov),
        });
    }
    out
}

/// `gen_sextant_sym` (`chafa-symbols.c:294-325`): render a 2x3 mosaic from the
/// low 6 bits of `val`.
fn gen_sextant_coverage(val: u8) -> [u8; SYMBOL_N_PIXELS] {
    let mut cov = [0u8; SYMBOL_N_PIXELS];
    for y in 0..3usize {
        for x in 0..2usize {
            let bit = y * 2 + x;
            if val & (1 << bit) != 0 {
                for v in 0..3usize {
                    for u in 0..4usize {
                        let mut row = y * 3 + v;
                        if row > 3 {
                            row -= 1;
                        }
                        cov[row * 8 + x * 4 + u] = 1;
                    }
                }
            }
        }
    }
    cov
}

/// Generate the 59 Sextant symbols (U+1FB00..=U+1FB3A), skipping the two values
/// that collide with the half/full block characters (the `>20`/`>41` bumps).
pub fn generate_sextant() -> Vec<GenSymbol> {
    let mut out = Vec::with_capacity(59);
    for cp in 0x1fb00u32..0x1fb3b {
        let mut bitmap = (cp - 0x1fb00 + 1) as i32;
        if bitmap > 20 {
            bitmap += 1;
        }
        if bitmap > 41 {
            bitmap += 1;
        }
        let cov = gen_sextant_coverage(bitmap as u8);
        out.push(GenSymbol {
            c: char::from_u32(cp).unwrap(),
            bitmap: coverage_to_bitmap(&cov),
        });
    }
    out
}

// --- Octant ---
//
// Based on code by Kang-Che Sung (MIT). Port of `chafa-symbols.c:362-510`.

struct OctantEntry {
    octant_bits: u8,
    data: u8,
}

const OCTANT_MAP: [OctantEntry; 26] = [
    OctantEntry {
        octant_bits: 0x00,
        data: 0x00,
    },
    OctantEntry {
        octant_bits: 0x01,
        data: 0xa8,
    },
    OctantEntry {
        octant_bits: 0x02,
        data: 0xab,
    },
    OctantEntry {
        octant_bits: 0x03,
        data: 0xc2,
    },
    OctantEntry {
        octant_bits: 0x05,
        data: 0x98,
    },
    OctantEntry {
        octant_bits: 0x0a,
        data: 0x9d,
    },
    OctantEntry {
        octant_bits: 0x0f,
        data: 0x80,
    },
    OctantEntry {
        octant_bits: 0x14,
        data: 0xe6,
    },
    OctantEntry {
        octant_bits: 0x28,
        data: 0xe7,
    },
    OctantEntry {
        octant_bits: 0x3f,
        data: 0xc5,
    },
    OctantEntry {
        octant_bits: 0x40,
        data: 0xa3,
    },
    OctantEntry {
        octant_bits: 0x50,
        data: 0x96,
    },
    OctantEntry {
        octant_bits: 0x55,
        data: 0x8c,
    },
    OctantEntry {
        octant_bits: 0x5a,
        data: 0x9e,
    },
    OctantEntry {
        octant_bits: 0x5f,
        data: 0x9b,
    },
    OctantEntry {
        octant_bits: 0x80,
        data: 0xa0,
    },
    OctantEntry {
        octant_bits: 0xa0,
        data: 0x97,
    },
    OctantEntry {
        octant_bits: 0xa5,
        data: 0x9a,
    },
    OctantEntry {
        octant_bits: 0xaa,
        data: 0x90,
    },
    OctantEntry {
        octant_bits: 0xaf,
        data: 0x9c,
    },
    OctantEntry {
        octant_bits: 0xc0,
        data: 0x82,
    },
    OctantEntry {
        octant_bits: 0xf0,
        data: 0x84,
    },
    OctantEntry {
        octant_bits: 0xf5,
        data: 0x99,
    },
    OctantEntry {
        octant_bits: 0xfa,
        data: 0x9f,
    },
    OctantEntry {
        octant_bits: 0xfc,
        data: 0x86,
    },
    OctantEntry {
        octant_bits: 0xff,
        data: 0x88,
    },
];

/// `find_unicode_octant_map_data` (`:398-418`): binary search; on miss returns
/// `-(first insertion index)` (matching the C signed-return convention).
fn find_octant_map_data(octant_bits: u8) -> i32 {
    let mut first = 0usize;
    let mut last = OCTANT_MAP.len();
    while first < last {
        let i = (first + last) / 2;
        let probe = OCTANT_MAP[i].octant_bits;
        if octant_bits == probe {
            return OCTANT_MAP[i].data as i32;
        }
        if octant_bits > probe {
            first = i + 1;
        } else {
            last = i;
        }
    }
    -(first as i32)
}

/// `octant_bits_to_unichar` (`:420-447`).
fn octant_bits_to_unichar(octant_bits: u8) -> u32 {
    let data = find_octant_map_data(octant_bits);

    if data < 0 {
        // ((octant_bits + data) as u32) | 0x1cd00; data is negative offset.
        return ((octant_bits as i32 + data) as u32) | 0x1cd00;
    }
    if data == 0x00 {
        return 0x00a0;
    }
    match (data >> 5) & 0x3 {
        0 => (data as u32 & 0x1f) | 0x2580,
        1 => (data as u32 & 0x1f) | 0x1cea0,
        2 => (data as u32 & 0x1f) | 0x1fb80,
        _ => (data as u32 & 0x1f) | 0x1fbe6,
    }
}

/// `octant_bits_to_coverage` (`:449-478`).
fn octant_bits_to_coverage(octant_bits: u8) -> [u8; SYMBOL_N_PIXELS] {
    let mut cov = [0u8; SYMBOL_N_PIXELS];
    for y in 0..SYMBOL_WIDTH_PIXELS {
        for x in 0..SYMBOL_WIDTH_PIXELS {
            let bit = (y & !1) + ((x >> 2) & 1);
            cov[y * SYMBOL_WIDTH_PIXELS + x] = (octant_bits >> bit) & 1;
        }
    }
    cov
}

/// Generate the Octant symbols (`:480-510`): iterate all 256 octant patterns,
/// keep only those whose codepoint lands in the dedicated octant block
/// (`0x1cd00..=0x1d000`), skipping ones that collide with existing block chars.
pub fn generate_octant() -> Vec<GenSymbol> {
    let mut out = Vec::new();
    for oct in 0u32..256 {
        let c = octant_bits_to_unichar(oct as u8);
        // Skip block symbols we already have (chafa: c < 0x1cd00 || c > 0x1d000).
        if !(0x1cd00..=0x1d000).contains(&c) {
            continue;
        }
        let cov = octant_bits_to_coverage(oct as u8);
        out.push(GenSymbol {
            c: char::from_u32(c).unwrap(),
            bitmap: coverage_to_bitmap(&cov),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(generate_braille().len(), 256);
        assert_eq!(generate_sextant().len(), 59);
        // Octant count verified against the chafa dump in the integration test.
        assert!(generate_octant().len() > 200);
    }

    #[test]
    fn braille_known_bitmaps() {
        let b = generate_braille();
        // U+2800: empty. U+2801: dots at (1,0),(2,0) -> bits 62,61.
        assert_eq!(b[0].bitmap, 0x0000000000000000);
        assert_eq!(b[1].bitmap, 0x6000000000000000);
    }

    #[test]
    fn sextant_first_bitmap() {
        // chafa dump: U+1FB00 bitmap = f0f0f00000000000.
        let s = generate_sextant();
        assert_eq!(s[0].c as u32, 0x1fb00);
        assert_eq!(s[0].bitmap, 0xf0f0f00000000000);
    }
}
