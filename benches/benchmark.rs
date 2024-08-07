#![feature(generic_arg_infer)]
#![feature(const_mut_refs)]
#![feature(generic_const_exprs)]

use std::hint::black_box;

use divan::{bench, Bencher};
use fastbuf::{Buffer, ReadBuf, WriteBuf};
use rand::Rng;

#[bench]
fn write_fastbuf(bencher: Bencher) {
    let rand = rand::thread_rng().gen_range(0..10);
    bencher.bench_local(|| {
        let mut buf = Buffer::<10000>::new();
        buf.pos = rand;
        buf.filled_pos = rand;
        buf.write(&[1, 2, 3, 4, 6, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2])
            .unwrap();
        black_box(&buf);
    });
}

fn main() {
    divan::main()
}
