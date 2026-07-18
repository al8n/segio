pub use varing::{
  ConstDecodeError as ConstDecodeVarintError, ConstEncodeError as ConstEncodeVarintError,
  DecodeError as DecodeVarintError, EncodeError as EncodeVarintError, InsufficientData,
  InsufficientSpace,
};

use core::num::NonZeroUsize;

macro_rules! try_op_error {
  (
    #[doc = $doc:literal]
    #[error($msg:literal)]
    $op:ident
  ) => {
    paste::paste! {
      #[doc = $doc]
      #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
      #[error($msg)]
      pub struct [< Try $op:camel Error >] {
        requested: NonZeroUsize,
        available: usize,
      }

      impl [< Try $op:camel Error >] {
        #[doc = "Creates a new `Try" $op:camel "Error` with the requested and available bytes."]
        ///
        /// # Panics
        ///
        /// - If `requested <= available` (this would not be an error condition).
        /// - The `requested` value must be a non-zero.
        #[inline]
        pub const fn new(requested: NonZeroUsize, available: usize) -> Self {
          assert!(requested.get() > available, concat!(stringify!([< Try $op:camel Error >]), ": requested must be greater than available"));

          Self {
            requested,
            available,
          }
        }

        /// Returns the number of bytes requested for the operation.
        ///
        /// This is the minimum number of bytes needed for the operation to succeed.
        #[inline]
        pub const fn requested(&self) -> NonZeroUsize {
          self.requested
        }

        /// Returns the number of bytes available in the buffer.
        ///
        /// This is the actual number of bytes that were available when the operation failed.
        #[inline]
        pub const fn available(&self) -> usize {
          self.available
        }
      }
    }
  };
}

try_op_error!(
  #[doc = "An error that occurs when trying to advance the buffer cursor beyond available data.
  
This error indicates that an attempt was made to move the buffer's read position forward
by more bytes than are currently available. This is a recoverable error - the buffer
position remains unchanged and the operation can be retried with a smaller advance amount."]
  #[error(
    "not enough bytes available to advance (requested {requested} but only {available} available)"
  )]
  advance
);

#[cfg(feature = "std")]
impl From<TryAdvanceError> for std::io::Error {
  fn from(e: TryAdvanceError) -> Self {
    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e)
  }
}

try_op_error!(
  #[doc = "An error that occurs when trying to read data from a buffer with insufficient bytes.
  
This error indicates that a read operation failed because the buffer does not contain
enough bytes to satisfy the request. Failed read operations do not consume any bytes - the buffer position remains unchanged."]
  #[error(
    "not enough bytes available to read value (requested {requested} but only {available} available)"
  )]
  read
);

#[cfg(feature = "std")]
impl From<TryReadError> for std::io::Error {
  fn from(e: TryReadError) -> Self {
    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e)
  }
}

try_op_error!(
  #[doc = "An error that occurs when trying to peek at data beyond the buffer's available bytes.
  
This error indicates that a peek operation failed because the buffer does not contain
enough bytes at the current position. Peek operations never modify the buffer position,
so this error leaves the buffer in its original state."]
  #[error(
    "not enough bytes available to peek value (requested {requested} but only {available} available)"
  )]
  peek
);

#[cfg(feature = "std")]
impl From<TryPeekError> for std::io::Error {
  fn from(e: TryPeekError) -> Self {
    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e)
  }
}

impl From<TryPeekError> for TryReadError {
  #[inline]
  fn from(e: TryPeekError) -> Self {
    TryReadError {
      requested: e.requested,
      available: e.available,
    }
  }
}

/// An error that occurs when trying to create a segment with an invalid range.
///
/// This error indicates that the requested range extends beyond the buffer's boundaries
/// or is otherwise invalid (e.g., start > end). The original buffer remains unchanged.
///
/// # Examples
///
/// ```rust
/// # use buffo::Chunk;
/// let data = b"Hello";
/// let buf = &data[..];
///
/// // Range extends beyond buffer
/// match buf.try_segment(2..10) {
///     Err(e) => {
///         assert_eq!(e.start(), 2);
///         assert_eq!(e.end(), 10);
///         assert_eq!(e.available(), 5);
///     }
///     _ => panic!("Expected error"),
/// }
///
/// // Invalid range (start > end)
/// assert!(buf.try_segment(4..2).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid segment range {start}..{end} for buffer with {available} bytes")]
pub struct TrySegmentError {
  start: usize,
  end: usize,
  available: usize,
}

impl TrySegmentError {
  /// Creates a new `TrySegmentError`.
  ///
  /// # Panics
  ///
  /// In debug builds, panics if the range is valid (`start <= end && end <= available`),
  /// since that would not represent an error condition.
  #[inline]
  pub const fn new(start: usize, end: usize, available: usize) -> Self {
    debug_assert!(
      start > end || end > available,
      "TrySegmentError: invalid error - range is valid"
    );

    Self {
      start,
      end,
      available,
    }
  }

  /// Returns the start index of the requested range.
  #[inline]
  pub const fn start(&self) -> usize {
    self.start
  }

  /// Returns the end index of the requested range (exclusive).
  #[inline]
  pub const fn end(&self) -> usize {
    self.end
  }

  /// Returns the total number of bytes available in the buffer.
  #[inline]
  pub const fn available(&self) -> usize {
    self.available
  }

  /// Returns the length of the requested range.
  ///
  /// Returns 0 if start > end (invalid range).
  #[inline]
  pub const fn requested(&self) -> usize {
    self.end.saturating_sub(self.start)
  }

  /// Returns whether the range itself is invalid (start > end).
  #[inline]
  pub const fn is_inverted(&self) -> bool {
    self.start > self.end
  }

  /// Returns how many bytes the range extends beyond the buffer.
  ///
  /// Returns 0 if the range doesn't extend beyond the buffer
  /// (e.g., when the error is due to start > end).
  #[inline]
  pub const fn overflow(&self) -> usize {
    self.end.saturating_sub(self.available)
  }
}

#[cfg(feature = "std")]
impl From<TrySegmentError> for std::io::Error {
  fn from(e: TrySegmentError) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
  }
}

/// An error that occurs when an offset is beyond the buffer's boundaries.
///
/// This error is typically used for operations that access a specific position
/// in the buffer, such as writing at an offset or splitting at a position.
///
/// # Example
///
/// ```rust
/// # use buffo::ChunkMut;
/// let mut buf = [0u8; 10];
/// let mut writer = &mut buf[..];
///
/// // Trying to write at offset 15 in a 10-byte buffer
/// match writer.try_put_u32_le_at(42, 15) {
///     Err(e) => {
///         // Error contains OutOfBounds information
///     }
///     _ => panic!("Expected error"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("offset {offset} is out of bounds for buffer length {length}")]
pub struct OutOfBounds {
  offset: usize,
  length: usize,
}

impl OutOfBounds {
  /// Creates a new `OutOfBounds` error.
  ///
  /// # Panics
  ///
  /// In debug builds, panics if `offset < length` (would not be out of bounds).
  #[inline]
  pub const fn new(offset: usize, length: usize) -> Self {
    debug_assert!(offset >= length, "OutOfBounds: offset must be >= length");

    Self { offset, length }
  }

  /// Returns the offset that caused the error.
  #[inline]
  pub const fn offset(&self) -> usize {
    self.offset
  }

  /// Returns the actual length of the buffer.
  #[inline]
  pub const fn length(&self) -> usize {
    self.length
  }

  /// Returns how far beyond the buffer the offset extends.
  ///
  /// This is equivalent to `offset() - length() + 1`, saturating at
  /// `usize::MAX` instead of overflowing at the maximal boundary.
  #[inline]
  pub const fn excess(&self) -> usize {
    self.offset.saturating_sub(self.length).saturating_add(1)
  }
}

#[cfg(feature = "std")]
impl From<OutOfBounds> for std::io::Error {
  fn from(e: OutOfBounds) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
  }
}

/// An error indicating insufficient space at a specific offset in a buffer.
///
/// This error provides detailed information about space requirements when a write
/// operation fails due to insufficient remaining capacity from a given offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
  "not enough bytes available at {offset} (requested {} but only {} available)",
  self.info.requested(), self.info.available()
)]
pub struct InsufficientSpaceAt {
  info: InsufficientSpace,
  /// The offset at which the write was attempted.
  offset: usize,
}

impl InsufficientSpaceAt {
  /// Creates a new `InsufficientSpaceAt` error.
  ///
  /// # Panics
  ///
  /// - If `requested <= available` (would not be an error).
  /// - The `requested` value must be a non-zero.
  #[inline]
  pub const fn new(requested: NonZeroUsize, available: usize, offset: usize) -> Self {
    assert!(
      requested.get() > available,
      "InsufficientSpaceAt: requested must be greater than available"
    );

    Self {
      info: InsufficientSpace::new(requested, available),
      offset,
    }
  }

  /// Returns the number of bytes requested to write.
  #[inline]
  pub const fn requested(&self) -> NonZeroUsize {
    self.info.requested()
  }

  /// Returns the number of bytes available from the offset.
  #[inline]
  pub const fn available(&self) -> usize {
    self.info.available()
  }

  /// Returns the offset at which the write was attempted.
  #[inline]
  pub const fn offset(&self) -> usize {
    self.offset
  }
}

/// An error indicating insufficient data available at a specific offset in a buffer.
///
/// This error provides detailed information about data requirements when a read
/// operation fails due to insufficient remaining capacity from a given offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsufficientDataAt {
  info: InsufficientData,
  /// The offset at which the read was attempted.
  offset: usize,
}

impl core::fmt::Display for InsufficientDataAt {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self.requested() {
      Some(requested) => write!(
        f,
        "not enough bytes available at {}: available {}, requested {}",
        self.offset,
        self.info.available(),
        requested
      ),
      None => write!(
        f,
        "not enough bytes available at {}: available {}",
        self.offset,
        self.info.available()
      ),
    }
  }
}

impl core::error::Error for InsufficientDataAt {}

impl InsufficientDataAt {
  /// Creates a new `InsufficientDataAt` error.
  ///
  /// - `available`: the number of bytes available from the offset.
  /// - `offset`: the offset at which the read was attempted.
  #[inline]
  pub const fn new(available: usize, offset: usize) -> Self {
    Self {
      info: InsufficientData::new(available),
      offset,
    }
  }

  /// Creates a new `InsufficientDataAt` error with a specific requested size.
  ///
  /// # Panics
  ///
  /// - If `requested <= available` (would not be an error).
  #[inline]
  pub const fn with_requested(available: usize, offset: usize, requested: NonZeroUsize) -> Self {
    assert!(
      requested.get() > available,
      "InsufficientDataAt: requested must be greater than available"
    );

    Self {
      info: InsufficientData::with_required(requested, available),
      offset,
    }
  }

  /// Returns the number of bytes requested to read.
  #[inline]
  pub const fn requested(&self) -> Option<NonZeroUsize> {
    self.info.required()
  }

  /// Returns the number of bytes available from the offset.
  #[inline]
  pub const fn available(&self) -> usize {
    self.info.available()
  }

  /// Returns the offset at which the read was attempted.
  #[inline]
  pub const fn offset(&self) -> usize {
    self.offset
  }
}

#[cfg(feature = "std")]
impl From<InsufficientDataAt> for std::io::Error {
  fn from(e: InsufficientDataAt) -> Self {
    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e)
  }
}

/// An error that occurs when trying to peek at a specific offset in the buffer.
///
/// This error is returned when the offset is out of bounds or when there is insufficient data to peek the requested data.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TryPeekAtError {
  /// An error that occurs when trying to peek at an offset that is out of bounds for the buffer.
  #[error(transparent)]
  OutOfBounds(#[from] OutOfBounds),
  /// An error that occurs when there is not enough data in the buffer to peek the requested data.
  #[error(transparent)]
  InsufficientData(#[from] InsufficientDataAt),
}

impl TryPeekAtError {
  /// Creates a new `TryPeekAtError::OutOfBounds` error.
  #[inline]
  pub const fn out_of_bounds(offset: usize, length: usize) -> Self {
    Self::OutOfBounds(OutOfBounds::new(offset, length))
  }

  /// Creates a new `TryPeekAtError::InsufficientData` error.
  #[inline]
  pub const fn insufficient_data(available: usize, offset: usize) -> Self {
    Self::InsufficientData(InsufficientDataAt::new(available, offset))
  }

  /// Creates a new `TryPeekAtError::InsufficientData` error.
  ///
  /// # Panics
  ///
  /// Panics (in all build profiles) if `requested <= available`, since that would
  /// not represent an error condition.
  #[inline]
  pub const fn insufficient_data_with_requested(
    available: usize,
    offset: usize,
    requested: NonZeroUsize,
  ) -> Self {
    Self::InsufficientData(InsufficientDataAt::with_requested(
      available, offset, requested,
    ))
  }
}

#[cfg(feature = "std")]
impl From<TryPeekAtError> for std::io::Error {
  fn from(e: TryPeekAtError) -> Self {
    // Variant-explicit, mirroring the write-side `TryPutAtError`: an out-of-range offset
    // is `InvalidInput` (a caller mistake), while a valid offset with too few bytes is
    // `UnexpectedEof` (short data).
    match e {
      TryPeekAtError::OutOfBounds(e) => std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
      TryPeekAtError::InsufficientData(e) => {
        std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e)
      }
    }
  }
}

/// An error that occurs when trying to write at a specific offset in the buffer.
///
/// This error is returned when the offset is out of bounds or when there is insufficient space to write the requested data.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TryPutAtError {
  /// An error that occurs when trying to write at an offset that is out of bounds for the buffer.
  #[error(transparent)]
  OutOfBounds(#[from] OutOfBounds),
  /// An error that occurs when there is not enough space in the buffer to write the requested data.
  #[error(transparent)]
  InsufficientSpace(#[from] InsufficientSpaceAt),
}

impl TryPutAtError {
  /// Creates a new `TryPutAtError::OutOfBounds` error.
  #[inline]
  pub const fn out_of_bounds(offset: usize, length: usize) -> Self {
    Self::OutOfBounds(OutOfBounds::new(offset, length))
  }

  /// Creates a new `TryPutAtError::InsufficientSpace` error.
  ///
  /// # Panics
  ///
  /// Panics (in all build profiles) if `requested <= available`, since that would
  /// not represent an error condition.
  #[inline]
  pub const fn insufficient_space(
    requested: NonZeroUsize,
    available: usize,
    offset: usize,
  ) -> Self {
    Self::InsufficientSpace(InsufficientSpaceAt::new(requested, available, offset))
  }
}

#[cfg(feature = "std")]
impl From<TryPutAtError> for std::io::Error {
  fn from(e: TryPutAtError) -> Self {
    match e {
      TryPutAtError::OutOfBounds(e) => std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
      TryPutAtError::InsufficientSpace(e) => std::io::Error::new(std::io::ErrorKind::WriteZero, e),
    }
  }
}

/// An error that occurs when trying to put type in LEB128 format at a specific offset in the buffer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeVarintAtError {
  /// The offset is out of bounds for the buffer length.
  #[error(transparent)]
  OutOfBounds(#[from] OutOfBounds),
  /// The buffer does not have enough capacity to encode the value.
  #[error(transparent)]
  InsufficientSpace(#[from] InsufficientSpaceAt),
  /// A custom error message.
  #[error("{0}")]
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  Other(&'static str),

  /// A custom error message.
  #[error("{0}")]
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(std::borrow::Cow<'static, str>),
}

impl EncodeVarintAtError {
  /// Creates a new `EncodeVarintAtError::OutOfBounds` error.
  #[inline]
  pub const fn out_of_bounds(offset: usize, length: usize) -> Self {
    Self::OutOfBounds(OutOfBounds::new(offset, length))
  }

  /// Creates a new `EncodeVarintAtError::Insufficient` error.
  ///
  /// # Panics
  ///
  /// - If `requested <= available` (would not be an error).
  #[inline]
  pub const fn insufficient_space(
    requested: NonZeroUsize,
    available: usize,
    offset: usize,
  ) -> Self {
    Self::InsufficientSpace(InsufficientSpaceAt::new(requested, available, offset))
  }

  /// Creates a new `EncodeVarintAtError` error from `EncodeVarintError`.
  #[inline]
  pub fn from_varint_error(err: EncodeVarintError, offset: usize) -> Self {
    match err {
      EncodeVarintError::InsufficientSpace(e) => {
        Self::insufficient_space(e.requested(), e.available(), offset)
      }
      EncodeVarintError::Other(msg) => Self::Other(msg),
      _ => Self::other("unknown error"),
    }
  }

  /// Creates a new `EncodeVarintAtError` error from `ConstEncodeVarintError`.
  #[inline]
  pub const fn from_const_varint_error(err: ConstEncodeVarintError, offset: usize) -> Self {
    match err {
      ConstEncodeVarintError::InsufficientSpace(e) => {
        Self::insufficient_space(e.requested(), e.available(), offset)
      }
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      ConstEncodeVarintError::Other(msg) => Self::Other(msg),
      #[cfg(any(feature = "std", feature = "alloc"))]
      ConstEncodeVarintError::Other(msg) => Self::Other(std::borrow::Cow::Borrowed(msg)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => Self::other("unknown error"),
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(std::borrow::Cow::Borrowed("unknown error")),
    }
  }

  /// Creates a new `EncodeVarintAtError::Other` error.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }

  /// Creates a new `EncodeVarintAtError::Other` error.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline]
  pub fn other(msg: impl Into<std::borrow::Cow<'static, str>>) -> Self {
    Self::Other(msg.into())
  }
}

#[cfg(feature = "std")]
impl From<EncodeVarintAtError> for std::io::Error {
  fn from(e: EncodeVarintAtError) -> Self {
    match e {
      EncodeVarintAtError::OutOfBounds(e) => {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
      }
      EncodeVarintAtError::InsufficientSpace(e) => {
        std::io::Error::new(std::io::ErrorKind::WriteZero, e)
      }
      EncodeVarintAtError::Other(msg) => std::io::Error::other(msg),
    }
  }
}

/// An error that occurs when trying to put type in LEB128 format at a specific offset in the buffer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeVarintAtError {
  /// The offset is out of bounds for the buffer length.
  #[error(transparent)]
  OutOfBounds(#[from] OutOfBounds),
  /// The buffer does not have enough capacity to encode the value.
  #[error(transparent)]
  InsufficientData(#[from] InsufficientDataAt),
  /// The decoded value would overflow the target type.
  #[error("decoded value would overflow the target type")]
  Overflow,
  /// A custom error message.
  #[error("{0}")]
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  Other(&'static str),

  /// A custom error message.
  #[error("{0}")]
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(std::borrow::Cow<'static, str>),
}

impl DecodeVarintAtError {
  /// Creates a new `DecodeVarintAtError::OutOfBounds` error.
  #[inline]
  pub const fn out_of_bounds(offset: usize, length: usize) -> Self {
    Self::OutOfBounds(OutOfBounds::new(offset, length))
  }

  /// Creates a new `DecodeVarintAtError::Insufficient` error.
  #[inline]
  pub const fn insufficient_data(available: usize, offset: usize) -> Self {
    Self::InsufficientData(InsufficientDataAt::new(available, offset))
  }

  /// Creates a new `DecodeVarintAtError` error from `EncodeVarintError`.
  #[inline]
  pub fn from_varint_error(err: DecodeVarintError, offset: usize) -> Self {
    match err {
      DecodeVarintError::InsufficientData(e) => match e.required() {
        Some(requested) => Self::InsufficientData(InsufficientDataAt::with_requested(
          e.available(),
          offset,
          requested,
        )),
        None => Self::insufficient_data(e.available(), offset),
      },
      DecodeVarintError::Overflow => Self::Overflow,
      DecodeVarintError::Other(msg) => Self::other(msg),
      _ => Self::other("unknown error"),
    }
  }

  /// Creates a new `DecodeVarintAtError::Other` error from `ConstDecodeVarintError`.
  #[inline]
  pub const fn from_const_varint_error(err: ConstDecodeVarintError, offset: usize) -> Self {
    match err {
      ConstDecodeVarintError::InsufficientData(e) => match e.required() {
        Some(requested) => Self::InsufficientData(InsufficientDataAt::with_requested(
          e.available(),
          offset,
          requested,
        )),
        None => Self::insufficient_data(e.available(), offset),
      },
      ConstDecodeVarintError::Overflow => Self::Overflow,
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      ConstDecodeVarintError::Other(msg) => Self::Other(msg),
      #[cfg(any(feature = "std", feature = "alloc"))]
      ConstDecodeVarintError::Other(msg) => Self::Other(std::borrow::Cow::Borrowed(msg)),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => Self::other("unknown error"),
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::Other(std::borrow::Cow::Borrowed("unknown error")),
    }
  }

  /// Creates a new `DecodeVarintAtError::Other` error.
  #[cfg(not(any(feature = "std", feature = "alloc")))]
  #[inline]
  pub const fn other(msg: &'static str) -> Self {
    Self::Other(msg)
  }

  /// Creates a new `DecodeVarintAtError::Other` error.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[inline]
  pub fn other(msg: impl Into<std::borrow::Cow<'static, str>>) -> Self {
    Self::Other(msg.into())
  }
}

#[cfg(feature = "std")]
impl From<DecodeVarintAtError> for std::io::Error {
  fn from(e: DecodeVarintAtError) -> Self {
    match e {
      // Variant-explicit: an out-of-range offset is a caller mistake (`InvalidInput`), not
      // EOF, so it must not fall through the `_ => UnexpectedEof` wildcard below. This
      // mirrors the write-side `EncodeVarintAtError`.
      DecodeVarintAtError::OutOfBounds(e) => {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
      }
      DecodeVarintAtError::Overflow => std::io::Error::new(std::io::ErrorKind::InvalidData, e),
      DecodeVarintAtError::Other(msg) => std::io::Error::other(msg),
      _ => std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e),
    }
  }
}

// Normalizes a `(requested, available)` pair taken from `bytes::TryGetError` (and, via the
// same nominal type under smol-bytes' std/alloc tiers, `smol_bytes::TryGetError`) so that it
// always satisfies the `requested >= 1` and `requested > available` invariants of these error
// types. These source error types expose public `requested`/`available` fields, so any pair
// (including `requested == 0` or `requested <= available`) is constructible; degenerate pairs
// are normalized here rather than panicked on, while valid non-degenerate inputs are preserved
// verbatim.
#[cfg(any(
  feature = "bytes_1",
  all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))
))]
#[inline]
fn normalize(requested: usize, available: usize) -> (NonZeroUsize, usize) {
  match NonZeroUsize::new(requested) {
    Some(requested) if requested.get() > available => (requested, available),
    _ => {
      // A valid error cannot have `available == usize::MAX` (no larger `requested`
      // exists), so clamp it down by one in that single case to keep the invariant
      // satisfiable; then `available + 1` is non-zero and strictly greater.
      let available = available.min(usize::MAX - 1);
      let requested = NonZeroUsize::new(available + 1).unwrap_or(NonZeroUsize::MIN);
      (requested, available)
    }
  }
}

// `bytes::TryGetError` and `smol_bytes::TryGetError` are the SAME nominal type whenever
// smol-bytes is built with `std`/`alloc` (smol re-exports `bytes::TryGetError`), so a single
// `From<TryGetError>` impl must cover both. Writing a separate
// `From<smol_bytes_01::TryGetError>` would be a duplicate impl (E0119) when both features are
// enabled. The cfg-selected `use` below picks whichever name is in scope. In pure-core smol
// (no std/alloc) no `TryGetError` conversion exists at all — buffo's pure-core
// `Chunk for Buffer` impl keeps the trait read defaults and never produces one, and smol's
// pure-core `TryGetError` identity is not knowable here.
#[cfg(any(
  feature = "bytes_1",
  all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))
))]
const _: () = {
  #[cfg(feature = "bytes_1")]
  use bytes_1::TryGetError;
  #[cfg(all(not(feature = "bytes_1"), feature = "smol_bytes_01"))]
  use smol_bytes_01::TryGetError;

  impl From<TryGetError> for TryAdvanceError {
    fn from(e: TryGetError) -> Self {
      let (requested, available) = normalize(e.requested, e.available);
      TryAdvanceError::new(requested, available)
    }
  }

  impl From<TryGetError> for TryReadError {
    fn from(e: TryGetError) -> Self {
      let (requested, available) = normalize(e.requested, e.available);
      TryReadError::new(requested, available)
    }
  }
};

#[cfg(test)]
mod tests {
  use super::*;
  use core::num::NonZeroUsize;

  // Fix 2: `OutOfBounds::excess()` must be total (saturating) — no overflow/underflow
  // in debug or release, including the reachable `try_split_off(usize::MAX)` boundary.
  #[test]
  fn out_of_bounds_excess_is_total() {
    use crate::Chunk;

    let vals = [0usize, 1, usize::MAX];
    for &offset in &vals {
      for &length in &vals {
        // `OutOfBounds::new` requires `offset >= length`.
        if offset >= length {
          let e = OutOfBounds::new(offset, length);
          assert_eq!(e.excess(), offset.saturating_sub(length).saturating_add(1));
        }
      }
    }

    // Reachable in-crate: splitting an empty buffer at `usize::MAX` yields
    // `OutOfBounds { offset: usize::MAX, length: 0 }`; `excess()` must saturate.
    let mut buf = &b""[..];
    let err = buf.try_split_off(usize::MAX).unwrap_err();
    assert_eq!(err.offset(), usize::MAX);
    assert_eq!(err.length(), 0);
    assert_eq!(err.excess(), usize::MAX);
  }

  // Fix 3: varint `Overflow` must map to a dedicated `Overflow` variant, not `Other`.
  #[test]
  fn decode_varint_at_error_maps_overflow() {
    assert!(matches!(
      DecodeVarintAtError::from_varint_error(DecodeVarintError::Overflow, 4),
      DecodeVarintAtError::Overflow
    ));
    assert!(matches!(
      DecodeVarintAtError::from_const_varint_error(ConstDecodeVarintError::Overflow, 4),
      DecodeVarintAtError::Overflow
    ));
  }

  // Fix 6: a source carrying `required = Some(n)` must preserve it as `requested() == Some(n)`.
  #[test]
  fn decode_varint_at_error_preserves_required() {
    let n = NonZeroUsize::new(10).unwrap();

    // varing enforces `required > available` at construction (here 10 > 3).
    match DecodeVarintAtError::from_varint_error(
      DecodeVarintError::insufficient_data_with_required(n, 3),
      7,
    ) {
      DecodeVarintAtError::InsufficientData(inner) => {
        assert_eq!(inner.requested(), Some(n));
        assert_eq!(inner.available(), 3);
        assert_eq!(inner.offset(), 7);
      }
      other => panic!("expected InsufficientData, got {other:?}"),
    }

    match DecodeVarintAtError::from_const_varint_error(
      ConstDecodeVarintError::insufficient_data_with_required(n, 3),
      7,
    ) {
      DecodeVarintAtError::InsufficientData(inner) => {
        assert_eq!(inner.requested(), Some(n));
      }
      other => panic!("expected InsufficientData, got {other:?}"),
    }

    // Without a known `required`, `requested()` stays `None`.
    match DecodeVarintAtError::from_varint_error(DecodeVarintError::insufficient_data(2), 1) {
      DecodeVarintAtError::InsufficientData(inner) => {
        assert_eq!(inner.requested(), None);
        assert_eq!(inner.available(), 2);
      }
      other => panic!("expected InsufficientData, got {other:?}"),
    }
  }

  // Fixes 3 + 4: `io::ErrorKind` mapping is a function of the variant only.
  #[cfg(feature = "std")]
  #[test]
  fn io_error_kind_mapping() {
    use std::io::ErrorKind;

    // Fix 3: Overflow -> InvalidData (malformed data, not EOF).
    let err: std::io::Error =
      DecodeVarintAtError::from_varint_error(DecodeVarintError::Overflow, 0).into();
    assert_eq!(err.kind(), ErrorKind::InvalidData);

    let nz = NonZeroUsize::new(4).unwrap();

    // Fix 4: InsufficientSpace -> WriteZero unconditionally. The `offset >= available`
    // case (5 >= 1) previously mapped to InvalidInput; it must now stay WriteZero.
    let err: std::io::Error = TryPutAtError::insufficient_space(nz, 1, 0).into();
    assert_eq!(err.kind(), ErrorKind::WriteZero);
    let err: std::io::Error = TryPutAtError::insufficient_space(nz, 1, 5).into();
    assert_eq!(err.kind(), ErrorKind::WriteZero);
    let err: std::io::Error = EncodeVarintAtError::insufficient_space(nz, 1, 0).into();
    assert_eq!(err.kind(), ErrorKind::WriteZero);
    let err: std::io::Error = EncodeVarintAtError::insufficient_space(nz, 1, 5).into();
    assert_eq!(err.kind(), ErrorKind::WriteZero);

    // OutOfBounds -> InvalidInput on both write-side wrappers.
    let err: std::io::Error = TryPutAtError::out_of_bounds(9, 4).into();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
    let err: std::io::Error = EncodeVarintAtError::out_of_bounds(9, 4).into();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    // Read side (variant-explicit, mirroring the write side): OutOfBounds -> InvalidInput,
    // InsufficientData -> UnexpectedEof.
    let err: std::io::Error = TryPeekAtError::out_of_bounds(9, 4).into();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
    let err: std::io::Error = TryPeekAtError::insufficient_data(0, 4).into();
    assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    let err: std::io::Error = DecodeVarintAtError::out_of_bounds(9, 4).into();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
    let err: std::io::Error = DecodeVarintAtError::insufficient_data(0, 4).into();
    assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
  }

  // Fix 1: `From<bytes::TryGetError>` must be total — never panics for any field values,
  // and preserves fields verbatim on valid (requested >= 1 && requested > available) inputs.
  #[cfg(feature = "bytes_1")]
  #[test]
  fn try_get_error_conversions_are_total() {
    use bytes_1::TryGetError;

    let vals = [0usize, 1, 5, usize::MAX];
    for &requested in &vals {
      for &available in &vals {
        let advance: TryAdvanceError = TryGetError {
          requested,
          available,
        }
        .into();
        let read: TryReadError = TryGetError {
          requested,
          available,
        }
        .into();

        // Reaching here already proves totality (no panic); the invariant must hold.
        assert!(advance.requested().get() > advance.available());
        assert!(read.requested().get() > read.available());

        // Field preservation on valid, non-degenerate inputs.
        if requested >= 1 && requested > available {
          assert_eq!(advance.requested().get(), requested);
          assert_eq!(advance.available(), available);
          assert_eq!(read.requested().get(), requested);
          assert_eq!(read.available(), available);
        }
      }
    }
  }
}
