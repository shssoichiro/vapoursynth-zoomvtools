//! Group of Frames code for managing a hierarchical frame structure

use std::num::{NonZeroU8, NonZeroUsize};

use anyhow::Result;
use smallvec::SmallVec;
use vapoursynth::{format::Format, frame::Frame};

use crate::{
    mv_frame::MVFrame,
    mv_plane::{MVPlaneSet, plane_height_luma, plane_super_offset, plane_width_luma},
    params::{ReduceFilter, Subpel, SubpelMethod},
    util::Pixel,
};

#[derive(Debug, Clone)]
pub struct MVGroupOfFrames {
    level_count: u16,
    width: [NonZeroUsize; 3],
    height: [NonZeroUsize; 3],
    pel: Subpel,
    hpad: [usize; 3],
    vpad: [usize; 3],
    x_ratio_uv: NonZeroUsize,
    y_ratio_uv: NonZeroUsize,
    pub frames: Box<[MVFrame]>,
}

impl MVGroupOfFrames {
    pub fn new(
        level_count: u16,
        width: NonZeroUsize,
        height: NonZeroUsize,
        pel: Subpel,
        hpad: usize,
        vpad: usize,
        yuv_mode: MVPlaneSet,
        x_ratio_uv: NonZeroUsize,
        y_ratio_uv: NonZeroUsize,
        bits_per_sample: NonZeroU8,
        pitch: &[NonZeroUsize; 3],
        format: Format,
    ) -> Result<Self> {
        // SAFETY: Width must be at least the value of its ratio
        let chroma_width = unsafe { NonZeroUsize::new_unchecked(width.get() / x_ratio_uv.get()) };
        // SAFETY: Height must be at least the value of its ratio
        let chroma_height = unsafe { NonZeroUsize::new_unchecked(height.get() / y_ratio_uv.get()) };
        let chroma_hpad = hpad / x_ratio_uv.get();
        let chroma_vpad = vpad / y_ratio_uv.get();

        let mut this = Self {
            level_count,
            width: [width, chroma_width, chroma_width],
            height: [height, chroma_height, chroma_height],
            pel,
            hpad: [hpad, chroma_hpad, chroma_hpad],
            vpad: [vpad, chroma_vpad, chroma_vpad],
            x_ratio_uv,
            y_ratio_uv,
            frames: Default::default(),
        };

        let mut frames = Vec::with_capacity(level_count as usize);

        for i in 0..level_count {
            let width_i = plane_width_luma(this.width[0], i, this.x_ratio_uv, this.hpad[0]);
            let height_i = plane_height_luma(this.height[0], i, this.y_ratio_uv, this.vpad[0]);
            let mut plane_offsets = SmallVec::with_capacity(3);
            for plane in 0..format.plane_count() {
                let offset = plane_super_offset(
                    plane > 0,
                    this.height[plane],
                    i,
                    this.pel,
                    this.vpad[plane],
                    pitch[plane],
                    this.y_ratio_uv,
                );
                plane_offsets.push(offset);
            }

            frames.push(MVFrame::new(
                width_i,
                height_i,
                Subpel::Full,
                this.hpad[0],
                this.vpad[0],
                yuv_mode,
                this.x_ratio_uv,
                this.y_ratio_uv,
                bits_per_sample,
                &plane_offsets,
                pitch,
            )?);
        }

        this.frames = frames.into_boxed_slice();

        Ok(this)
    }

    pub fn reduce<T: Pixel>(&mut self, mode: MVPlaneSet, filter: ReduceFilter, frame: &mut Frame) {
        for i in 0..(self.level_count as usize - 1) {
            self.frames[i]
                .clone()
                .reduce_to::<T>(&mut self.frames[i + 1], mode, filter, frame);
            self.frames[i + 1].pad::<T>(MVPlaneSet::YUVPLANES, frame);
        }
    }

    pub fn pad<T: Pixel>(&mut self, mode: MVPlaneSet, frame: &mut Frame) {
        self.frames[0].pad::<T>(mode, frame);
    }

    pub fn refine<T: Pixel>(&mut self, mode: MVPlaneSet, subpel: SubpelMethod, frame: &mut Frame) {
        self.frames[0].refine::<T>(mode, subpel, frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_env;
    use vapoursynth::format::PresetFormat;

    #[test]
    fn test_mvgof_struct_fields() {
        // Test that the struct can be created and basic field access works
        // This test doesn't require complex VapourSynth setup
        let env = create_test_env(64, 48, PresetFormat::YUV420P8, 1).unwrap();
        let (node, _) = env.get_output(0).unwrap();
        let video_info = node.info();
        let format = match video_info.format {
            vapoursynth::prelude::Property::Constant(format) => format,
            _ => panic!("Should have constant format"),
        };

        let level_count = 2;
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(48).unwrap();
        let pel = Subpel::Full;
        let hpad = 8;
        let vpad = 8;
        let yuv_mode = MVPlaneSet::YPLANE; // Use single plane to simplify
        let x_ratio_uv = NonZeroUsize::new(1).unwrap();
        let y_ratio_uv = NonZeroUsize::new(1).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let pitch = [
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(80).unwrap(),
            NonZeroUsize::new(80).unwrap(),
        ];

        let result = MVGroupOfFrames::new(
            level_count,
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &pitch,
            format,
        );

        assert!(result.is_ok(), "Should create MVGroupOfFrames successfully");
        let gof = result.unwrap();

        // Test basic structure properties
        assert_eq!(gof.level_count, level_count);
        assert_eq!(gof.frames.len(), level_count as usize);
        assert_eq!(gof.pel, pel);
        assert_eq!(gof.x_ratio_uv, x_ratio_uv);
        assert_eq!(gof.y_ratio_uv, y_ratio_uv);

        // Test that width/height arrays are populated correctly
        assert_eq!(gof.width[0], width);
        assert_eq!(gof.height[0], height);

        // Test that hpad/vpad arrays are populated correctly
        assert_eq!(gof.hpad[0], hpad);
        assert_eq!(gof.vpad[0], vpad);

        // Test that all frames were created
        assert!(!gof.frames.is_empty());
        for frame in gof.frames.iter() {
            assert!(!frame.planes.is_empty(), "Each frame should have planes");
        }
    }

    #[test]
    fn test_mvgof_different_level_counts() {
        let env = create_test_env(64, 48, PresetFormat::YUV420P8, 1).unwrap();
        let (node, _) = env.get_output(0).unwrap();
        let video_info = node.info();
        let format = match video_info.format {
            vapoursynth::prelude::Property::Constant(format) => format,
            _ => panic!("Should have constant format"),
        };

        for level_count in [1, 2, 3, 5] {
            let width = NonZeroUsize::new(64).unwrap();
            let height = NonZeroUsize::new(48).unwrap();
            let pel = Subpel::Full;
            let hpad = 8;
            let vpad = 8;
            let yuv_mode = MVPlaneSet::YPLANE;
            let x_ratio_uv = NonZeroUsize::new(1).unwrap();
            let y_ratio_uv = NonZeroUsize::new(1).unwrap();
            let bits_per_sample = NonZeroU8::new(8).unwrap();
            let pitch = [
                NonZeroUsize::new(80).unwrap(),
                NonZeroUsize::new(80).unwrap(),
                NonZeroUsize::new(80).unwrap(),
            ];

            let result = MVGroupOfFrames::new(
                level_count,
                width,
                height,
                pel,
                hpad,
                vpad,
                yuv_mode,
                x_ratio_uv,
                y_ratio_uv,
                bits_per_sample,
                &pitch,
                format,
            );

            assert!(result.is_ok(), "Should create with {} levels", level_count);
            let gof = result.unwrap();
            assert_eq!(gof.frames.len(), level_count as usize);
            assert_eq!(gof.level_count, level_count);
        }
    }

    #[test]
    fn test_mvgof_debug_and_clone() {
        let env = create_test_env(32, 32, PresetFormat::YUV420P8, 1).unwrap();
        let (node, _) = env.get_output(0).unwrap();
        let video_info = node.info();
        let format = match video_info.format {
            vapoursynth::prelude::Property::Constant(format) => format,
            _ => panic!("Should have constant format"),
        };

        let level_count = 2;
        let width = NonZeroUsize::new(32).unwrap();
        let height = NonZeroUsize::new(32).unwrap();
        let pel = Subpel::Full;
        let hpad = 4;
        let vpad = 4;
        let yuv_mode = MVPlaneSet::YPLANE;
        let x_ratio_uv = NonZeroUsize::new(1).unwrap();
        let y_ratio_uv = NonZeroUsize::new(1).unwrap();
        let bits_per_sample = NonZeroU8::new(8).unwrap();
        let pitch = [
            NonZeroUsize::new(40).unwrap(),
            NonZeroUsize::new(40).unwrap(),
            NonZeroUsize::new(40).unwrap(),
        ];

        let gof = MVGroupOfFrames::new(
            level_count,
            width,
            height,
            pel,
            hpad,
            vpad,
            yuv_mode,
            x_ratio_uv,
            y_ratio_uv,
            bits_per_sample,
            &pitch,
            format,
        )
        .unwrap();

        // Test Debug implementation
        let debug_str = format!("{:?}", gof);
        assert!(debug_str.contains("MVGroupOfFrames"));
        assert!(debug_str.contains("level_count"));

        // Test Clone implementation
        let cloned_gof = gof.clone();
        assert_eq!(gof.level_count, cloned_gof.level_count);
        assert_eq!(gof.frames.len(), cloned_gof.frames.len());
        assert_eq!(gof.width, cloned_gof.width);
        assert_eq!(gof.height, cloned_gof.height);
        assert_eq!(gof.pel, cloned_gof.pel);
    }

    // Note: More comprehensive tests would require complex VapourSynth Frame object creation
    // which is better suited for integration tests. These unit tests focus on the basic
    // constructor behavior and struct invariants that can be tested without complex mocking.
}
