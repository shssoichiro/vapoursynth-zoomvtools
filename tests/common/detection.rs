use anyhow::{Context, Result};
use vapoursynth::prelude::Environment;

/// Checks if C MVTools plugin is available
pub fn check_mvtools_available() -> Result<()> {
    let test_script = r#"
import vapoursynth as vs
core = vs.core
clip = core.std.BlankClip(width=64, height=64, format=vs.YUV420P8, length=10)
try:
    super = core.mv.Super(clip)
    super.set_output()
except AttributeError as e:
    raise Exception(f"C MVTools (mv namespace) not available: {e}")
"#;

    Environment::from_script(test_script).context(
        "C MVTools plugin not detected. Please install libmvtools.so to /usr/lib/vapoursynth/\n\
         Installation instructions:\n\
         - Arch Linux:    pacman -S vapoursynth-plugin-mvtools\n\
         - Ubuntu/Debian: apt-get install vapoursynth-mvtools\n\
         - From source:   https://github.com/dubhater/vapoursynth-mvtools",
    )?;

    Ok(())
}
