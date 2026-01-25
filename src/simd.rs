#[inline]
#[allow(non_snake_case)]
// Unstable function copied from std lib
pub const fn _MM_SHUFFLE(z: u32, y: u32, x: u32, w: u32) -> i32 {
    ((z << 6) | (y << 4) | (x << 2) | w) as i32
}
