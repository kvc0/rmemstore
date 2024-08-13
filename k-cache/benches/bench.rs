use criterion::criterion_main;

#[allow(clippy::unwrap_used)] // it doesn't matter in benchmarks or tests
mod benchmarks;

criterion_main! {
    benchmarks::cache_bench::benches,
}
