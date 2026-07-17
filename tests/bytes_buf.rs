#![allow(unused_imports)]

#[cfg(test)]
#[cfg(all(feature = "bytes_1", feature = "std"))]
mod bytes_buf_tests {
  use buffo::{Chunk, ChunkExt};
  use bytes_1::Bytes;

  #[test]
  fn test_remaining() {
    let data = vec![1, 2, 3, 4, 5];
    let buf = Bytes::from(data);
    assert_eq!(buf.remaining(), 5);
    assert!(buf.has_remaining());

    let empty = Bytes::new();
    assert_eq!(empty.remaining(), 0);
    assert!(!empty.has_remaining());
  }

  #[test]
  fn test_buffer() {
    let data = vec![1, 2, 3, 4, 5];
    let buf = Bytes::from(data);
    assert_eq!(buf.buffer(), &[1, 2, 3, 4, 5]);
  }

  #[test]
  fn test_advance() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    buf.advance(2);
    assert_eq!(buf.remaining(), 3);
    assert_eq!(buf.buffer(), &[3, 4, 5]);

    buf.advance(3);
    assert_eq!(buf.remaining(), 0);
    assert!(!buf.has_remaining());
  }

  #[test]
  #[should_panic]
  fn test_advance_panic() {
    let data = vec![1, 2, 3];
    let mut buf = Bytes::from(data);
    buf.advance(5); // Should panic
  }

  #[test]
  fn test_try_advance() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    assert!(buf.try_advance(3).is_ok());
    assert_eq!(buf.remaining(), 2);

    let err = buf.try_advance(5).unwrap_err();
    assert_eq!(err.requested().get(), 5);
    assert_eq!(err.available(), 2);
  }

  #[test]
  fn test_truncate() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    buf.truncate(3);
    assert_eq!(buf.remaining(), 3);
    assert_eq!(buf.buffer(), &[1, 2, 3]);

    // Truncating to larger size has no effect
    buf.truncate(10);
    assert_eq!(buf.remaining(), 3);
  }

  #[test]
  fn test_segment() {
    let data = b"Hello, World!".to_vec();
    let buf = Bytes::from(data);

    let hello = buf.segment(0..5);
    assert_eq!(hello.buffer(), b"Hello");

    let world = buf.segment(7..12);
    assert_eq!(world.buffer(), b"World");

    // Original unchanged
    assert_eq!(buf.remaining(), 13);
  }

  #[allow(clippy::reversed_empty_ranges)]
  #[test]
  fn test_try_segment() {
    let data = vec![1, 2, 3, 4, 5];
    let buf = Bytes::from(data);

    assert!(buf.try_segment(1..4).is_ok());
    assert!(buf.try_segment(0..10).is_err());
    assert!(buf.try_segment(3..2).is_err()); // Invalid range
  }

  #[test]
  fn test_split_off() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    let tail = buf.split_off(2);
    assert_eq!(buf.buffer(), &[1, 2]);
    assert_eq!(tail.buffer(), &[3, 4, 5]);
  }

  #[test]
  fn test_split_to() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    let head = buf.split_to(2);
    assert_eq!(head.buffer(), &[1, 2]);
    assert_eq!(buf.buffer(), &[3, 4, 5]);
  }

  #[test]
  fn test_peek_primitives() {
    let data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let buf = Bytes::from(data);

    // Peek doesn't advance
    assert_eq!(buf.peek_u8(), 0x12);
    assert_eq!(buf.remaining(), 8);

    assert_eq!(buf.peek_u16_le(), 0x3412);
    assert_eq!(buf.peek_u16_be(), 0x1234);

    assert_eq!(buf.peek_u32_le(), 0x78563412);
    assert_eq!(buf.peek_u32_be(), 0x12345678);

    // Chunk unchanged
    assert_eq!(buf.remaining(), 8);
  }

  #[test]
  fn test_read_primitives() {
    let data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let mut buf = Bytes::from(data);

    assert_eq!(buf.read_u8(), 0x12);
    assert_eq!(buf.remaining(), 7);

    assert_eq!(buf.read_u16_le(), 0x5634);
    assert_eq!(buf.remaining(), 5);

    assert_eq!(buf.read_u32_be(), 0x789ABCDE);
    assert_eq!(buf.remaining(), 1);

    assert_eq!(buf.read_u8(), 0xF0);
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_peek_checked() {
    let data = vec![0x12];
    let buf = Bytes::from(data);

    assert_eq!(buf.peek_u8_checked(), Some(0x12));
    assert_eq!(buf.peek_u16_le_checked(), None);
    assert_eq!(buf.peek_u32_be_checked(), None);
  }

  #[test]
  fn test_read_checked() {
    let data = vec![0x12, 0x34];
    let mut buf = Bytes::from(data);

    assert_eq!(buf.read_u8_checked(), Some(0x12));
    assert_eq!(buf.read_u16_le_checked(), None); // Only 1 byte left
    assert_eq!(buf.remaining(), 1); // Read failed, cursor unchanged

    assert_eq!(buf.read_u8_checked(), Some(0x34));
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_try_methods() {
    let data = vec![0x12, 0x34];
    let mut buf = Bytes::from(data);

    assert!(buf.try_peek_u16_le().is_ok());

    let err = buf.try_peek_u32_le().unwrap_err();
    assert_eq!(err.requested().get(), 4);
    assert_eq!(err.available(), 2);

    assert!(buf.try_read_u16_be().is_ok());
    assert_eq!(buf.remaining(), 0);

    let err = buf.try_read_u8().unwrap_err();
    assert_eq!(err.requested().get(), 1);
    assert_eq!(err.available(), 0);
  }

  #[test]
  fn test_arrays() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let mut buf = Bytes::from(data);

    let arr: [u8; 3] = buf.peek_array();
    assert_eq!(arr, [1, 2, 3]);
    assert_eq!(buf.remaining(), 8);

    let arr: [u8; 3] = buf.read_array();
    assert_eq!(arr, [1, 2, 3]);
    assert_eq!(buf.remaining(), 5);

    assert!(buf.peek_array_checked::<10>().is_none());
    assert!(buf.read_array_checked::<10>().is_none());
  }

  #[test]
  fn test_floating_point() {
    let data = vec![
      0x00, 0x00, 0x80, 0x3F, // 1.0f32 in little-endian
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // 1.0f64 in little-endian
    ];
    let mut buf = Bytes::from(data);

    assert_eq!(buf.read_f32_le(), 1.0);
    assert_eq!(buf.read_f64_le(), 1.0);
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_to_vec() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    buf.advance(2);
    let vec = buf.to_vec();
    assert_eq!(vec, vec![3, 4, 5]);
  }

  #[test]
  fn test_to_bytes() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data.clone());

    buf.advance(1);
    let bytes = buf.to_bytes();
    assert_eq!(bytes.as_ref(), &[2, 3, 4, 5]);

    // Original unchanged
    assert_eq!(buf.buffer(), &[2, 3, 4, 5]);
  }

  #[test]
  fn test_to_bytes_mut() {
    let data = vec![1, 2, 3, 4, 5];
    let mut buf = Bytes::from(data);

    buf.advance(1);
    let mut bytes_mut = buf.to_bytes_mut();
    bytes_mut[0] = 10;
    assert_eq!(bytes_mut.as_ref(), &[10, 3, 4, 5]);

    // Original unchanged
    assert_eq!(buf.buffer(), &[2, 3, 4, 5]);
  }

  #[test]
  fn test_bytes_specific_features() {
    let data = vec![1, 2, 3, 4, 5];
    let buf = Bytes::from(data);

    // Test zero-copy cloning
    let clone1 = buf.clone();
    let clone2 = buf.clone();

    assert_eq!(clone1.buffer(), clone2.buffer());
    assert_eq!(clone1.buffer(), buf.buffer());

    // Test slicing maintains reference counting
    let slice = buf.segment(1..4);
    assert_eq!(slice.buffer(), &[2, 3, 4]);
  }

  #[test]
  fn test_varint() {
    use varing::Varint as _;

    let mut data = vec![0; 10];

    // Encode a varint
    let encoded_len = 42u32.encode(&mut data).unwrap();
    let buf = Bytes::from(data);

    // Peek varint
    let (len, value) = buf.peek_varint::<u32>().unwrap();
    assert_eq!(value, 42);
    assert_eq!(len, encoded_len);
    assert_eq!(buf.remaining(), 10);

    // Read varint
    let mut buf = buf;
    let (len, value) = buf.read_varint::<u32>().unwrap();
    assert_eq!(value, 42);
    assert_eq!(len, encoded_len);
    assert_eq!(buf.remaining(), 10 - encoded_len.get());
  }

  #[test]
  fn test_segment_edge() {
    let empty = Bytes::new();
    let seg = empty.segment(..);
    assert_eq!(seg.remaining(), 0);

    let empty: &[u8] = &[];
    let seg = empty.segment(..);
    assert_eq!(seg.remaining(), 0);
  }

  #[test]
  fn test_try_segment_edge() {
    let empty = Bytes::new();
    let seg = empty.try_segment(..).unwrap();
    assert_eq!(seg.remaining(), 0);

    let empty: &[u8] = &[];
    let seg = empty.try_segment(..).unwrap();
    assert_eq!(seg.remaining(), 0);
  }

  #[test]
  #[should_panic]
  fn test_advance_panic_empty() {
    let empty = Bytes::new();
    let mut buf = empty;
    buf.advance(1); // Should panic
  }
}
