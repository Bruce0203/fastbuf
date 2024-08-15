```
benchmark                                    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ read_array_with_fast_collection_cursor    25.35 ns      │ 209.1 ns      │ 25.47 ns      │ 27.95 ns      │ 1000    │ 1000000
├─ read_array_with_fastbuf_buffer            15.68 ns      │ 54.3 ns       │ 18.49 ns      │ 19.83 ns      │ 1000    │ 1000000
├─ write_array_with_fast_collections_cursor  11.51 ns      │ 45.14 ns      │ 14.35 ns      │ 14.57 ns      │ 1000    │ 1000000
╰─ write_array_with_fastbuf_buffer           9.391 ns      │ 31.3 ns       │ 11.1 ns       │ 10.69 ns      │ 1000    │ 1000000

```
