use std::cmp::{max, min};

/// Port of C's `nearbyintf` function which rounds `.5` values to the nearest even number.
/// Why on earth IEEE 754 decided this should be the standard, I will never know.
#[must_use]
pub fn round_ties_to_even(x: f32) -> f32 {
    let truncated = x.trunc();
    let fractional = x - truncated;

    match fractional.abs() {
        f if f < 0.5 => truncated,
        f if f > 0.5 => truncated + x.signum(),
        _ => {
            // Exactly 0.5 - round to even
            if truncated as i32 % 2 == 0 {
                truncated
            } else {
                truncated + x.signum()
            }
        }
    }
}

/// find the median between a, b and c
#[must_use]
pub fn median<T: Ord + Copy>(a: T, b: T, c: T) -> T {
    max(min(a, b), min(max(a, b), c))
}
