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

macro_rules! verify_asm {
    ($module:ident, $func:ident($dest:expr, $($args:expr),* $(,)?)) => {{
        if stringify!($module) != "rust" {
            // Compare the rust version against the SIMD version we are testing.
            let mut rust_dest = $dest.clone();
            unsafe {
                super::rust::$func(
                    &mut rust_dest,
                    $($args),*
                );
            }
            unsafe {
                super::$module::$func(
                    $dest,
                    $($args),*
                );
            }
            assert_eq!(rust_dest, *$dest,
                "Mismatch between Rust and {} in {}",
                stringify!($module),
                stringify!($func)
            );
        } else {
            // We are testing the rust version directly
            unsafe {
                super::$module::$func(
                    $dest,
                    $($args),*
                );
            }
        }
    }};
}
