//! Criterion Benchmarks for Hyperbolic KV Operations
//!
//! Provides microbenchmarks for individual operations to measure
//! instruction counts, memory access patterns, and kernel performance.

use crate::hyperbolic::{random_lorentz_vec, AtomCoord, LorentzVec, ToroidalPhase};
use crate::picard::PicardReducer;
use crate::vptree::VpTree;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn bench_minkowski_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("minkowski_dot");
    let mut rng = StdRng::seed_from_u64(42);

    let a = random_lorentz_vec(&mut rng, 100.0);
    let b = random_lorentz_vec(&mut rng, 100.0);

    group.throughput(Throughput::Elements(1));
    group.bench_function("Q32.31", |bencher| {
        bencher.iter(|| {
            let dot = black_box(&a).minkowski_dot(black_box(&b));
            black_box(dot);
        });
    });
    group.finish();
}

fn bench_hyperbolic_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyperbolic_distance");
    let mut rng = StdRng::seed_from_u64(42);

    let a = random_lorentz_vec(&mut rng, 100.0);
    let b = random_lorentz_vec(&mut rng, 100.0);

    group.throughput(Throughput::Elements(1));
    group.bench_function("arcosh", |bencher| {
        bencher.iter(|| {
            let dist = black_box(&a).hyperbolic_distance(black_box(&b));
            black_box(dist);
        });
    });
    group.finish();
}

fn bench_toroidal_phase(c: &mut Criterion) {
    let mut group = c.benchmark_group("toroidal_phase");
    let mut rng = StdRng::seed_from_u64(42);

    let v = random_lorentz_vec(&mut rng, 100.0);

    group.throughput(Throughput::Elements(1));
    group.bench_function("from_lorentz", |bencher| {
        bencher.iter(|| {
            let phase = ToroidalPhase::from_lorentz(black_box(&v));
            black_box(phase);
        });
    });
    group.finish();
}

fn bench_toroidal_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("toroidal_distance");
    let mut rng = StdRng::seed_from_u64(42);

    let v1 = random_lorentz_vec(&mut rng, 100.0);
    let v2 = random_lorentz_vec(&mut rng, 100.0);
    let p1 = ToroidalPhase::from_lorentz(&v1);
    let p2 = ToroidalPhase::from_lorentz(&v2);

    group.throughput(Throughput::Elements(1));
    group.bench_function("distance_sq", |bencher| {
        bencher.iter(|| {
            let dist = black_box(&p1).distance_sq(black_box(&p2));
            black_box(dist);
        });
    });
    group.finish();
}

fn bench_vptree_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_build");

    for size in [1000, 10000, 100000].iter() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut atoms = Vec::with_capacity(*size);

        for i in 0..*size {
            let h3 = random_lorentz_vec(&mut rng, 100.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            atoms.push(AtomCoord::new(h3, t2, i as u64, false));
        }

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                let mut tree = VpTree::new(32);
                tree.build(black_box(atoms.clone()));
                black_box(tree);
            });
        });
    }
    group.finish();
}

fn bench_vptree_knn(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_knn");

    for size in [1000, 10000, 100000].iter() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut atoms = Vec::with_capacity(*size);

        for i in 0..*size {
            let h3 = random_lorentz_vec(&mut rng, 100.0);
            let t2 = ToroidalPhase::from_lorentz(&h3);
            atoms.push(AtomCoord::new(h3, t2, i as u64, false));
        }

        let mut tree = VpTree::new(32);
        tree.build(atoms.clone());

        let query_h3 = random_lorentz_vec(&mut rng, 100.0);
        let query_t2 = ToroidalPhase::from_lorentz(&query_h3);
        let query = AtomCoord::new(query_h3, query_t2, u64::MAX, false);

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("k=32", size), size, |bencher, _| {
            bencher.iter(|| {
                let results = tree.knn_search(black_box(&query), 32);
                black_box(results);
            });
        });
    }
    group.finish();
}

fn bench_vptree_evict(c: &mut Criterion) {
    let mut group = c.benchmark_group("vptree_evict");

    for size in [1000, 10000, 100000].iter() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut atoms = Vec::with_capacity(*size);

        for i in 0..*size {
            let h3 = random_lorentz_vec(&mut rng, 1000.0); // High x0 for eviction
            let t2 = ToroidalPhase::from_lorentz(&h3);
            atoms.push(AtomCoord::new(h3, t2, i as u64, false));
        }

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |bencher, _| {
            bencher.iter(|| {
                let mut t = VpTree::new(32);
                t.build(black_box(atoms.clone()));
                let _ = t.evict_max_radial();
                black_box(t);
            });
        });
    }
    group.finish();
}

fn bench_picard_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("picard_reduce");

    for count in [100, 1000, 10000].iter() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut atoms: Vec<LorentzVec> = (0..*count)
            .map(|_| random_lorentz_vec(&mut rng, 10000.0))
            .collect();

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("iterations=32", count), count, |bencher, _| {
            bencher.iter(|| {
                let mut atoms_clone = atoms.clone();
                PicardReducer::reduce_cpu_fallback(black_box(&mut atoms_clone), 32);
                black_box(atoms_clone);
            });
        });
    }
    group.finish();
}

fn bench_atom_retrieval_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("atom_retrieval_score");
    let mut rng = StdRng::seed_from_u64(42);

    let h3_a = random_lorentz_vec(&mut rng, 100.0);
    let h3_b = random_lorentz_vec(&mut rng, 100.0);
    let t2_a = ToroidalPhase::from_lorentz(&h3_a);
    let t2_b = ToroidalPhase::from_lorentz(&h3_b);

    let atom_a = AtomCoord::new(h3_a, t2_a, 1, false);
    let atom_b = AtomCoord::new(h3_b, t2_b, 2, false);

    group.throughput(Throughput::Elements(1));
    group.bench_function("score", |b| {
        b.iter(|| {
            let score = black_box(&atom_a).retrieval_score(black_box(&atom_b));
            black_box(score);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_minkowski_dot,
    bench_hyperbolic_distance,
    bench_toroidal_phase,
    bench_toroidal_distance,
    bench_vptree_build,
    bench_vptree_knn,
    bench_vptree_evict,
    bench_picard_reduce,
    bench_atom_retrieval_score,
);

criterion_main!(benches);
