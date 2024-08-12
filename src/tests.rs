use super::*;

#[test]
fn test_write_and_read() {
    let mut buffer: Buffer<16> = Buffer::new();
    let data = b"hello";

    buffer.write(data);
    assert_eq!(buffer.remaining_space(), 11);

    let read_data = buffer.read(5);
    assert_eq!(read_data, data);
}

#[test]
fn test_try_write_success() {
    let mut buffer: Buffer<16> = Buffer::new();
    let data = b"hello";

    assert!(buffer.try_write(data).is_ok());
    assert_eq!(buffer.remaining_space(), 11);
}

#[test]
fn test_try_write_fail() {
    let mut buffer: Buffer<8> = Buffer::new();
    let data = b"too long data";

    assert!(buffer.try_write(data).is_err());
    assert_eq!(buffer.remaining_space(), 8);
}

#[test]
fn test_clear() {
    let mut buffer: Buffer<16> = Buffer::new();
    let data = b"hello";

    buffer.write(data);
    buffer.clear();
    assert_eq!(buffer.remaining_space(), 16);
    assert_eq!(buffer.remaining(), 0);
}

#[test]
fn test_advance() {
    let mut buffer: Buffer<16> = Buffer::new();
    let data = b"hello world";

    buffer.write(data);
    buffer.advance(6);
    assert_eq!(buffer.remaining(), 5);

    let remaining_data = buffer.read(5);
    assert_eq!(remaining_data, b"world");
}

#[test]
fn test_get_continuous() {
    let mut buffer: Buffer<16> = Buffer::new();
    let data = b"hello world";

    buffer.write(data);
    let continuous_data = buffer.get_continuous(5);
    assert_eq!(continuous_data, b"hello");
}

#[test]
fn test_debug() {
    let mut buffer: Buffer<16> = Buffer::new();
    let data = b"test";

    buffer.write(data);
    let debug_str = format!("{:?}", buffer);
    assert_eq!(debug_str, "[116, 101, 115, 116]");
}
