use criterion::{black_box, criterion_group};
// #![feature(test)]

// extern crate test;

// use test::Bencher;

// #[bench]
// fn bench_fib_10(b: &mut Bencher) {
//     b.iter(|| {
//         black_box(10 + 10);
//         // let _ = fib(10);
//     });
// }

fn configure_group<M>(group: &mut criterion::BenchmarkGroup<M>)
where
    M: criterion::measurement::Measurement,
{
    group.sample_size(1000);
    group.sampling_mode(criterion::SamplingMode::Flat);
}

fn bench_iter_recursive(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("iter/recursive");
    configure_group(&mut group);
    group.bench_function("dfs/sequential", |b| {
        b.iter(|| {
            black_box(10 + 10);
            // use serde_json_merge::{Dfs, Iter};
            // black_box(value.clone().iter_recursive::<Dfs>().count());
        });
    });
}

criterion_group!(bench_test, bench_iter_recursive);

fn main() {
    bench_test();

    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
