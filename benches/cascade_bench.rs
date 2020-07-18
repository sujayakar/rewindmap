use std::time::Instant;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rewindmap::simple::{bisect_left, FractionalCascade};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::distributions::Uniform;

fn binary_searches<T: Copy + Ord>(arrays: &[Vec<T>], key: T) -> Vec<usize> {
    let mut out = Vec::with_capacity(arrays.len());
    for array in arrays {
        out.push(bisect_left(array, key));
    }
    out
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(0);

    for n in 100..101 {
        let n = 1000;
        let m = 100;

        let key = rng.gen_range(0, m);
        let dist = Uniform::new(0, m);
        let mut v: Vec<Vec<usize>> = (0..n)
            .map(|_| (&mut rng).sample_iter(dist).take(m).collect())
            .collect();
        for l in &mut v {
            l.sort();
        }

        let start = Instant::now();
        let f = FractionalCascade::new(v.clone());
        println!("Index construction: {:?}", start.elapsed());
        let name = format!("fc_query_{}", n);
        c.bench_function(&name, |b| b.iter(|| f.bisect_left(black_box(key))));
        c.bench_function("binary_search", |b| b.iter(|| binary_searches(&v, black_box(key))));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
