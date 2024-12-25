use core::{fmt::Debug, marker::PhantomData, ptr::slice_from_raw_parts};
use std::{alloc::Allocator, ptr::slice_from_raw_parts_mut};

use crate::{const_min, declare_impl, Buf, Chunk, ReadBuf, ReadToBuf, WriteBuf, WriteBufferError};

#[cfg(feature = "std")]
type ALLOC = std::alloc::Global;
#[cfg(not(feature = "std"))]
type ALLOC = crate::EmptyAlloc;

#[cfg(feature = "std")]
pub type BoxedBuffer<T, const N: usize, A = ALLOC> = Buffer<T, N, A, Box<[u8; N]>>;

pub type ByteBuffer<const N: usize, A = ALLOC> = Buffer<u8, N, A>;

#[cfg(feature = "std")]
pub type BoxedByteBuffer<const N: usize, A = ALLOC> = BoxedBuffer<u8, N, A>;

pub struct Buffer<T: Copy, const N: usize, A: Allocator = ALLOC, C: Chunk<T, N, A> = [T; N]> {
    chunk: C,
    filled_pos: LenUint,
    pos: LenUint,
    _marker: PhantomData<(A, T)>,
}

impl<T: Copy + Clone, const N: usize, A: Allocator, C: Chunk<T, N, A>> Copy for Buffer<T, N, A, C> {}

#[cfg(target_pointer_width = "64")]
type LenUint = u32;
#[cfg(target_pointer_width = "32")]
type LenUint = u16;

declare_impl! {
    (impl<T: Copy, const N: usize, C: Chunk<T, N, ALLOC>> Buffer<T, N, ALLOC, C>),
    (impl<T: Copy, const N: usize, C: const Chunk<T, N, ALLOC>> Buffer<T, N, ALLOC, C>) {
        #[inline(always)]
        pub const fn new() -> Self {
            Self {
                chunk: C::new_uninit(),
                filled_pos: 0,
                pos: 0,
                _marker: PhantomData,
            }
        }
    }
}

declare_impl! {
    (impl<T: Copy + Clone, A: Allocator, const N: usize, C: Chunk<T, N, A>> Buffer<T, N, A, C>),
    (impl<T: Copy + Clone, A: Allocator, const N: usize, C: const Chunk<T, N, A>> Buffer<T, N, A, C>) {
        #[inline(always)]
        pub const fn new_in(alloc: A) -> Self {
            Self {
                chunk: C::new_uninit_in(alloc),
                filled_pos: 0,
                pos: 0,
                _marker: PhantomData,
            }
        }
    }
}

impl<T: Copy + Clone, const N: usize, A: Allocator, C: Chunk<T, N, A>> Clone
    for Buffer<T, N, A, C>
{
    fn clone(&self) -> Self {
        Self {
            chunk: self.chunk.clone(),
            filled_pos: self.filled_pos.clone(),
            pos: self.pos.clone(),
            _marker: self._marker.clone(),
        }
    }
}

declare_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator, C: Chunk<T, N, A>> Buf<T> for Buffer<T, N, A, C>),
    (impl<T: Copy + Clone, const N: usize, A: Allocator, C: const Chunk<T, N, A>> const Buf<T> for Buffer<T, N, A, C>) {
        #[inline(always)]
        fn clear(&mut self) {
            self.filled_pos = 0;
            self.pos = 0;
        }

        #[inline(always)]
        fn as_ptr(&self) -> *const T {
            self.chunk.as_ptr()
        }

        #[inline(always)]
        fn as_mut_ptr(&mut self) -> *mut T {
            self.chunk.as_mut_ptr()
        }

        #[inline(always)]
        fn capacity(&self) -> usize {
            N
        }
    }
}

declare_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator, C: Chunk<T, N, A>> WriteBuf<T> for Buffer<T, N, A, C>),
    (impl<T: Copy + Clone, const N: usize, A: Allocator, C: const Chunk<T, N, A>> const WriteBuf<T> for Buffer<T, N, A, C>) {
        #[inline(always)]
        fn try_write(&mut self, data: &[T]) -> Result<(), WriteBufferError> {
            let filled_pos = self.filled_pos as usize;
            let new_filled_pos = filled_pos + data.len();
            if new_filled_pos <= N {
                self.filled_pos = new_filled_pos as LenUint;
                #[cfg(not(feature = "const-trait"))]
                unsafe {
                    self.chunk
                        .as_mut_slice()
                        .get_unchecked_mut(filled_pos..new_filled_pos)
                        .copy_from_slice(data);
                }

                #[cfg(feature = "const-trait")]
                unsafe {
                    (&mut *slice_from_raw_parts_mut(self.chunk.as_mut_ptr().wrapping_add(filled_pos),data.len())).copy_from_slice(data);
                }
                Ok(())
            } else {
                Err(WriteBufferError::BufferFull)
            }
        }

        #[inline(always)]
        fn write(&mut self, data: &[T]) {
            let filled_pos = self.filled_pos as usize;
            let new_filled_pos_len = filled_pos + data.len();
            self.filled_pos = new_filled_pos_len as LenUint;
            #[cfg(not(feature = "const-trait"))]
            self.chunk.as_mut_slice()[filled_pos..new_filled_pos_len].copy_from_slice(data);
            #[cfg(feature = "const-trait")]
            unsafe {

                (&mut *slice_from_raw_parts_mut(self.chunk.as_mut_ptr().wrapping_add(filled_pos), data.len())).copy_from_slice(data);
            }
        }

        #[inline(always)]
        fn remaining_space(&self) -> usize {
            N - self.filled_pos as usize
        }

        #[inline(always)]
        fn filled_pos(&self) -> usize {
            self.filled_pos as usize
        }

        #[inline(always)]
        unsafe fn set_filled_pos(&mut self, filled_pos: usize) {
            self.filled_pos = filled_pos as LenUint;
        }
    }
}

declare_impl! {
    (impl<T: Copy + Clone, const N: usize, A: Allocator, C: Chunk<T, N, A>> ReadBuf<T> for Buffer<T, N, A, C>),
    (impl<T: Copy + Clone, const N: usize, A: Allocator, C: const Chunk<T, N, A>> const ReadBuf<T> for Buffer<T, N, A, C>) {
        #[inline(always)]
        fn read(&mut self, len: usize) -> &[T] {
            let pos = self.pos as usize;
            let slice_len = const_min!(len, self.filled_pos as usize - pos);
            let new_pos = pos + slice_len;
            self.pos = new_pos as LenUint;
            unsafe { &*slice_from_raw_parts(self.chunk.as_ptr().wrapping_add(pos), slice_len) }
        }

        #[inline(always)]
        unsafe fn get_continuous(&self, len: usize) -> &[T] {
            let pos = self.pos as usize;
            let filled_pos = self.filled_pos as usize;
            let slice_len = const_min!(len, filled_pos - pos);
            unsafe { &*slice_from_raw_parts(self.chunk.as_ptr().wrapping_add(pos), slice_len) }
        }

        #[inline(always)]
        fn remaining(&self) -> usize {
            (self.filled_pos - self.pos) as usize
        }

        #[inline(always)]
        fn advance(&mut self, len: usize) {
            let pos = self.pos as usize;
            if cfg!(feature = "const-trait") {
                let filled_pos = self.filled_pos;
                let new_pos = (pos + len) as LenUint;
                self.pos = if filled_pos > new_pos {
                    new_pos
                } else {
                    filled_pos
                };
            } else {
                self.pos = const_min!(self.filled_pos, (pos + len) as LenUint);
            }
        }

        #[inline(always)]
        fn pos(&self) -> usize {
            self.pos as usize
        }

        #[inline(always)]
        unsafe fn set_pos(&mut self, pos: usize) {
            self.pos = pos as LenUint;
        }
    }
}

#[cfg(feature = "std")]
impl<S: std::io::Read> ReadToBuf<u8> for S {
    #[inline(always)]
    fn read_to_buf(&mut self, buf: &mut impl Buf<u8>) -> Result<(), ()> {
        let filled_pos = buf.filled_pos() as usize;
        let slice = unsafe {
            &mut *slice_from_raw_parts_mut(
                buf.as_mut_ptr().wrapping_add(filled_pos),
                buf.capacity() - filled_pos,
            )
        };
        let read_length = self.read(slice).map_err(|_| ())?;
        if read_length == 0 {
            Err(())?
        }
        unsafe { buf.set_filled_pos(filled_pos + read_length) };
        Ok(())
    }
}

#[cfg(feature = "std")]
declare_impl! {
    (impl<const N: usize, A: Allocator, C: Chunk<u8, N, A>> std::io::Write for Buffer<u8, N, A, C>),
    (impl<const N: usize, A: Allocator, C: const Chunk<u8, N, A>> std::io::Write for Buffer<u8, N, A, C>) {
        #[inline(always)]
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let backup_filled_pos = self.filled_pos();
            self.try_write(buf)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "write buffer failed"))?;
            Ok(self.filled_pos() - backup_filled_pos)
        }

        #[inline(always)]
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}

declare_impl! {
    (impl<T: Debug + Copy, const N: usize, A: Allocator, C: Chunk<T, N, A>> Debug for Buffer<T, N, A, C>),
    (impl<T: Debug + Copy, const N: usize, A: Allocator, C: const Chunk<T, N, A>> Debug for Buffer<T, N, A, C>) {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.chunk.as_slice()[self.pos()..self.filled_pos()].fmt(f)
        }

    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use test::{black_box, Bencher};

    use super::*;

    #[test]
    fn test_debug() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
                let data = b"test";

                buffer.write(data);
                let debug_str = format!("{:?}", buffer);
                assert_eq!(debug_str, "[116, 101, 115, 116]");
            }
        }
        test!(ByteBuffer::<16>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<16>::new());
    }

    #[test]
    fn test_write_and_read() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello";

            buffer.write(data);
            assert_eq!(buffer.remaining_space(), 11);

            let read_data = buffer.read(5);
            assert_eq!(read_data, data);
            }
        }
        test!(ByteBuffer::<16>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<16>::new());
    }

    #[test]
    fn test_try_write_success() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello";

            assert!(buffer.try_write(data).is_ok());
            assert_eq!(buffer.remaining_space(), 11);
        }
        }

        test!(ByteBuffer::<16>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<16>::new());
    }

    #[test]
    fn test_try_write_fail() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"too long data";

            assert!(buffer.try_write(data).is_err());
            assert_eq!(buffer.remaining_space(), 8);
            buffer.try_write(&[]).unwrap();
        }}

        test!(ByteBuffer::<8>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<8>::new());
    }

    #[test]
    fn test_clear() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello";

            buffer.write(data);
            buffer.clear();
            assert_eq!(buffer.remaining_space(), 16);
            assert_eq!(buffer.remaining(), 0);
        }}

        test!(ByteBuffer::<16>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<16>::new());
    }

    #[test]
    fn test_advance() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello world";

            buffer.write(data);
            buffer.advance(6);
            assert_eq!(buffer.remaining(), 5);

            let remaining_data = buffer.read(5);
            assert_eq!(remaining_data, b"world");
        }}

        test!(ByteBuffer::<16>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<16>::new());
    }

    #[test]
    fn test_get_continuous() {
        macro_rules! test {
            ($($buf:tt)*) => {
                let mut buffer = $($buf)*;
            let data = b"hello world";

            buffer.write(data);
            let continuous_data = unsafe { buffer.get_continuous(5) };
            assert_eq!(continuous_data, b"hello");
        }}

        test!(ByteBuffer::<16>::new());
        #[cfg(all(not(feature = "const-trait"), feature = "std"))]
        test!(BoxedByteBuffer::<16>::new());
    }

    const N: usize = 1000;

    #[bench]
    fn bench_buffer_try_write(b: &mut Bencher) {
        let ref mut buffer: ByteBuffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        black_box(&src);
        b.iter(|| {
            unsafe { buffer.set_filled_pos(0) };
            let _ = black_box(&buffer.try_write(&src));
        });
        black_box(&buffer);
    }

    #[bench]
    fn bench_buffer_write(b: &mut Bencher) {
        let ref mut buffer: ByteBuffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        black_box(&src);
        b.iter(|| {
            unsafe { buffer.set_filled_pos(0) };
            let _ = black_box(&buffer.write(&src));
        });
        black_box(&buffer);
    }

    #[bench]
    fn bench_buffer_read(b: &mut Bencher) {
        let ref mut buffer: ByteBuffer<N> = Buffer::new();
        let src: &[u8] = &vec![0; N];
        buffer.write(src);
        b.iter(|| {
            unsafe { buffer.set_pos(0) };
            let _ = black_box(&buffer.read(N));
        });
        black_box(&buffer);
    }
}
