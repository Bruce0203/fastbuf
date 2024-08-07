#![allow(soft_unstable)]
#![feature(generic_arg_infer)]
#![feature(const_mut_refs)]
#![feature(generic_const_exprs)]

use std::{hint::black_box, sync::LazyLock};

use fast_collections::Cursor;
use fastbuf::{Buffer, WriteBuf};
use rand::Rng;

const BUF_LEN: usize = 10000;

static RAND_LEN: LazyLock<usize> = LazyLock::new(|| rand::thread_rng().gen_range(7..10));

#[divan::bench(args = [get_model()])]
fn write_array_with_fastbuf_buffer(model: &Vec<u8>) {
    let mut buf = Buffer::<BUF_LEN>::new();
    buf.fill(model.len(), |unfilled| {
        unfilled.copy_from_slice(&model);
        model.len()
    });
    black_box(&buf);
}

#[divan::bench(args = [get_model()])]
fn write_array_with_fast_collections_cursor(model: &Vec<u8>) {
    let mut cur = Cursor::<u8, BUF_LEN>::new();
    if model.len() < cur.capacity() - cur.filled_len() {
        cur.as_array()[0..model.len()].copy_from_slice(model.as_slice());
    }
    black_box(&cur);
}

fn get_model() -> Vec<u8> {
    let value = [*RAND_LEN as u8; 100];
    let mut vec = value.to_vec();
    unsafe { vec.set_len(*RAND_LEN as usize) };
    vec
}

fn main() {
    divan::main()
}
