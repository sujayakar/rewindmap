use std::time::Instant;
use criterion::black_box;
use rewindmap::simple::FractionalCascade;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::distributions::Uniform;

fn main() {
    let mut rng = StdRng::seed_from_u64(0);
    let n = 1000;
    let m = 1000;
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

    let start = Instant::now();
    let num_queries = 10000;
    for _ in 0..num_queries {
        black_box(f.bisect_left(black_box(key)));
    }
    println!("Query: {:?}", start.elapsed() / num_queries);
}
