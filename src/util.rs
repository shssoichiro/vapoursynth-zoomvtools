use crate::params::Subpel;

/// Calculate the height of a plane in luma samples.
pub fn plane_height_luma(src_height: usize, level: u16, y_ratio_uv: usize, vpad: usize) -> usize {
    todo!()
}

/// Calculate the width of a plane in luma samples.
pub fn plane_width_luma(src_width: usize, level: u16, x_ratio_uv: usize, hpad: usize) -> usize {
    todo!()
}

pub fn plane_super_offset(
    chroma: bool,
    src_height: usize,
    level: u16,
    pel: Subpel,
    vpad: usize,
    plane_pitch: usize,
    y_ratio_uv: usize,
) -> usize {
    todo!()
}
