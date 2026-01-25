//! Tensor operation benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use a16_tensor::Tensor;

fn bench_tensor_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_creation");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                Tensor::zeros(black_box(&[size]))
            });
        });
    }
    group.finish();
}

fn bench_tensor_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_add");
    
    for size in [100, 1000, 10000].iter() {
        let a = Tensor::randn(&[*size]);
        let b = Tensor::randn(&[*size]);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, _| {
            bench.iter(|| {
                black_box(a.add(&b))
            });
        });
    }
    group.finish();
}

fn bench_tensor_matmul(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_matmul");
    
    for size in [16, 32, 64, 128].iter() {
        let a = Tensor::randn(&[*size, *size]);
        let b = Tensor::randn(&[*size, *size]);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, _| {
            bench.iter(|| {
                black_box(a.matmul(&b))
            });
        });
    }
    group.finish();
}

fn bench_tensor_relu(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_relu");
    
    for size in [100, 1000, 10000].iter() {
        let t = Tensor::randn(&[*size]);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bench, _| {
            bench.iter(|| {
                black_box(t.relu())
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_tensor_creation,
    bench_tensor_add,
    bench_tensor_matmul,
    bench_tensor_relu,
);
criterion_main!(benches);
