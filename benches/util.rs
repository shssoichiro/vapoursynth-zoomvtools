use std::{hint::black_box, num::NonZeroUsize};

use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use vapoursynth_zoomvtools::util::vs_bitblt;

pub fn bench_vs_bitblt_8bit_same_stride(c: &mut Criterion) {
    c.bench_function("vs_bitblt 8-bit same stride", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let dest_stride = resolution;
        let mut dest = vec![0u8; dest_stride.get() * dest_stride.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            vs_bitblt(
                black_box(&mut dest),
                black_box(dest_stride),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
            )
        })
    });
}

pub fn bench_vs_bitblt_8bit_different_stride(c: &mut Criterion) {
    c.bench_function("vs_bitblt 8-bit different stride", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let dest_stride = resolution.saturating_add(8);
        let mut dest = vec![0u8; dest_stride.get() * dest_stride.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            vs_bitblt(
                black_box(&mut dest),
                black_box(dest_stride),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
            )
        })
    });
}

pub fn bench_vs_bitblt_16bit_same_stride(c: &mut Criterion) {
    c.bench_function("vs_bitblt 16-bit same stride", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let dest_stride = resolution;
        let mut dest = vec![0u16; dest_stride.get() * dest_stride.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            vs_bitblt(
                black_box(&mut dest),
                black_box(dest_stride),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
            )
        })
    });
}

pub fn bench_vs_bitblt_16bit_different_stride(c: &mut Criterion) {
    c.bench_function("vs_bitblt 16-bit different stride", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let dest_stride = resolution.saturating_add(8);
        let mut dest = vec![0u16; dest_stride.get() * dest_stride.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            vs_bitblt(
                black_box(&mut dest),
                black_box(dest_stride),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
            )
        })
    });
}

criterion_group!(
    bench_vs_bitblt,
    bench_vs_bitblt_8bit_same_stride,
    bench_vs_bitblt_8bit_different_stride,
    bench_vs_bitblt_16bit_same_stride,
    bench_vs_bitblt_16bit_different_stride
);
criterion_main!(bench_vs_bitblt);
