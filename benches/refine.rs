use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use std::num::NonZeroU8;
use std::{hint::black_box, num::NonZeroUsize};
use vapoursynth_zoomvtools::mv_plane::MVPlane;
use vapoursynth_zoomvtools::params::Subpel;
use vapoursynth_zoomvtools::refine::{
    refine_diagonal_bilinear, refine_horizontal_bicubic, refine_horizontal_bilinear,
    refine_horizontal_wiener, refine_vertical_bicubic, refine_vertical_bilinear,
    refine_vertical_wiener,
};

pub fn bench_refine_ext_pel2_8bit(c: &mut Criterion) {
    c.bench_function("refine_ext_pel2 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_2x_resolution = resolution.saturating_mul(NonZeroUsize::new(2).unwrap());
        let mut dest = vec![0u8; 4 * resolution.get() * resolution.get()];
        let mut src = vec![0u8; src_2x_resolution.get() * src_2x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Half),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(8).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel2(
                black_box(&src),
                black_box(src_2x_resolution),
                black_box(false),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel2_8bit_padded(c: &mut Criterion) {
    c.bench_function("refine_ext_pel2 8-bit padded", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_2x_resolution = resolution.saturating_mul(NonZeroUsize::new(2).unwrap());
        let mut dest = vec![0u8; 4 * resolution.get() * resolution.get()];
        let mut src = vec![0u8; src_2x_resolution.get() * src_2x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Half),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(8).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel2(
                black_box(&src),
                black_box(src_2x_resolution),
                black_box(true),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel2_16bit(c: &mut Criterion) {
    c.bench_function("refine_ext_pel2 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_2x_resolution = resolution.saturating_mul(NonZeroUsize::new(2).unwrap());
        let mut dest = vec![0u16; 4 * resolution.get() * resolution.get()];
        let mut src = vec![0u16; src_2x_resolution.get() * src_2x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Half),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(16).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel2(
                black_box(&src),
                black_box(src_2x_resolution),
                black_box(false),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel2_16bit_padded(c: &mut Criterion) {
    c.bench_function("refine_ext_pel2 16-bit padded", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_2x_resolution = resolution.saturating_mul(NonZeroUsize::new(2).unwrap());
        let mut dest = vec![0u16; 4 * resolution.get() * resolution.get()];
        let mut src = vec![0u16; src_2x_resolution.get() * src_2x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Half),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(16).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel2(
                black_box(&src),
                black_box(src_2x_resolution),
                black_box(true),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel4_8bit(c: &mut Criterion) {
    c.bench_function("refine_ext_pel4 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_4x_resolution = resolution.saturating_mul(NonZeroUsize::new(4).unwrap());
        let mut dest = vec![0u8; 16 * resolution.get() * resolution.get()];
        let mut src = vec![0u8; src_4x_resolution.get() * src_4x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Quarter),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(8).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel4(
                black_box(&src),
                black_box(src_4x_resolution),
                black_box(false),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel4_8bit_padded(c: &mut Criterion) {
    c.bench_function("refine_ext_pel4 8-bit padded", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_4x_resolution = resolution.saturating_mul(NonZeroUsize::new(4).unwrap());
        let mut dest = vec![0u8; 16 * resolution.get() * resolution.get()];
        let mut src = vec![0u8; src_4x_resolution.get() * src_4x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Quarter),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(8).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel4(
                black_box(&src),
                black_box(src_4x_resolution),
                black_box(true),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel4_16bit(c: &mut Criterion) {
    c.bench_function("refine_ext_pel4 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_4x_resolution = resolution.saturating_mul(NonZeroUsize::new(4).unwrap());
        let mut dest = vec![0u16; 16 * resolution.get() * resolution.get()];
        let mut src = vec![0u16; src_4x_resolution.get() * src_4x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Quarter),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(16).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel4(
                black_box(&src),
                black_box(src_4x_resolution),
                black_box(false),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_ext_pel4_16bit_padded(c: &mut Criterion) {
    c.bench_function("refine_ext_pel4 16-bit padded", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let src_4x_resolution = resolution.saturating_mul(NonZeroUsize::new(4).unwrap());
        let mut dest = vec![0u16; 16 * resolution.get() * resolution.get()];
        let mut src = vec![0u16; src_4x_resolution.get() * src_4x_resolution.get()];
        let mut mvp = MVPlane::new(
            black_box(resolution),
            black_box(resolution),
            black_box(Subpel::Quarter),
            black_box(0),
            black_box(0),
            black_box(NonZeroU8::new(16).unwrap()),
            black_box(0),
            black_box(resolution),
        )
        .unwrap();

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            mvp.refine_ext_pel4(
                black_box(&src),
                black_box(src_4x_resolution),
                black_box(true),
                black_box(&mut dest),
            )
        })
    });
}

pub fn bench_refine_horizontal_bilinear_8bit(c: &mut Criterion) {
    c.bench_function("refine_horizontal_bilinear 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u8; resolution.get() * resolution.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_horizontal_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_horizontal_bilinear_16bit(c: &mut Criterion) {
    c.bench_function("refine_horizontal_bilinear 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u16; resolution.get() * resolution.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_horizontal_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

pub fn bench_refine_vertical_bilinear_8bit(c: &mut Criterion) {
    c.bench_function("refine_vertical_bilinear 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u8; resolution.get() * resolution.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_vertical_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_vertical_bilinear_16bit(c: &mut Criterion) {
    c.bench_function("refine_vertical_bilinear 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u16; resolution.get() * resolution.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_vertical_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

pub fn bench_refine_diagonal_bilinear_8bit(c: &mut Criterion) {
    c.bench_function("refine_diagonal_bilinear 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let padded_width = resolution.get() + 1;
        let padded_height = resolution.get() + 1;
        let mut dest = vec![0u8; padded_width * (resolution.get() + 1)];
        let mut src = vec![0u8; padded_width * padded_height];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_diagonal_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(NonZeroUsize::new(padded_width).unwrap()),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_diagonal_bilinear_16bit(c: &mut Criterion) {
    c.bench_function("refine_diagonal_bilinear 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let padded_width = resolution.get() + 1;
        let padded_height = resolution.get() + 1;
        let mut dest = vec![0u16; padded_width * (resolution.get() + 1)];
        let mut src = vec![0u16; padded_width * padded_height];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_diagonal_bilinear(
                black_box(&mut dest),
                black_box(&src),
                black_box(NonZeroUsize::new(padded_width).unwrap()),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

pub fn bench_refine_horizontal_bicubic_8bit(c: &mut Criterion) {
    c.bench_function("refine_horizontal_bicubic 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u8; resolution.get() * resolution.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_horizontal_bicubic(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_horizontal_bicubic_16bit(c: &mut Criterion) {
    c.bench_function("refine_horizontal_bicubic 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u16; resolution.get() * resolution.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_horizontal_bicubic(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

pub fn bench_refine_vertical_bicubic_8bit(c: &mut Criterion) {
    c.bench_function("refine_vertical_bicubic 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u8; resolution.get() * resolution.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_vertical_bicubic(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_vertical_bicubic_16bit(c: &mut Criterion) {
    c.bench_function("refine_vertical_bicubic 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u16; resolution.get() * resolution.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_vertical_bicubic(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

pub fn bench_refine_horizontal_wiener_8bit(c: &mut Criterion) {
    c.bench_function("refine_horizontal_wiener 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u8; resolution.get() * resolution.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_horizontal_wiener(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_horizontal_wiener_16bit(c: &mut Criterion) {
    c.bench_function("refine_horizontal_wiener 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u16; resolution.get() * resolution.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_horizontal_wiener(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

pub fn bench_refine_vertical_wiener_8bit(c: &mut Criterion) {
    c.bench_function("refine_vertical_wiener 8-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u8; resolution.get() * resolution.get()];
        let mut src = vec![0u8; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_vertical_wiener(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(8).unwrap()),
            )
        })
    });
}

pub fn bench_refine_vertical_wiener_16bit(c: &mut Criterion) {
    c.bench_function("refine_vertical_wiener 16-bit", |b| {
        let mut rng = Xoshiro128StarStar::from_seed(*b"deadbeeflolcakes");
        let resolution = NonZeroUsize::new(256).unwrap();
        let mut dest = vec![0u16; resolution.get() * resolution.get()];
        let mut src = vec![0u16; resolution.get() * resolution.get()];

        for p in src.iter_mut() {
            *p = rng.random();
        }

        b.iter(|| {
            refine_vertical_wiener(
                black_box(&mut dest),
                black_box(&src),
                black_box(resolution),
                black_box(resolution),
                black_box(resolution),
                black_box(NonZeroU8::new(16).unwrap()),
            )
        })
    });
}

criterion_group!(
    bench_refine_ext_pel2,
    bench_refine_ext_pel2_8bit,
    bench_refine_ext_pel2_8bit_padded,
    bench_refine_ext_pel2_16bit,
    bench_refine_ext_pel2_16bit_padded
);
criterion_group!(
    bench_refine_ext_pel4,
    bench_refine_ext_pel4_8bit,
    bench_refine_ext_pel4_8bit_padded,
    bench_refine_ext_pel4_16bit,
    bench_refine_ext_pel4_16bit_padded
);
criterion_group!(
    bench_refine_bilinear,
    bench_refine_horizontal_bilinear_8bit,
    bench_refine_horizontal_bilinear_16bit,
    bench_refine_vertical_bilinear_8bit,
    bench_refine_vertical_bilinear_16bit,
    bench_refine_diagonal_bilinear_8bit,
    bench_refine_diagonal_bilinear_16bit
);
criterion_group!(
    bench_refine_bicubic,
    bench_refine_horizontal_bicubic_8bit,
    bench_refine_horizontal_bicubic_16bit,
    bench_refine_vertical_bicubic_8bit,
    bench_refine_vertical_bicubic_16bit
);
criterion_group!(
    bench_refine_wiener,
    bench_refine_horizontal_wiener_8bit,
    bench_refine_horizontal_wiener_16bit,
    bench_refine_vertical_wiener_8bit,
    bench_refine_vertical_wiener_16bit
);
criterion_main!(
    bench_refine_ext_pel2,
    bench_refine_ext_pel4,
    bench_refine_bilinear,
    bench_refine_bicubic,
    bench_refine_wiener
);
