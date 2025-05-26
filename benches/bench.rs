use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

// TODO: Write actual benchmarks
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(mv_analyse, criterion_benchmark);
criterion_group!(mv_compensate, criterion_benchmark);
criterion_group!(mv_recalculate, criterion_benchmark);
criterion_group!(mv_super, criterion_benchmark);
criterion_main!(mv_analyse, mv_compensate, mv_recalculate, mv_super);
