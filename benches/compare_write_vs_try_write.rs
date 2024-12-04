use std::hint::black_box;

use divan::{bench, Bencher};
use fastbuf::{Buffer, WriteBuf};

#[bench(sample_size = 10000, sample_count = 10000)]
fn bench_try_write(bencher: Bencher) {
    bencher.bench_local(|| {
        for _i in 0..100 {
            let mut buf = Buffer::<200000>::new();
            let _ = black_box(buf.try_write(&[1]));
            black_box(&buf);
        }
    });
}

#[bench(sample_size = 10000, sample_count = 10000)]
fn bench_write(bencher: Bencher) {
    bencher.bench_local(|| {
        for _i in 0..100 {
            let mut buf = Buffer::<200000>::new();
            buf.write(&[1]);
            black_box(&buf);
        }
    });
}

fn main() {
    std::thread::Builder::new()
        .stack_size(1280 * 1024 * 1024)
        .spawn(|| divan::main())
        .unwrap()
        .join()
        .unwrap();
}
