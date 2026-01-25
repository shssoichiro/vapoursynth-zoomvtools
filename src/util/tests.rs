#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use super::*;

#[test]
fn vs_bitblt_same_stride() {
    // Test case where src_stride == dst_stride == row_size
    let src = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut dest = vec![0u8; 9];
    let stride = NonZeroUsize::new(3).unwrap();
    let row_size = NonZeroUsize::new(3).unwrap();
    let height = NonZeroUsize::new(3).unwrap();

    vs_bitblt(&mut dest, stride, &src, stride, row_size, height);

    assert_eq!(dest, src, "Entire buffer should be copied exactly");
}

#[test]
fn vs_bitblt_different_stride() {
    // Test case where strides are larger than row_size
    let src = vec![
        1u8, 2, 3, 0, 0, // src_stride = 5
        4, 5, 6, 0, 0, 7, 8, 9, 0, 0,
    ];
    let mut dest = vec![
        0u8, 0, 0, 0, 0, 0, // dest_stride = 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let src_stride = NonZeroUsize::new(5).unwrap();
    let dest_stride = NonZeroUsize::new(6).unwrap();
    let row_size = NonZeroUsize::new(3).unwrap();
    let height = NonZeroUsize::new(3).unwrap();

    vs_bitblt(&mut dest, dest_stride, &src, src_stride, row_size, height);

    // Check that each row was copied correctly
    assert_eq!(&dest[0..3], &[1, 2, 3], "First row should match");
    assert_eq!(&dest[6..9], &[4, 5, 6], "Second row should match");
    assert_eq!(&dest[12..15], &[7, 8, 9], "Third row should match");

    // Check that padding remains untouched
    assert_eq!(
        &dest[3..6],
        &[0, 0, 0],
        "First row padding should be unchanged"
    );
    assert_eq!(
        &dest[9..12],
        &[0, 0, 0],
        "Second row padding should be unchanged"
    );
    assert_eq!(
        &dest[15..18],
        &[0, 0, 0],
        "Third row padding should be unchanged"
    );
}

#[test]
fn median_distinct_values() {
    // Test all permutations of three distinct values
    assert_eq!(median(1, 2, 3), 2);
    assert_eq!(median(1, 3, 2), 2);
    assert_eq!(median(2, 1, 3), 2);
    assert_eq!(median(2, 3, 1), 2);
    assert_eq!(median(3, 1, 2), 2);
    assert_eq!(median(3, 2, 1), 2);
}

#[test]
fn median_duplicate_values() {
    // Test cases with two identical values
    assert_eq!(median(1, 1, 2), 1);
    assert_eq!(median(1, 2, 1), 1);
    assert_eq!(median(2, 1, 1), 1);

    assert_eq!(median(1, 2, 2), 2);
    assert_eq!(median(2, 1, 2), 2);
    assert_eq!(median(2, 2, 1), 2);

    assert_eq!(median(5, 5, 3), 5);
    assert_eq!(median(5, 3, 5), 5);
    assert_eq!(median(3, 5, 5), 5);
}

#[test]
fn median_all_same_values() {
    // Test cases where all three values are identical
    assert_eq!(median(5, 5, 5), 5);
    assert_eq!(median(0, 0, 0), 0);
    assert_eq!(median(100, 100, 100), 100);
}

#[test]
fn median_different_types() {
    // Test with different integer types
    assert_eq!(median(1u8, 2u8, 3u8), 2u8);
    assert_eq!(median(10u16, 20u16, 30u16), 20u16);
    assert_eq!(median(100u32, 200u32, 300u32), 200u32);
    assert_eq!(median(1000u64, 2000u64, 3000u64), 2000u64);

    // Test with signed integers
    assert_eq!(median(-1i32, 0i32, 1i32), 0i32);
    assert_eq!(median(-10i32, -5i32, -1i32), -5i32);

    // Test with characters
    assert_eq!(median('a', 'b', 'c'), 'b');
    assert_eq!(median('z', 'a', 'm'), 'm');
}

#[test]
fn median_edge_cases() {
    // Test with extreme values for u8
    assert_eq!(median(0u8, 255u8, 128u8), 128u8);
    assert_eq!(median(0u8, 0u8, 255u8), 0u8);
    assert_eq!(median(255u8, 255u8, 0u8), 255u8);

    // Test with negative numbers
    assert_eq!(median(-100i32, 0i32, 100i32), 0i32);
    assert_eq!(median(-1i32, -2i32, -3i32), -2i32);
}

#[test]
fn median_ordering_edge_cases() {
    // Test cases that might reveal issues with the ordering logic
    assert_eq!(median(10, 5, 15), 10);
    assert_eq!(median(15, 10, 5), 10);
    assert_eq!(median(5, 15, 10), 10);

    // Test with large differences
    assert_eq!(median(1, 1000, 500), 500);
    assert_eq!(median(1000, 1, 500), 500);
    assert_eq!(median(500, 1000, 1), 500);
}

#[test]
fn round_ties_to_even_ties() {
    // Positive ties
    assert_eq!(round_ties_to_even(0.5), 0.0);
    assert_eq!(round_ties_to_even(1.5), 2.0);
    assert_eq!(round_ties_to_even(2.5), 2.0);
    assert_eq!(round_ties_to_even(3.5), 4.0);

    // Negative ties
    assert_eq!(round_ties_to_even(-0.5), 0.0);
    assert_eq!(round_ties_to_even(-1.5), -2.0);
    assert_eq!(round_ties_to_even(-2.5), -2.0);
    assert_eq!(round_ties_to_even(-3.5), -4.0);
}

#[test]
fn round_ties_to_even_non_ties() {
    // Values just below/above .5 should round away from or toward zero accordingly
    assert_eq!(round_ties_to_even(1.4999), 1.0);
    assert_eq!(round_ties_to_even(1.5001), 2.0);
    assert_eq!(round_ties_to_even(-1.4999), -1.0);
    assert_eq!(round_ties_to_even(-1.5001), -2.0);

    // Near zero
    assert_eq!(round_ties_to_even(0.49), 0.0);
    assert_eq!(round_ties_to_even(0.51), 1.0);
    assert_eq!(round_ties_to_even(-0.49), 0.0);
    assert_eq!(round_ties_to_even(-0.51), -1.0);
}

#[test]
fn round_ties_to_even_integers_and_bounds() {
    // Exact integers remain unchanged
    assert_eq!(round_ties_to_even(2.0), 2.0);
    assert_eq!(round_ties_to_even(-2.0), -2.0);
    assert_eq!(round_ties_to_even(0.0), 0.0);

    // Large values at tie boundaries
    assert_eq!(round_ties_to_even(123456.5), 123456.0); // even
    assert_eq!(round_ties_to_even(123455.5), 123456.0); // odd -> up
    assert_eq!(round_ties_to_even(-123456.5), -123456.0); // even
    assert_eq!(round_ties_to_even(-123455.5), -123456.0); // odd -> down
}
