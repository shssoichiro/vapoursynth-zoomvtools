use anyhow::Result;
use vapoursynth::{
    format::{FormatID, PresetFormat},
    prelude::Environment,
};

pub fn create_test_env(
    width: usize,
    height: usize,
    format: PresetFormat,
    frames: usize,
) -> Result<Environment> {
    let format = i32::from(FormatID::from(format));
    let script = format!(
        r#"
import vapoursynth as vs
core = vs.core
clip = core.std.BlankClip(width={width}, height={height}, format={format}, length={frames})
clip.set_output()
"#,
    );

    let env = Environment::from_script(&script)?;
    Ok(env)
}
