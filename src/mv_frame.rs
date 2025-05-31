use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::frame::Frame;

use crate::{
    mv_plane::{MVPlane, MVPlaneSet},
    params::{ReduceFilter, Subpel, SubpelMethod},
    util::{Pixel, plane_with_padding_mut, plane_with_padding_split},
};

#[derive(Debug, Clone)]
pub struct MVFrame {
    pub planes: SmallVec<[MVPlane; 3]>,
}

impl MVFrame {
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        yuv_mode: MVPlaneSet,
        x_ratio_uv: NonZeroUsize,
        y_ratio_uv: NonZeroUsize,
        bits_per_sample: NonZeroU8,
        plane_offsets: &SmallVec<[usize; 3]>,
        pitch: &[NonZeroUsize; 3],
    ) -> Result<Self> {
        // SAFETY: Width must be at least the value of its ratio
        let chroma_width = unsafe { NonZeroUsize::new_unchecked(width.get() / x_ratio_uv.get()) };
        // SAFETY: Height must be at least the value of its ratio
        let chroma_height = unsafe { NonZeroUsize::new_unchecked(height.get() / y_ratio_uv.get()) };
        let chroma_hpad = hpad / x_ratio_uv.get();
        let chroma_vpad = vpad / y_ratio_uv.get();

        let width = [width, chroma_width, chroma_width];
        let height = [height, chroma_height, chroma_height];
        let hpad = [hpad, chroma_hpad, chroma_hpad];
        let vpad = [vpad, chroma_vpad, chroma_vpad];

        let mut planes = SmallVec::new();
        for i in 0..3 {
            if (yuv_mode.bits() & (1 << i)) == 0 {
                continue;
            }

            let plane = MVPlane::new(
                width[i],
                height[i],
                pel,
                hpad[i],
                vpad[i],
                bits_per_sample,
                plane_offsets[i],
                pitch[i],
            )?;
            planes.push(plane);
        }

        Ok(Self { planes })
    }

    pub(crate) fn reduce_to<T: Pixel>(
        &self,
        reduced_frame: &mut MVFrame,
        mode: MVPlaneSet,
        filter: ReduceFilter,
        frame: &mut Frame,
    ) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get(i) {
                if (mode.bits() & (1 << i)) > 0 {
                    let reduced_pitch = reduced_frame.planes[i].pitch;
                    let (width, height) = (
                        reduced_frame.planes[i].width,
                        reduced_frame.planes[i].height,
                    );
                    // Use the new helper function to avoid cloning the source data
                    // SAFETY: The windows inside each plane are set up so that they do not overlap.
                    unsafe {
                        let (src, dest) = plane_with_padding_split::<T>(frame, i)
                            .expect("Super: plane should exist but does not");
                        plane.reduce_to::<T>(
                            &mut reduced_frame.planes[i],
                            filter,
                            dest,
                            src,
                            reduced_pitch,
                            self.planes[i].pitch,
                            width,
                            height,
                        );
                    }
                }
            }
        }
    }

    pub(crate) fn pad<T: Pixel>(&mut self, mode: MVPlaneSet, frame: &mut Frame) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get_mut(i) {
                if (mode.bits() & (1 << i)) > 0 {
                    plane.pad(
                        plane_with_padding_mut::<T>(frame, i)
                            .expect("Super: source plane should exist but does not"),
                    );
                }
            }
        }
    }

    pub(crate) fn refine<T: Pixel>(
        &mut self,
        mode: MVPlaneSet,
        subpel: SubpelMethod,
        frame: &mut Frame,
    ) {
        for i in 0..3 {
            if let Some(plane) = self.planes.get_mut(i) {
                if (mode.bits() & (1 << i)) > 0 {
                    plane.refine::<T>(
                        subpel,
                        plane_with_padding_mut::<T>(frame, i)
                            .expect("Super: source plane should exist but does not"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    /// Test helper to create minimal valid parameters for MVFrame::new
    fn create_test_params() -> (
        NonZeroUsize,
        NonZeroUsize,
        Subpel,
        usize,
        usize,
        MVPlaneSet,
        NonZeroUsize,
        NonZeroUsize,
        NonZeroU8,
        SmallVec<[usize; 3]>,
        [NonZeroUsize; 3],
    ) {
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(48).unwrap();
        let pel = Subpel::Full;
        let hpad = 8;
        let vpad = 8;
        let yuv_mode = MVPlaneSet::YUVPLANES;
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let plane_offsets = SmallVec::from([0, 1000, 2000]);
        let pitch = [
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(40).unwrap(),
            NonZeroUsize::new(40).unwrap(),
        ];

        (
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        )
    }

    /// Test helper to create a basic MVFrame for testing methods
    fn create_test_mvframe() -> MVFrame {
        let (
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        ) = create_test_params();

        MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        )
        .unwrap()
    }

    #[test]
    fn test_mvframe_new_creates_correct_number_of_planes() {
        let (
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        ) = create_test_params();

        let frame = MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        )
        .unwrap();

        // MVPlaneSet::YUVPLANES includes all 3 planes
        assert_eq!(frame.planes.len(), 3, "Should create 3 planes for YUV mode");
    }

    #[test]
    fn test_mvframe_new_y_plane_only() {
        let (
            width,
            height,
            pel,
            hpad,
            vpad,
            _,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        ) = create_test_params();

        let frame = MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            MVPlaneSet::YPLANE,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        )
        .unwrap();

        assert_eq!(
            frame.planes.len(),
            1,
            "Should create 1 plane for Y-only mode"
        );
    }

    #[test]
    fn test_mvframe_new_chroma_calculation() {
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(48).unwrap();
        let pel = Subpel::Full;
        let hpad = 8;
        let vpad = 8;
        let yuv_mode = MVPlaneSet::YUVPLANES;
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let plane_offsets = SmallVec::from([0, 1000, 2000]);
        let pitch = [
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(40).unwrap(),
            NonZeroUsize::new(40).unwrap(),
        ];

        let frame = MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        )
        .unwrap();

        // Y plane should have original dimensions
        assert_eq!(frame.planes[0].width, width);
        assert_eq!(frame.planes[0].height, height);

        // U and V planes should have half dimensions (64/2=32, 48/2=24)
        let expected_chroma_width = NonZeroUsize::new(32).unwrap();
        let expected_chroma_height = NonZeroUsize::new(24).unwrap();
        assert_eq!(frame.planes[1].width, expected_chroma_width);
        assert_eq!(frame.planes[1].height, expected_chroma_height);
        assert_eq!(frame.planes[2].width, expected_chroma_width);
        assert_eq!(frame.planes[2].height, expected_chroma_height);
    }

    #[test]
    fn test_mvframe_new_padding_calculation() {
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(48).unwrap();
        let pel = Subpel::Full;
        let hpad = 16;
        let vpad = 12;
        let yuv_mode = MVPlaneSet::YUVPLANES;
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let plane_offsets = SmallVec::from([0, 1000, 2000]);
        let pitch = [
            NonZeroUsize::new(96).unwrap(),
            NonZeroUsize::new(48).unwrap(),
            NonZeroUsize::new(48).unwrap(),
        ];

        let frame = MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        )
        .unwrap();

        // Y plane should have original padding
        assert_eq!(frame.planes[0].hpad, hpad);
        assert_eq!(frame.planes[0].vpad, vpad);

        // Chroma planes should have reduced padding (16/2=8, 12/2=6)
        assert_eq!(frame.planes[1].hpad, 8);
        assert_eq!(frame.planes[1].vpad, 6);
        assert_eq!(frame.planes[2].hpad, 8);
        assert_eq!(frame.planes[2].vpad, 6);
    }

    #[test]
    fn test_mvframe_new_different_pel_values() {
        let (
            width,
            height,
            _,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        ) = create_test_params();

        // Test with different pel values
        for &pel in &[Subpel::Full, Subpel::Half, Subpel::Quarter] {
            let frame = MVFrame::new(
                width,
                height,
                pel,
                hpad,
                vpad,
                yuv_mode,
                x_ratio_uv,
                y_ratio_uv,
                bits_per_sample,
                &plane_offsets,
                &pitch,
            )
            .unwrap();

            // All planes should have the same pel value
            for plane in &frame.planes {
                assert_eq!(
                    plane.pel, pel,
                    "All planes should have the correct pel value"
                );
            }
        }
    }

    #[test]
    fn test_mvframe_new_edge_case_small_dimensions() {
        let width = NonZeroUsize::new(4).unwrap(); // Large enough for x_ratio_uv of 2
        let height = NonZeroUsize::new(4).unwrap(); // Large enough for y_ratio_uv of 2
        let pel = Subpel::Full;
        let hpad = 0;
        let vpad = 0;
        let yuv_mode = MVPlaneSet::YUVPLANES;
        let x_ratio_uv = NonZeroUsize::new(2).unwrap();
        let y_ratio_uv = NonZeroUsize::new(2).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let plane_offsets = SmallVec::from([0, 1000, 2000]);
        let pitch = [
            NonZeroUsize::new(4).unwrap(),
            NonZeroUsize::new(2).unwrap(),
            NonZeroUsize::new(2).unwrap(),
        ];

        // This should work with valid small dimensions
        let result = MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        );

        assert!(result.is_ok(), "Should handle small valid dimensions");
        let frame = result.unwrap();

        // Verify chroma dimensions are calculated correctly: 4/2 = 2
        assert_eq!(frame.planes[1].width.get(), 2);
        assert_eq!(frame.planes[1].height.get(), 2);
    }

    #[test]
    fn test_mvframe_new_plane_selection() {
        let (
            width,
            height,
            pel,
            hpad,
            vpad,
            _,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        ) = create_test_params();

        // Test different plane combinations
        let test_cases = [
            (MVPlaneSet::YPLANE, 1),
            (MVPlaneSet::UPLANE, 1),
            (MVPlaneSet::VPLANE, 1),
            (MVPlaneSet::YUPLANES, 2),
            (MVPlaneSet::YVPLANES, 2),
            (MVPlaneSet::UVPLANES, 2),
            (MVPlaneSet::YUVPLANES, 3),
        ];

        for (yuv_mode, expected_count) in test_cases {
            let frame = MVFrame::new(
                width,
                height,
                pel,
                hpad,
                vpad,
                yuv_mode,
                x_ratio_uv,
                y_ratio_uv,
                bits_per_sample,
                &plane_offsets,
                &pitch,
            )
            .unwrap();

            assert_eq!(
                frame.planes.len(),
                expected_count,
                "Mode {:?} should create {} planes",
                yuv_mode,
                expected_count
            );
        }
    }

    #[quickcheck]
    fn test_mvframe_new_property_based(
        width: u16,
        height: u16,
        hpad: u8,
        vpad: u8,
        bits: u8,
    ) -> TestResult {
        // Ensure inputs are within valid ranges
        if width == 0 || height == 0 || bits == 0 || bits > 16 {
            return TestResult::discard();
        }

        let width = NonZeroUsize::new(width as usize).unwrap();
        let height = NonZeroUsize::new(height as usize).unwrap();
        let pel = Subpel::Full;
        let hpad = hpad as usize;
        let vpad = vpad as usize;
        let yuv_mode = MVPlaneSet::YPLANE; // Use single plane to simplify test
        let x_ratio_uv = NonZeroUsize::new(1).unwrap();
        let y_ratio_uv = NonZeroUsize::new(1).unwrap();
        let bits_per_sample = NonZeroU8::new(bits).unwrap();
        let plane_offsets = SmallVec::from([0, 1000, 2000]);
        let pitch = [
            NonZeroUsize::new(width.get() + hpad * 2).unwrap(),
            NonZeroUsize::new(width.get() + hpad * 2).unwrap(),
            NonZeroUsize::new(width.get() + hpad * 2).unwrap(),
        ];

        match MVFrame::new(
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &plane_offsets,
            &pitch,
        ) {
            Ok(frame) => {
                // Verify basic properties
                assert!(!frame.planes.is_empty());
                assert_eq!(frame.planes[0].width, width);
                assert_eq!(frame.planes[0].height, height);
                TestResult::passed()
            }
            Err(_) => TestResult::discard(), // Some combinations might be invalid
        }
    }

    #[test]
    fn test_mvframe_clone() {
        let frame = create_test_mvframe();
        let cloned_frame = frame.clone();

        assert_eq!(frame.planes.len(), cloned_frame.planes.len());
        for (original, cloned) in frame.planes.iter().zip(cloned_frame.planes.iter()) {
            assert_eq!(original.width, cloned.width);
            assert_eq!(original.height, cloned.height);
            assert_eq!(original.pel, cloned.pel);
        }
    }

    #[test]
    fn test_mvframe_debug_format() {
        let frame = create_test_mvframe();
        let debug_str = format!("{:?}", frame);
        assert!(debug_str.contains("MVFrame"));
        assert!(debug_str.contains("planes"));
    }

    // Note: The reduce_to, pad, and refine methods require a vapoursynth Frame object
    // which is complex to mock in unit tests. These methods would be better tested
    // in integration tests where we can create actual Frame objects from the vapoursynth API.
    // The following tests focus on the structure and error handling aspects we can test.

    #[test]
    fn test_mvframe_plane_access_patterns() {
        let frame = create_test_mvframe();

        // Test that we can safely access planes
        for (i, plane) in frame.planes.iter().enumerate() {
            assert!(i < 3, "Should not have more than 3 planes");
            assert!(plane.width.get() > 0, "Plane width should be positive");
            assert!(plane.height.get() > 0, "Plane height should be positive");
        }
    }

    #[test]
    fn test_mvframe_plane_dimensions_consistency() {
        let frame = create_test_mvframe();

        // For YUV mode with 2:1 ratios, check dimension relationships
        if frame.planes.len() >= 3 {
            let y_plane = &frame.planes[0];
            let u_plane = &frame.planes[1];
            let v_plane = &frame.planes[2];

            // Chroma planes should have half the dimensions of luma
            assert_eq!(y_plane.width.get() / 2, u_plane.width.get());
            assert_eq!(y_plane.height.get() / 2, u_plane.height.get());
            assert_eq!(u_plane.width, v_plane.width);
            assert_eq!(u_plane.height, v_plane.height);
        }
    }

    #[test]
    fn test_mvframe_memory_layout_expectations() {
        let frame = create_test_mvframe();

        for plane in &frame.planes {
            // Padded dimensions should be larger than or equal to original
            assert!(plane.padded_width.get() >= plane.width.get());
            assert!(plane.padded_height.get() >= plane.height.get());

            // Pitch should be at least as wide as padded width
            assert!(plane.pitch.get() >= plane.padded_width.get());

            // Padding values should be consistent
            assert_eq!(plane.padded_width.get(), plane.width.get() + 2 * plane.hpad);
            assert_eq!(
                plane.padded_height.get(),
                plane.height.get() + 2 * plane.vpad
            );
        }
    }

    #[test]
    fn test_mvframe_subpel_window_offsets() {
        let (
            width,
            height,
            _,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            plane_offsets,
            pitch,
        ) = create_test_params();

        // Test different pel values create different numbers of subpel windows
        for &pel in &[Subpel::Full, Subpel::Half, Subpel::Quarter] {
            let frame = MVFrame::new(
                width,
                height,
                pel,
                hpad,
                vpad,
                yuv_mode,
                x_ratio_uv,
                y_ratio_uv,
                bits_per_sample,
                &plane_offsets,
                &pitch,
            )
            .unwrap();

            let expected_windows = usize::from(pel) * usize::from(pel);
            for plane in &frame.planes {
                assert_eq!(
                    plane.subpel_window_offsets.len(),
                    expected_windows,
                    "Pel {:?} should create {} subpel windows",
                    pel,
                    expected_windows
                );
            }
        }
    }
}
