#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use std::num::NonZeroUsize;

use pastey::paste;

const SATD_SIZES: &[(usize, usize)] = &[
    (4, 4),
    (8, 4),
    (8, 8),
    (16, 8),
    (16, 16),
    (32, 16),
    (32, 32),
    (64, 32),
    (64, 64),
    (128, 64),
    (128, 128),
];

macro_rules! get_satd_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<satd_identical_u8_ $module>]() {
                for &(w, h) in SATD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u8> = vec![42u8; w * h];
                    let ref_: Vec<u8> = vec![42u8; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_satd(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 0, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<satd_identical_u16_ $module>]() {
                for &(w, h) in SATD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u16> = vec![1000u16; w * h];
                    let ref_: Vec<u16> = vec![1000u16; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_satd(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 0, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<satd_uniform_diff_u8_ $module>]() {
                for &(w, h) in SATD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u8> = vec![10u8; w * h];
                    let ref_: Vec<u8> = vec![7u8; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    // For a constant-difference block, only the DC coefficient of the Hadamard
                    // transform survives, giving SATD = diff * width * height / 2.
                    let result = verify_asm!(ret $module, get_satd(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 3 * (w * h) as u64 / 2, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<satd_uniform_diff_u16_ $module>]() {
                for &(w, h) in SATD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u16> = vec![1000u16; w * h];
                    let ref_: Vec<u16> = vec![700u16; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_satd(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 300 * (w * h) as u64 / 2, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<satd_with_padding_u8_ $module>]() {
                for &(w, h) in SATD_SIZES {
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
                    let result = verify_asm!(ret $module, get_satd(width, height, &src, src_pitch, &ref_, src_pitch));
                    assert_eq!(result, 3 * (w * h) as u64 / 2, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<satd_with_padding_u16_ $module>]() {
                for &(w, h) in SATD_SIZES {
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
                    let result = verify_asm!(ret $module, get_satd(width, height, &src, src_pitch, &ref_, src_pitch));
                    assert_eq!(result, 300 * (w * h) as u64 / 2, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<satd_zeros_u8_ $module>]() {
                let w = 8;
                let h = 8;
                let src: Vec<u8> = vec![0u8; w * h];
                let ref_: Vec<u8> = vec![0u8; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_satd(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, 0);
            }

            #[test]
            fn [<satd_max_u16_ $module>]() {
                let w = 128;
                let h = 128;
                let src: Vec<u16> = vec![u16::MAX; w * h];
                let ref_: Vec<u16> = vec![0u16; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_satd(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, u16::MAX as u64 * (w * h) as u64 / 2);
            }
        }
    };
}

get_satd_tests!(rust);

#[test]
#[should_panic]
fn satd_unsupported_size_panics() {
    let w = 3;
    let h = 3;
    let src: Vec<u8> = vec![1u8; w * h];
    let ref_: Vec<u8> = vec![0u8; w * h];
    let width = NonZeroUsize::new(w).unwrap();
    let height = NonZeroUsize::new(h).unwrap();
    let pitch = NonZeroUsize::new(w).unwrap();
    let _ = super::get_satd(width, height, &src, pitch, &ref_, pitch);
}
