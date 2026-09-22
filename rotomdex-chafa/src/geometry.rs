//! Geometry and core algorithm constants.
//!
//! Values mirror chafa's headers exactly (see references per-constant).

/// Symbol cell width in pixels (`chafa-symbol-map.h:29`).
pub const SYMBOL_WIDTH_PIXELS: usize = 8;
/// Symbol cell height in pixels (`chafa-symbol-map.h:30`).
pub const SYMBOL_HEIGHT_PIXELS: usize = 8;
/// Pixels per symbol cell = 64, i.e. one `u64` bitmap (`chafa-private.h:40`).
pub const SYMBOL_N_PIXELS: usize = SYMBOL_WIDTH_PIXELS * SYMBOL_HEIGHT_PIXELS;

/// Maximum number of shape candidates carried into Phase B
/// (`chafa-symbol-renderer.c:41`).
pub const N_CANDIDATES_MAX: usize = 8;

/// Sentinel "worse than any real error" value (`chafa-symbol-renderer.c:37`):
/// `G_MAXINT / 8`.
pub const SYMBOL_ERROR_MAX: i32 = i32::MAX / 8;

/// Ring-buffer size for per-cell scratch, enabling wide-symbol lookback
/// (`chafa-symbol-renderer.c:792`).
pub const N_BUF_CELLS: usize = 4;

/// Upper bound on auto-detected thread count (`chafa-features.c:49`).
pub const AUTO_THREAD_COUNT_MAX: usize = 24;

/// Map a pixel coordinate `(x, y)` within an 8x8 cell to its bit position in a
/// packed `u64` coverage bitmap.
///
/// Bit order is **MSB-first, row-major** (`coverage_to_bitmap`,
/// `chafa-symbols.c`): pixel `(x, y)` occupies bit `63 - (y*8 + x)`, so bit 63
/// is the top-left pixel and bit 0 is the bottom-right. This bit order is
/// load-bearing for shape matching — getting it wrong makes every match wrong.
#[inline]
pub const fn bit_index(x: usize, y: usize) -> u32 {
    (SYMBOL_N_PIXELS - 1 - (y * SYMBOL_WIDTH_PIXELS + x)) as u32
}

/// Map a linear pixel index `i` (`0..64`, row-major) to its bit position.
#[inline]
pub const fn bit_index_lin(i: usize) -> u32 {
    (SYMBOL_N_PIXELS - 1 - i) as u32
}

/// Pack a row-major coverage array (`coverage[i] ∈ {0, 1}`) into a `u64` bitmap.
///
/// Port of `coverage_to_bitmap` (`chafa-symbols.c`).
#[inline]
pub fn coverage_to_bitmap(coverage: &[u8; SYMBOL_N_PIXELS]) -> u64 {
    let mut bitmap = 0u64;
    for (i, &cov) in coverage.iter().enumerate() {
        if cov != 0 {
            bitmap |= 1u64 << bit_index_lin(i);
        }
    }
    bitmap
}

/// Unpack a `u64` bitmap into a row-major coverage array (inverse of
/// [`coverage_to_bitmap`]): `coverage[i] = (bitmap >> (63 - i)) & 1`.
#[inline]
pub fn bitmap_to_coverage(bitmap: u64) -> [u8; SYMBOL_N_PIXELS] {
    let mut coverage = [0u8; SYMBOL_N_PIXELS];
    for (i, cov) in coverage.iter_mut().enumerate() {
        *cov = ((bitmap >> bit_index_lin(i)) & 1) as u8;
    }
    coverage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_left_is_msb() {
        assert_eq!(bit_index(0, 0), 63);
        assert_eq!(bit_index(7, 7), 0);
        assert_eq!(bit_index_lin(0), 63);
        assert_eq!(bit_index_lin(63), 0);
    }

    #[test]
    fn coverage_bitmap_roundtrip() {
        let mut cov = [0u8; SYMBOL_N_PIXELS];
        // A few representative pixels.
        cov[0] = 1; // top-left -> bit 63
        cov[7] = 1; // top-right -> bit 56
        cov[63] = 1; // bottom-right -> bit 0
        cov[32] = 1; // middle-ish
        let bm = coverage_to_bitmap(&cov);
        assert_eq!(bm & (1 << 63), 1 << 63);
        assert_eq!(bm & (1 << 56), 1 << 56);
        assert_eq!(bm & 1, 1);
        assert_eq!(bitmap_to_coverage(bm), cov);
    }

    #[test]
    fn full_and_empty() {
        let full = [1u8; SYMBOL_N_PIXELS];
        assert_eq!(coverage_to_bitmap(&full), u64::MAX);
        let empty = [0u8; SYMBOL_N_PIXELS];
        assert_eq!(coverage_to_bitmap(&empty), 0);
        assert_eq!(bitmap_to_coverage(u64::MAX), full);
    }
}
