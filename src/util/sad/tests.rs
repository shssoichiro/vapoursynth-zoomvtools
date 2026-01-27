#![allow(clippy::unwrap_used, reason = "allow in test files")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "allow in test files")]

use std::num::NonZeroUsize;

use pastey::paste;

const SAD_SIZES: &[(usize, usize)] = &[
    (2, 2),
    (2, 4),
    (4, 2),
    (4, 4),
    (4, 8),
    (8, 1),
    (8, 2),
    (8, 4),
    (8, 8),
    (8, 16),
    (16, 1),
    (16, 2),
    (16, 4),
    (16, 8),
    (16, 16),
    (16, 32),
    (32, 8),
    (32, 16),
    (32, 32),
    (32, 64),
    (64, 16),
    (64, 32),
    (64, 64),
    (64, 128),
    (128, 32),
    (128, 64),
    (128, 128),
];

macro_rules! get_sad_tests {
    ($module:ident) => {
        paste! {
            #[test]
            fn [<sad_identical_u8_ $module>]() {
                for &(w, h) in SAD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u8> = vec![42u8; w * h];
                    let ref_: Vec<u8> = vec![42u8; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 0, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<sad_identical_u16_ $module>]() {
                for &(w, h) in SAD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u16> = vec![1000u16; w * h];
                    let ref_: Vec<u16> = vec![1000u16; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 0, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<sad_uniform_diff_u8_ $module>]() {
                for &(w, h) in SAD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u8> = vec![10u8; w * h];
                    let ref_: Vec<u8> = vec![7u8; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 3 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<sad_uniform_diff_u16_ $module>]() {
                for &(w, h) in SAD_SIZES {
                    let width = NonZeroUsize::new(w).unwrap();
                    let height = NonZeroUsize::new(h).unwrap();
                    let src: Vec<u16> = vec![1000u16; w * h];
                    let ref_: Vec<u16> = vec![700u16; w * h];
                    let pitch = NonZeroUsize::new(w).unwrap();
                    let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                    assert_eq!(result, 300 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<sad_with_padding_u8_ $module>]() {
                for &(w, h) in SAD_SIZES {
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
                    let result = verify_asm!(ret $module, get_sad(width, height, &src, src_pitch, &ref_, src_pitch));
                    assert_eq!(result, 3 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<sad_with_padding_u16_ $module>]() {
                for &(w, h) in SAD_SIZES {
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
                    let result = verify_asm!(ret $module, get_sad(width, height, &src, src_pitch, &ref_, src_pitch));
                    assert_eq!(result, 300 * (w * h) as u64, "failed at {w}x{h}");
                }
            }

            #[test]
            fn [<sad_zeros_u8_ $module>]() {
                let w = 8;
                let h = 8;
                let src: Vec<u8> = vec![0u8; w * h];
                let ref_: Vec<u8> = vec![0u8; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, 0);
            }

            #[test]
            fn [<sad_one_side_zero_u8_ $module>]() {
                let w = 8;
                let h = 8;
                let src: Vec<u8> = vec![5u8; w * h];
                let ref_: Vec<u8> = vec![0u8; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, 5 * (w * h) as u64);
            }

            #[test]
            fn [<sad_max_u16_ $module>]() {
                let w = 128;
                let h = 128;
                let src: Vec<u16> = vec![u16::MAX; w * h];
                let ref_: Vec<u16> = vec![0u16; w * h];
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, u16::MAX as u64 * (w * h) as u64);
            }

            #[test]
            fn [<sad_sequential_u8_ $module>]() {
                // 4x4 block: src = [1..=16], ref = [16..=31]
                // Each element differs by 15, so SAD = 15 * 16 = 240
                let w = 4;
                let h = 4;
                let src: Vec<u8> = (1..=16).collect();
                let ref_: Vec<u8> = (16..=31).collect();
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, 240);
            }

            #[test]
            fn [<sad_sequential_u16_ $module>]() {
                // 4x4 block: src = [1000..=1015], ref = [2000..=2015]
                // Each element differs by 1000, so SAD = 1000 * 16 = 16000
                let w = 4;
                let h = 4;
                let src: Vec<u16> = (1000..=1015).collect();
                let ref_: Vec<u16> = (2000..=2015).collect();
                let width = NonZeroUsize::new(w).unwrap();
                let height = NonZeroUsize::new(h).unwrap();
                let pitch = NonZeroUsize::new(w).unwrap();
                let result = verify_asm!(ret $module, get_sad(width, height, &src, pitch, &ref_, pitch));
                assert_eq!(result, 16000);
            }
        }
    };
}

get_sad_tests!(rust);
