use std::{hint::black_box, num::NonZeroUsize};

use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use vapoursynth_zoomvtools::average::average2;

pub fn bench_average2_8bit(c: &mut Criterion) {
    c.bench_function("average2 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut src1 = vec![0u8; resolution.get() * resolution.get()];
        let mut src2 = vec![0u8; resolution.get() * resolution.get()];
        let mut dest = vec![0u8; resolution.get() * resolution.get()];

        for p in src1.iter_mut() {
            *p = rng.random();
        }
        for p in src2.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            average2(
                black_box(&src1),
                black_box(&src2),
                black_box(&mut dest),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
            )
        })
    });
}

pub fn bench_average2_16bit(c: &mut Criterion) {
    c.bench_function("average2 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut src1 = vec![0u16; resolution.get() * resolution.get()];
        let mut src2 = vec![0u16; resolution.get() * resolution.get()];
        let mut dest = vec![0u16; resolution.get() * resolution.get()];

        for p in src1.iter_mut() {
            *p = rng.random();
        }
        for p in src2.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            average2(
                black_box(&src1),
                black_box(&src2),
                black_box(&mut dest),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
            )
        })
    });
}

criterion_group!(bench_average2, bench_average2_8bit, bench_average2_16bit);
criterion_main!(bench_average2);
