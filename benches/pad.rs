use std::{hint::black_box, num::NonZeroUsize};

use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use vapoursynth_zoomvtools::pad::pad_reference_frame;

pub fn bench_pad_reference_frame_8bit(c: &mut Criterion) {
    c.bench_function("pad_reference_frame 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let vpad = 16;
        let hpad = 16;
        let alloc_height = resolution.saturating_add(2 * vpad);
        let pitch = resolution.saturating_add(2 * hpad);
        let offset = 0;
        let mut plane = vec![0u8; pitch.get() * alloc_height.get()];

        for p in plane.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            pad_reference_frame(
                black_box(offset),
                black_box(pitch),
                black_box(hpad),
                black_box(vpad),
                black_box(resolution),
                black_box(resolution),
                black_box(&mut plane),
            )
        })
    });
}

pub fn bench_pad_reference_frame_16bit(c: &mut Criterion) {
    c.bench_function("pad_reference_frame 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let vpad = 16;
        let hpad = 16;
        let alloc_height = resolution.saturating_add(2 * vpad);
        let pitch = resolution.saturating_add(2 * hpad);
        let offset = 0;
        let mut plane = vec![0u16; pitch.get() * alloc_height.get()];

        for p in plane.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            pad_reference_frame(
                black_box(offset),
                black_box(pitch),
                black_box(hpad),
                black_box(vpad),
                black_box(resolution),
                black_box(resolution),
                black_box(&mut plane),
            )
        })
    });
}

criterion_group!(
    bench_pad_reference_frame,
    bench_pad_reference_frame_8bit,
    bench_pad_reference_frame_16bit
);
criterion_main!(bench_pad_reference_frame);
