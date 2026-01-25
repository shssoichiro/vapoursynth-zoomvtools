use std::num::NonZeroUsize;

mod average;
mod bilinear;
mod cubic;
mod quadratic;
mod triangle;

pub use average::reduce_average;
pub use bilinear::reduce_bilinear;
pub use cubic::reduce_cubic;
pub use quadratic::reduce_quadratic;
pub use triangle::reduce_triangle;

pub type ReduceFn<T> = fn(&mut [T], &[T], NonZeroUsize, NonZeroUsize, NonZeroUsize, NonZeroUsize);
