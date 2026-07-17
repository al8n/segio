use crate::must_non_zero;

use super::error::{
  OutOfBounds, TryAdvanceError, TryPeekAtError, TryPeekError, TryReadError, TrySegmentError,
};

use core::num::NonZeroUsize;
use core::ops::{Bound, RangeBounds};

use varing::Varint;

use super::error::{DecodeVarintAtError, DecodeVarintError};

use super::panic_advance;

pub use peeker::Peeker;
pub use ref_peeker::RefPeeker;

mod peeker;
mod ref_peeker;

macro_rules! peek_fixed {
  ($($ty:ident), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Peeks a `" $ty "` value from the buffer in little-endian byte order without advancing the cursor."]
        ///
        /// Returns the decoded value. The buffer position remains unchanged after this operation.
        ///
        /// # Panics
        ///
        #[doc = "Panics if the buffer has fewer than `size_of::<" $ty ">()` bytes available."]
        /// Use the `*_checked` or `try_*` variants for non-panicking peeks.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert_eq!(buf.peek_" $ty "_le(), <" $ty ">::from_le_bytes((&data[..size_of::<" $ty ">()]).try_into().unwrap()));"]
        /// assert_eq!(buf.remaining(), data.len()); // Cursor unchanged
        /// ```
        #[inline]
        fn [<peek_ $ty _le>](&self) -> $ty {
          <$ty>::from_le_bytes(peek_array::<_, { core::mem::size_of::<$ty>() }>(self))
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in little-endian byte order without advancing the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_le`](Chunk::peek_" $ty "_le)."]
        /// Returns `Some(value)` if sufficient data is available, otherwise returns `None`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.peek_" $ty "_le_checked().is_some());"]
        ///
        /// let small_buf = &[0x34][..];
        #[doc = "assert!(small_buf.peek_" $ty "_le_checked().is_none()); // Not enough bytes"]
        /// ```
        #[inline]
        fn [<peek_ $ty _le_checked>](&self) -> Option<$ty> {
          peek_array_checked::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_le_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in little-endian byte order without advancing the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_le`](Chunk::peek_" $ty "_le)."]
        /// Returns `Ok(value)` on success, or `Err(TryPeekError)` with details about
        /// requested vs available bytes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.try_peek_" $ty "_le().is_ok());"]
        ///
        /// let small_buf = &[0x34][..];
        #[doc = "let err = small_buf.try_peek_" $ty "_le().unwrap_err();"]
        /// // err contains details about requested vs available bytes
        /// ```
        #[inline]
        fn [<try_peek_ $ty _le>](&self) -> Result<$ty, TryPeekError> {
          try_peek_array::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_le_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in big-endian byte order without advancing the cursor."]
        ///
        /// Returns the decoded value. The buffer position remains unchanged after this operation.
        ///
        /// # Panics
        ///
        #[doc = "Panics if the buffer has fewer than `size_of::<" $ty ">()` bytes available."]
        /// Use the `*_checked` or `try_*` variants for non-panicking peeks.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert_eq!(buf.peek_" $ty "_be(), <" $ty ">::from_be_bytes((&data[..size_of::<" $ty ">()]).try_into().unwrap()));"]
        /// assert_eq!(buf.remaining(), data.len()); // Cursor unchanged
        /// ```
        #[inline]
        fn [<peek_ $ty _be>](&self) -> $ty {
          <$ty>::from_be_bytes(peek_array::<_, { core::mem::size_of::<$ty>() }>(self))
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in big-endian byte order without advancing the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_be`](Chunk::peek_" $ty "_be)."]
        /// Returns `Some(value)` if sufficient data is available, otherwise returns `None`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.peek_" $ty "_be_checked().is_some());"]
        ///
        /// let small_buf = &[0x12][..];
        #[doc = "assert!(small_buf.peek_" $ty "_be_checked().is_none()); // Not enough bytes"]
        /// ```
        #[inline]
        fn [<peek_ $ty _be_checked>](&self) -> Option<$ty> {
          peek_array_checked::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_be_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in big-endian byte order without advancing the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_be`](Chunk::peek_" $ty "_be)."]
        /// Returns `Ok(value)` on success, or `Err(TryPeekError)` with details about
        /// requested vs available bytes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.try_peek_" $ty "_be().is_ok());"]
        ///
        /// let small_buf = &[0x12][..];
        #[doc = "let err = small_buf.try_peek_" $ty "_be().unwrap_err();"]
        /// // err contains details about requested vs available bytes
        /// ```
        #[inline]
        fn [<try_peek_ $ty _be>](&self) -> Result<$ty, TryPeekError> {
          try_peek_array::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_be_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in native-endian byte order without advancing the cursor."]
        ///
        /// The byte order depends on the target platform's endianness (little-endian on x86/x64,
        /// big-endian on some embedded platforms).
        ///
        /// Returns the decoded value. The buffer position remains unchanged after this operation.
        ///
        /// # Panics
        ///
        #[doc = "Panics if the buffer has fewer than `size_of::<" $ty ">()` bytes available."]
        /// Use the `*_checked` or `try_*` variants for non-panicking peeks.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "let value = buf.peek_" $ty "_ne();"]
        /// assert_eq!(buf.remaining(), data.len()); // Cursor unchanged
        /// ```
        #[inline]
        fn [<peek_ $ty _ne>](&self) -> $ty {
          <$ty>::from_ne_bytes(peek_array::<_, { core::mem::size_of::<$ty>() }>(self))
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in native-endian byte order without advancing the cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`peek_" $ty "_ne`](Chunk::peek_" $ty "_ne)."]
        /// Returns `Some(value)` if sufficient data is available, otherwise returns `None`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.peek_" $ty "_ne_checked().is_some());"]
        ///
        /// let small_buf = &[0x34][..];
        #[doc = "assert!(small_buf.peek_" $ty "_ne_checked().is_none()); // Not enough bytes"]
        /// ```
        #[inline]
        fn [<peek_ $ty _ne_checked>](&self) -> Option<$ty> {
          peek_array_checked::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_ne_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer in native-endian byte order without advancing the cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`peek_" $ty "_ne`](Chunk::peek_" $ty "_ne)."]
        /// Returns `Ok(value)` on success, or `Err(TryPeekError)` with details about
        /// requested vs available bytes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.try_peek_" $ty "_ne().is_ok());"]
        ///
        /// let small_buf = &[0x34][..];
        #[doc = "let err = small_buf.try_peek_" $ty "_ne().unwrap_err();"]
        /// // err contains details about requested vs available bytes
        /// ```
        #[inline]
        fn [<try_peek_ $ty _ne>](&self) -> Result<$ty, TryPeekError> {
          try_peek_array::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_ne_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in little-endian byte order."]
        ///
        /// Returns the decoded value without modifying the buffer position.
        /// The offset is relative to the current buffer position.
        ///
        /// # Panics
        ///
        #[doc = "Panics if `offset + size_of::<" $ty ">() > self.remaining()`."]
        /// Use the `*_checked` or `try_*` variants for non-panicking peeks.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "let value = buf.peek_" $ty "_le_at(2); // Peek at offset 2"]
        /// assert_eq!(buf.remaining(), data.len()); // Buffer unchanged
        /// ```
        #[inline]
        fn [<peek_ $ty _le_at>](&self, offset: usize) -> $ty {
          <$ty>::from_le_bytes(peek_array_at::<_, { core::mem::size_of::<$ty>() }>(self, offset))
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in little-endian byte order."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_le_at`](Chunk::peek_" $ty "_le_at)."]
        /// Returns `Some(value)` if sufficient data is available at the offset, otherwise returns `None`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.peek_" $ty "_le_at_checked(2).is_some());"]
        #[doc = "assert!(buf.peek_" $ty "_le_at_checked(100).is_none()); // Out of bounds"]
        /// ```
        #[inline]
        fn [<peek_ $ty _le_at_checked>](&self, offset: usize) -> Option<$ty> {
          peek_array_at_checked::<_, { core::mem::size_of::<$ty>() }>(self, offset).map(<$ty>::from_le_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in little-endian byte order."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_le_at`](Chunk::peek_" $ty "_le_at)."]
        /// Returns `Ok(value)` on success, or `Err(TryPeekAtError)` with details about
        /// the error condition (out of bounds or insufficient data).
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.try_peek_" $ty "_le_at(2).is_ok());"]
        #[doc = "let err = buf.try_peek_" $ty "_le_at(100).unwrap_err();"]
        /// // err contains details about the error
        /// ```
        #[inline]
        fn [<try_peek_ $ty _le_at>](&self, offset: usize) -> Result<$ty, TryPeekAtError> {
          try_peek_array_at::<_, { core::mem::size_of::<$ty>() }>(self, offset).map(<$ty>::from_le_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in big-endian byte order."]
        ///
        /// Returns the decoded value without modifying the buffer position.
        /// The offset is relative to the current buffer position.
        ///
        /// # Panics
        ///
        #[doc = "Panics if `offset + size_of::<" $ty ">() > self.remaining()`."]
        /// Use the `*_checked` or `try_*` variants for non-panicking peeks.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "let value = buf.peek_" $ty "_be_at(2); // Peek at offset 2"]
        /// assert_eq!(buf.remaining(), data.len()); // Buffer unchanged
        /// ```
        #[inline]
        fn [<peek_ $ty _be_at>](&self, offset: usize) -> $ty {
          <$ty>::from_be_bytes(peek_array_at::<_, { core::mem::size_of::<$ty>() }>(self, offset))
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in big-endian byte order."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_be_at`](Chunk::peek_" $ty "_be_at)."]
        /// Returns `Some(value)` if sufficient data is available at the offset, otherwise returns `None`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.peek_" $ty "_be_at_checked(2).is_some());"]
        #[doc = "assert!(buf.peek_" $ty "_be_at_checked(100).is_none()); // Out of bounds"]
        /// ```
        #[inline]
        fn [<peek_ $ty _be_at_checked>](&self, offset: usize) -> Option<$ty> {
          peek_array_at_checked::<_, { core::mem::size_of::<$ty>() }>(self, offset).map(<$ty>::from_be_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in big-endian byte order."]
        ///
        #[doc = "This is the non-panicking version of [`peek_" $ty "_be_at`](Chunk::peek_" $ty "_be_at)."]
        /// Returns `Ok(value)` on success, or `Err(TryPeekAtError)` with details about
        /// the error condition (out of bounds or insufficient data).
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.try_peek_" $ty "_be_at(2).is_ok());"]
        #[doc = "let err = buf.try_peek_" $ty "_be_at(100).unwrap_err();"]
        /// // err contains details about the error
        /// ```
        #[inline]
        fn [<try_peek_ $ty _be_at>](&self, offset: usize) -> Result<$ty, TryPeekAtError> {
          try_peek_array_at::<_, { core::mem::size_of::<$ty>() }>(self, offset).map(<$ty>::from_be_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in native-endian byte order."]
        ///
        /// The byte order depends on the target platform's endianness.
        /// Returns the decoded value without modifying the buffer position.
        /// The offset is relative to the current buffer position.
        ///
        /// # Panics
        ///
        #[doc = "Panics if `offset + size_of::<" $ty ">() > self.remaining()`."]
        /// Use the `*_checked` or `try_*` variants for non-panicking peeks.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "let value = buf.peek_" $ty "_ne_at(2); // Peek at offset 2"]
        /// assert_eq!(buf.remaining(), data.len()); // Buffer unchanged
        /// ```
        #[inline]
        fn [<peek_ $ty _ne_at>](&self, offset: usize) -> $ty {
          <$ty>::from_ne_bytes(peek_array_at::<_, { core::mem::size_of::<$ty>() }>(self, offset))
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in native-endian byte order."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`peek_" $ty "_ne_at`](Chunk::peek_" $ty "_ne_at)."]
        /// Returns `Some(value)` if sufficient data is available at the offset, otherwise returns `None`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.peek_" $ty "_ne_at_checked(2).is_some());"]
        #[doc = "assert!(buf.peek_" $ty "_ne_at_checked(100).is_none()); // Out of bounds"]
        /// ```
        #[inline]
        fn [<peek_ $ty _ne_at_checked>](&self, offset: usize) -> Option<$ty> {
          peek_array_at_checked::<_, { core::mem::size_of::<$ty>() }>(self, offset).map(<$ty>::from_ne_bytes)
        }

        #[doc = "Peeks a `" $ty "` value from the buffer at the specified offset in native-endian byte order."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`peek_" $ty "_ne_at`](Chunk::peek_" $ty "_ne_at)."]
        /// Returns `Ok(value)` on success, or `Err(TryPeekAtError)` with details about
        /// the error condition (out of bounds or insufficient data).
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let buf = &data[..];
        #[doc = "assert!(buf.try_peek_" $ty "_ne_at(0).is_ok());"]
        #[doc = "let err = buf.try_peek_" $ty "_ne_at(100).unwrap_err();"]
        /// // err contains details about the error
        /// ```
        #[inline]
        fn [<try_peek_ $ty _ne_at>](&self, offset: usize) -> Result<$ty, TryPeekAtError> {
          try_peek_array_at::<_, { core::mem::size_of::<$ty>() }>(self, offset).map(<$ty>::from_ne_bytes)
        }
      )*
    }
  };
}

macro_rules! read_fixed {
  ($($ty:ident), +$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Reads a `" $ty "` value from the buffer in little-endian byte order and advances the cursor."]
        ///
        #[doc = "Returns the decoded value and advances the cursor by `size_of::<" $ty ">()` bytes."]
        ///
        /// # Panics
        ///
        #[doc = "Panics if the buffer has fewer than `size_of::<" $ty ">()` bytes available."]
        /// Use the `*_checked` or `try_*` variants for non-panicking reads.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "let value = buf.read_" $ty "_le();"]
        #[doc = "assert_eq!(buf.remaining(), data.len() - size_of::<" $ty ">()); // Cursor advanced"]
        /// ```
        #[inline]
        fn [<read_ $ty _le>](&mut self) -> $ty {
          <$ty>::from_le_bytes(read_array::<_, { core::mem::size_of::<$ty>() }>(self))
        }

        #[doc = "Reads a `" $ty "` value from the buffer in little-endian byte order and advances the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`read_" $ty "_le`](Chunk::read_" $ty "_le)."]
        /// Returns `Some(value)` and advances the cursor on success, or `None` if insufficient data.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "if let Some(value) = buf.read_" $ty "_le_checked() {"]
        #[doc = "    // Cursor advanced by size_of::<" $ty ">()"]
        /// } else {
        ///     // Not enough data, cursor unchanged
        /// }
        /// ```
        #[inline]
        fn [<read_ $ty _le_checked>](&mut self) -> Option<$ty> {
          read_array_checked::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_le_bytes)
        }

        #[doc = "Reads a `" $ty "` value from the buffer in little-endian byte order and advances the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`read_" $ty "_le`](Chunk::read_" $ty "_le)."]
        /// Returns `Ok(value)` and advances the cursor on success, or `Err(TryReadError)`
        /// with details about requested vs available bytes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "match buf.try_read_" $ty "_le() {"]
        ///     Ok(value) => {
        #[doc = "        // Cursor advanced by size_of::<" $ty ">()"]
        ///     },
        ///     Err(e) => {
        ///         // e contains details about requested vs available bytes
        ///     }
        /// }
        /// ```
        #[inline]
        fn [<try_read_ $ty _le>](&mut self) -> Result<$ty, TryReadError> {
          try_read_array::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_le_bytes)
        }

        #[doc = "Reads a `" $ty "` value from the buffer in big-endian byte order and advances the cursor."]
        ///
        #[doc = "Returns the decoded value and advances the cursor by `size_of::<" $ty ">()` bytes."]
        ///
        /// # Panics
        ///
        #[doc = "Panics if the buffer has fewer than `size_of::<" $ty ">()` bytes available."]
        /// Use the `*_checked` or `try_*` variants for non-panicking reads.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "let value = buf.read_" $ty "_be();"]
        #[doc = "assert_eq!(buf.remaining(), data.len() - size_of::<" $ty ">()); // Cursor advanced"]
        /// ```
        #[inline]
        fn [<read_ $ty _be>](&mut self) -> $ty {
          <$ty>::from_be_bytes(read_array::<_, { core::mem::size_of::<$ty>() }>(self))
        }

        #[doc = "Reads a `" $ty "` value from the buffer in big-endian byte order and advances the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`read_" $ty "_be`](Chunk::read_" $ty "_be)."]
        /// Returns `Some(value)` and advances the cursor on success, or `None` if insufficient data.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "if let Some(value) = buf.read_" $ty "_be_checked() {"]
        #[doc = "    // Cursor advanced by size_of::<" $ty ">()"]
        /// } else {
        ///     // Not enough data, cursor unchanged
        /// }
        /// ```
        #[inline]
        fn [<read_ $ty _be_checked>](&mut self) -> Option<$ty> {
          read_array_checked::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_be_bytes)
        }

        #[doc = "Reads a `" $ty "` value from the buffer in big-endian byte order and advances the cursor."]
        ///
        #[doc = "This is the non-panicking version of [`read_" $ty "_be`](Chunk::read_" $ty "_be)."]
        /// Returns `Ok(value)` and advances the cursor on success, or `Err(TryReadError)`
        /// with details about requested vs available bytes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "match buf.try_read_" $ty "_be() {"]
        ///     Ok(value) => {
        #[doc = "        // Cursor advanced by size_of::<" $ty ">()"]
        ///     },
        ///     Err(e) => {
        ///         // e contains details about requested vs available bytes
        ///     }
        /// }
        /// ```
        #[inline]
        fn [<try_read_ $ty _be>](&mut self) -> Result<$ty, TryReadError> {
          try_read_array::<_, { core::mem::size_of::<$ty>() }>(self).map(|val| { <$ty>::from_be_bytes(val) })
        }

        #[doc = "Reads a `" $ty "` value from the buffer in native-endian byte order and advances the cursor."]
        ///
        /// The byte order depends on the target platform's endianness (little-endian on x86/x64,
        /// big-endian on some embedded platforms).
        ///
        #[doc = "Returns the decoded value and advances the cursor by `size_of::<" $ty ">()` bytes."]
        ///
        /// # Panics
        ///
        #[doc = "Panics if the buffer has fewer than `size_of::<" $ty ">()` bytes available."]
        /// Use the `*_checked` or `try_*` variants for non-panicking reads.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "let value = buf.read_" $ty "_ne();"]
        #[doc = "assert_eq!(buf.remaining(), data.len() - size_of::<" $ty ">()); // Cursor advanced"]
        /// ```
        #[inline]
        fn [<read_ $ty _ne>](&mut self) -> $ty {
          <$ty>::from_ne_bytes(read_array::<_, { core::mem::size_of::<$ty>() }>(self))
        }

        #[doc = "Reads a `" $ty "` value from the buffer in native-endian byte order and advances the cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`read_" $ty "_ne`](Chunk::read_" $ty "_ne)."]
        /// Returns `Some(value)` and advances the cursor on success, or `None` if insufficient data.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "if let Some(value) = buf.read_" $ty "_ne_checked() {"]
        #[doc = "    // Cursor advanced by size_of::<" $ty ">()"]
        /// } else {
        ///     // Not enough data, cursor unchanged
        /// }
        /// ```
        #[inline]
        fn [<read_ $ty _ne_checked>](&mut self) -> Option<$ty> {
          read_array_checked::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_ne_bytes)
        }

        #[doc = "Reads a `" $ty "` value from the buffer in native-endian byte order and advances the cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`read_" $ty "_ne`](Chunk::read_" $ty "_ne)."]
        /// Returns `Ok(value)` and advances the cursor on success, or `Err(TryReadError)`
        /// with details about requested vs available bytes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use buffo::Chunk;
        ///
        /// let data = [147, 23, 89, 201, 156, 74, 33, 198, 67, 142, 91, 205, 38, 177, 124, 59, 183, 96, 241, 167, 82, 135, 49, 213];
        /// let mut buf = &data[..];
        #[doc = "match buf.try_read_" $ty "_ne() {"]
        ///     Ok(value) => {
        #[doc = "        // Cursor advanced by size_of::<" $ty ">()"]
        ///     },
        ///     Err(e) => {
        ///         // e contains details about requested vs available bytes
        ///     }
        /// }
        /// ```
        #[inline]
        fn [<try_read_ $ty _ne>](&mut self) -> Result<$ty, TryReadError> {
          try_read_array::<_, { core::mem::size_of::<$ty>() }>(self).map(<$ty>::from_ne_bytes)
        }
      )*
    }
  };
}

/// A trait for chunk-like buffer types (such as [`Chunk`] and `ChunkMut`) that can be
/// instantiated directly as empty.
pub trait EmptyChunk {
  /// Creates an empty buffer instance.
  ///
  /// The returned value must represent an empty view of the underlying data.
  /// For types implementing [`Chunk`], this implies `remaining() == 0` and
  /// `buffer().is_empty()`. For mutable chunk-like types (such as `ChunkMut`),
  /// this implies there are no readable or writable bytes available according
  /// to that type's API.
  ///
  /// This is a convenience method for creating an empty buffer of the implementing type.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, EmptyChunk};
  ///
  /// let empty_buf = <&[u8]>::empty();
  /// assert_eq!(empty_buf.remaining(), 0);
  /// assert!(empty_buf.buffer().is_empty());
  /// ```
  fn empty() -> Self
  where
    Self: Sized;
}

/// A trait for implementing custom buffers that can read and navigate through byte sequences.
///
/// **Coherence obligation:** an implementation must keep `buffer().len()` equal to
/// [`remaining()`](Chunk::remaining), and neither `buffer()` nor `remaining()` may panic.
/// The non-panicking default methods (the `*_checked` and `try_*` families) rely on this
/// invariant to stay panic-free and correct; an incoherent implementation can make them
/// panic or misbehave.
///
/// This trait provides a comprehensive set of methods for reading data from buffers with different
/// error handling strategies:
/// - **Panicking methods** (e.g., `read_*`): Fast operations that panic on insufficient data
/// - **Checked methods** (e.g., `*_checked`): Return `Option` - `None` on failure, `Some(value)` on success
/// - **Fallible methods** (e.g., `try_*`): Return `Result` with detailed error information
///
/// # Method Categories
///
/// - **Buffer inspection**: `remaining()`, `has_remaining()`, `buffer()`
/// - **Navigation**: `advance()`, `try_advance()`
/// - **Buffer manipulation**: `truncate()`, `split_to()`, `split_off()`, `segment()`
/// - **Peeking data**: `peek_u8()`, `peek_u16_le()`, etc. (read without advancing)
/// - **Reading data**: `read_u8()`, `read_u16_le()`, etc. (read and advance cursor)
pub trait Chunk {
  /// Returns the number of bytes available for reading in the buffer.
  ///
  /// This represents how many bytes can be read from the current cursor position
  /// to the end of the buffer.
  ///
  /// **Implementor obligation:** this must equal `buffer().len()` and must not panic.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  /// assert_eq!(buf.remaining(), 5);
  ///
  /// buf.advance(2);
  /// assert_eq!(buf.remaining(), 3);
  /// ```
  fn remaining(&self) -> usize;

  /// Returns the remaining bytes of the buffer as a slice.
  ///
  /// This provides direct access to all bytes from the current cursor position
  /// to the end of the buffer.
  ///
  /// **Implementor obligation:** the returned length must equal
  /// [`remaining()`](Chunk::remaining), and this method must not panic.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  /// assert_eq!(buf.buffer(), &[1, 2, 3, 4, 5]);
  ///
  /// buf.advance(2);
  /// assert_eq!(buf.buffer(), &[3, 4, 5]);
  /// ```
  fn buffer(&self) -> &[u8];

  /// Returns a slice of the buffer starting from the specified offset.
  ///
  /// This is similar to [`buffer`](Chunk::buffer) but starts from the given offset
  /// rather than the current cursor position.
  ///
  /// # Panics
  ///
  /// Panics if `offset > self.remaining()`.
  /// Use [`buffer_from_checked`](Chunk::buffer_from_checked) for non-panicking access.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1u8, 2, 3, 4, 5];
  /// let buf = &data[..];
  ///
  /// assert_eq!(buf.buffer(), &[1, 2, 3, 4, 5]);
  /// assert_eq!(buf.buffer_from(2), &[3, 4, 5]);
  /// ```
  #[inline]
  fn buffer_from(&self, offset: usize) -> &[u8] {
    &self.buffer()[offset..]
  }

  /// Returns a slice of the buffer starting from the specified offset.
  ///
  /// This is the non-panicking version of [`buffer_from`](Chunk::buffer_from).
  /// Returns `Some(slice)` if `offset <= self.remaining()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1u8, 2, 3, 4, 5];
  /// let buf = &data[..];
  ///
  /// assert_eq!(buf.buffer_from_checked(2), Some(&[3, 4, 5][..]));
  /// assert!(buf.buffer_from_checked(5).unwrap().is_empty()); // empty buffer
  /// assert_eq!(buf.buffer_from_checked(10), None); // Out of bounds
  /// ```
  #[inline]
  fn buffer_from_checked(&self, offset: usize) -> Option<&[u8]> {
    if offset > self.remaining() {
      None
    } else {
      Some(&self.buffer()[offset..])
    }
  }

  /// Advances the internal cursor by the specified number of bytes.
  ///
  /// This moves the read position forward, making the advanced bytes no longer
  /// available for reading. The operation consumes the bytes without returning them.
  ///
  /// # Panics
  ///
  /// Panics if `cnt > self.remaining()`.
  /// Use [`try_advance`](Chunk::try_advance) for non-panicking advancement.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// buf.advance(2);
  /// assert_eq!(buf.remaining(), 3);
  /// assert_eq!(buf.buffer(), &[3, 4, 5]);
  /// ```
  fn advance(&mut self, cnt: usize);

  /// Returns a slice containing the first `len` bytes of the buffer.
  ///
  /// This provides access to a prefix of the buffer for efficient manipulation
  /// of a specific portion without affecting the rest of the buffer.
  ///
  /// # Panics
  ///
  /// Panics if `len > self.remaining()`.
  /// Use [`prefix_checked`](Chunk::prefix_checked) for non-panicking access.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let mut buf = [1u8, 2, 3, 4, 5];
  /// let slice = &buf[..];
  /// let prefix = Chunk::prefix(&slice, 3);
  /// assert_eq!(prefix, [1u8, 2, 3].as_slice());
  /// ```
  #[inline]
  fn prefix(&self, len: usize) -> &[u8] {
    &self.buffer()[..len]
  }

  /// Returns a slice containing the first `len` bytes of the buffer.
  ///
  /// This is the non-panicking version of [`prefix`](Chunk::prefix).
  /// Returns `Some(slice)` if `len <= self.remaining()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let mut buf = [1u8, 2, 3, 4, 5];
  ///
  /// assert_eq!(Chunk::prefix_checked(&&buf[..], 3).unwrap(), &[1, 2, 3]);
  /// assert_eq!(Chunk::prefix_checked(&&buf[..], 5).unwrap(), &[1, 2, 3, 4, 5]);
  /// assert!(Chunk::prefix_checked(&&buf[..], 10).is_none());
  /// ```
  #[inline]
  fn prefix_checked(&self, len: usize) -> Option<&[u8]> {
    if self.remaining() < len {
      None
    } else {
      Some(&self.buffer()[..len])
    }
  }

  /// Returns a slice containing the last `len` bytes of the buffer.
  ///
  /// This provides access to a suffix of the buffer for efficient manipulation
  /// of the trailing portion without affecting the rest of the buffer.
  ///
  /// # Panics
  ///
  /// Panics if `len > self.remaining()`.
  /// Use [`suffix_checked`](Chunk::suffix_checked) for non-panicking access.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let mut buf = [1u8, 2, 3, 4, 5];
  ///
  /// let slice = &buf[..];
  /// let suffix = Chunk::suffix(&slice, 2);
  /// assert_eq!(suffix, &[4, 5]);
  /// ```
  #[inline]
  fn suffix(&self, len: usize) -> &[u8] {
    let total_len = self.remaining();
    &self.buffer()[total_len - len..]
  }

  /// Returns a slice containing the last `len` bytes of the buffer.
  ///
  /// This is the non-panicking version of [`suffix`](Chunk::suffix).
  /// Returns `Some(slice)` if `len <= self.remaining()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let mut buf = [1u8, 2, 3, 4, 5];
  /// let slice = &buf[..];
  /// assert_eq!(Chunk::suffix_checked(&slice, 2).unwrap(), &[4, 5]);
  /// assert_eq!(Chunk::suffix_checked(&slice, 5).unwrap(), &[1, 2, 3, 4, 5]);
  /// assert!(Chunk::suffix_checked(&slice, 10).is_none());
  /// ```
  #[inline]
  fn suffix_checked(&self, len: usize) -> Option<&[u8]> {
    self
      .remaining()
      .checked_sub(len)
      .map(|start| &self.buffer()[start..])
  }

  /// Creates an independent buffer containing a segment of the current buffer's data.
  ///
  /// This method returns a new buffer instance that represents a portion of the current
  /// buffer defined by the given range. The original buffer remains unchanged,
  /// and the new buffer has its own independent cursor starting at the beginning of the segment.
  ///
  /// # Panics
  ///
  /// Panics if the range is out of bounds relative to the current buffer's available data.
  /// Use [`try_segment`](Chunk::try_segment) for non-panicking segmentation.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = b"Hello, World!";
  /// let buf = &data[..];
  ///
  /// let hello = buf.segment(0..5);
  /// let world = buf.segment(7..12);
  ///
  /// assert_eq!(hello.buffer(), b"Hello");
  /// assert_eq!(world.buffer(), b"World");
  /// // Original buffer unchanged
  /// assert_eq!(buf.remaining(), 13);
  /// ```
  fn segment(&self, range: impl RangeBounds<usize>) -> Self
  where
    Self: Sized;

  /// Shortens the buffer to the specified length, keeping the first `len` bytes.
  ///
  /// If `len` is greater than the buffer's current available bytes, this has no effect.
  /// This operation cannot fail and will never panic.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// buf.truncate(3);
  /// assert_eq!(buf.remaining(), 3);
  /// assert_eq!(buf.buffer(), &[1, 2, 3]);
  ///
  /// // Truncating to a length >= available has no effect
  /// buf.truncate(10);
  /// assert_eq!(buf.remaining(), 3);
  /// ```
  fn truncate(&mut self, len: usize);

  /// Splits the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[0, at)`, and the returned buffer
  /// contains elements `[at, remaining())`. The memory layout remains unchanged.
  ///
  /// **Implementor Notes:** This should be an `O(1)` operation.
  ///
  /// # Panics
  ///
  /// Panics if `at > self.remaining()`.
  /// Use [`split_off_checked`](Chunk::split_off_checked) or
  /// [`try_split_off`](Chunk::try_split_off) for non-panicking splits.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// let tail = Chunk::split_off(&mut buf, 2);
  /// assert_eq!(buf.buffer(), &[1, 2]);
  /// assert_eq!(tail.buffer(), &[3, 4, 5]);
  /// ```
  #[must_use = "consider Chunk::truncate if you don't need the other half"]
  fn split_off(&mut self, at: usize) -> Self
  where
    Self: Sized;

  /// Splits the buffer into two at the given index.
  ///
  /// This is the non-panicking version of [`split_off`](Chunk::split_off).
  /// Returns `Some((left, right))` if `at <= self.remaining()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// assert!(Chunk::split_off_checked(&mut buf, 2).is_some());
  ///
  /// let mut small_buf = &[1u8][..];
  /// assert!(Chunk::split_off_checked(&mut small_buf, 5).is_none());
  /// ```
  #[must_use = "consider Chunk::truncate if you don't need the other half"]
  fn split_off_checked(&mut self, at: usize) -> Option<Self>
  where
    Self: Sized,
  {
    if at > self.remaining() {
      None
    } else {
      Some(self.split_off(at))
    }
  }

  /// Splits the buffer into two at the given index.
  ///
  /// This is the non-panicking version of [`split_off`](Chunk::split_off) that
  /// returns detailed error information on failure.
  /// Returns `Ok(right_half)` on success, or `Err(OutOfBounds)` with details about
  /// the attempted split position and available bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// assert!(Chunk::try_split_off(&mut buf, 2).is_ok());
  ///
  /// let mut small_buf = &[1u8][..];
  /// let err = Chunk::try_split_off(&mut small_buf, 5).unwrap_err();
  /// // err contains details about requested vs available
  /// ```
  #[must_use = "consider Chunk::truncate if you don't need the other half"]
  fn try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds>
  where
    Self: Sized,
  {
    if at > self.remaining() {
      Err(OutOfBounds::new(at, self.remaining()))
    } else {
      Ok(self.split_off(at))
    }
  }

  /// Splits the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[at, remaining())`, and the returned
  /// buffer contains elements `[0, at)`.
  ///
  /// **Implementor Notes:** This should be an `O(1)` operation.
  ///
  /// # Panics
  ///
  /// Panics if `at > self.remaining()`.
  /// Use [`split_to_checked`](Chunk::split_to_checked) or
  /// [`try_split_to`](Chunk::try_split_to) for non-panicking splits.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = b"hello world";
  /// let mut buf = &data[..];
  ///
  /// let hello = buf.split_to(5);
  /// assert_eq!(hello.buffer(), b"hello");
  /// assert_eq!(buf.buffer(), b" world");
  /// ```
  #[must_use = "consider Chunk::advance if you don't need the other half"]
  fn split_to(&mut self, at: usize) -> Self
  where
    Self: Sized;

  /// Splits the buffer into two at the given index.
  ///
  /// This is the non-panicking version of [`split_to`](Chunk::split_to).
  /// Returns `Some(left_half)` if `at <= self.remaining()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// assert!(Chunk::split_to_checked(&mut buf, 3).is_some());
  /// assert!(Chunk::split_to_checked(&mut buf, 10).is_none());
  /// ```
  #[must_use = "consider Chunk::advance if you don't need the other half"]
  fn split_to_checked(&mut self, at: usize) -> Option<Self>
  where
    Self: Sized,
  {
    if at > self.remaining() {
      None
    } else {
      Some(self.split_to(at))
    }
  }

  /// Splits the buffer into two at the given index.
  ///
  /// This is the non-panicking version of [`split_to`](Chunk::split_to) that
  /// returns detailed error information on failure.
  /// Returns `Ok(left_half)` on success, or `Err(OutOfBounds)` with details about
  /// the attempted split position and available bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// assert!(Chunk::try_split_to(&mut buf, 3).is_ok());
  ///
  /// let err = Chunk::try_split_to(&mut buf, 10).unwrap_err();
  /// // err contains detailed information about the failure
  /// ```
  #[must_use = "consider Chunk::advance if you don't need the other half"]
  fn try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds>
  where
    Self: Sized,
  {
    if at > self.remaining() {
      Err(OutOfBounds::new(at, self.remaining()))
    } else {
      Ok(self.split_to(at))
    }
  }

  /// Returns `true` if there are bytes available for reading in the buffer.
  ///
  /// This is equivalent to `self.remaining() > 0`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3];
  /// let mut buf = &data[..];
  /// assert!(Chunk::has_remaining(&buf));
  ///
  /// buf.advance(3);
  /// assert!(!Chunk::has_remaining(&buf));
  /// ```
  fn has_remaining(&self) -> bool {
    self.remaining() > 0
  }

  /// Attempts to advance the internal cursor by the specified number of bytes.
  ///
  /// This is the non-panicking version of [`advance`](Chunk::advance).
  /// Returns `Ok(())` if the advancement was successful, or `Err(TryAdvanceError)`
  /// with details about requested vs available bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// assert!(buf.try_advance(3).is_ok());
  /// assert_eq!(buf.remaining(), 2);
  ///
  /// let err = buf.try_advance(5).unwrap_err();
  /// // err contains details about requested vs available
  /// ```
  fn try_advance(&mut self, cnt: usize) -> Result<(), TryAdvanceError> {
    if cnt == 0 {
      return Ok(());
    }

    let remaining = self.remaining();
    if remaining < cnt {
      return Err(TryAdvanceError::new(must_non_zero(cnt), remaining));
    }

    self.advance(cnt);
    Ok(())
  }

  /// Attempts to create a new buffer containing a segment of the current buffer's data.
  ///
  /// The returned buffer is independent with its own cursor starting at the beginning of the segment.
  /// The original buffer remains unchanged. This is the non-panicking version of
  /// [`segment`](Chunk::segment).
  ///
  /// Returns `Ok(segment)` if the range is valid, or `Err(TrySegmentError)` if the range
  /// extends beyond the current buffer's available data.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = b"Hello, World!";
  /// let buf = &data[..];
  ///
  /// assert!(buf.try_segment(0..5).is_ok());
  /// assert!(buf.try_segment(0..20).is_err()); // Out of bounds
  /// ```
  #[inline]
  fn try_segment(&self, range: impl RangeBounds<usize>) -> Result<Self, TrySegmentError>
  where
    Self: Sized,
  {
    check_segment(range, self.remaining()).map(|(start, end)| self.segment(start..end))
  }

  // Macro generates peek methods for primitive types
  peek_fixed!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

  // Macro generates read methods for primitive types
  read_fixed!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

  /// Peeks a `u8` value from the buffer without advancing the internal cursor.
  ///
  /// Returns the first byte from the buffer without consuming it.
  /// The buffer position remains unchanged after this operation.
  ///
  /// # Panics
  ///
  /// Panics if the buffer is empty.
  /// Use [`peek_u8_checked`](Chunk::peek_u8_checked) for non-panicking peeks.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42, 1, 2, 3];
  /// let buf = &data[..];
  ///
  /// assert_eq!(buf.peek_u8(), 42);
  /// assert_eq!(buf.remaining(), 4); // Unchanged
  /// ```
  #[inline]
  fn peek_u8(&self) -> u8 {
    self.buffer()[0]
  }

  /// Peeks a `u8` value from the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`peek_u8`](Chunk::peek_u8).
  /// Returns `Some(byte)` if data is available, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42];
  /// let buf = &data[..];
  /// assert_eq!(buf.peek_u8_checked(), Some(42));
  ///
  /// let empty_buf = &[][..];
  /// assert_eq!(empty_buf.peek_u8_checked(), None);
  /// ```
  #[inline]
  fn peek_u8_checked(&self) -> Option<u8> {
    self.buffer().first().copied()
  }

  /// Peeks a `u8` value from the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`peek_u8`](Chunk::peek_u8).
  /// Returns `Ok(byte)` if data is available, otherwise returns `Err`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42];
  /// let buf = &data[..];
  /// assert_eq!(buf.peek_u8_checked(), Some(42));
  ///
  /// let empty_buf = &[][..];
  /// assert!(empty_buf.try_peek_u8().is_err());
  /// ```
  #[inline]
  fn try_peek_u8(&self) -> Result<u8, TryPeekError> {
    self
      .buffer()
      .first()
      .copied()
      .ok_or_else(|| TryPeekError::new(super::NON_ZERO_1, self.remaining()))
  }

  /// Peeks a `u8` value from the buffer at the specified offset without advancing the cursor.
  ///
  /// Returns the byte at the given offset without consuming it.
  /// The buffer position remains unchanged after this operation.
  ///
  /// # Panics
  ///
  /// Panics if `offset >= self.remaining()`.
  /// Use [`peek_u8_at_checked`](Chunk::peek_u8_at_checked) for non-panicking peeks.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42, 1, 2, 3];
  /// let buf = &data[..];
  ///
  /// assert_eq!(buf.peek_u8_at(0), 42);
  /// assert_eq!(buf.peek_u8_at(2), 2);
  /// assert_eq!(buf.remaining(), 4); // Unchanged
  /// ```
  #[inline]
  fn peek_u8_at(&self, offset: usize) -> u8 {
    self.buffer_from(offset)[0]
  }

  /// Peeks a `u8` value from the buffer at the specified offset without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_u8_at`](Chunk::peek_u8_at).
  /// Returns `Some(byte)` if the offset is valid, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42, 1, 2];
  /// let buf = &data[..];
  /// assert_eq!(buf.peek_u8_at_checked(1), Some(1));
  /// assert_eq!(buf.peek_u8_at_checked(10), None);
  /// ```
  #[inline]
  fn peek_u8_at_checked(&self, offset: usize) -> Option<u8> {
    self.buffer().get(offset).copied()
  }

  /// Peeks a `u8` value from the buffer at the specified offset without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_u8_at`](Chunk::peek_u8_at).
  /// Returns `Ok(byte)` if the offset is valid, otherwise returns `Err(TryPeekAtError)`.
  ///
  /// # Errors
  ///
  /// An `offset` strictly past the end returns `OutOfBounds`, while `offset == remaining()`
  /// (in range, but with no byte to read) returns `InsufficientData`. This read-side
  /// boundary at `offset == len` is the deliberate opposite of the write-side `put_*_at`,
  /// where `offset == remaining_mut()` returns `OutOfBounds`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42, 1, 2];
  /// let buf = &data[..];
  /// assert_eq!(buf.try_peek_u8_at(1), Ok(1));
  ///
  /// let err = buf.try_peek_u8_at(10).unwrap_err();
  /// // err contains details about the error
  /// ```
  #[inline]
  fn try_peek_u8_at(&self, offset: usize) -> Result<u8, TryPeekAtError> {
    let buffer = self.buffer();
    let buf_len = buffer.len();

    // Two-stage taxonomy, mirroring `try_peek_array_at`: an offset strictly past the
    // end is `OutOfBounds`, while an in-range offset with too few bytes (for a single
    // byte, exactly `offset == buf_len`) is `InsufficientData`.
    match buf_len.checked_sub(offset) {
      None => Err(TryPeekAtError::out_of_bounds(offset, buf_len)),
      Some(0) => Err(TryPeekAtError::insufficient_data_with_requested(
        0,
        offset,
        super::NON_ZERO_1,
      )),
      Some(_) => Ok(buffer[offset]),
    }
  }

  /// Peeks an `i8` value from the buffer at the specified offset without advancing the cursor.
  ///
  /// Returns the byte at the given offset as a signed integer without consuming it.
  /// The buffer position remains unchanged after this operation.
  ///
  /// # Panics
  ///
  /// Panics if `offset >= self.remaining()`.
  /// Use [`peek_i8_at_checked`](Chunk::peek_i8_at_checked) for non-panicking peeks.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8, 1, 2, 3]; // 255 as i8 is -1
  /// let buf = &data[..];
  ///
  /// assert_eq!(buf.peek_i8_at(0), -1);
  /// assert_eq!(buf.remaining(), 4); // Unchanged
  /// ```
  #[inline]
  fn peek_i8_at(&self, offset: usize) -> i8 {
    self.peek_u8_at(offset) as i8
  }

  /// Peeks an `i8` value from the buffer at the specified offset without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_i8_at`](Chunk::peek_i8_at).
  /// Returns `Some(byte)` if the offset is valid, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8, 1, 2]; // 255 as i8 is -1
  /// let buf = &data[..];
  /// assert_eq!(buf.peek_i8_at_checked(0), Some(-1));
  /// assert_eq!(buf.peek_i8_at_checked(10), None);
  /// ```
  #[inline]
  fn peek_i8_at_checked(&self, offset: usize) -> Option<i8> {
    self.peek_u8_at_checked(offset).map(|v| v as i8)
  }

  /// Peeks an `i8` value from the buffer at the specified offset without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_i8_at`](Chunk::peek_i8_at).
  /// Returns `Ok(byte)` if the offset is valid, otherwise returns `Err(TryPeekAtError)`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8, 1, 2]; // 255 as i8 is -1
  /// let buf = &data[..];
  /// assert_eq!(buf.try_peek_i8_at(0), Ok(-1));
  ///
  /// let err = buf.try_peek_i8_at(10).unwrap_err();
  /// // err contains details about the error
  /// ```
  #[inline]
  fn try_peek_i8_at(&self, offset: usize) -> Result<i8, TryPeekAtError> {
    self.try_peek_u8_at(offset).map(|v| v as i8)
  }

  /// Reads a `u8` value from the buffer and advances the internal cursor.
  ///
  /// Returns the first byte from the buffer and advances the cursor by 1 byte.
  ///
  /// # Panics
  ///
  /// Panics if the buffer is empty.
  /// Use [`read_u8_checked`](Chunk::read_u8_checked) for non-panicking reads.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42, 1, 2, 3];
  /// let mut buf = &data[..];
  ///
  /// assert_eq!(buf.read_u8(), 42);
  /// assert_eq!(buf.remaining(), 3); // Cursor advanced
  /// ```
  #[inline]
  fn read_u8(&mut self) -> u8 {
    let val = self.peek_u8();
    self.advance(1);
    val
  }

  /// Reads a `u8` value from the buffer and advances the internal cursor.
  ///
  /// This is the non-panicking version of [`read_u8`](Chunk::read_u8).
  /// Returns `Some(byte)` and advances the cursor on success, or `None` if the buffer is empty.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42];
  /// let mut buf = &data[..];
  ///
  /// assert_eq!(buf.read_u8_checked(), Some(42));
  /// assert_eq!(buf.remaining(), 0);
  ///
  /// assert_eq!(buf.read_u8_checked(), None); // Empty now
  /// ```
  #[inline]
  fn read_u8_checked(&mut self) -> Option<u8> {
    self.peek_u8_checked().inspect(|_| {
      self.advance(1);
    })
  }

  /// Reads a `u8` value from the buffer and advances the internal cursor.
  ///
  /// This is the non-panicking version of [`read_u8`](Chunk::read_u8).
  /// Returns `Ok(byte)` and advances the cursor on success, or `Err` if the buffer is empty.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [42];
  /// let mut buf = &data[..];
  ///
  /// assert_eq!(buf.read_u8_checked(), Some(42));
  /// assert_eq!(buf.remaining(), 0);
  ///
  /// assert!(buf.try_read_u8().is_err()); // Empty now
  /// ```
  #[inline]
  fn try_read_u8(&mut self) -> Result<u8, TryReadError> {
    self
      .read_u8_checked()
      .ok_or_else(|| TryReadError::new(super::NON_ZERO_1, self.remaining()))
  }

  /// Peeks an `i8` value from the buffer without advancing the internal cursor.
  ///
  /// Returns the first byte from the buffer as a signed integer without consuming it.
  /// The buffer position remains unchanged after this operation.
  ///
  /// # Panics
  ///
  /// Panics if the buffer is empty.
  /// Use [`peek_i8_checked`](Chunk::peek_i8_checked) for non-panicking peeks.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8, 1, 2, 3]; // 255 as i8 is -1
  /// let buf = &data[..];
  ///
  /// assert_eq!(buf.peek_i8(), -1);
  /// assert_eq!(buf.remaining(), 4); // Unchanged
  /// ```
  #[inline]
  fn peek_i8(&self) -> i8 {
    self.peek_u8() as i8
  }

  /// Peeks an `i8` value from the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`peek_i8`](Chunk::peek_i8).
  /// Returns `Some(byte)` if data is available, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8]; // -1 as i8
  /// let buf = &data[..];
  /// assert_eq!(buf.peek_i8_checked(), Some(-1));
  ///
  /// let empty_buf = &[][..];
  /// assert_eq!(empty_buf.peek_i8_checked(), None);
  /// ```
  #[inline]
  fn peek_i8_checked(&self) -> Option<i8> {
    self.peek_u8_checked().map(|v| v as i8)
  }

  /// Peeks an `i8` value from the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`peek_i8`](Chunk::peek_i8).
  /// Returns `Ok(byte)` if data is available, otherwise returns `Err`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8]; // -1 as i8
  /// let buf = &data[..];
  /// assert_eq!(buf.try_peek_i8(), Ok(-1));
  ///
  /// let empty_buf = &[][..];
  /// assert!(empty_buf.try_peek_i8().is_err());
  /// ```
  #[inline]
  fn try_peek_i8(&self) -> Result<i8, TryPeekError> {
    self.try_peek_u8().map(|v| v as i8)
  }

  /// Reads an `i8` value from the buffer and advances the internal cursor.
  ///
  /// Returns the first byte from the buffer as a signed integer and advances the cursor by 1 byte.
  ///
  /// # Panics
  ///
  /// Panics if the buffer is empty.
  /// Use [`read_i8_checked`](Chunk::read_i8_checked) for non-panicking reads.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8, 1, 2, 3]; // 255 as i8 is -1
  /// let mut buf = &data[..];
  ///
  /// assert_eq!(buf.read_i8(), -1);
  /// assert_eq!(buf.remaining(), 3); // Cursor advanced
  /// ```
  #[inline]
  fn read_i8(&mut self) -> i8 {
    self.read_u8() as i8
  }

  /// Reads an `i8` value from the buffer and advances the internal cursor.
  ///
  /// This is the non-panicking version of [`read_i8`](Chunk::read_i8).
  /// Returns `Some(byte)` and advances the cursor on success, or `None` if the buffer is empty.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8]; // -1 as i8
  /// let mut buf = &data[..];
  ///
  /// assert_eq!(buf.read_i8_checked(), Some(-1));
  /// assert_eq!(buf.remaining(), 0);
  ///
  /// assert_eq!(buf.read_i8_checked(), None); // Empty now
  /// ```
  #[inline]
  fn read_i8_checked(&mut self) -> Option<i8> {
    self.read_u8_checked().map(|v| v as i8)
  }

  /// Reads an `i8` value from the buffer and advances the internal cursor.
  ///
  /// This is the non-panicking version of [`read_i8`](Chunk::read_i8).
  /// Returns `Ok(byte)` and advances the cursor on success, or `Err(TryReadError)`
  /// if the buffer is empty.
  ///
  /// # Examples
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [255u8]; // -1 as i8
  /// let mut buf = &data[..];
  /// assert_eq!(buf.try_read_i8(), Ok(-1));
  /// assert_eq!(buf.remaining(), 0);
  /// assert!(buf.try_read_i8().is_err()); // Empty now
  /// ```
  #[inline]
  fn try_read_i8(&mut self) -> Result<i8, TryReadError> {
    self.try_read_u8().map(|v| v as i8)
  }

  /// Converts the read buffer to a `Vec<u8>` instance.
  ///
  /// Creates a new vector containing a copy of all available bytes in the buffer.
  /// The original buffer remains unchanged.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::Chunk;
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  ///
  /// let vec = buf.to_vec();
  /// assert_eq!(vec, vec![1, 2, 3, 4, 5]);
  /// ```
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  fn to_vec(&self) -> ::std::vec::Vec<u8> {
    self.buffer().to_vec()
  }

  /// Converts the read buffer to a `Bytes` instance.
  ///
  /// Creates a new `Bytes` instance containing a copy of all available bytes in the buffer.
  /// The original buffer remains unchanged.
  #[cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc"))))
  )]
  fn to_bytes(&self) -> ::bytes_1::Bytes {
    ::bytes_1::Bytes::copy_from_slice(self.buffer())
  }

  /// Converts the read buffer to a `BytesMut` instance.
  ///
  /// Creates a new `BytesMut` instance containing a copy of all available bytes in the buffer.
  /// The original buffer remains unchanged.
  #[cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc"))))
  )]
  #[allow(clippy::wrong_self_convention)]
  fn to_bytes_mut(&self) -> ::bytes_1::BytesMut {
    ::bytes_1::BytesMut::from(self.to_bytes())
  }

  /// Converts the read buffer to a smol-bytes [`Bytes`](::smol_bytes_01::Bytes) instance.
  ///
  /// Creates a new `Bytes` instance containing a copy of all available bytes in the buffer.
  /// The original buffer remains unchanged.
  ///
  /// # UTF-8 holders
  ///
  /// buffo intentionally does not implement [`Chunk`]/[`ChunkMut`] for smol-bytes' `Utf8*`
  /// types, because exposing a raw `&mut [u8]` window would let a caller violate their
  /// "always valid UTF-8" invariant. To work with the bytes of a `Utf8Bytes`, `Utf8BytesMut`,
  /// or `Utf8Buffer`, obtain the underlying byte-level type with smol-bytes' `as_inner()` /
  /// `into_inner()` and use [`Chunk`]/[`ChunkMut`] on that; re-entry to the `Utf8*` type
  /// revalidates through smol-bytes' own fallible constructors.
  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
  )]
  fn to_smol_bytes(&self) -> ::smol_bytes_01::Bytes {
    ::smol_bytes_01::Bytes::copy_from_slice(self.buffer())
  }

  /// Converts the read buffer to a smol-bytes [`BytesMut`](::smol_bytes_01::BytesMut) instance.
  ///
  /// Creates a new `BytesMut` instance containing a copy of all available bytes in the buffer.
  /// The original buffer remains unchanged.
  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
  )]
  #[allow(clippy::wrong_self_convention)]
  fn to_smol_bytes_mut(&self) -> ::smol_bytes_01::BytesMut {
    ::smol_bytes_01::BytesMut::from(self.to_smol_bytes())
  }
}

/// Extension trait for `Chunk` that provides additional methods
pub trait ChunkExt: Chunk {
  /// Peeks a fixed-size array from the beginning of the buffer without advancing the cursor.
  ///
  /// This method creates a copy of the first `N` bytes from the buffer without
  /// consuming them. The buffer position remains unchanged after this operation.
  ///
  /// # Panics
  ///
  /// Panics if the buffer contains fewer than `N` bytes.
  /// Use [`peek_array_checked`](Chunk::peek_array_checked) or
  /// [`try_peek_array`](Chunk::try_peek_array) for non-panicking peeks.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  ///
  /// let first_three: [u8; 3] = buf.peek_array();
  /// assert_eq!(first_three, [1, 2, 3]);
  /// // Buffer unchanged
  /// assert_eq!(buf.remaining(), 5);
  /// ```
  #[inline]
  fn peek_array<const N: usize>(&self) -> [u8; N] {
    peek_array::<_, N>(self)
  }

  /// Peeks a fixed-size array from the beginning of the buffer without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_array`](Chunk::peek_array).
  /// Returns `Some(array)` if sufficient data is available, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3];
  /// let buf = &data[..];
  ///
  /// assert!(buf.peek_array_checked::<2>().is_some());
  /// assert!(buf.peek_array_checked::<5>().is_none());
  /// ```
  #[inline]
  fn peek_array_checked<const N: usize>(&self) -> Option<[u8; N]> {
    peek_array_checked::<_, N>(self)
  }

  /// Peeks a fixed-size array from the beginning of the buffer without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_array`](Chunk::peek_array) that
  /// returns detailed error information on failure.
  /// Returns `Ok(array)` on success, or `Err(TryPeekError)` with details about
  /// requested vs available bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3];
  /// let buf = &data[..];
  ///
  /// assert!(buf.try_peek_array::<2>().is_ok());
  ///
  /// let err = buf.try_peek_array::<5>().unwrap_err();
  /// // err contains details about requested vs available
  /// ```
  #[inline]
  fn try_peek_array<const N: usize>(&self) -> Result<[u8; N], TryPeekError> {
    try_peek_array::<_, N>(self)
  }

  /// Peeks a fixed-size array from the buffer at the specified offset without advancing the cursor.
  ///
  /// This method creates a copy of `N` bytes from the buffer starting at the given offset
  /// without consuming them. The buffer position remains unchanged after this operation.
  ///
  /// # Panics
  ///
  /// Panics if `offset + N > self.remaining()`.
  /// Use [`peek_array_at_checked`](ChunkExt::peek_array_at_checked) or
  /// [`try_peek_array_at`](ChunkExt::try_peek_array_at) for non-panicking peeks.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3, 4, 5, 6, 7, 8];
  /// let buf = &data[..];
  ///
  /// let array_at_2: [u8; 3] = buf.peek_array_at(2);
  /// assert_eq!(array_at_2, [3, 4, 5]);
  /// // Buffer unchanged
  /// assert_eq!(buf.remaining(), 8);
  /// ```
  #[inline]
  fn peek_array_at<const N: usize>(&self, offset: usize) -> [u8; N] {
    peek_array_at::<_, N>(self, offset)
  }

  /// Peeks a fixed-size array from the buffer at the specified offset without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_array_at`](ChunkExt::peek_array_at).
  /// Returns `Some(array)` if sufficient data is available at the offset, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  ///
  /// assert!(buf.peek_array_at_checked::<3>(1).is_some());
  /// assert!(buf.peek_array_at_checked::<3>(4).is_none()); // Not enough bytes
  /// ```
  #[inline]
  fn peek_array_at_checked<const N: usize>(&self, offset: usize) -> Option<[u8; N]> {
    peek_array_at_checked::<_, N>(self, offset)
  }

  /// Peeks a fixed-size array from the buffer at the specified offset without advancing the cursor.
  ///
  /// This is the non-panicking version of [`peek_array_at`](ChunkExt::peek_array_at) that
  /// returns detailed error information on failure.
  /// Returns `Ok(array)` on success, or `Err(TryPeekAtError)` with details about
  /// the error condition.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  ///
  /// assert!(buf.try_peek_array_at::<3>(1).is_ok());
  ///
  /// let err = buf.try_peek_array_at::<3>(4).unwrap_err();
  /// let err = buf.try_peek_array_at::<3>(6).unwrap_err();
  /// // err contains details about the error
  /// ```
  #[inline]
  fn try_peek_array_at<const N: usize>(&self, offset: usize) -> Result<[u8; N], TryPeekAtError> {
    try_peek_array_at::<_, N>(self, offset)
  }

  /// Reads a fixed-size array from the buffer and advances the internal cursor.
  ///
  /// This method creates a copy of the first `N` bytes from the buffer and
  /// advances the cursor by `N` bytes, consuming the data.
  ///
  /// # Panics
  ///
  /// Panics if the buffer contains fewer than `N` bytes.
  /// Use [`read_array_checked`](Chunk::read_array_checked) or
  /// [`try_read_array`](Chunk::try_read_array) for non-panicking reads.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let mut buf = &data[..];
  ///
  /// let first_three: [u8; 3] = buf.read_array();
  /// assert_eq!(first_three, [1, 2, 3]);
  /// assert_eq!(buf.remaining(), 2); // Cursor advanced
  /// ```
  #[inline]
  fn read_array<const N: usize>(&mut self) -> [u8; N] {
    let output = self.peek_array::<N>();
    self.advance(N);
    output
  }

  /// Reads a fixed-size array from the buffer and advances the internal cursor.
  ///
  /// This is the non-panicking version of [`read_array`](Chunk::read_array).
  /// Returns `Some(array)` and advances the cursor on success, or `None` if insufficient data.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3];
  /// let mut buf = &data[..];
  ///
  /// assert!(buf.read_array_checked::<2>().is_some());
  /// assert_eq!(buf.remaining(), 1);
  ///
  /// assert!(buf.read_array_checked::<2>().is_none());
  /// assert_eq!(buf.remaining(), 1); // Cursor not advanced on failure
  /// ```
  #[inline]
  fn read_array_checked<const N: usize>(&mut self) -> Option<[u8; N]> {
    self.peek_array_checked::<N>().inspect(|_| {
      self.advance(N);
    })
  }

  /// Reads a fixed-size array from the buffer and advances the internal cursor.
  ///
  /// This is the non-panicking version of [`read_array`](Chunk::read_array) that
  /// returns detailed error information on failure.
  /// Returns `Ok(array)` and advances the cursor on success, or `Err(TryReadError)`
  /// with details about requested vs available bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, ChunkExt};
  ///
  /// let data = [1, 2, 3];
  /// let mut buf = &data[..];
  ///
  /// assert!(buf.try_read_array::<2>().is_ok());
  /// assert_eq!(buf.remaining(), 1);
  ///
  /// let err = buf.try_read_array::<2>().unwrap_err();
  /// // err contains details about requested vs available
  /// ```
  #[inline]
  fn try_read_array<const N: usize>(&mut self) -> Result<[u8; N], TryReadError> {
    self
      .try_peek_array::<N>()
      .inspect(|_| {
        self.advance(N);
      })
      .map_err(Into::into)
  }

  /// Peeks a variable-length encoded type from the buffer without advancing the internal cursor.
  ///
  /// Returns the length of the value and the value itself.
  #[inline]
  fn peek_varint<V: Varint>(&self) -> Result<(NonZeroUsize, V), DecodeVarintError> {
    V::decode(self.buffer())
  }

  /// Reads a variable-length encoded type from the buffer and advances the internal cursor.
  ///
  /// Returns the length of the value read and the value itself.
  #[inline]
  fn read_varint<V: Varint>(&mut self) -> Result<(NonZeroUsize, V), DecodeVarintError> {
    V::decode(self.buffer()).inspect(|(len, _)| {
      debug_assert!(len.get() <= self.remaining());
      self.advance(len.get());
    })
  }

  /// Skips a variable-length encoded type in the buffer without advancing the internal cursor.
  ///
  /// In varint encoding, each byte uses 7 bits for the value and the highest bit (MSB)
  /// as a continuation flag. A set MSB (1) indicates more bytes follow, while an unset MSB (0)
  /// marks the last byte of the varint.
  ///
  /// This is a **structural** scan: it locates the end of the varint from the
  /// continuation bits alone and does **not** validate the value against any specific
  /// integer type's width. A long-but-terminated run is accepted; an unterminated run
  /// yields `InsufficientData`. For type-bounded decoding that rejects values too wide
  /// for the target type, use the typed [`read_varint`](ChunkExt::read_varint) /
  /// [`peek_varint`](ChunkExt::peek_varint) instead.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use buffo::{ChunkExt, Chunk};
  ///
  /// let buf = [0x96, 0x01]; // Varint encoding of 150
  /// let chunk = &buf[..];
  /// assert_eq!(chunk.try_scan_varint().map(|val| val.get()), Ok(2));
  /// assert_eq!(chunk.remaining(), 2); // Cursor not advanced
  ///
  /// let buf = [0x7F]; // Varint encoding of 127
  /// let chunk = &buf[..];
  /// assert_eq!(chunk.try_scan_varint().map(|val| val.get()), Ok(1));
  /// assert_eq!(chunk.remaining(), 1); // Cursor not advanced
  /// ```
  fn try_scan_varint(&self) -> Result<NonZeroUsize, DecodeVarintError> {
    // Structural scan: locate the varint's end via its continuation bits without
    // bounding the value to any integer type's width. Type-bounded rejection is the
    // job of the typed `read_varint`/`peek_varint`.
    varing::try_consume_varint(self.buffer()).map_err(Into::into)
  }

  /// Skips a variable-length encoded type in the buffer at the specified offset without advancing the cursor.
  ///
  /// In varint encoding, each byte uses 7 bits for the value and the highest bit (MSB)
  /// as a continuation flag. A set MSB (1) indicates more bytes follow, while an unset MSB (0)
  /// marks the last byte of the varint.
  ///
  /// This is a **structural** scan: it locates the end of the varint from the
  /// continuation bits alone and does **not** validate the value against any specific
  /// integer type's width. A long-but-terminated run is accepted; an unterminated run
  /// yields `InsufficientData`. For type-bounded decoding that rejects values too wide
  /// for the target type, use the typed [`read_varint`](ChunkExt::read_varint) /
  /// [`peek_varint`](ChunkExt::peek_varint) instead.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use buffo::{ChunkExt, Chunk};
  ///
  /// let buf = [0, 0x96, 0x01]; // Varint encoding of 150
  /// let chunk = &buf[..];
  /// assert_eq!(chunk.try_scan_varint_at(1).map(|val| val.get()), Ok(2));
  /// assert_eq!(chunk.remaining(), 3); // Cursor not advanced
  ///
  /// let buf = [0, 0x7F]; // Varint encoding of 127
  /// let chunk = &buf[..];
  /// assert_eq!(chunk.try_scan_varint_at(1).map(|val| val.get()), Ok(1));
  /// assert_eq!(chunk.remaining(), 2); // Cursor not advanced
  ///
  /// // Out of bounds example
  /// let err = chunk.try_scan_varint_at(10).unwrap_err();
  /// ```
  fn try_scan_varint_at(&self, offset: usize) -> Result<NonZeroUsize, DecodeVarintAtError> {
    self
      .buffer_from_checked(offset)
      .ok_or_else(|| DecodeVarintAtError::out_of_bounds(offset, self.remaining()))
      .and_then(|buf| {
        // Structural scan (see `try_scan_varint`): no width cap.
        varing::try_consume_varint(buf)
          .map_err(|e| DecodeVarintAtError::from_const_varint_error(e, offset))
      })
  }

  /// Skips a variable-length encoded type in the buffer and advances the internal cursor.
  ///
  /// In varint encoding, each byte uses 7 bits for the value and the highest bit (MSB)
  /// as a continuation flag. A set MSB (1) indicates more bytes follow, while an unset MSB (0)
  /// marks the last byte of the varint.
  ///
  /// This is a **structural** scan: it locates the end of the varint from the
  /// continuation bits alone and does **not** validate the value against any specific
  /// integer type's width. A long-but-terminated run is accepted; an unterminated run
  /// yields `InsufficientData`. For type-bounded decoding that rejects values too wide
  /// for the target type, use the typed [`read_varint`](ChunkExt::read_varint) /
  /// [`peek_varint`](ChunkExt::peek_varint) instead.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use buffo::{ChunkExt, Chunk};
  ///
  /// let buf = [0x96, 0x01]; // Varint encoding of 150
  /// let mut chunk = &buf[..];
  /// assert_eq!(chunk.try_consume_varint().map(|val| val.get()), Ok(2));
  /// assert_eq!(chunk.remaining(), 0); // Cursor advanced
  ///
  /// let buf = [0x7F]; // Varint encoding of 127
  /// let mut chunk = &buf[..];
  /// assert_eq!(chunk.try_consume_varint().map(|val| val.get()), Ok(1));
  /// assert_eq!(chunk.remaining(), 0); // Cursor advanced
  /// ```
  fn try_consume_varint(&mut self) -> Result<NonZeroUsize, DecodeVarintError> {
    // Structural scan (see `try_scan_varint`): advance ONLY on `Ok`, so a malformed
    // (unterminated) run leaves the cursor untouched.
    varing::try_consume_varint(self.buffer())
      .inspect(|len| {
        self.advance(len.get());
      })
      .map_err(Into::into)
  }
}

impl<T: Chunk> ChunkExt for T {}

impl EmptyChunk for &[u8] {
  #[inline]
  fn empty() -> Self
  where
    Self: Sized,
  {
    &[]
  }
}

impl Chunk for &[u8] {
  #[inline]
  fn remaining(&self) -> usize {
    <[u8]>::len(self)
  }

  #[inline]
  fn has_remaining(&self) -> bool {
    !<[u8]>::is_empty(self)
  }

  #[inline]
  fn advance(&mut self, cnt: usize) {
    if self.len() < cnt {
      panic_advance(&TryAdvanceError::new(must_non_zero(cnt), self.len()));
    }

    *self = &self[cnt..];
  }

  #[inline]
  fn try_advance(&mut self, cnt: usize) -> Result<(), TryAdvanceError> {
    if self.len() < cnt {
      return Err(TryAdvanceError::new(must_non_zero(cnt), self.len()));
    }

    *self = &self[cnt..];
    Ok(())
  }

  #[inline]
  fn truncate(&mut self, len: usize) {
    if len > self.len() {
      return;
    }

    *self = &self[..len];
  }

  #[inline]
  fn segment(&self, range: impl RangeBounds<usize>) -> Self {
    let len = self.len();

    let begin = match range.start_bound() {
      Bound::Included(&n) => n,
      Bound::Excluded(&n) => n.checked_add(1).expect("out of range"),
      Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
      Bound::Included(&n) => n.checked_add(1).expect("out of range"),
      Bound::Excluded(&n) => n,
      Bound::Unbounded => {
        if begin == 0 {
          return self;
        } else {
          len
        }
      }
    };

    &self[begin..end]
  }

  #[inline]
  fn try_segment(&self, range: impl RangeBounds<usize>) -> Result<Self, TrySegmentError> {
    check_segment(range, self.len()).map(|(begin, end)| &self[begin..end])
  }

  #[inline]
  fn split_off(&mut self, at: usize) -> Self {
    let (left, right) = self.split_at(at);
    *self = left;
    right
  }

  #[inline]
  fn split_off_checked(&mut self, at: usize) -> Option<Self> {
    let (left, right) = self.split_at_checked(at)?;
    *self = left;
    Some(right)
  }

  #[inline]
  fn split_to(&mut self, at: usize) -> Self {
    let (left, right) = self.split_at(at);
    *self = right;
    left
  }

  #[inline]
  fn split_to_checked(&mut self, at: usize) -> Option<Self> {
    let (left, right) = self.split_at_checked(at)?;
    *self = right;
    Some(left)
  }

  #[inline]
  fn try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self
      .split_at_checked(at)
      .ok_or_else(|| OutOfBounds::new(at, self.len()))
      .map(|(left, right)| {
        *self = left;
        right
      })
  }

  #[inline]
  fn try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self
      .split_at_checked(at)
      .ok_or_else(|| OutOfBounds::new(at, self.len()))
      .map(|(left, right)| {
        *self = right;
        left
      })
  }

  #[inline]
  fn buffer(&self) -> &[u8] {
    self
  }

  #[inline]
  fn buffer_from(&self, offset: usize) -> &[u8] {
    &self[offset..]
  }

  #[inline]
  fn buffer_from_checked(&self, offset: usize) -> Option<&[u8]> {
    self.split_at_checked(offset).map(|(_, right)| right)
  }
}

#[cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc")))]
const _: () = {
  use bytes_1::{Buf as _, Bytes};

  macro_rules! read_fixed_specification {
    ($($ty:ident), +$(,)?) => {
      paste::paste! {
        $(
          fn [<read_ $ty _le>](&mut self) -> $ty {
            self.[<get_ $ty _le>]()
          }

          fn [<read_ $ty _le_checked>](&mut self) -> Option<$ty> {
            self.[<try_get_ $ty _le>]().ok()
          }

          fn [<try_read_ $ty _le>](&mut self) -> Result<$ty, TryReadError> {
            self.[<try_get_ $ty _le>]().map_err(Into::into)
          }

          fn [<read_ $ty _be>](&mut self) -> $ty {
            self.[<get_ $ty>]()
          }

          fn [<read_ $ty _be_checked>](&mut self) -> Option<$ty> {
            self.[<try_get_ $ty>]().ok()
          }

          fn [<try_read_ $ty _be>](&mut self) -> Result<$ty, TryReadError> {
            self.[<try_get_ $ty>]().map_err(Into::into)
          }

          fn [<read_ $ty _ne>](&mut self) -> $ty {
            self.[<get_ $ty _ne>]()
          }

          fn [<read_ $ty _ne_checked>](&mut self) -> Option<$ty> {
            self.[<try_get_ $ty _ne>]().ok()
          }

          fn [<try_read_ $ty _ne>](&mut self) -> Result<$ty, TryReadError> {
            self.[<try_get_ $ty _ne>]().map_err(Into::into)
          }
        )*
      }
    };
  }

  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc"))))
  )]
  impl EmptyChunk for Bytes {
    /// ```rust
    /// use buffo::{EmptyChunk, Chunk};
    /// use bytes_1::Bytes;
    ///
    /// let empty = Bytes::empty();
    /// assert_eq!(empty.remaining(), 0);
    /// assert!(!empty.has_remaining());
    /// ```
    #[inline]
    fn empty() -> Self
    where
      Self: Sized,
    {
      Bytes::new()
    }
  }

  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc"))))
  )]
  impl Chunk for Bytes {
    #[inline]
    fn remaining(&self) -> usize {
      self.len()
    }

    #[inline]
    fn has_remaining(&self) -> bool {
      !self.is_empty()
    }

    #[inline]
    fn buffer(&self) -> &[u8] {
      self.as_ref()
    }

    #[inline]
    fn buffer_from(&self, offset: usize) -> &[u8] {
      &self.as_ref()[offset..]
    }

    #[inline]
    fn buffer_from_checked(&self, offset: usize) -> Option<&[u8]> {
      self
        .as_ref()
        .split_at_checked(offset)
        .map(|(_, right)| right)
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
      bytes_1::Buf::advance(self, cnt);
    }

    #[inline]
    fn truncate(&mut self, len: usize) {
      Bytes::truncate(self, len);
    }

    #[inline]
    fn segment(&self, range: impl RangeBounds<usize>) -> Self {
      Bytes::slice(self, range)
    }

    #[inline]
    fn split_off(&mut self, at: usize) -> Self {
      Bytes::split_off(self, at)
    }

    #[inline]
    fn split_to(&mut self, at: usize) -> Self {
      Bytes::split_to(self, at)
    }

    #[inline]
    fn read_u8(&mut self) -> u8 {
      self.get_u8()
    }

    #[inline]
    fn read_u8_checked(&mut self) -> Option<u8> {
      self.try_get_u8().ok()
    }

    #[inline]
    fn try_read_u8(&mut self) -> Result<u8, TryReadError> {
      self.try_get_u8().map_err(Into::into)
    }

    #[inline]
    fn read_i8(&mut self) -> i8 {
      self.get_i8()
    }

    #[inline]
    fn read_i8_checked(&mut self) -> Option<i8> {
      self.try_get_i8().ok()
    }

    #[inline]
    fn try_read_i8(&mut self) -> Result<i8, TryReadError> {
      self.try_get_i8().map_err(Into::into)
    }

    #[cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc")))]
    fn to_bytes(&self) -> ::bytes_1::Bytes {
      self.clone()
    }

    read_fixed_specification!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);
  }
};

// smol-bytes: the heap-capable byte holders (`shared::Bytes`, `compact::Bytes`, `BytesMut`).
// These only exist in smol's `std`/`alloc` tiers, and under those tiers smol re-exports
// `bytes::Buf`, so the read specializations route through `smol_bytes_01::Buf` (NOT buffo's
// own optional `bytes_1`, which may be disabled here).
#[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
const _: () = {
  use smol_bytes_01::{compact, Buf as _, Bytes, BytesMut};

  macro_rules! read_fixed_specification {
    ($($ty:ident), +$(,)?) => {
      paste::paste! {
        $(
          fn [<read_ $ty _le>](&mut self) -> $ty {
            self.[<get_ $ty _le>]()
          }

          fn [<read_ $ty _le_checked>](&mut self) -> Option<$ty> {
            self.[<try_get_ $ty _le>]().ok()
          }

          fn [<try_read_ $ty _le>](&mut self) -> Result<$ty, TryReadError> {
            self.[<try_get_ $ty _le>]().map_err(Into::into)
          }

          fn [<read_ $ty _be>](&mut self) -> $ty {
            self.[<get_ $ty>]()
          }

          fn [<read_ $ty _be_checked>](&mut self) -> Option<$ty> {
            self.[<try_get_ $ty>]().ok()
          }

          fn [<try_read_ $ty _be>](&mut self) -> Result<$ty, TryReadError> {
            self.[<try_get_ $ty>]().map_err(Into::into)
          }

          fn [<read_ $ty _ne>](&mut self) -> $ty {
            self.[<get_ $ty _ne>]()
          }

          fn [<read_ $ty _ne_checked>](&mut self) -> Option<$ty> {
            self.[<try_get_ $ty _ne>]().ok()
          }

          fn [<try_read_ $ty _ne>](&mut self) -> Result<$ty, TryReadError> {
            self.[<try_get_ $ty _ne>]().map_err(Into::into)
          }
        )*
      }
    };
  }

  // The immutable holders (`shared::Bytes` and `compact::Bytes`) share the same `Chunk` body;
  // it is emitted once via this macro and invoked for each type. `remaining`/`buffer` map to
  // the LEN-based accessors, so `buffer().len() == remaining()`. `advance`/`split_off`/
  // `split_to`/`segment` all panic iff the argument exceeds `len`, matching smol's own bounds.
  macro_rules! smol_immutable_chunk {
    ($ty:ty, { $($extra:tt)* }) => {
      #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
      )]
      impl Chunk for $ty {
        #[inline]
        fn remaining(&self) -> usize {
          <$ty>::len(self)
        }

        #[inline]
        fn has_remaining(&self) -> bool {
          !<$ty>::is_empty(self)
        }

        #[inline]
        fn buffer(&self) -> &[u8] {
          <$ty>::as_slice(self)
        }

        #[inline]
        fn advance(&mut self, cnt: usize) {
          smol_bytes_01::Buf::advance(self, cnt);
        }

        #[inline]
        fn truncate(&mut self, len: usize) {
          <$ty>::truncate(self, len);
        }

        #[inline]
        fn segment(&self, range: impl RangeBounds<usize>) -> Self {
          <$ty>::slice(self, range)
        }

        #[inline]
        fn split_off(&mut self, at: usize) -> Self {
          <$ty>::split_off(self, at)
        }

        #[inline]
        fn split_to(&mut self, at: usize) -> Self {
          <$ty>::split_to(self, at)
        }

        #[inline]
        fn read_u8(&mut self) -> u8 {
          self.get_u8()
        }

        #[inline]
        fn read_u8_checked(&mut self) -> Option<u8> {
          self.try_get_u8().ok()
        }

        #[inline]
        fn try_read_u8(&mut self) -> Result<u8, TryReadError> {
          self.try_get_u8().map_err(Into::into)
        }

        #[inline]
        fn read_i8(&mut self) -> i8 {
          self.get_i8()
        }

        #[inline]
        fn read_i8_checked(&mut self) -> Option<i8> {
          self.try_get_i8().ok()
        }

        #[inline]
        fn try_read_i8(&mut self) -> Result<i8, TryReadError> {
          self.try_get_i8().map_err(Into::into)
        }

        #[cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc")))]
        fn to_bytes(&self) -> ::bytes_1::Bytes {
          self.clone().into_bytes()
        }

        $($extra)*

        read_fixed_specification!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);
      }
    };
  }

  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
  )]
  impl EmptyChunk for Bytes {
    /// ```rust
    /// use buffo::{EmptyChunk, Chunk};
    /// use smol_bytes_01::Bytes;
    ///
    /// let empty = Bytes::empty();
    /// assert_eq!(empty.remaining(), 0);
    /// assert!(!empty.has_remaining());
    /// ```
    #[inline]
    fn empty() -> Self
    where
      Self: Sized,
    {
      Bytes::new()
    }
  }

  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
  )]
  impl EmptyChunk for compact::Bytes {
    /// ```rust
    /// use buffo::{EmptyChunk, Chunk};
    /// use smol_bytes_01::compact::Bytes;
    ///
    /// let empty = Bytes::empty();
    /// assert_eq!(empty.remaining(), 0);
    /// assert!(!empty.has_remaining());
    /// ```
    #[inline]
    fn empty() -> Self
    where
      Self: Sized,
    {
      compact::Bytes::new()
    }
  }

  // `shared::Bytes` clones into a `smol_bytes_01::Bytes` for free (it IS that type), so it
  // overrides `to_smol_bytes` to avoid the copy. `compact::Bytes` keeps the default
  // (`copy_from_slice`), which produces a `shared::Bytes` and preserves inline data.
  smol_immutable_chunk!(Bytes, {
    #[inline]
    fn to_smol_bytes(&self) -> ::smol_bytes_01::Bytes {
      self.clone()
    }
  });
  smol_immutable_chunk!(compact::Bytes, {});

  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
  )]
  impl EmptyChunk for BytesMut {
    /// ```rust
    /// use buffo::{EmptyChunk, Chunk};
    /// use smol_bytes_01::BytesMut;
    ///
    /// let empty = BytesMut::empty();
    /// assert_eq!(empty.remaining(), 0);
    /// assert!(!empty.has_remaining());
    /// ```
    #[inline]
    fn empty() -> Self
    where
      Self: Sized,
    {
      BytesMut::new()
    }
  }

  #[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc"))))
  )]
  impl Chunk for BytesMut {
    #[inline]
    fn remaining(&self) -> usize {
      BytesMut::len(self)
    }

    #[inline]
    fn has_remaining(&self) -> bool {
      !BytesMut::is_empty(self)
    }

    #[inline]
    fn buffer(&self) -> &[u8] {
      BytesMut::as_slice(self)
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
      smol_bytes_01::Buf::advance(self, cnt);
    }

    #[inline]
    fn truncate(&mut self, len: usize) {
      BytesMut::truncate(self, len);
    }

    // O(n) copy into an independent buffer, with bounds/panic behaviour delegated to the
    // `&[u8]` impl so it panics iff the range exceeds `len`.
    #[inline]
    fn segment(&self, range: impl RangeBounds<usize>) -> Self {
      let slice = BytesMut::as_slice(self);
      BytesMut::from(<&[u8] as Chunk>::segment(&slice, range))
    }

    // smol's `BytesMut::split_off` panics only at `at > capacity`; the buffo contract is
    // `at > len`. The pre-check restores that contract. `Err(inline)` is the inline-storage
    // case (a small buffer split into a `Buffer`), NOT an error — map it to `BytesMut`.
    #[inline]
    fn split_off(&mut self, at: usize) -> Self {
      check_out_of_bounds("split_off", at, BytesMut::len(self));
      match BytesMut::split_off(self, at) {
        Ok(tail) => tail,
        Err(inline) => BytesMut::from(inline),
      }
    }

    // smol's `BytesMut::split_to` already panics at `at > len`, so no pre-check is needed.
    #[inline]
    fn split_to(&mut self, at: usize) -> Self {
      match BytesMut::split_to(self, at) {
        Ok(head) => head,
        Err(inline) => BytesMut::from(inline),
      }
    }

    #[inline]
    fn read_u8(&mut self) -> u8 {
      self.get_u8()
    }

    #[inline]
    fn read_u8_checked(&mut self) -> Option<u8> {
      self.try_get_u8().ok()
    }

    #[inline]
    fn try_read_u8(&mut self) -> Result<u8, TryReadError> {
      self.try_get_u8().map_err(Into::into)
    }

    #[inline]
    fn read_i8(&mut self) -> i8 {
      self.get_i8()
    }

    #[inline]
    fn read_i8_checked(&mut self) -> Option<i8> {
      self.try_get_i8().ok()
    }

    #[inline]
    fn try_read_i8(&mut self) -> Result<i8, TryReadError> {
      self.try_get_i8().map_err(Into::into)
    }

    read_fixed_specification!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);
  }
};

// smol-bytes: the pure-core `Buffer` (works in no_std + no_alloc). It uses only inherent,
// core-capable methods — in particular the inherent (safe) `Buffer::advance`, NOT
// `bytes::Buf` and NOT the unsafe write-cursor `advance_mut`. No read/write specializations
// are provided, so this impl is identical across core/alloc/std and never touches
// `TryGetError` (whose identity varies in pure core).
#[cfg(feature = "smol_bytes_01")]
const _: () = {
  use smol_bytes_01::Buffer;

  #[cfg_attr(docsrs, doc(cfg(feature = "smol_bytes_01")))]
  impl EmptyChunk for Buffer {
    /// ```rust
    /// use buffo::{EmptyChunk, Chunk};
    /// use smol_bytes_01::Buffer;
    ///
    /// let empty = Buffer::empty();
    /// assert_eq!(empty.remaining(), 0);
    /// assert!(!empty.has_remaining());
    /// ```
    #[inline]
    fn empty() -> Self
    where
      Self: Sized,
    {
      Buffer::new()
    }
  }

  #[cfg_attr(docsrs, doc(cfg(feature = "smol_bytes_01")))]
  impl Chunk for Buffer {
    #[inline]
    fn remaining(&self) -> usize {
      Buffer::remaining(self)
    }

    #[inline]
    fn has_remaining(&self) -> bool {
      !Buffer::is_empty(self)
    }

    #[inline]
    fn buffer(&self) -> &[u8] {
      Buffer::as_slice(self)
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
      Buffer::advance(self, cnt);
    }

    #[inline]
    fn truncate(&mut self, len: usize) {
      Buffer::truncate(self, len);
    }

    #[inline]
    fn segment(&self, range: impl RangeBounds<usize>) -> Self {
      Buffer::slice(self, range)
    }

    #[inline]
    fn split_off(&mut self, at: usize) -> Self {
      Buffer::split_off(self, at)
    }

    #[inline]
    fn split_to(&mut self, at: usize) -> Self {
      Buffer::split_to(self, at)
    }
  }
};

#[inline]
fn check_segment<R: RangeBounds<usize>>(
  range: R,
  len: usize,
) -> Result<(usize, usize), TrySegmentError> {
  let begin = match range.start_bound() {
    Bound::Included(&n) => n,
    Bound::Excluded(&n) => match n.checked_add(1) {
      Some(val) => val,
      None => usize::MAX,
    },
    Bound::Unbounded => 0,
  };

  let end = match range.end_bound() {
    Bound::Included(&n) => match n.checked_add(1) {
      Some(val) => val,
      None => usize::MAX,
    },
    Bound::Excluded(&n) => n,
    Bound::Unbounded => len,
  };

  if begin > len || end > len || begin > end {
    return Err(TrySegmentError::new(begin, end, len));
  }

  Ok((begin, end))
}

#[inline]
fn read_array<B: Chunk + ?Sized, const N: usize>(buf: &mut B) -> [u8; N] {
  let output = peek_array::<B, N>(buf);
  buf.advance(N);
  output
}

#[inline]
fn read_array_checked<B: Chunk + ?Sized, const N: usize>(buf: &mut B) -> Option<[u8; N]> {
  peek_array_checked::<B, N>(buf).inspect(|_| {
    buf.advance(N);
  })
}

#[inline]
fn try_read_array<B: Chunk + ?Sized, const N: usize>(buf: &mut B) -> Result<[u8; N], TryReadError> {
  try_peek_array::<B, N>(buf)
    .inspect(|_| {
      buf.advance(N);
    })
    .map_err(Into::into)
}

#[inline]
fn peek_array<B: Chunk + ?Sized, const N: usize>(buf: &B) -> [u8; N] {
  <[u8; N]>::try_from(&buf.buffer()[..N]).expect("Already checked there are enough bytes")
}

#[inline]
fn peek_array_checked<B: Chunk + ?Sized, const N: usize>(buf: &B) -> Option<[u8; N]> {
  if N == 0 {
    return Some([0u8; N]);
  }

  if buf.remaining() < N {
    None
  } else {
    Some(<[u8; N]>::try_from(&buf.buffer()[..N]).expect("Already checked there are enough bytes"))
  }
}

#[inline]
fn try_peek_array<B: Chunk + ?Sized, const N: usize>(buf: &B) -> Result<[u8; N], TryPeekError> {
  if N == 0 {
    return Ok([0u8; N]);
  }

  if buf.remaining() < N {
    Err(TryPeekError::new(must_non_zero(N), buf.remaining()))
  } else {
    Ok(<[u8; N]>::try_from(&buf.buffer()[..N]).expect("Already checked there are enough bytes"))
  }
}

#[inline]
fn peek_array_at<B: Chunk + ?Sized, const N: usize>(buf: &B, offset: usize) -> [u8; N] {
  buf.buffer_from(offset)[..N].try_into().unwrap()
}

#[inline]
fn peek_array_at_checked<B: Chunk + ?Sized, const N: usize>(
  buf: &B,
  offset: usize,
) -> Option<[u8; N]> {
  match buf.buffer_from_checked(offset) {
    Some(slice) if slice.len() >= N => Some(slice[..N].try_into().unwrap()),
    _ => None,
  }
}

#[inline]
fn try_peek_array_at<B: Chunk + ?Sized, const N: usize>(
  buf: &B,
  offset: usize,
) -> Result<[u8; N], TryPeekAtError> {
  let buffer = buf.buffer();
  let buf_len = buffer.len();

  match buf_len.checked_sub(offset) {
    None => Err(TryPeekAtError::out_of_bounds(offset, buf_len)),
    Some(_) if N == 0 => Ok([0u8; N]),
    Some(remaining) if remaining >= N => Ok(
      buffer[offset..offset + N]
        .try_into()
        .expect("Already checked there are enough bytes"),
    ),
    Some(remaining) => {
      let n = must_non_zero(N);
      let err = TryPeekAtError::insufficient_data_with_requested(remaining, offset, n);
      Err(err)
    }
  }
}

#[inline]
fn check_out_of_bounds(method_name: &'static str, at: usize, remaining: usize) {
  assert!(
    at <= remaining,
    "{method_name} out of bounds: {at} > {remaining}",
  );
}

#[inline]
fn check_segment_range_bounds(begin: usize, end: usize, current_remaining: usize) {
  assert!(
    begin <= end,
    "range start must not be greater than end: {begin} <= {end}",
  );
  assert!(
    end <= current_remaining,
    "range end out of bounds: {end} <= {current_remaining}",
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Wrapper<'a>(&'a [u8]);

  impl Chunk for Wrapper<'_> {
    fn remaining(&self) -> usize {
      self.0.len()
    }

    fn buffer(&self) -> &[u8] {
      self.0
    }

    fn advance(&mut self, cnt: usize) {
      if self.0.len() < cnt {
        panic_advance(&TryAdvanceError::new(must_non_zero(cnt), self.0.len()));
      }

      self.0 = &self.0[cnt..];
    }

    fn segment(&self, range: impl RangeBounds<usize>) -> Self
    where
      Self: Sized,
    {
      let len = self.0.len();
      let (begin, end) = check_segment(range, len).expect("out of range");
      Wrapper(&self.0[begin..end])
    }

    fn truncate(&mut self, len: usize) {
      if len > self.0.len() {
        return;
      }

      self.0 = &self.0[..len];
    }

    fn split_off(&mut self, at: usize) -> Self
    where
      Self: Sized,
    {
      let (left, right) = self.0.split_at(at);
      self.0 = left;
      Wrapper(right)
    }

    fn split_to(&mut self, at: usize) -> Self
    where
      Self: Sized,
    {
      let (left, right) = self.0.split_at(at);
      self.0 = right;
      Wrapper(left)
    }
  }

  #[test]
  #[should_panic]
  fn test_advance_panic() {
    let buf = [0u8; 5];
    let mut slice = &buf[..];
    slice.advance(10);
  }

  #[test]
  fn test_segment_edge() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let output = slice
      .try_segment((Bound::Excluded(1), Bound::Included(3)))
      .unwrap();
    assert_eq!(output.0, &[3, 4]);

    assert!(slice
      .try_segment((Bound::Excluded(usize::MAX), Bound::Included(usize::MAX)))
      .is_err());
  }

  #[test]
  #[should_panic]
  fn test_segment_panic() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let output = slice.segment((Bound::Excluded(1), Bound::Included(3)));
    assert_eq!(output.0, &[3, 4]);

    slice.segment((Bound::Excluded(usize::MAX), Bound::Included(usize::MAX)));
  }

  #[test]
  fn test_blanket_has_remaining() {
    let buf = [0u8; 5];
    let slice = Wrapper(&buf[..]);
    assert!(slice.has_remaining());

    let empty = Wrapper(&[]);
    assert!(!empty.has_remaining());
  }

  #[test]
  fn test_blanket_split_off() {
    let buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&buf[..]);
    let right = slice.split_off(2);
    assert_eq!(slice.0, &[1, 2]);
    assert_eq!(right.0, &[3, 4, 5]);
  }

  #[test]
  fn test_blanket_split_off_checked() {
    let buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&buf[..]);
    let right = slice.split_off_checked(2).unwrap();
    assert_eq!(slice.0, &[1, 2]);
    assert_eq!(right.0, &[3, 4, 5]);

    assert!(slice.split_off_checked(10).is_none());
  }

  #[test]
  fn test_blanket_try_split_off() {
    let buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&buf[..]);
    let right = slice.try_split_off(2).unwrap();
    assert_eq!(slice.0, &[1, 2]);
    assert_eq!(right.0, &[3, 4, 5]);

    assert!(slice.try_split_off(10).is_err());
  }

  #[test]
  fn test_blanket_split_to() {
    let buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&buf[..]);
    let right = slice.split_to(2);
    assert_eq!(slice.0, &[3, 4, 5]);
    assert_eq!(right.0, &[1, 2]);
  }

  #[test]
  fn test_blanket_split_to_checked() {
    let buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&buf[..]);
    let right = slice.split_to_checked(2).unwrap();
    assert_eq!(slice.0, &[3, 4, 5]);
    assert_eq!(right.0, &[1, 2]);

    assert!(slice.split_to_checked(10).is_none());
  }

  #[test]
  fn test_blanket_try_split_to() {
    let buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&buf[..]);
    let right = slice.try_split_to(2).unwrap();
    assert_eq!(slice.0, &[3, 4, 5]);
    assert_eq!(right.0, &[1, 2]);

    assert!(slice.try_split_to(10).is_err());
  }

  #[test]
  fn test_blanket_buffer_from() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    assert_eq!(slice.buffer_from(2), &[3, 4, 5]);
  }

  #[test]
  fn test_blanket_buffer_from_checked() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    assert_eq!(slice.buffer_from_checked(2), Some(&[3, 4, 5][..]));
    assert_eq!(slice.buffer_from_checked(10), None);
  }

  #[test]
  fn test_blanket_peek_array_0() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: [u8; 0] = slice.peek_array();
    assert_eq!(arr, [0u8; 0]);

    let buf = [];
    let slice = Wrapper(&buf[..]);
    let arr: [u8; 0] = slice.peek_array();
    assert_eq!(arr, [0u8; 0]);
  }

  #[test]
  fn test_blanket_peek_checked_array_0() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Option<[u8; 0]> = slice.peek_array_checked();
    assert_eq!(arr, Some([]));
  }

  #[test]
  fn test_blanket_try_peek_array_0() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Result<[u8; 0], TryPeekError> = slice.try_peek_array();
    assert_eq!(arr, Ok([]));
  }

  #[test]
  fn test_blanket_peek_at_array_0() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: [u8; 0] = slice.peek_array_at(2);
    assert_eq!(arr, [0u8; 0]);

    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: [u8; 0] = slice.peek_array_at(5);
    assert_eq!(arr, [0u8; 0]);

    let buf = [];
    let slice = Wrapper(&buf[..]);
    let arr: [u8; 0] = slice.peek_array_at(0);
    assert_eq!(arr, [0u8; 0]);
  }

  #[test]
  fn test_blanket_peek_at_checked_array_0() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Option<[u8; 0]> = slice.peek_array_at_checked(2);
    assert_eq!(arr, Some([]));

    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Option<[u8; 0]> = slice.peek_array_at_checked(5);
    assert_eq!(arr, Some([]));

    let buf = [];
    let slice = Wrapper(&buf[..]);
    let arr: Option<[u8; 0]> = slice.peek_array_at_checked(0);
    assert_eq!(arr, Some([]));

    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Option<[u8; 0]> = slice.peek_array_at_checked(10);
    assert_eq!(arr, None);
  }

  #[test]
  fn test_blanket_try_peek_at_array_0() {
    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Result<[u8; 0], TryPeekAtError> = slice.try_peek_array_at(2);
    assert_eq!(arr, Ok([]));

    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Result<[u8; 0], TryPeekAtError> = slice.try_peek_array_at(5);
    assert_eq!(arr, Ok([]));

    let buf = [];
    let slice = Wrapper(&buf[..]);
    let arr: Result<[u8; 0], TryPeekAtError> = slice.try_peek_array_at(0);
    assert_eq!(arr, Ok([]));

    let buf = [1, 2, 3, 4, 5];
    let slice = Wrapper(&buf[..]);
    let arr: Result<[u8; 0], TryPeekAtError> = slice.try_peek_array_at(10);
    assert!(arr.is_err());
  }

  #[test]
  #[cfg(all(feature = "bytes_1", any(feature = "std", feature = "alloc")))]
  fn test_to_bytes() {
    let slice = Wrapper(&[1, 2, 3, 4, 5]);
    let bytes = slice.to_bytes();
    assert_eq!(bytes, bytes_1::Bytes::from(&[1, 2, 3, 4, 5][..]));
  }

  // Fix 1 (P2): the untyped varint scan is STRUCTURAL — it locates the end of the varint
  // from its continuation bits and does NOT bound the value to any specific integer type's
  // width. A long-but-terminated run is accepted here; type-bounded rejection is the job of
  // the typed `read_varint`/`peek_varint`. (There is no correct finite universal cap: wider
  // `Varint` impls — e.g. ruint/bnum/primitive-types U256/U512 — are reachable via feature
  // unification through the generic `V: Varint` APIs.)
  #[test]
  fn varint_scan_is_structural() {
    // `[0x80; 20] ++ [0x00]` = 21 bytes: a well-formed MSB-terminated run, longer than any
    // fixed-width integer's varint encoding. The structural scan accepts it.
    let mut overlong = [0x80u8; 21];
    overlong[20] = 0x00;

    // `try_scan_varint` accepts it structurally and never advances the cursor.
    let buf = &overlong[..];
    assert_eq!(buf.try_scan_varint().map(|v| v.get()), Ok(21));
    assert_eq!(buf.remaining(), 21);

    // `try_scan_varint_at` accepts it too.
    let buf_at = &overlong[..];
    assert_eq!(buf_at.try_scan_varint_at(0).map(|v| v.get()), Ok(21));

    // `try_consume_varint` accepts it AND advances the cursor by the full structural length.
    let mut buf = &overlong[..];
    assert_eq!(buf.try_consume_varint().map(|v| v.get()), Ok(21));
    assert_eq!(
      buf.remaining(),
      0,
      "consume advances by the structural length"
    );

    // Parity: the TYPED decode REJECTS the same input (too wide for `u64`). This documents
    // the intended structural-vs-typed difference.
    let mut buf = &overlong[..];
    assert!(buf.read_varint::<u64>().is_err());

    // A normal short varint still scans and consumes successfully.
    let valid = [0x96u8, 0x01]; // encodes 150
    let buf = &valid[..];
    assert_eq!(buf.try_scan_varint().map(|v| v.get()), Ok(2));
    assert_eq!(buf.remaining(), 2); // scan does not advance
    let mut buf = &valid[..];
    assert_eq!(buf.try_consume_varint().map(|v| v.get()), Ok(2));
    assert_eq!(buf.remaining(), 0); // consume advances
  }

  // Fix 2 (P2): `try_peek_u8_at`/`try_peek_i8_at` must use the same two-stage taxonomy as
  // every other `*_at` method: `offset > len` is `OutOfBounds`, `offset == len` (in range
  // but no byte to read) is `InsufficientData`.
  #[test]
  fn peek_u8_at_taxonomy_is_two_stage() {
    let buf = &[1u8][..]; // length 1

    // Valid offset -> Ok.
    assert_eq!(buf.try_peek_u8_at(0), Ok(1));

    // offset == len -> InsufficientData (not OutOfBounds).
    assert!(matches!(
      buf.try_peek_u8_at(1),
      Err(TryPeekAtError::InsufficientData(_))
    ));

    // offset > len -> OutOfBounds.
    assert!(matches!(
      buf.try_peek_u8_at(2),
      Err(TryPeekAtError::OutOfBounds(_))
    ));

    // The i8 variant mirrors the u8 taxonomy.
    assert_eq!(buf.try_peek_i8_at(0), Ok(1));
    assert!(matches!(
      buf.try_peek_i8_at(1),
      Err(TryPeekAtError::InsufficientData(_))
    ));
    assert!(matches!(
      buf.try_peek_i8_at(2),
      Err(TryPeekAtError::OutOfBounds(_))
    ));
  }

  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[test]
  fn smol_shared_bytes_chunk() {
    use smol_bytes_01::Bytes;
    // `alloc` is aliased as `std` in the no_std+alloc tier (see `lib.rs`), but the
    // `vec!` macro still needs an explicit import into scope (unlike the real `std`
    // prelude, `no_std` builds do not bring it in automatically).
    use std::vec;

    let mut buf = Bytes::from(vec![1u8, 2, 3, 4, 5]);
    assert_eq!(Chunk::remaining(&buf), 5);
    assert!(Chunk::has_remaining(&buf));
    assert_eq!(Chunk::buffer(&buf), &[1, 2, 3, 4, 5]);
    // coherence: buffer().len() == remaining()
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());

    assert_eq!(buf.read_u8(), 1);
    assert_eq!(buf.read_u16_be(), 0x0203);
    assert_eq!(Chunk::remaining(&buf), 2);
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());
    Chunk::advance(&mut buf, 1);
    assert_eq!(Chunk::buffer(&buf), &[5]);
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());

    let base = Bytes::from(b"hello world".to_vec());
    assert_eq!(Chunk::segment(&base, 0..5).buffer(), b"hello");

    let mut b2 = base.clone();
    let tail = Chunk::split_off(&mut b2, 5);
    assert_eq!(b2.buffer(), b"hello");
    assert_eq!(tail.buffer(), b" world");

    let mut b3 = base.clone();
    let head = Chunk::split_to(&mut b3, 5);
    assert_eq!(head.buffer(), b"hello");
    assert_eq!(b3.buffer(), b" world");

    let mut b4 = base.clone();
    Chunk::truncate(&mut b4, 3);
    assert_eq!(b4.buffer(), b"hel");

    // to_smol_bytes on shared is a cheap clone; conversions round-trip the bytes.
    assert_eq!(base.to_smol_bytes().buffer(), b"hello world");
    assert_eq!(base.to_smol_bytes_mut().buffer(), b"hello world");

    let e = <Bytes as EmptyChunk>::empty();
    assert_eq!(Chunk::remaining(&e), 0);
    assert!(!Chunk::has_remaining(&e));
  }

  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[test]
  fn smol_compact_bytes_chunk() {
    use smol_bytes_01::compact::Bytes;

    let mut buf = Bytes::copy_from_slice([10u8, 20, 30, 40]);
    assert_eq!(Chunk::remaining(&buf), 4);
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());
    assert_eq!(buf.read_u8(), 10);
    assert_eq!(buf.read_u8(), 20);
    assert_eq!(Chunk::buffer(&buf), &[30, 40]);
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());

    let seg = Chunk::segment(&buf, 0..1);
    assert_eq!(seg.buffer(), &[30]);

    let e = <Bytes as EmptyChunk>::empty();
    assert_eq!(Chunk::remaining(&e), 0);
  }

  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[test]
  fn smol_bytes_mut_split_inline_path() {
    use smol_bytes_01::BytesMut;

    // 11 bytes -> inline storage; smol's split returns `Err(Buffer)`, mapped to `BytesMut`.
    let mut b = BytesMut::from(&b"hello world"[..]);
    assert!(b.is_inline());
    let tail = Chunk::split_off(&mut b, 5);
    assert_eq!(Chunk::buffer(&b), b"hello");
    assert_eq!(Chunk::buffer(&tail), b" world");

    let mut b2 = BytesMut::from(&b"hello world"[..]);
    assert!(b2.is_inline());
    let head = Chunk::split_to(&mut b2, 5);
    assert_eq!(Chunk::buffer(&head), b"hello");
    assert_eq!(Chunk::buffer(&b2), b" world");
  }

  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[test]
  fn smol_bytes_mut_split_heap_path() {
    use smol_bytes_01::BytesMut;

    let mut b = BytesMut::with_capacity(128);
    b.extend_from_slice(b"hello world");
    assert!(b.is_heap());
    let tail = Chunk::split_off(&mut b, 5);
    assert_eq!(Chunk::buffer(&b), b"hello");
    assert_eq!(Chunk::buffer(&tail), b" world");

    let mut b2 = BytesMut::with_capacity(128);
    b2.extend_from_slice(b"hello world");
    let head = Chunk::split_to(&mut b2, 5);
    assert_eq!(Chunk::buffer(&head), b"hello");
    assert_eq!(Chunk::buffer(&b2), b" world");
  }

  // Regression: buffo's `Chunk::split_off` must panic when `len < at <= capacity`. smol's own
  // `BytesMut::split_off` would NOT panic there (its bound is capacity); the pre-check in
  // buffo's impl restores the `at > len` contract.
  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[test]
  #[should_panic]
  fn smol_bytes_mut_split_off_beyond_len_panics() {
    use smol_bytes_01::BytesMut;

    let mut b = BytesMut::with_capacity(128);
    b.extend_from_slice(&[1, 2, 3, 4, 5]); // len 5, capacity >= 128
    assert!(b.capacity() >= 10);
    // 10 <= capacity but 10 > len(5): buffo must panic even though smol alone would not.
    let _ = Chunk::split_off(&mut b, 10);
  }

  #[cfg(all(feature = "smol_bytes_01", any(feature = "std", feature = "alloc")))]
  #[test]
  fn smol_bytes_mut_chunk_coherence() {
    use smol_bytes_01::BytesMut;

    let mut b = BytesMut::from(&b"abcdefgh"[..]);
    assert_eq!(Chunk::remaining(&b), Chunk::buffer(&b).len());
    assert_eq!(b.read_u32_be(), 0x61626364);
    assert_eq!(Chunk::remaining(&b), Chunk::buffer(&b).len());
    Chunk::advance(&mut b, 1);
    assert_eq!(Chunk::buffer(&b), b"fgh");
    assert_eq!(Chunk::remaining(&b), Chunk::buffer(&b).len());
    Chunk::truncate(&mut b, 1);
    assert_eq!(Chunk::buffer(&b), b"f");
  }

  #[cfg(feature = "smol_bytes_01")]
  #[test]
  fn smol_buffer_chunk() {
    use smol_bytes_01::Buffer;

    let mut buf = Buffer::try_from(&b"hello world"[..]).unwrap();
    assert_eq!(Chunk::remaining(&buf), 11);
    assert!(Chunk::has_remaining(&buf));
    // coherence: buffer().len() == remaining()
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());

    // read/peek use the trait defaults (Buffer has no specializations, never touches TryGetError)
    assert_eq!(buf.peek_u8(), b'h');
    assert_eq!(buf.read_u8(), b'h');
    assert_eq!(Chunk::buffer(&buf), b"ello world");
    assert_eq!(Chunk::remaining(&buf), Chunk::buffer(&buf).len());

    let seg = Chunk::segment(&buf, 0..4);
    assert_eq!(Chunk::buffer(&seg), b"ello");

    let mut b2 = Buffer::try_from(&b"hello world"[..]).unwrap();
    let tail = Chunk::split_off(&mut b2, 5);
    assert_eq!(Chunk::buffer(&b2), b"hello");
    assert_eq!(Chunk::buffer(&tail), b" world");

    let mut b3 = Buffer::try_from(&b"hello world"[..]).unwrap();
    let head = Chunk::split_to(&mut b3, 5);
    assert_eq!(Chunk::buffer(&head), b"hello");
    assert_eq!(Chunk::buffer(&b3), b" world");

    Chunk::truncate(&mut buf, 3);
    assert_eq!(Chunk::buffer(&buf), b"ell");

    let e = <Buffer as EmptyChunk>::empty();
    assert_eq!(Chunk::remaining(&e), 0);
    assert!(!Chunk::has_remaining(&e));
  }

  // Buffer split_off/split_to panic iff `at > len` (exact match to smol's own bound; no
  // pre-check required, unlike BytesMut).
  #[cfg(feature = "smol_bytes_01")]
  #[test]
  #[should_panic]
  fn smol_buffer_split_off_beyond_len_panics() {
    use smol_bytes_01::Buffer;

    let mut buf = Buffer::try_from(&[1u8, 2, 3][..]).unwrap();
    let _ = Chunk::split_off(&mut buf, 4);
  }
}
