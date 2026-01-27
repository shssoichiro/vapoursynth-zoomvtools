use std::{hint::black_box, num::NonZeroUsize};

use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use vapoursynth_zoomvtools::util::luma_sum;

pub fn bench_luma_sum_4x4_8bit(c: &mut Criterion) {
    c.bench_function("luma_sum 4x4 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let width = NonZeroUsize::new(4).unwrap();
        let height = NonZeroUsize::new(4).unwrap();
        let mut src = vec![0u8; width.get() * height.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            luma_sum(
                black_box(width),
                black_box(height),
                black_box(&src),
                black_box(width),
            )
        })
    });
}

pub fn bench_luma_sum_4x4_16bit(c: &mut Criterion) {
    c.bench_function("luma_sum 4x4 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let width = NonZeroUsize::new(4).unwrap();
        let height = NonZeroUsize::new(4).unwrap();
        let mut src = vec![0u16; width.get() * height.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            luma_sum(
                black_box(width),
                black_box(height),
                black_box(&src),
                black_box(width),
            )
        })
    });
}

pub fn bench_luma_sum_16x16_8bit(c: &mut Criterion) {
    c.bench_function("luma_sum 16x16 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let width = NonZeroUsize::new(16).unwrap();
        let height = NonZeroUsize::new(16).unwrap();
        let mut src = vec![0u8; width.get() * height.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            luma_sum(
                black_box(width),
                black_box(height),
                black_box(&src),
                black_box(width),
            )
        })
    });
}

pub fn bench_luma_sum_16x16_16bit(c: &mut Criterion) {
    c.bench_function("luma_sum 16x16 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let width = NonZeroUsize::new(16).unwrap();
        let height = NonZeroUsize::new(16).unwrap();
        let mut src = vec![0u16; width.get() * height.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            luma_sum(
                black_box(width),
                black_box(height),
                black_box(&src),
                black_box(width),
            )
        })
    });
}

pub fn bench_luma_sum_64x64_8bit(c: &mut Criterion) {
    c.bench_function("luma_sum 64x64 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(64).unwrap();
        let mut src = vec![0u8; width.get() * height.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            luma_sum(
                black_box(width),
                black_box(height),
                black_box(&src),
                black_box(width),
            )
        })
    });
}

pub fn bench_luma_sum_64x64_16bit(c: &mut Criterion) {
    c.bench_function("luma_sum 64x64 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let width = NonZeroUsize::new(64).unwrap();
        let height = NonZeroUsize::new(64).unwrap();
        let mut src = vec![0u16; width.get() * height.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            luma_sum(
                black_box(width),
                black_box(height),
                black_box(&src),
                black_box(width),
            )
        })
    });
}

criterion_group!(
    bench_luma_sum,
    bench_luma_sum_4x4_8bit,
    bench_luma_sum_4x4_16bit,
    bench_luma_sum_16x16_8bit,
    bench_luma_sum_16x16_16bit,
    bench_luma_sum_64x64_8bit,
    bench_luma_sum_64x64_16bit
);
criterion_main!(bench_luma_sum);
