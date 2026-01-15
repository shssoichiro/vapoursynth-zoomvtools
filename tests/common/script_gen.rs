#![allow(dead_code)]

use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct TestClipConfig {
    pub width: usize,
    pub height: usize,
    pub format: &'static str, // e.g., "vs.YUV420P8"
    pub length: usize,
    pub content_type: ClipContentType,
}

#[derive(Debug, Clone)]
pub enum ClipContentType {
    Blank,
    Gradient,
    Checkerboard,
    MovingBox { speed_x: i32, speed_y: i32 },
    Noise { seed: u64 },
}

impl ClipContentType {
    fn to_script(&self) -> String {
        match self {
            Self::Blank => String::new(),
            Self::Gradient => {
                // Generate gradient pattern using std.Expr
                // Use a simple gradient based on x coordinate
                String::new() // Just use blank for now - gradient requires more complex setup
            }
            Self::Checkerboard => {
                // Generate checkerboard pattern
                String::new() // Just use blank for now
            }
            Self::MovingBox { speed_x, speed_y } => {
                // Create a simple pattern with motion by shifting frame content
                // This creates detectable motion for the motion estimation algorithms
                format!(
                    r#"
# Create a base clip with a simple pattern (white center on black background)
import vapoursynth as vs
box_size = min(clip.width, clip.height) // 4
border_h = (clip.width - box_size) // 2
border_v = (clip.height - box_size) // 2

# Create white box in center
base = core.std.BlankClip(clip, color=[0, 128, 128])  # Black background
box = core.std.BlankClip(clip, width=box_size, height=box_size, color=[255, 128, 128])  # White box
box_bordered = core.std.AddBorders(box, left=border_h, right=border_h, top=border_v, bottom=border_v, color=[0, 128, 128])

# Shift the pattern per frame to create motion
def shift_clip(n):
    # Ensure shifts are mod 2 for YUV420 chroma alignment
    shift_x = ((n * {speed_x}) % (clip.width - box_size)) & ~1
    shift_y = ((n * {speed_y}) % (clip.height - box_size)) & ~1
    if shift_x == 0 and shift_y == 0:
        return box_bordered
    # Shift by cropping and adding borders
    cropped = core.std.Crop(box_bordered, left=shift_x, top=shift_y, right=0, bottom=0)
    shifted = core.std.AddBorders(cropped, left=0, top=0, right=shift_x, bottom=shift_y, color=[0, 128, 128])
    return shifted

clip = core.std.FrameEval(clip, shift_clip)
"#
                )
            }
            Self::Noise { seed } => {
                format!(
                    r#"
clip = core.grain.Add(clip, var={seed}, constant=True)
"#
                )
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilterParams {
    pub hpad: Option<i64>,
    pub vpad: Option<i64>,
    pub pel: Option<i64>,
    pub levels: Option<i64>,
    pub chroma: Option<i64>,
    pub sharp: Option<i64>,
    pub rfilter: Option<i64>,
    // Analyse params
    pub blksize: Option<i64>,
    pub blksizev: Option<i64>,
    pub search: Option<i64>,
    pub searchparam: Option<i64>,
    pub pelsearch: Option<i64>,
    pub isb: Option<i64>,
    pub lambda: Option<i64>,
    pub chroma_analyse: Option<i64>,
    pub delta: Option<i64>,
    pub truemotion: Option<i64>,
    pub lsad: Option<i64>,
    pub plevel: Option<i64>,
    pub global_motion: Option<i64>,
    pub pnew: Option<i64>,
    pub pzero: Option<i64>,
    pub pglobal: Option<i64>,
    pub overlap: Option<i64>,
    pub overlapv: Option<i64>,
    pub divide: Option<i64>,
    pub badsad: Option<i64>,
    pub badrange: Option<i64>,
    pub meander: Option<i64>,
    pub trymany: Option<i64>,
    pub fields: Option<i64>,
    pub tff: Option<i64>,
    pub search_coarse: Option<i64>,
    pub dct: Option<i64>,
}

impl FilterParams {
    pub fn to_kwargs(&self) -> String {
        let mut parts = Vec::new();

        // Super params
        if let Some(v) = self.hpad {
            parts.push(format!("hpad={}", v));
        }
        if let Some(v) = self.vpad {
            parts.push(format!("vpad={}", v));
        }
        if let Some(v) = self.pel {
            parts.push(format!("pel={}", v));
        }
        if let Some(v) = self.levels {
            parts.push(format!("levels={}", v));
        }
        if let Some(v) = self.chroma {
            parts.push(format!("chroma={}", v));
        }
        if let Some(v) = self.sharp {
            parts.push(format!("sharp={}", v));
        }
        if let Some(v) = self.rfilter {
            parts.push(format!("rfilter={}", v));
        }

        // Analyse params
        if let Some(v) = self.blksize {
            parts.push(format!("blksize={}", v));
        }
        if let Some(v) = self.blksizev {
            parts.push(format!("blksizev={}", v));
        }
        if let Some(v) = self.search {
            parts.push(format!("search={}", v));
        }
        if let Some(v) = self.searchparam {
            parts.push(format!("searchparam={}", v));
        }
        if let Some(v) = self.pelsearch {
            parts.push(format!("pelsearch={}", v));
        }
        if let Some(v) = self.isb {
            parts.push(format!("isb={}", v));
        }
        if let Some(v) = self.lambda {
            parts.push(format!("lambda={}", v));
        }
        if let Some(v) = self.chroma_analyse {
            parts.push(format!("chroma={}", v));
        }
        if let Some(v) = self.delta {
            parts.push(format!("delta={}", v));
        }
        if let Some(v) = self.truemotion {
            parts.push(format!("truemotion={}", v));
        }
        if let Some(v) = self.lsad {
            parts.push(format!("lsad={}", v));
        }
        if let Some(v) = self.plevel {
            parts.push(format!("plevel={}", v));
        }
        if let Some(v) = self.global_motion {
            parts.push(format!("global={}", v));
        }
        if let Some(v) = self.pnew {
            parts.push(format!("pnew={}", v));
        }
        if let Some(v) = self.pzero {
            parts.push(format!("pzero={}", v));
        }
        if let Some(v) = self.pglobal {
            parts.push(format!("pglobal={}", v));
        }
        if let Some(v) = self.overlap {
            parts.push(format!("overlap={}", v));
        }
        if let Some(v) = self.overlapv {
            parts.push(format!("overlapv={}", v));
        }
        if let Some(v) = self.divide {
            parts.push(format!("divide={}", v));
        }
        if let Some(v) = self.badsad {
            parts.push(format!("badsad={}", v));
        }
        if let Some(v) = self.badrange {
            parts.push(format!("badrange={}", v));
        }
        if let Some(v) = self.meander {
            parts.push(format!("meander={}", v));
        }
        if let Some(v) = self.trymany {
            parts.push(format!("trymany={}", v));
        }
        if let Some(v) = self.fields {
            parts.push(format!("fields={}", v));
        }
        if let Some(v) = self.tff {
            parts.push(format!("tff={}", v));
        }
        if let Some(v) = self.search_coarse {
            parts.push(format!("search_coarse={}", v));
        }
        if let Some(v) = self.dct {
            parts.push(format!("dct={}", v));
        }

        parts.join(", ")
    }
}

pub fn generate_comparison_script(
    clip_config: &TestClipConfig,
    super_params: &FilterParams,
    analyse_params: Option<&FilterParams>,
) -> String {
    let mut script = format!(
        r#"
import vapoursynth as vs
core = vs.core

# Generate base clip
clip = core.std.BlankClip(width={}, height={}, format={}, length={})
{}

# Apply C MVTools
c_super = core.mv.Super(clip{})
"#,
        clip_config.width,
        clip_config.height,
        clip_config.format,
        clip_config.length,
        clip_config.content_type.to_script(),
        if !super_params.to_kwargs().is_empty() {
            format!(", {}", super_params.to_kwargs())
        } else {
            String::new()
        }
    );

    if let Some(analyse) = analyse_params {
        writeln!(
            &mut script,
            "c_vectors = core.mv.Analyse(c_super{})",
            if !analyse.to_kwargs().is_empty() {
                format!(", {}", analyse.to_kwargs())
            } else {
                String::new()
            }
        )
        .unwrap();
    }

    write!(
        &mut script,
        r#"
# Apply Rust ZoomVTools
r_super = core.zoomv.Super(clip{})
"#,
        if !super_params.to_kwargs().is_empty() {
            format!(", {}", super_params.to_kwargs())
        } else {
            String::new()
        }
    )
    .unwrap();

    if let Some(analyse) = analyse_params {
        writeln!(
            &mut script,
            "r_vectors = core.zoomv.Analyse(r_super{})",
            if !analyse.to_kwargs().is_empty() {
                format!(", {}", analyse.to_kwargs())
            } else {
                String::new()
            }
        )
        .unwrap();
        script.push_str("c_vectors.set_output(0)\nr_vectors.set_output(1)\n");
    } else {
        script.push_str("c_super.set_output(0)\nr_super.set_output(1)\n");
    }

    script
}
