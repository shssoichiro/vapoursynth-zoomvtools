use crate::params::Subpel;

pub fn plane_height_luma(src_height: usize, level: u16, y_ratio_uv: usize, vpad: usize) -> usize {
    let mut height = src_height;

    for _i in 1..=level {
        height = if vpad >= y_ratio_uv {
            (height / y_ratio_uv).div_ceil(2) * y_ratio_uv
        } else {
            ((height / y_ratio_uv) / 2) * y_ratio_uv
        };
    }

    height
}

pub fn plane_width_luma(src_width: usize, level: u16, x_ratio_uv: usize, hpad: usize) -> usize {
    let mut width = src_width;

    for _i in 1..=level {
        width = if hpad >= x_ratio_uv {
            (width / x_ratio_uv).div_ceil(2) * x_ratio_uv
        } else {
            ((width / x_ratio_uv) / 2) * x_ratio_uv
        };
    }

    width
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
    // storing subplanes in superframes may be implemented by various ways
    let mut height = src_height; // luma or chroma

    let mut offset;

    if level == 0 {
        offset = 0;
    } else {
        let pel = usize::from(pel);
        offset = pel * pel * plane_pitch * (src_height + vpad * 2);

        for i in 1..level {
            // FIXME: Are we sure this should pass `src_height` and not `height?`
            height = if chroma {
                plane_height_luma(src_height * y_ratio_uv, i, y_ratio_uv, vpad * y_ratio_uv)
                    / y_ratio_uv
            } else {
                plane_height_luma(src_height, i, y_ratio_uv, vpad)
            };

            offset += plane_pitch * (height + vpad * 2);
        }
    }

    offset
}
