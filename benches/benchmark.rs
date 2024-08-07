#![allow(soft_unstable)]
#![feature(generic_arg_infer)]
#![feature(const_mut_refs)]
#![feature(generic_const_exprs)]

use std::{
    hint::black_box,
    mem::{transmute, MaybeUninit},
    sync::LazyLock,
};

use fast_collections::Cursor;
use fastbuf::{Buffer, ReadBuf, WriteBuf};
use rand::Rng;

pub const SAMPLE_SIZE: u32 = 1000;
pub const SAMPLE_COUNT: u32 = 1000;
pub type LenHeader = u8;
pub const TARGET_STRUCT_LEN: usize = 300;

const BUF_LEN: usize = 400;

static RAND_LEN: LazyLock<usize> = LazyLock::new(|| rand::thread_rng().gen_range(7..10));

type LenUint = u32;

#[divan::bench(args = [get_model()], sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn write_array_with_fastbuf_buffer(model: &Vec<u8>) {
    let mut buf = get_buf();
    buf.write_many(&[&(model.len() as u16).to_be_bytes(), model]);
    black_box(&buf);
}

#[divan::bench(args = [get_model()], sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn write_array_with_fast_collections_cursor(model: &Vec<u8>) {
    let mut cur = get_cur();
    let model_len = model.len();
    if model_len < cur.capacity() - cur.filled_len() {
        cur.as_array()[..2].copy_from_slice(&(model.len() as u16).to_be_bytes());
        cur.as_array()[2..2 + model_len].copy_from_slice(model);
    }
    *unsafe { cur.filled_len_mut() } += model_len;
    black_box(&cur);
}

#[divan::bench(args = [get_model()], sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn read_array_with_fastbuf_buffer(model: &Vec<u8>) {
    let mut buf = get_buf();
    #[allow(invalid_value)]
    let mut bytes: [u8; size_of::<LenHeader>()] =
        [unsafe { MaybeUninit::uninit().assume_init() }; _];
    bytes.copy_from_slice(buf.read(size_of::<LenHeader>()));
    let len = LenHeader::from_be_bytes(bytes);
    let value = buf.read(len as usize);
    let mut slice: [u8; TARGET_STRUCT_LEN] = [unsafe { MaybeUninit::uninit().assume_init() }; _];
    slice[..value.len()].copy_from_slice(value);
    black_box(&slice);
}

#[divan::bench(args = [get_model()], sample_size = SAMPLE_SIZE, sample_count = SAMPLE_COUNT)]
fn read_array_with_fast_collection_cursor(model: &Vec<u8>) {
    let mut cur = get_cur();
    #[allow(invalid_value)]
    let mut bytes: [u8; size_of::<LenHeader>()] =
        [unsafe { MaybeUninit::uninit().assume_init() }; _];
    bytes.copy_from_slice(&cur.filled()[..size_of::<LenHeader>()]);
    let len = LenHeader::from_be_bytes(bytes);
    let pos = cur.pos();
    let value = &cur.as_array()[pos..pos + len as usize];
    let mut slice: [u8; TARGET_STRUCT_LEN] = [unsafe { MaybeUninit::uninit().assume_init() }; _];
    slice[..value.len()].copy_from_slice(value);
    black_box(&slice);
}

fn get_model() -> Vec<u8> {
    let value = [*RAND_LEN as u8; 100];
    let mut vec = value.to_vec();
    unsafe { vec.set_len(*RAND_LEN as usize) };
    vec
}

fn get_buf() -> Buffer<BUF_LEN> {
    let mut buf = Buffer::<BUF_LEN>::new();
    buf.filled_pos = *RAND_LEN as LenUint + 100;
    buf.pos = *RAND_LEN as LenUint;
    buf
}

fn get_cur() -> Cursor<u8, BUF_LEN> {
    let mut cur = Cursor::<u8, BUF_LEN>::new();
    *unsafe { cur.filled_len_mut() } = *RAND_LEN as usize + 100;
    *unsafe { cur.pos_mut() } = *RAND_LEN as usize;
    cur
}

fn main() {
    divan::main()
}
