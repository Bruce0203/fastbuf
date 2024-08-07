use std::hint::black_box;

use divan::{bench, Bencher};
use fastbuf::{Buffer, ReadBuf, WriteBuf};

#[bench]
fn benchmark(bencher: Bencher) {
    let mut buf = Buffer::<1000>::new();
    buf.write(&[1, 2, 3, 4]).unwrap();
    bencher.bench_local(|| {
        for _i in 0..10 {
            black_box(buf.get_continuous(10));
        }
    });
}

fn main() {
    divan::main()
}
