use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use std::{hint::black_box, num::NonZeroUsize};
use vapoursynth_zoomvtools::reduce::{
    reduce_average, reduce_bilinear, reduce_cubic, reduce_quadratic, reduce_triangle,
};

pub fn bench_reduce_average_8bit(c: &mut Criterion) {
    c.bench_function("reduce_average 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u8; dest_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u8; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_average(
                black_box(&mut dest),
                black_box(&src),
                black_box(dest_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_average_16bit(c: &mut Criterion) {
    c.bench_function("reduce_average 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u16; dest_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u16; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_average(
                black_box(&mut dest),
                black_box(&src),
                black_box(dest_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_bilinear_8bit(c: &mut Criterion) {
    c.bench_function("reduce_bilinear 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u8; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u8; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_bilinear_16bit(c: &mut Criterion) {
    c.bench_function("reduce_bilinear 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u16; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u16; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_cubic_8bit(c: &mut Criterion) {
    c.bench_function("reduce_cubic 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u8; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u8; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_cubic(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_cubic_16bit(c: &mut Criterion) {
    c.bench_function("reduce_cubic 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u16; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u16; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_cubic(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_quadratic_8bit(c: &mut Criterion) {
    c.bench_function("reduce_quadratic 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u8; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u8; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_quadratic(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_quadratic_16bit(c: &mut Criterion) {
    c.bench_function("reduce_quadratic 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u16; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u16; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_quadratic(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_triangle_8bit(c: &mut Criterion) {
    c.bench_function("reduce_triangle 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u8; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u8; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_triangle(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

pub fn bench_reduce_triangle_16bit(c: &mut Criterion) {
    c.bench_function("reduce_triangle 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let src_resolution = NonZeroUsize::new(256).unwrap();
        let dest_resolution = NonZeroUsize::new(src_resolution.get() / 2).unwrap();
        let mut dest = vec![0u16; src_resolution.get() * dest_resolution.get()];
        let mut src = vec![0u16; src_resolution.get() * src_resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            reduce_triangle(
                black_box(&mut dest),
                black_box(&src),
                black_box(src_resolution),
                black_box(src_resolution),
                black_box(dest_resolution),
                black_box(dest_resolution),
            )
        })
    });
}

criterion_group!(
    bench_reduce_average,
    bench_reduce_average_8bit,
    bench_reduce_average_16bit
);
criterion_group!(
    bench_reduce_bilinear,
    bench_reduce_bilinear_8bit,
    bench_reduce_bilinear_16bit
);
criterion_group!(
    bench_reduce_cubic,
    bench_reduce_cubic_8bit,
    bench_reduce_cubic_16bit
);
criterion_group!(
    bench_reduce_quadratic,
    bench_reduce_quadratic_8bit,
    bench_reduce_quadratic_16bit
);
criterion_group!(
    bench_reduce_triangle,
    bench_reduce_triangle_8bit,
    bench_reduce_triangle_16bit
);
criterion_main!(
    bench_reduce_average,
    bench_reduce_bilinear,
    bench_reduce_cubic,
    bench_reduce_quadratic,
    bench_reduce_triangle
);
