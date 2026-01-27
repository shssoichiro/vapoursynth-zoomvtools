#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use std::num::NonZeroUsize;

use pastey::paste;

macro_rules! luma_sum_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<luma_sum_uniform_u8_ $module>]() {
                for &(w, h) in &[
                    (4, 4), (8, 4), (8, 8), (16, 2), (16, 8), (16, 16),
                    (32, 16), (32, 32), (64, 32), (64, 64), (128, 64), (128, 128),
                ] {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u8> = vec![7u8; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, luma_sum(width, height, &src, pitch));
                    assert_eq!(result, 7 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<luma_sum_uniform_u16_ $module>]() {
                for &(w, h) in &[
                    (4, 4), (8, 4), (8, 8), (16, 2), (16, 8), (16, 16),
                    (32, 16), (32, 32), (64, 32), (64, 64), (128, 64), (128, 128),
                ] {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u16> = vec![300u16; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, luma_sum(width, height, &src, pitch));
                    assert_eq!(result, 300 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<luma_sum_with_padding_u8_ $module>]() {
                for &(w, h) in &[
                    (4, 4), (8, 4), (8, 8), (16, 2), (16, 8), (16, 16),
                    (32, 16), (32, 32), (64, 32), (64, 64), (128, 64), (128, 128),
                ] {
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
                    let result = verify_asm!(ret $module, luma_sum(width, height, &src, src_pitch));
                    assert_eq!(result, 7 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<luma_sum_with_padding_u16_ $module>]() {
                for &(w, h) in &[
                    (4, 4), (8, 4), (8, 8), (16, 2), (16, 8), (16, 16),
                    (32, 16), (32, 32), (64, 32), (64, 64), (128, 64), (128, 128),
                ] {
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
                    let result = verify_asm!(ret $module, luma_sum(width, height, &src, src_pitch));
                    assert_eq!(result, 300 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<luma_sum_sequential_u8_ $module>]() {
                let w = 4;
                let h = 4;
                let src: Vec<u8> = (1..=16).collect();
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, luma_sum(width, height, &src, pitch));
                assert_eq!(result, 136);
            }

            #[test]
            fn [<luma_sum_sequential_u16_ $module>]() {
                let w = 4;
                let h = 4;
                let src: Vec<u16> = (1000..=1015).collect();
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, luma_sum(width, height, &src, pitch));
                assert_eq!(result, 16120);
            }

            #[test]
            fn [<luma_sum_zeros_u8_ $module>]() {
                let w = 8;
                let h = 8;
                let src: Vec<u8> = vec![0u8; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, luma_sum(width, height, &src, pitch));
                assert_eq!(result, 0);
            }

            #[test]
            fn [<luma_sum_max_u16_ $module>]() {
                let w = 128;
                let h = 128;
                let src: Vec<u16> = vec![u16::MAX; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, luma_sum(width, height, &src, pitch));
                assert_eq!(result, u16::MAX as u64 * (w * h) as u64);
            }
        }
    };
}

luma_sum_tests!(rust);

#[test]
#[should_panic]
fn luma_sum_unsupported_size_panics() {
    let w = 3;
    let h = 3;
    let src: Vec<u8> = vec![1u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    let _ = super::luma_sum(width, height, &src, pitch);
}
