use std::time::Instant;

use criterion::{BenchmarkId, Criterion};

pub fn cache_comparison(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cache");

    for threads in [1, 2, 4, 8] {
        group.bench_function(BenchmarkId::new("moka", threads), |bencher| {
            let mcache = moka::sync::SegmentedCache::<u64, u64>::builder(8)
                .max_capacity(8192)
                .build();
            bencher.iter_custom(|n| {
                let threads = threads.min(n);
                let ipt = n / threads;

                let barrier = std::sync::Barrier::new(threads as usize + 1);
                std::thread::scope(|scope| {
                    for _ in 0..threads {
                        scope.spawn(|| {
                            barrier.wait();
                            for _ in 0..ipt {
                                let i = rand::random();
                                mcache.insert(i, i + 1);
                            }
                        });
                    }
                    barrier.wait();
                    Instant::now()
                })
                .elapsed()
            })
        });

        group.bench_function(BenchmarkId::new("kcache", threads), |bencher| {
            let kcache = k_cache::SegmentedCache::<u64, u64>::new(8, 8192);
            bencher.iter_custom(|n| {
                let threads = threads.min(n);
                let ipt = n / threads;

                let barrier = std::sync::Barrier::new(threads as usize + 1);
                std::thread::scope(|scope| {
                    for _ in 0..threads {
                        scope.spawn(|| {
                            barrier.wait();
                            for _ in 0..ipt {
                                let i = rand::random();
                                kcache.put(i, i + 1);
                            }
                        });
                    }
                    barrier.wait();
                    Instant::now()
                })
                .elapsed()
            })
        });
    }
}

criterion::criterion_group!(benches, cache_comparison);
