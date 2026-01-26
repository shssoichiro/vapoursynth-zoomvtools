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
