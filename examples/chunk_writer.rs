use buffo::{Chunk, ChunkMut, ChunkWriter};

#[derive(Debug)]
pub enum Error {}

pub trait Encode {
  fn encoded_len(&self) -> usize;

  fn encode<B: ChunkMut>(&self, buf: impl Into<ChunkWriter<B>>) -> Result<usize, Error>;
}

pub trait Decode {
  fn decode<B: Chunk>(buf: B) -> Result<(usize, Self), Error>
  where
    Self: Sized;
}

#[derive(Debug)]
pub struct Foo {
  pub value: u32,
  pub other: Vec<u8>,
}

impl Encode for Foo {
  fn encoded_len(&self) -> usize {
    4 + 4 + self.other.len() // 4 bytes for u32 + 4 bytes for length + length of the Vec<u8>
  }

  fn encode<B: ChunkMut>(&self, buf: impl Into<ChunkWriter<B>>) -> Result<usize, Error> {
    let mut buf = buf.into();
    let mut offset = buf.write_u32_le(self.value);
    offset += buf.write_u32_le(self.other.len() as u32);
    offset += buf.write_slice(self.other.as_ref());
    Ok(offset)
  }
}

impl Decode for Foo {
  fn decode<B: Chunk>(mut buf: B) -> Result<(usize, Self), Error> {
    let value = buf.read_u32_le();
    let other_len = buf.read_u32_le() as usize;
    let other = buf.prefix(other_len).to_vec();
    buf.advance(other_len);
    Ok((8 + other_len, Foo { value, other }))
  }
}

#[derive(Debug)]
pub struct Bar {
  id: u64,
  foo: Foo,
  n: u32,
}

impl Encode for Bar {
  fn encoded_len(&self) -> usize {
    8 + self.foo.encoded_len() + 4 // 8 bytes for u64 + encoded length of Foo + 4 bytes for u32
  }

  fn encode<B: ChunkMut>(&self, buf: impl Into<ChunkWriter<B>>) -> Result<usize, Error> {
    let mut buf = buf.into();
    let mut offset = buf.write_u64_le(self.id);
    offset += self.foo.encode::<&mut B>(buf.as_mut())?;
    offset += buf.write_u32_le(self.n);
    Ok(offset)
  }
}

impl Decode for Bar {
  fn decode<B: Chunk>(mut buf: B) -> Result<(usize, Self), Error> {
    let id = buf.read_u64_le();
    let (foo_len, decoded) = Foo::decode(buf.buffer())?;
    buf.advance(foo_len);
    let n = buf.read_u32_le();
    Ok((
      12 + foo_len,
      Bar {
        id,
        foo: decoded,
        n,
      },
    ))
  }
}

fn main() {
  let mut buffer = [0u8; 100]; // Example buffer
  let val = Foo {
    value: 42,
    other: b"Hello".to_vec(),
  };

  let encoded = match val.encode(&mut buffer[..]) {
    Ok(len) => len,
    Err(_) => panic!("Failed to encode Foo"),
  };

  let (_, decoded) = Foo::decode(&buffer[..encoded]).unwrap();
  assert_eq!(decoded.value, 42);
  assert_eq!(decoded.other, b"Hello");

  let bar = Bar {
    id: 123456789,
    foo: decoded,
    n: 7,
  };

  let encoded_bar = match bar.encode(&mut buffer[..]) {
    Ok(len) => len,
    Err(_) => panic!("Failed to encode Bar"),
  };
  let (_, decoded_bar) = Bar::decode(&buffer[..encoded_bar]).unwrap();
  assert_eq!(decoded_bar.id, 123456789);
  assert_eq!(decoded_bar.foo.value, 42);
  assert_eq!(decoded_bar.foo.other, b"Hello".to_vec());
  assert_eq!(decoded_bar.n, 7);

  let mut vec = vec![0u8; 100]; // Example vector buffer
  let encoded_bar = match bar.encode(vec.as_mut_slice()) {
    Ok(len) => len,
    Err(_) => panic!("Failed to encode Bar"),
  };

  let (_, decoded_bar) = Bar::decode(&vec[..encoded_bar]).unwrap();
  assert_eq!(decoded_bar.id, 123456789);
  assert_eq!(decoded_bar.foo.value, 42);
  assert_eq!(decoded_bar.foo.other, b"Hello".to_vec());
  assert_eq!(decoded_bar.n, 7);
}
