#![allow(clippy::unwrap_used, reason = "allow in test files")]

use std::num::NonZeroUsize;

use parameterized::parameterized;

use super::luma_sum;

#[parameterized(
    w = { 4, 8, 8, 16, 16, 16, 32, 32, 64, 64, 128, 128 },
    h = { 4, 4, 8,  2,  8, 16, 16, 32, 32, 64,  64, 128 },
)]
fn luma_sum_uniform_u8(w: usize, h: usize) {
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src: Vec<u8> = vec![7u8; w * h];
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(luma_sum(width, height, &src, pitch), 7 * (w * h) as u64);
}

#[parameterized(
    w = { 4, 8, 8, 16, 16, 16, 32, 32, 64, 64, 128, 128 },
    h = { 4, 4, 8,  2,  8, 16, 16, 32, 32, 64,  64, 128 },
)]
fn luma_sum_uniform_u16(w: usize, h: usize) {
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src: Vec<u16> = vec![300u16; w * h];
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(luma_sum(width, height, &src, pitch), 300 * (w * h) as u64);
}

#[parameterized(
    w = { 4, 8, 8, 16, 16, 16, 32, 32, 64, 64, 128, 128 },
    h = { 4, 4, 8,  2,  8, 16, 16, 32, 32, 64,  64, 128 },
)]
fn luma_sum_with_padding_u8(w: usize, h: usize) {
    let padding = 16;
    let pitch = w + padding;
    let mut src: Vec<u8> = vec![255u8; pitch * h];
    // Fill only the block region with the test value
    for row in 0..h {
        for col in 0..w {
            src[row * pitch + col] = 7;
        }
    }
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src_pitch = NonZeroUsize::new(pitch).unwrap();
    assert_eq!(luma_sum(width, height, &src, src_pitch), 7 * (w * h) as u64);
}

#[parameterized(
    w = { 4, 8, 8, 16, 16, 16, 32, 32, 64, 64, 128, 128 },
    h = { 4, 4, 8,  2,  8, 16, 16, 32, 32, 64,  64, 128 },
)]
fn luma_sum_with_padding_u16(w: usize, h: usize) {
    let padding = 16;
    let pitch = w + padding;
    let mut src: Vec<u16> = vec![65535u16; pitch * h];
    // Fill only the block region with the test value
    for row in 0..h {
        for col in 0..w {
            src[row * pitch + col] = 300;
        }
    }
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let src_pitch = NonZeroUsize::new(pitch).unwrap();
    assert_eq!(
        luma_sum(width, height, &src, src_pitch),
        300 * (w * h) as u64
    );
}

#[test]
fn luma_sum_sequential_u8() {
    let w = 4;
    let h = 4;
    let src: Vec<u8> = (1..=16).collect();
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(luma_sum(width, height, &src, pitch), 136);
}

#[test]
fn luma_sum_sequential_u16() {
    let w = 4;
    let h = 4;
    let src: Vec<u16> = (1000..=1015).collect();
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(luma_sum(width, height, &src, pitch), 16120);
}

#[test]
fn luma_sum_zeros_u8() {
    let w = 8;
    let h = 8;
    let src: Vec<u8> = vec![0u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(luma_sum(width, height, &src, pitch), 0);
}

#[test]
fn luma_sum_max_u16() {
    let w = 128;
    let h = 128;
    let src: Vec<u16> = vec![u16::MAX; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    assert_eq!(
        luma_sum(width, height, &src, pitch),
        u16::MAX as u64 * (w * h) as u64
    );
}

#[test]
#[should_panic]
fn luma_sum_unsupported_size_panics() {
    let w = 3;
    let h = 3;
    let src: Vec<u8> = vec![1u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    let _ = luma_sum(width, height, &src, pitch);
}
