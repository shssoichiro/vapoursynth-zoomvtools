#![allow(clippy::unwrap_used, reason = "allow in test files")]

use std::num::NonZeroUsize;

use parameterized::parameterized;

use super::get_sad;

#[parameterized(
    w = { 2, 2, 4, 4, 4, 8, 8, 8, 8,  8, 16, 16, 16, 16, 16, 16, 32,  32, 32, 32, 64, 64, 64,  64, 128, 128, 128 },
    h = { 2, 4, 2, 4, 8, 1, 2, 4, 8, 16,  1,  2,  4,  8, 16, 32,  8, 16, 32, 64, 16, 32, 64, 128,  32,  64, 128 },
)]
fn sad_identical_u8(w: usize, h: usize) {
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src: Vec<u8> = vec![42u8; w * h];
    let ref_: Vec<u8> = vec![42u8; w * h];
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(get_sad(width, height, &src, pitch, &ref_, pitch), 0);
}

#[parameterized(
    w = { 2, 2, 4, 4, 4, 8, 8, 8, 8,  8, 16, 16, 16, 16, 16, 16, 32,  32, 32, 32, 64, 64, 64,  64, 128, 128, 128 },
    h = { 2, 4, 2, 4, 8, 1, 2, 4, 8, 16,  1,  2,  4,  8, 16, 32,  8, 16, 32, 64, 16, 32, 64, 128,  32,  64, 128 },
)]
fn sad_identical_u16(w: usize, h: usize) {
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src: Vec<u16> = vec![1000u16; w * h];
    let ref_: Vec<u16> = vec![1000u16; w * h];
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(get_sad(width, height, &src, pitch, &ref_, pitch), 0);
}

#[parameterized(
    w = { 2, 2, 4, 4, 4, 8, 8, 8, 8,  8, 16, 16, 16, 16, 16, 16, 32,  32, 32, 32, 64, 64, 64,  64, 128, 128, 128 },
    h = { 2, 4, 2, 4, 8, 1, 2, 4, 8, 16,  1,  2,  4,  8, 16, 32,  8, 16, 32, 64, 16, 32, 64, 128,  32,  64, 128 },
)]
fn sad_uniform_diff_u8(w: usize, h: usize) {
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src: Vec<u8> = vec![10u8; w * h];
    let ref_: Vec<u8> = vec![7u8; w * h];
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(
        get_sad(width, height, &src, pitch, &ref_, pitch),
        3 * (w * h) as u64
    );
}

#[parameterized(
    w = { 2, 2, 4, 4, 4, 8, 8, 8, 8,  8, 16, 16, 16, 16, 16, 16, 32,  32, 32, 32, 64, 64, 64,  64, 128, 128, 128 },
    h = { 2, 4, 2, 4, 8, 1, 2, 4, 8, 16,  1,  2,  4,  8, 16, 32,  8, 16, 32, 64, 16, 32, 64, 128,  32,  64, 128 },
)]
fn sad_uniform_diff_u16(w: usize, h: usize) {
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src: Vec<u16> = vec![1000u16; w * h];
    let ref_: Vec<u16> = vec![700u16; w * h];
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(
        get_sad(width, height, &src, pitch, &ref_, pitch),
        300 * (w * h) as u64
    );
}

#[parameterized(
    w = { 2, 2, 4, 4, 4, 8, 8, 8, 8,  8, 16, 16, 16, 16, 16, 16, 32,  32, 32, 32, 64, 64, 64,  64, 128, 128, 128 },
    h = { 2, 4, 2, 4, 8, 1, 2, 4, 8, 16,  1,  2,  4,  8, 16, 32,  8, 16, 32, 64, 16, 32, 64, 128,  32,  64, 128 },
)]
fn sad_with_padding_u8(w: usize, h: usize) {
    let padding = 16;
    let pitch = w + padding;
    let mut src: Vec<u8> = vec![255u8; pitch * h];
    let mut ref_: Vec<u8> = vec![255u8; pitch * h];
    // Fill only the block region with test values
    for row in 0..h {
        for col in 0..w {
            src[row * pitch + col] = 10;
            ref_[row * pitch + col] = 7;
        }
    }
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src_pitch = NonZeroUsize::new(pitch).unwrap();
    assert_eq!(
        get_sad(width, height, &src, src_pitch, &ref_, src_pitch),
        3 * (w * h) as u64
    );
}

#[parameterized(
    w = { 2, 2, 4, 4, 4, 8, 8, 8, 8,  8, 16, 16, 16, 16, 16, 16, 32,  32, 32, 32, 64, 64, 64,  64, 128, 128, 128 },
    h = { 2, 4, 2, 4, 8, 1, 2, 4, 8, 16,  1,  2,  4,  8, 16, 32,  8, 16, 32, 64, 16, 32, 64, 128,  32,  64, 128 },
)]
fn sad_with_padding_u16(w: usize, h: usize) {
    let padding = 16;
    let pitch = w + padding;
    let mut src: Vec<u16> = vec![65535u16; pitch * h];
    let mut ref_: Vec<u16> = vec![65535u16; pitch * h];
    // Fill only the block region with test values
    for row in 0..h {
        for col in 0..w {
            src[row * pitch + col] = 1000;
            ref_[row * pitch + col] = 700;
        }
    }
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src_pitch = NonZeroUsize::new(pitch).unwrap();
    assert_eq!(
        get_sad(width, height, &src, src_pitch, &ref_, src_pitch),
        300 * (w * h) as u64
    );
}

#[test]
fn sad_zeros_u8() {
    let w = 8;
    let h = 8;
    let src: Vec<u8> = vec![0u8; w * h];
    let ref_: Vec<u8> = vec![0u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(get_sad(width, height, &src, pitch, &ref_, pitch), 0);
}

#[test]
fn sad_one_side_zero_u8() {
    let w = 8;
    let h = 8;
    let src: Vec<u8> = vec![5u8; w * h];
    let ref_: Vec<u8> = vec![0u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(
        get_sad(width, height, &src, pitch, &ref_, pitch),
        5 * (w * h) as u64
    );
}

#[test]
fn sad_max_u16() {
    let w = 128;
    let h = 128;
    let src: Vec<u16> = vec![u16::MAX; w * h];
    let ref_: Vec<u16> = vec![0u16; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(
        get_sad(width, height, &src, pitch, &ref_, pitch),
        u16::MAX as u64 * (w * h) as u64
    );
}

#[test]
fn sad_sequential_u8() {
    // 4x4 block: src = [1..=16], ref = [16..=31]
    // Each element differs by 15, so SAD = 15 * 16 = 240
    let w = 4;
    let h = 4;
    let src: Vec<u8> = (1..=16).collect();
    let ref_: Vec<u8> = (16..=31).collect();
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(get_sad(width, height, &src, pitch, &ref_, pitch), 240);
}

#[test]
fn sad_sequential_u16() {
    // 4x4 block: src = [1000..=1015], ref = [2000..=2015]
    // Each element differs by 1000, so SAD = 1000 * 16 = 16000
    let w = 4;
    let h = 4;
    let src: Vec<u16> = (1000..=1015).collect();
    let ref_: Vec<u16> = (2000..=2015).collect();
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(get_sad(width, height, &src, pitch, &ref_, pitch), 16000);
}

#[test]
#[should_panic]
fn sad_unsupported_size_panics() {
    let w = 3;
    let h = 3;
    let src: Vec<u8> = vec![1u8; w * h];
    let ref_: Vec<u8> = vec![0u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    let _ = get_sad(width, height, &src, pitch, &ref_, pitch);
}
