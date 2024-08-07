use divan::{bench, Bencher};
use fastbuf::Buffer;

#[bench]
fn benchmark(bencher: Bencher) {
    let buf = Buffer::<10>::new();
    bencher.bench_local(|| {})
}
