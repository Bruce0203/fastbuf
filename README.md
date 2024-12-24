# FastBuf
[![Documentation](https://docs.rs/fastbuf/badge.svg)](https://docs.rs/fastbuf)
[![crates.io](https://img.shields.io/crates/v/fastbuf.svg)](https://crates.io/crates/fastbuf)

```rust 
use fastbuf::{Buffer, WriteBuf, ReadBuf};
let mut buffer: Buffer<100> = Buffer::new();
buffer.write(&[0; 100]);
let read: &[u8] = buffer.read(100);
```
