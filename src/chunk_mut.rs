use core::num::NonZeroUsize;

use crate::EmptyChunk;

use super::{
  error::{InsufficientSpace, TryAdvanceError, TryPutAtError},
  must_non_zero, panic_advance,
};

use super::error::{EncodeVarintAtError, EncodeVarintError};
use varing::Varint;

pub use putter::Putter;

mod putter;

macro_rules! put_fixed {
  ($($ty:ty),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Puts a `" $ty "` value in little-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if `offset >= self.remaining_mut()` or if `offset + size_of::<T>() > self.remaining_mut()`.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.put_" $ty "_le_at(0x1234 as " $ty ", 2);"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        /// // Value is written in little-endian format starting at offset 2
        /// ```
        #[inline]
        fn [< put_ $ty _le_at>](&mut self, value: $ty, offset: usize) -> usize {
          self.put_slice_at(&value.to_le_bytes(), offset)
        }

        #[doc = "Tries to put `" $ty "` value in little-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_le_at`](ChunkMut::put_" $ty "_le_at)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if the offset is out of bounds
        /// or there's insufficient space for the value.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.put_" $ty "_le_at_checked(0x1234 as " $ty ", 2).is_some());"]
        #[doc = "assert!(slice.put_" $ty "_le_at_checked(0x1234 as " $ty ", 30).is_none()); // Out of bounds"]
        /// ```
        #[inline]
        fn [< put_ $ty _le_at_checked>](&mut self, value: $ty, offset: usize) -> Option<usize> {
          self.put_slice_at_checked(&value.to_le_bytes(), offset)
        }

        #[doc = "Tries to put `" $ty "` value in little-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_le_at`](ChunkMut::put_" $ty "_le_at)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(TryPutAtError)` with detailed
        /// error information if the offset is out of bounds or there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_put_" $ty "_le_at(0x1234 as " $ty ", 2).is_ok());"]
        ///
        #[doc = "let err = slice.try_put_" $ty "_le_at(0x1234 as " $ty ", 30).unwrap_err();"]
        /// // err contains detailed information about what went wrong
        /// ```
        #[inline]
        fn [< try_put_ $ty _le_at>](&mut self, value: $ty, offset: usize) -> Result<usize, TryPutAtError> {
          self.try_put_slice_at(&value.to_le_bytes(), offset)
        }

        #[doc = "Puts `" $ty "` value in big-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if `offset >= self.remaining_mut()` or if `offset + size_of::<T>() > self.remaining_mut()`.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.put_" $ty "_be_at(0x1234 as " $ty ", 2);"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        /// // Value is written in big-endian format starting at offset 2
        /// ```
        #[inline]
        fn [< put_ $ty _be_at>](&mut self, value: $ty, offset: usize) -> usize {
          self.put_slice_at(&value.to_be_bytes(), offset)
        }

        #[doc = "Tries to put `" $ty "` value in big-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_be_at`](ChunkMut::put_" $ty "_be_at)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if the offset is out of bounds
        /// or there's insufficient space for the value.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.put_" $ty "_be_at_checked(0x1234 as " $ty ", 2).is_some());"]
        #[doc = "assert!(slice.put_" $ty "_be_at_checked(0x1234 as " $ty ", 30).is_none()); // Out of bounds"]
        /// ```
        #[inline]
        fn [< put_ $ty _be_at_checked>](&mut self, value: $ty, offset: usize) -> Option<usize> {
          self.put_slice_at_checked(&value.to_be_bytes(), offset)
        }

        #[doc = "Tries to put `" $ty "` value in big-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_be_at`](ChunkMut::put_" $ty "_be_at)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(TryPutAtError)` with detailed
        /// error information if the offset is out of bounds or there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_put_" $ty "_be_at(0x1234 as " $ty ", 2).is_ok());"]
        ///
        #[doc = "let err = slice.try_put_" $ty "_be_at(0x1234 as " $ty ", 30).unwrap_err();"]
        /// // err contains detailed information about what went wrong
        /// ```
        #[inline]
        fn [< try_put_ $ty _be_at>](&mut self, value: $ty, offset: usize) -> Result<usize, TryPutAtError> {
          self.try_put_slice_at(&value.to_be_bytes(), offset)
        }

        #[doc = "Puts `" $ty "` value in native-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness (little-endian on x86/x64,
        /// big-endian on some embedded platforms).
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if `offset >= self.remaining_mut()` or if `offset + size_of::<T>() > self.remaining_mut()`.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.put_" $ty "_ne_at(0x1234 as " $ty ", 2);"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        /// // Value is written in native-endian format starting at offset 2
        /// ```
        #[inline]
        fn [< put_ $ty _ne_at>](&mut self, value: $ty, offset: usize) -> usize {
          self.put_slice_at(&value.to_ne_bytes(), offset)
        }

        #[doc = "Tries to put `" $ty "` value in native-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`put_" $ty "_ne_at`](ChunkMut::put_" $ty "_ne_at)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if the offset is out of bounds
        /// or there's insufficient space for the value.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.put_" $ty "_ne_at_checked(0x1234 as " $ty ", 2).is_some());"]
        #[doc = "assert!(slice.put_" $ty "_ne_at_checked(0x1234 as " $ty ", 30).is_none()); // Out of bounds"]
        /// ```
        #[inline]
        fn [< put_ $ty _ne_at_checked>](&mut self, value: $ty, offset: usize) -> Option<usize> {
          self.put_slice_at_checked(&value.to_ne_bytes(), offset)
        }

        #[doc = "Tries to put `" $ty "` value in native-endian byte order to the buffer at the specified offset without advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`put_" $ty "_ne_at`](ChunkMut::put_" $ty "_ne_at)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(TryPutAtError)` with detailed
        /// error information if the offset is out of bounds or there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_put_" $ty "_ne_at(0x1234 as " $ty ", 2).is_ok());"]
        ///
        #[doc = "let err = slice.try_put_" $ty "_ne_at(0x1234 as " $ty ", 30).unwrap_err();"]
        /// // err contains detailed information about what went wrong
        /// ```
        #[inline]
        fn [< try_put_ $ty _ne_at>](&mut self, value: $ty, offset: usize) -> Result<usize, TryPutAtError> {
          self.try_put_slice_at(&value.to_ne_bytes(), offset)
        }

        #[doc = "Puts `" $ty "` value in little-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if the buffer has insufficient space to hold the value.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.put_" $ty "_le(0x1234 as " $ty ");"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        /// // Value is written in little-endian format at the beginning
        ///
        /// assert_eq!(slice.remaining_mut(), 24);
        /// ```
        #[inline]
        fn [< put_ $ty _le>](&mut self, value: $ty) -> usize {
          self.put_slice(&value.to_le_bytes())
        }

        #[doc = "Tries to put `" $ty "` value in little-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_le`](ChunkMut::put_" $ty "_le)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.put_" $ty "_le_checked(0x1234 as " $ty ").is_some());"]
        /// assert_eq!(slice.remaining_mut(), 24);
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "assert!(small_slice.put_" $ty "_le_checked(0x1234 as " $ty ").is_none()); // Not enough space"]
        /// ```
        #[inline]
        fn [< put_ $ty _le_checked>](&mut self, value: $ty) -> Option<usize> {
          self.put_slice_checked(&value.to_le_bytes())
        }

        #[doc = "Tries to put `" $ty "` value in little-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_le`](ChunkMut::put_" $ty "_le)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with detailed
        /// error information if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_put_" $ty "_le(0x1234 as " $ty ").is_ok());"]
        /// assert_eq!(slice.remaining_mut(), 24);
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "let err = small_slice.try_put_" $ty "_le(0x1234 as " $ty ").unwrap_err();"]
        /// // err contains information about required vs available space
        /// ```
        #[inline]
        fn [< try_put_ $ty _le>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          self.try_put_slice(&value.to_le_bytes())
        }

        #[doc = "Puts a `" $ty "` value in big-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if the buffer has insufficient space to hold the value.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.put_" $ty "_be(0x1234 as " $ty ");"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        /// // Value is written in big-endian format at the beginning
        ///
        /// assert_eq!(slice.remaining_mut(), 24);
        /// ```
        #[inline]
        fn [< put_ $ty _be>](&mut self, value: $ty) -> usize {
          self.put_slice(&value.to_be_bytes())
        }

        #[doc = "Tries to put `" $ty "` value in big-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_be`](ChunkMut::put_" $ty "_be)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.put_" $ty "_be_checked(0x1234 as " $ty ").is_some());"]
        /// assert_eq!(slice.remaining_mut(), 24);
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "assert!(small_slice.put_" $ty "_be_checked(0x1234 as " $ty ").is_none()); // Not enough space"]
        /// ```
        #[inline]
        fn [< put_ $ty _be_checked>](&mut self, value: $ty) -> Option<usize> {
          self.put_slice_checked(&value.to_be_bytes())
        }

        #[doc = "Tries to put `" $ty "` value in big-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`put_" $ty "_be`](ChunkMut::put_" $ty "_be)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with detailed
        /// error information if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_put_" $ty "_be(0x1234 as " $ty ").is_ok());"]
        /// assert_eq!(slice.remaining_mut(), 24);
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "let err = small_slice.try_put_" $ty "_be(0x1234 as " $ty ").unwrap_err();"]
        /// // err contains information about required vs available space
        /// ```
        #[inline]
        fn [< try_put_ $ty _be>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          self.try_put_slice(&value.to_be_bytes())
        }

        #[doc = "Puts `" $ty "` value in native-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness (little-endian on x86/x64,
        /// big-endian on some embedded platforms).
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if the buffer has insufficient space to hold the value.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.put_" $ty "_ne(0x1234 as " $ty ");"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        /// // Value is written in native-endian format at the beginning
        ///
        /// assert_eq!(slice.remaining_mut(), 24);
        /// ```
        #[inline]
        fn [< put_ $ty _ne>](&mut self, value: $ty) -> usize {
          self.put_slice(&value.to_ne_bytes())
        }

        #[doc = "Tries to put `" $ty "` value in native-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`put_" $ty "_ne`](ChunkMut::put_" $ty "_ne)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.put_" $ty "_ne_checked(0x1234 as " $ty ").is_some());"]
        /// assert_eq!(slice.remaining_mut(), 24);
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "assert!(small_slice.put_" $ty "_ne_checked(0x1234 as " $ty ").is_none()); // Not enough space"]
        /// ```
        #[inline]
        fn [< put_ $ty _ne_checked>](&mut self, value: $ty) -> Option<usize> {
          self.put_slice_checked(&value.to_ne_bytes())
        }

        #[doc = "Tries to put `" $ty "` value in native-endian byte order to the beginning of the buffer without advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`put_" $ty "_ne`](ChunkMut::put_" $ty "_ne)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with detailed
        /// error information if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_put_" $ty "_ne(0x1234 as " $ty ").is_ok());"]
        /// assert_eq!(slice.remaining_mut(), 24);
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "let err = small_slice.try_put_" $ty "_ne(0x1234 as " $ty ").unwrap_err();"]
        /// // err contains information about required vs available space
        /// ```
        #[inline]
        fn [< try_put_ $ty _ne>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          self.try_put_slice(&value.to_ne_bytes())
        }
      )*
    }
  };
  (@forward $($ty:ty),+$(,)?) => {
    paste::paste! {
      $(
        #[inline]
        fn [< put_ $ty _le_at>](&mut self, value: $ty, offset: usize) -> usize {
          (**self).[< put_ $ty _le_at>](value, offset)
        }

        #[inline]
        fn [< put_ $ty _le_at_checked>](&mut self, value: $ty, offset: usize) -> Option<usize> {
          (**self).[< put_ $ty _le_at_checked>](value, offset)
        }

        #[inline]
        fn [< try_put_ $ty _le_at>](&mut self, value: $ty, offset: usize) -> Result<usize, TryPutAtError> {
          (**self).[< try_put_ $ty _le_at>](value, offset)
        }

        #[inline]
        fn [< put_ $ty _be_at>](&mut self, value: $ty, offset: usize) -> usize {
          (**self).[< put_ $ty _be_at>](value, offset)
        }

        #[inline]
        fn [< put_ $ty _be_at_checked>](&mut self, value: $ty, offset: usize) -> Option<usize> {
          (**self).[< put_ $ty _be_at_checked>](value, offset)
        }

        #[inline]
        fn [< try_put_ $ty _be_at>](&mut self, value: $ty, offset: usize) -> Result<usize, TryPutAtError> {
          (**self).[< try_put_ $ty _be_at>](value, offset)
        }

        #[inline]
        fn [< put_ $ty _ne_at>](&mut self, value: $ty, offset: usize) -> usize {
          (**self).[< put_ $ty _ne_at>](value, offset)
        }

        #[inline]
        fn [< put_ $ty _ne_at_checked>](&mut self, value: $ty, offset: usize) -> Option<usize> {
          (**self).[< put_ $ty _ne_at_checked>](value, offset)
        }

        #[inline]
        fn [< try_put_ $ty _ne_at>](&mut self, value: $ty, offset: usize) -> Result<usize, TryPutAtError> {
          (**self).[< try_put_ $ty _ne_at>](value, offset)
        }

        #[inline]
        fn [< put_ $ty _le>](&mut self, value: $ty) -> usize {
          (**self).[< put_ $ty _le>](value)
        }

        #[inline]
        fn [< put_ $ty _le_checked>](&mut self, value: $ty) -> Option<usize> {
          (**self).[< put_ $ty _le_checked>](value)
        }

        #[inline]
        fn [< try_put_ $ty _le>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          (**self).[< try_put_ $ty _le>](value)
        }

        #[inline]
        fn [< put_ $ty _be>](&mut self, value: $ty) -> usize {
          (**self).[< put_ $ty _be>](value)
        }

        #[inline]
        fn [< put_ $ty _be_checked>](&mut self, value: $ty) -> Option<usize> {
          (**self).[< put_ $ty _be_checked>](value)
        }

        #[inline]
        fn [< try_put_ $ty _be>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          (**self).[< try_put_ $ty _be>](value)
        }

        #[inline]
        fn [< put_ $ty _ne>](&mut self, value: $ty) -> usize {
          (**self).[< put_ $ty _ne>](value)
        }

        #[inline]
        fn [< put_ $ty _ne_checked>](&mut self, value: $ty) -> Option<usize> {
          (**self).[< put_ $ty _ne_checked>](value)
        }

        #[inline]
        fn [< try_put_ $ty _ne>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          (**self).[< try_put_ $ty _ne>](value)
        }
      )*
    }
  };
}

macro_rules! write_fixed {
  ($($ty:ty),+$(,)?) => {
    paste::paste! {
      $(
        #[doc = "Writes `" $ty "` value in little-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if the buffer has insufficient space to hold the value.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.write_" $ty "_le(0x1234 as " $ty ");"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        /// // Value is written in little-endian format at the beginning
        /// ```
        #[inline]
        fn [< write_ $ty _le>](&mut self, value: $ty) -> usize {
          self.write_slice(&value.to_le_bytes())
        }

        #[doc = "Tries to write `" $ty "` value in little-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`write_" $ty "_le`](ChunkMut::write_" $ty "_le)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.write_" $ty "_le_checked(0x1234 as " $ty ").is_some());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "assert!(small_slice.write_" $ty "_le_checked(0x1234 as " $ty ").is_none()); // Not enough space"]
        /// ```
        #[inline]
        fn [< write_ $ty _le_checked>](&mut self, value: $ty) -> Option<usize> {
          self.write_slice_checked(&value.to_le_bytes())
        }

        #[doc = "Tries to write `" $ty "` value in little-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`write_" $ty "_le`](ChunkMut::write_" $ty "_le)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with detailed
        /// error information if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_write_" $ty "_le(0x1234 as " $ty ").is_ok());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "let err = small_slice.try_write_" $ty "_le(0x1234 as " $ty ").unwrap_err();"]
        /// // err contains information about required vs available space
        /// ```
        #[inline]
        fn [< try_write_ $ty _le>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          self.try_write_slice(&value.to_le_bytes())
        }

        #[doc = "Writes `" $ty "` value in big-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if the buffer has insufficient space to hold the value.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.write_" $ty "_be(0x1234 as " $ty ");"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        /// // Value is written in big-endian format at the beginning
        /// ```
        #[inline]
        fn [< write_ $ty _be>](&mut self, value: $ty) -> usize {
          self.write_slice(&value.to_be_bytes())
        }

        #[doc = "Tries to write `" $ty "` value in big-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`write_" $ty "_be`](ChunkMut::write_" $ty "_be)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.write_" $ty "_be_checked(0x1234 as " $ty ").is_some());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "assert!(small_slice.write_" $ty "_be_checked(0x1234 as " $ty ").is_none()); // Not enough space"]
        /// ```
        #[inline]
        fn [< write_ $ty _be_checked>](&mut self, value: $ty) -> Option<usize> {
          self.write_slice_checked(&value.to_be_bytes())
        }

        #[doc = "Tries to write `" $ty "` value in big-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        #[doc = "This is the non-panicking version of [`write_" $ty "_be`](ChunkMut::write_" $ty "_be)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with detailed
        /// error information if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_write_" $ty "_be(0x1234 as " $ty ").is_ok());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "let err = small_slice.try_write_" $ty "_be(0x1234 as " $ty ").unwrap_err();"]
        /// // err contains information about required vs available space
        /// ```
        #[inline]
        fn [< try_write_ $ty _be>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          self.try_write_slice(&value.to_be_bytes())
        }

        #[doc = "Writes `" $ty "` value in native-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness (little-endian on x86/x64,
        /// big-endian on some embedded platforms).
        ///
        #[doc = "Returns the number of bytes written (always `size_of::<" $ty ">()` for this type)."]
        ///
        /// # Panics
        ///
        /// Panics if the buffer has insufficient space to hold the value.
        /// Use the `*_checked` or `try_*` variants for non-panicking writes.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "let written = slice.write_" $ty "_ne(0x1234 as " $ty ");"]
        #[doc = "assert_eq!(written, size_of::<" $ty ">());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        /// // Value is written in native-endian format at the beginning
        /// ```
        #[inline]
        fn [< write_ $ty _ne>](&mut self, value: $ty) -> usize {
          self.write_slice(&value.to_ne_bytes())
        }

        #[doc = "Tries to write `" $ty "` value in native-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`write_" $ty "_ne`](ChunkMut::write_" $ty "_ne)."]
        ///
        /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.write_" $ty "_ne_checked(0x1234 as " $ty ").is_some());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "assert!(small_slice.write_" $ty "_ne_checked(0x1234 as " $ty ").is_none()); // Not enough space"]
        /// ```
        #[inline]
        fn [< write_ $ty _ne_checked>](&mut self, value: $ty) -> Option<usize> {
          self.write_slice_checked(&value.to_ne_bytes())
        }

        #[doc = "Tries to write `" $ty "` value in native-endian byte order to the beginning of the buffer, advancing the internal cursor."]
        ///
        /// The byte order depends on the target platform's endianness.
        #[doc = "This is the non-panicking version of [`write_" $ty "_ne`](ChunkMut::write_" $ty "_ne)."]
        ///
        /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with detailed
        /// error information if there's insufficient space.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use bufkit::ChunkMut;
        ///
        /// let mut buf = [0u8; 24];
        /// let mut slice = &mut buf[..];
        #[doc = "assert!(slice.try_write_" $ty "_ne(0x1234 as " $ty ").is_ok());"]
        #[doc = "assert_eq!(slice.remaining_mut(), 24 - size_of::<" $ty ">());"]
        ///
        /// let mut small_buf = [0u8; 1];
        /// let mut small_slice = &mut small_buf[..];
        #[doc = "let err = small_slice.try_write_" $ty "_ne(0x1234 as " $ty ").unwrap_err();"]
        /// // err contains information about required vs available space
        /// ```
        #[inline]
        fn [< try_write_ $ty _ne>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          self.try_write_slice(&value.to_ne_bytes())
        }
      )*
    }
  };
  (@forward $($ty:ty),+$(,)?) => {
    paste::paste! {
      $(
        #[inline]
        fn [< write_ $ty _le>](&mut self, value: $ty) -> usize {
          (**self).[< write_ $ty _le>](value)
        }

        #[inline]
        fn [< write_ $ty _le_checked>](&mut self, value: $ty) -> Option<usize> {
          (**self).[< write_ $ty _le_checked>](value)
        }

        #[inline]
        fn [< try_write_ $ty _le>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          (**self).[< try_write_ $ty _le>](value)
        }

        #[inline]
        fn [< write_ $ty _be>](&mut self, value: $ty) -> usize {
          (**self).[< write_ $ty _be>](value)
        }

        #[inline]
        fn [< write_ $ty _be_checked>](&mut self, value: $ty) -> Option<usize> {
          (**self).[< write_ $ty _be_checked>](value)
        }

        #[inline]
        fn [< try_write_ $ty _be>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          (**self).[< try_write_ $ty _be>](value)
        }

        #[inline]
        fn [< write_ $ty _ne>](&mut self, value: $ty) -> usize {
          (**self).[< write_ $ty _ne>](value)
        }

        #[inline]
        fn [< write_ $ty _ne_checked>](&mut self, value: $ty) -> Option<usize> {
          (**self).[< write_ $ty _ne_checked>](value)
        }

        #[inline]
        fn [< try_write_ $ty _ne>](&mut self, value: $ty) -> Result<usize, InsufficientSpace> {
          (**self).[< try_write_ $ty _ne>](value)
        }
      )*
    }
  };
}

/// A trait for implementing custom buffers that can store and manipulate byte sequences.
///
/// **Implementers Notes:** Implementations should not have any hidden allocation logic.
///
/// **Coherence obligation:** an implementation must keep `buffer_mut().len()` equal to
/// [`remaining_mut()`](ChunkMut::remaining_mut), and neither `buffer_mut()` nor
/// `remaining_mut()` may panic. The non-panicking default methods (the `*_checked` and
/// `try_*` families) rely on this invariant to stay panic-free and correct; an incoherent
/// implementation can make them panic or misbehave.
///
/// This trait provides a comprehensive set of methods for writing data to buffers with different
/// error handling strategies:
/// - **Panicking methods** (e.g., `put_*`): Fast operations that panic on insufficient space
/// - **Checked methods** (e.g., `*_checked`): Return `Option` - `None` on failure, `Some(bytes_written)` on success
/// - **Fallible methods** (e.g., `try_*`): Return `Result` with detailed error information
///
/// # Method Categories
///
/// - **Buffer inspection**: `remaining_mut()`, `has_remaining_mut()`, `buffer()`, `buffer_mut()`
/// - **Buffer manipulation**: `resize()`, `truncate_mut()`, `fill()`
/// - **Slice operations**: `prefix_mut()`, `suffix_mut()`, `split_at_mut()`
/// - **Writing data**: `put_slice()`, `put_u8()`, `put_u16_le()`, etc.
/// - **Writing at offset**: `put_slice_at()`, `put_u8_at()`, `put_u16_le_at()`, etc.
pub trait ChunkMut {
  /// Returns `true` if the buffer has available space for writing.
  ///
  /// This is equivalent to `self.remaining_mut() > 0`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// assert!(ChunkMut::has_remaining_mut(&slice));
  ///
  /// let mut empty: &mut [u8] = &mut [];
  /// assert!(!ChunkMut::has_remaining_mut(&empty));
  /// ```
  fn has_remaining_mut(&self) -> bool {
    self.remaining_mut() > 0
  }

  /// Returns the number of bytes available for writing in the buffer.
  ///
  /// For fixed-size buffers like `&mut [u8]`, this returns the total buffer size.
  /// For growable buffers like `Vec<u8>`, this typically returns the current length.
  ///
  /// **Implementor obligation:** this must equal `buffer_mut().len()` and must not panic.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// assert_eq!(slice.remaining_mut(), 24);
  /// ```
  fn remaining_mut(&self) -> usize;

  /// Shortens the buffer to the specified length, keeping the first `len` bytes.
  ///
  /// If `len` is greater than or equal to the buffer's current length, this has no effect.
  /// This operation cannot fail and will never panic.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  ///
  /// let mut slice = &mut buf[..];
  /// ChunkMut::truncate_mut(&mut slice, 3);
  /// assert_eq!(slice, [1, 2, 3].as_slice());
  ///
  /// // Truncating to a length >= current length has no effect
  /// ChunkMut::truncate_mut(&mut slice, 10);
  /// assert_eq!(slice, [1, 2, 3].as_slice());
  /// ```
  fn truncate_mut(&mut self, new_len: usize);

  /// Returns the entire initialized buffer as a mutable slice.
  ///
  /// This provides direct access to all buffer contents for efficient manipulation.
  /// The returned slice covers all initialized bytes in the buffer.
  ///
  /// **Implementor obligation:** the returned length must equal
  /// [`remaining_mut()`](ChunkMut::remaining_mut), and this method must not panic.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4];
  /// let mut slice = &mut buf[..];
  /// let slice = slice.buffer_mut();
  /// slice[0] = 0xFF;
  /// assert_eq!(buf[0], 0xFF);
  /// ```
  fn buffer_mut(&mut self) -> &mut [u8];

  /// Returns a mutable slice of the buffer starting from the specified offset.
  ///
  /// This is similar to [`buffer_mut`](ChunkMut::buffer_mut) but starts from the given offset
  /// rather than the current cursor position.
  ///
  /// # Panics
  ///
  /// Panics if `offset > self.remaining_mut()`.
  /// Use [`buffer_mut_from_checked`](ChunkMut::buffer_mut_from_checked) for non-panicking access.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut data = [1u8, 2, 3, 4, 5];
  /// let mut buf = &mut data[..];
  ///
  /// let slice = buf.buffer_mut_from(2);
  /// slice[0] = 99; // Modify the buffer
  /// assert_eq!(buf.buffer_mut(), &[1, 2, 99, 4, 5]);
  /// ```
  #[inline]
  fn buffer_mut_from(&mut self, offset: usize) -> &mut [u8] {
    &mut self.buffer_mut()[offset..]
  }

  /// Returns a mutable slice of the buffer starting from the specified offset.
  ///
  /// This is the non-panicking version of [`buffer_mut_from`](ChunkMut::buffer_mut_from).
  /// Returns `Some(slice)` if `offset <= self.remaining_mut()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut data = [1u8, 2, 3, 4, 5];
  /// let mut buf = &mut data[..];
  ///
  /// if let Some(slice) = buf.buffer_mut_from_checked(2) {
  ///     slice[0] = 99;
  /// }
  /// assert_eq!(buf.buffer_mut(), &[1, 2, 99, 4, 5]);
  /// assert!(buf.buffer_mut_from_checked(5).unwrap().is_empty()); // empty buffer
  /// assert!(buf.buffer_mut_from_checked(10).is_none()); // Out of bounds
  /// ```
  #[inline]
  fn buffer_mut_from_checked(&mut self, offset: usize) -> Option<&mut [u8]> {
    if offset > self.remaining_mut() {
      None
    } else {
      Some(&mut self.buffer_mut()[offset..])
    }
  }

  /// Advances the internal cursor by the specified number of bytes.
  ///
  /// This moves the write cursor forward, marking the skipped bytes as already
  /// filled so that subsequent writes begin after them. The bytes are reserved
  /// without being written to.
  ///
  /// **Implementor obligation:** advancing by `cnt` must reduce `remaining_mut()` by
  /// exactly `cnt` (and shrink `buffer_mut()` to match), preserving the coherence
  /// invariant.
  ///
  /// # Panics
  ///
  /// Panics if `cnt > self.remaining_mut()`.
  /// Use [`try_advance_mut`](ChunkMut::try_advance_mut) for non-panicking advancement.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut data = [1, 2, 3, 4, 5];
  /// let mut buf = &mut data[..];
  ///
  /// buf.advance_mut(2);
  /// assert_eq!(buf.remaining_mut(), 3);
  /// assert_eq!(buf.buffer_mut(), &[3, 4, 5]);
  /// ```
  fn advance_mut(&mut self, cnt: usize);

  /// Attempts to advance the internal cursor by the specified number of bytes.
  ///
  /// This is the non-panicking version of [`advance_mut`](ChunkMut::advance_mut).
  /// Returns `Ok(())` if the advancement was successful, or `Err(TryAdvanceError)`
  /// with details about requested vs available bytes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut data = [1, 2, 3, 4, 5];
  /// let mut buf = &mut data[..];
  ///
  /// assert!(buf.try_advance_mut(3).is_ok());
  /// assert_eq!(buf.remaining_mut(), 2);
  ///
  /// let err = buf.try_advance_mut(5).unwrap_err();
  /// // err contains details about requested vs available
  /// ```
  fn try_advance_mut(&mut self, cnt: usize) -> Result<(), TryAdvanceError> {
    let remaining = self.remaining_mut();
    if remaining < cnt {
      return Err(TryAdvanceError::new(must_non_zero(cnt), remaining));
    }

    self.advance_mut(cnt);
    Ok(())
  }

  /// Fills the entire buffer with the specified byte value.
  ///
  /// This overwrites all bytes in the buffer with `value`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4];
  /// let mut slice = &mut buf[..];
  /// slice.fill(0xFF);
  /// assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
  /// ```
  fn fill(&mut self, value: u8) {
    self.buffer_mut().fill(value);
  }

  /// Returns a mutable slice containing the first `len` bytes of the buffer.
  ///
  /// This provides access to a prefix of the buffer for efficient manipulation
  /// of a specific portion without affecting the rest of the buffer.
  ///
  /// # Panics
  ///
  /// Panics if `len > self.remaining_mut()`.
  /// Use [`prefix_mut_checked`](ChunkMut::prefix_mut_checked) for non-panicking access.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  /// let mut slice = &mut buf[..];
  /// let prefix = slice.prefix_mut(3);
  /// prefix.fill(0xFF);
  /// assert_eq!(buf, [0xFF, 0xFF, 0xFF, 4, 5]);
  /// ```
  fn prefix_mut(&mut self, len: usize) -> &mut [u8] {
    &mut self.buffer_mut()[..len]
  }

  /// Returns a mutable slice containing the first `len` bytes of the buffer.
  ///
  /// This is the non-panicking version of [`prefix_mut`](ChunkMut::prefix_mut).
  /// Returns `Some(slice)` if `len <= self.remaining_mut()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.prefix_mut_checked(3).unwrap(), &[1, 2, 3]);
  /// assert_eq!(slice.prefix_mut_checked(5).unwrap(), &[1, 2, 3, 4, 5]);
  /// assert!(slice.prefix_mut_checked(10).is_none());
  /// ```
  fn prefix_mut_checked(&mut self, len: usize) -> Option<&mut [u8]> {
    if self.remaining_mut() < len {
      None
    } else {
      Some(&mut self.buffer_mut()[..len])
    }
  }

  /// Returns a mutable slice containing the last `len` bytes of the buffer.
  ///
  /// This provides access to a suffix of the buffer for efficient manipulation
  /// of the trailing portion without affecting the rest of the buffer.
  ///
  /// # Panics
  ///
  /// Panics if `len > self.remaining_mut()`.
  /// Use [`suffix_mut_checked`](ChunkMut::suffix_mut_checked) for non-panicking access.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  /// let mut slice = &mut buf[..];
  /// let suffix = slice.suffix_mut(2);
  /// suffix.fill(0xFF);
  /// assert_eq!(buf, [1, 2, 3, 0xFF, 0xFF]);
  /// ```
  fn suffix_mut(&mut self, len: usize) -> &mut [u8] {
    let total_len = self.remaining_mut();
    &mut self.buffer_mut()[total_len - len..]
  }

  /// Returns a mutable slice containing the last `len` bytes of the buffer.
  ///
  /// This is the non-panicking version of [`suffix_mut`](ChunkMut::suffix_mut).
  /// Returns `Some(slice)` if `len <= self.remaining_mut()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.suffix_mut_checked(2).unwrap(), &[4, 5]);
  /// assert_eq!(slice.suffix_mut_checked(5).unwrap(), &[1, 2, 3, 4, 5]);
  /// assert!(slice.suffix_mut_checked(10).is_none());
  /// ```
  fn suffix_mut_checked(&mut self, len: usize) -> Option<&mut [u8]> {
    self
      .remaining_mut()
      .checked_sub(len)
      .map(|start| &mut self.buffer_mut()[start..])
  }

  /// Divides the buffer into two mutable slices at the given index.
  ///
  /// Returns a tuple where the first slice contains indices `[0, mid)` and
  /// the second slice contains indices `[mid, len)`.
  ///
  /// # Panics
  ///
  /// Panics if `mid > self.remaining_mut()`.
  /// Use [`split_at_mut_checked`](ChunkMut::split_at_mut_checked) for non-panicking splitting.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  /// let mut slice = &mut buf[..];
  /// let (left, right) = slice.split_at_mut(2);
  /// assert_eq!(left, &[1, 2]);
  /// assert_eq!(right, &[3, 4, 5]);
  /// ```
  fn split_at_mut(&mut self, mid: usize) -> (&mut [u8], &mut [u8]) {
    self.buffer_mut().split_at_mut(mid)
  }

  /// Divides the buffer into two mutable slices at the given index.
  ///
  /// This is the non-panicking version of [`split_at_mut`](ChunkMut::split_at_mut).
  /// Returns `Some((left, right))` if `mid <= self.remaining_mut()`, otherwise returns `None`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [1, 2, 3, 4, 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.split_at_mut_checked(2).is_some());
  /// assert!(slice.split_at_mut_checked(10).is_none());
  /// ```
  fn split_at_mut_checked(&mut self, mid: usize) -> Option<(&mut [u8], &mut [u8])> {
    self.buffer_mut().split_at_mut_checked(mid)
  }

  /// Writes slice of bytes to the beginning of the buffer and advances the internal cursor.
  ///
  /// Copies all bytes from `slice` into the buffer starting at the beginning.
  /// Returns the number of bytes written (always equal to `slice.len()`).
  ///
  /// # Panics
  ///
  /// Panics if `slice.len() > self.remaining_mut()`.
  /// Use [`write_slice_checked`](ChunkMut::write_slice_checked) or
  /// [`try_write_slice`](ChunkMut::try_write_slice) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.write_slice(&[1, 2, 3]);
  /// assert_eq!(written, 3);
  /// assert_eq!(&buf[..3], &[1, 2, 3]);
  /// ```
  fn write_slice(&mut self, slice: &[u8]) -> usize {
    let len = slice.len();
    self.buffer_mut()[..len].copy_from_slice(slice);
    self.advance_mut(len);
    len
  }

  /// Tries to write slice of bytes to the beginning of the buffer and advance the internal cursor.
  ///
  /// This is the non-panicking version of [`write_slice`](ChunkMut::write_slice).
  /// Returns `Some(bytes_written)` on success, or `None` if the buffer is too small.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.write_slice_checked(&[1, 2, 3]), Some(3));
  /// assert_eq!(slice.write_slice_checked(&[1, 2, 3, 4, 5, 6]), None);
  /// ```
  fn write_slice_checked(&mut self, slice: &[u8]) -> Option<usize> {
    let len = slice.len();
    if len == 0 {
      return Some(0);
    }
    if len <= self.remaining_mut() {
      self.buffer_mut()[..len].copy_from_slice(slice);
      self.advance_mut(len);
      Some(len)
    } else {
      None
    }
  }

  /// Tries to write slice of bytes to the beginning of the buffer and advance the internal cursor.
  ///
  /// This is the non-panicking version of [`write_slice`](ChunkMut::write_slice) that
  /// returns detailed error information on failure.
  /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with details about
  /// the attempted write size and available space.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_write_slice(&[1, 2, 3]).is_ok());
  ///
  /// let err = slice.try_write_slice(&[1, 2, 3, 4, 5, 6]).unwrap_err();
  /// // err contains details about requested vs available space
  /// ```
  fn try_write_slice(&mut self, slice: &[u8]) -> Result<usize, InsufficientSpace> {
    let len = slice.len();
    if len == 0 {
      return Ok(0);
    }

    let space = self.remaining_mut();
    if len <= space {
      self.buffer_mut()[..len].copy_from_slice(slice);
      self.advance_mut(len);
      Ok(len)
    } else {
      Err(InsufficientSpace::new(must_non_zero(len), space))
    }
  }

  write_fixed!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

  /// Writes `u8` value to the beginning of the buffer, advancing the internal cursor.
  ///
  /// Returns the number of bytes written (always `1` for this type).
  ///
  /// # Panics
  ///
  /// Panics if the buffer has no space available.
  /// Use [`write_u8_checked`](ChunkMut::write_u8_checked) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  /// let written = slice.write_u8(0xFF);
  /// assert_eq!(written, 1);
  /// assert_eq!(slice.remaining_mut(), 4);
  /// assert_eq!(buf[0], 0xFF);
  /// ```
  #[inline]
  fn write_u8(&mut self, value: u8) -> usize {
    self.write_slice(&[value])
  }

  /// Tries to write `u8` value to the beginning of the buffer, advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`write_u8`](ChunkMut::write_u8).
  /// Returns `Some(1)` on success, or `None` if the buffer has no space.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 1];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.write_u8_checked(0xFF), Some(1));
  /// assert_eq!(slice.remaining_mut(), 0);
  ///
  /// let mut empty = &mut [][..];
  /// assert_eq!(empty.write_u8_checked(0xFF), None);
  /// ```
  #[inline]
  fn write_u8_checked(&mut self, value: u8) -> Option<usize> {
    self.write_slice_checked(&[value])
  }

  /// Writes `i8` value to the beginning of the buffer, advancing the internal cursor.
  ///
  /// Returns the number of bytes written (always `1` for this type).
  ///
  /// # Panics
  ///
  /// Panics if the buffer has no space available.
  /// Use [`write_i8_checked`](ChunkMut::write_i8_checked) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  /// let written = slice.write_i8(-42);
  /// assert_eq!(written, 1);
  /// assert_eq!(slice.remaining_mut(), 4);
  /// assert_eq!(buf[0], 214); // -42 as u8 is
  /// ```
  #[inline]
  fn write_i8(&mut self, value: i8) -> usize {
    self.write_slice(&[value as u8])
  }

  /// Tries to write `i8` value to the beginning of the buffer, advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`write_i8`](ChunkMut::write_i8).
  /// Returns `Some(1)` on success, or `None` if the buffer has no space.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 1];
  ///
  /// let mut slice = &mut buf[..];
  /// assert_eq!(slice.write_i8_checked(-42), Some(1));
  /// assert_eq!(slice.remaining_mut(), 0);
  /// let mut empty = &mut [][..];
  /// assert_eq!(empty.write_i8_checked(-42), None);
  /// ```
  #[inline]
  fn write_i8_checked(&mut self, value: i8) -> Option<usize> {
    self.write_slice_checked(&[value as u8])
  }

  /// Tries to write `u8` value to the beginning of the buffer, advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`write_u8`](ChunkMut::write_u8) that
  /// returns detailed error information on failure.
  /// Returns `Ok(1)` on success, or `Err(InsufficientSpace)` with details about
  /// the available space if the buffer is full.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_write_u8(0xFF).is_ok());
  /// assert_eq!(slice.remaining_mut(), 4);
  ///
  /// let mut empty = &mut [][..];
  /// let err = empty.try_write_u8(0xFF).unwrap_err();
  /// // err contains details about requested vs available space
  /// ```
  #[inline]
  fn try_write_u8(&mut self, value: u8) -> Result<usize, InsufficientSpace> {
    self.try_write_slice(&[value])
  }

  /// Tries to write `i8` value to the beginning of the buffer, advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`write_i8`](ChunkMut::write_i8) that
  /// returns detailed error information on failure.
  /// Returns `Ok(1)` on success, or `Err(InsufficientSpace)` with details about
  /// the available space if the buffer is full.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_write_i8(-42).is_ok());
  /// assert_eq!(slice.remaining_mut(), 4);
  ///
  /// let mut empty = &mut [][..];
  /// let err = empty.try_write_i8(-42).unwrap_err();
  /// // err contains details about requested vs available space
  /// ```
  #[inline]
  fn try_write_i8(&mut self, value: i8) -> Result<usize, InsufficientSpace> {
    self.try_write_slice(&[value as u8])
  }

  /// Puts slice of bytes to the beginning of the buffer without advancing the internal cursor.
  ///
  /// Copies all bytes from `slice` into the buffer starting at the beginning.
  /// Returns the number of bytes written (always equal to `slice.len()`).
  ///
  /// # Panics
  ///
  /// Panics if `slice.len() > self.remaining_mut()`.
  /// Use [`put_slice_checked`](ChunkMut::put_slice_checked) or
  /// [`try_put_slice`](ChunkMut::try_put_slice) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_slice(&[1, 2, 3]);
  /// assert_eq!(written, 3);
  /// assert_eq!(&buf[..3], &[1, 2, 3]);
  /// ```
  fn put_slice(&mut self, slice: &[u8]) -> usize {
    let len = slice.len();
    self.buffer_mut()[..len].copy_from_slice(slice);
    len
  }

  /// Tries to put slice of bytes to the beginning of the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_slice`](ChunkMut::put_slice).
  /// Returns `Some(bytes_written)` on success, or `None` if the buffer is too small.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.put_slice_checked(&[1, 2, 3]), Some(3));
  /// assert_eq!(slice.put_slice_checked(&[1, 2, 3, 4, 5, 6]), None);
  /// ```
  fn put_slice_checked(&mut self, slice: &[u8]) -> Option<usize> {
    let len = slice.len();
    if len == 0 {
      return Some(0);
    }
    if len <= self.remaining_mut() {
      self.buffer_mut()[..len].copy_from_slice(slice);
      Some(len)
    } else {
      None
    }
  }

  /// Tries to put slice of bytes to the beginning of the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_slice`](ChunkMut::put_slice) that
  /// returns detailed error information on failure.
  /// Returns `Ok(bytes_written)` on success, or `Err(InsufficientSpace)` with details about
  /// the attempted write size and available space.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_put_slice(&[1, 2, 3]).is_ok());
  ///
  /// let err = slice.try_put_slice(&[1, 2, 3, 4, 5, 6]).unwrap_err();
  /// // err contains details about requested vs available space
  /// ```
  fn try_put_slice(&mut self, slice: &[u8]) -> Result<usize, InsufficientSpace> {
    let len = slice.len();
    if len == 0 {
      return Ok(0);
    }

    let space = self.remaining_mut();
    if len <= space {
      self.buffer_mut()[..len].copy_from_slice(slice);
      Ok(len)
    } else {
      Err(InsufficientSpace::new(must_non_zero(len), space))
    }
  }

  /// Puts slice of bytes to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// Copies all bytes from `slice` into the buffer starting at the given `offset`.
  /// Returns the number of bytes written (always equal to `slice.len()`).
  ///
  /// # Panics
  ///
  /// Panics if `offset + slice.len() > self.remaining_mut()`.
  /// Use [`put_slice_at_checked`](ChunkMut::put_slice_at_checked) or
  /// [`try_put_slice_at`](ChunkMut::try_put_slice_at) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_slice_at(&[1, 2, 3], 2);
  /// assert_eq!(written, 3);
  /// assert_eq!(&buf[2..5], &[1, 2, 3]);
  /// ```
  fn put_slice_at(&mut self, slice: &[u8], offset: usize) -> usize {
    let len = slice.len();
    self.buffer_mut()[offset..offset + len].copy_from_slice(slice);
    len
  }

  /// Tries to put slice of bytes to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_slice_at`](ChunkMut::put_slice_at).
  /// Returns `Some(bytes_written)` on success, or `None` if there's insufficient space
  /// or the offset is out of bounds.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.put_slice_at_checked(&[1, 2], 3), Some(2));
  /// assert_eq!(slice.put_slice_at_checked(&[1, 2, 3, 4, 5], 30), None); // Not enough space
  /// assert_eq!(slice.put_slice_at_checked(&[1], 30), None); // Offset out of bounds
  /// ```
  fn put_slice_at_checked(&mut self, slice: &[u8], offset: usize) -> Option<usize> {
    let len = slice.len();
    let space = self.remaining_mut();
    if len == 0 {
      // Match `try_put_slice_at`: a zero-length write validates only the offset
      // (`offset <= space`) and never indexes `buffer_mut()`, so it cannot panic
      // on an incoherent impl at the `offset == space` boundary.
      return if offset > space { None } else { Some(0) };
    }
    if offset <= space && len <= space - offset {
      self.buffer_mut()[offset..offset + len].copy_from_slice(slice);
      Some(len)
    } else {
      None
    }
  }

  /// Tries to put slice of bytes to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_slice_at`](ChunkMut::put_slice_at) that
  /// returns detailed error information on failure.
  /// Returns `Ok(bytes_written)` on success, or `Err(TryPutAtError)` with details about
  /// what went wrong (out of bounds offset, insufficient space, etc.).
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_put_slice_at(&[1, 2], 3).is_ok());
  ///
  /// let err = slice.try_put_slice_at(&[1, 2, 3, 4, 5], 30).unwrap_err();
  /// let err = slice.try_put_slice_at(&[1, 2, 3, 4, 5], 20).unwrap_err();
  /// // err contains detailed information about the failure
  /// ```
  fn try_put_slice_at(&mut self, slice: &[u8], offset: usize) -> Result<usize, TryPutAtError> {
    let len = slice.len();
    let space = self.remaining_mut();

    if len == 0 {
      // For empty slices, still validate the offset:
      // allow `offset` up to and including `space`, but error on `offset > space`.
      if offset > space {
        return Err(TryPutAtError::out_of_bounds(offset, space));
      }
      return Ok(0);
    }
    if offset >= space {
      return Err(TryPutAtError::out_of_bounds(offset, space));
    }

    if len <= space - offset {
      self.buffer_mut()[offset..offset + len].copy_from_slice(slice);
      Ok(len)
    } else {
      Err(TryPutAtError::insufficient_space(
        must_non_zero(len),
        space - offset,
        offset,
      ))
    }
  }

  put_fixed!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

  /// Puts `u8` value to the beginning of the buffer without advancing the internal cursor.
  ///
  /// Returns the number of bytes written (always `1` for this type).
  ///
  /// # Panics
  ///
  /// Panics if the buffer has no space available.
  /// Use [`put_u8_checked`](ChunkMut::put_u8_checked) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_u8(0xFF);
  /// assert_eq!(written, 1);
  /// assert_eq!(buf[0], 0xFF);
  /// ```
  #[inline]
  fn put_u8(&mut self, value: u8) -> usize {
    self.put_slice(&[value])
  }

  /// Tries to put `u8` value to the beginning of the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_u8`](ChunkMut::put_u8).
  /// Returns `Some(1)` on success, or `None` if the buffer has no space.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 1];
  /// let mut slice = &mut buf[..];
  ///
  /// assert_eq!(slice.put_u8_checked(0xFF), Some(1));
  ///
  /// let mut empty = &mut [][..];
  /// assert_eq!(empty.put_u8_checked(0xFF), None);
  /// ```
  #[inline]
  fn put_u8_checked(&mut self, value: u8) -> Option<usize> {
    self.put_slice_checked(&[value])
  }

  /// Puts `i8` value to the beginning of the buffer without advancing the internal cursor.
  ///
  /// Returns the number of bytes written (always `1` for this type).
  ///
  /// # Panics
  ///
  /// Panics if the buffer has no space available.
  /// Use [`put_i8_checked`](ChunkMut::put_i8_checked) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_i8(-42);
  /// assert_eq!(written, 1);
  /// assert_eq!(buf[0], 214); // -42 as u8 is
  /// ```
  #[inline]
  fn put_i8(&mut self, value: i8) -> usize {
    self.put_slice(&[value as u8])
  }

  /// Tries to put `i8` value to the beginning of the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_i8`](ChunkMut::put_i8).
  /// Returns `Some(1)` on success, or `None` if the buffer has no space.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 1];
  ///
  /// let mut slice = &mut buf[..];
  /// assert_eq!(slice.put_i8_checked(-42), Some(1));
  /// let mut empty = &mut [][..];
  /// assert_eq!(empty.put_i8_checked(-42), None);
  /// ```
  #[inline]
  fn put_i8_checked(&mut self, value: i8) -> Option<usize> {
    self.put_slice_checked(&[value as u8])
  }

  /// Puts `u8` value to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// Returns the number of bytes written (always `1` for this type).
  ///
  /// # Panics
  ///
  /// Panics if `offset >= self.remaining_mut()`.
  /// Use [`put_u8_at_checked`](ChunkMut::put_u8_at_checked) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_u8_at(0xFF, 5);
  /// assert_eq!(written, 1);
  /// assert_eq!(buf[5], 0xFF);
  /// ```
  #[inline]
  fn put_u8_at(&mut self, value: u8, offset: usize) -> usize {
    self.put_slice_at(&[value], offset)
  }

  /// Tries to put `u8` value to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_u8_at`](ChunkMut::put_u8_at).
  /// Returns `Some(1)` on success, or `None` if the offset is out of bounds.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// assert_eq!(slice.put_u8_at_checked(0xFF, 5), Some(1));
  /// let err = slice.put_u8_at_checked(0xFF, 30);
  /// assert_eq!(err, None); // Offset out of bounds
  /// ```
  #[inline]
  fn put_u8_at_checked(&mut self, value: u8, offset: usize) -> Option<usize> {
    self.put_slice_at_checked(&[value], offset)
  }

  /// Puts `i8` value to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// Returns the number of bytes written (always `1` for this type).
  ///
  /// # Panics
  ///
  /// Panics if `offset >= self.remaining_mut()`.
  /// Use [`put_i8_at_checked`](ChunkMut::put_i8_at_checked) for non-panicking writes.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_i8_at(-42, 5);
  /// assert_eq!(written, 1);
  /// assert_eq!(buf[5], 214); // -42 as u8 is 214
  /// ```
  #[inline]
  fn put_i8_at(&mut self, value: i8, offset: usize) -> usize {
    self.put_slice_at(&[value as u8], offset)
  }

  /// Tries to put `i8` value to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_i8_at`](ChunkMut::put_i8_at).
  /// Returns `Some(1)` on success, or `None` if the offset is out of bounds.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// assert_eq!(slice.put_i8_at_checked(-42, 5), Some(1));
  /// let err = slice.put_i8_at_checked(-42, 30);
  /// assert_eq!(err, None); // Offset out of bounds
  /// ```
  #[inline]
  fn put_i8_at_checked(&mut self, value: i8, offset: usize) -> Option<usize> {
    self.put_slice_at_checked(&[value as u8], offset)
  }

  /// Tries to put `u8` value to the beginning of the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_u8`](ChunkMut::put_u8) that
  /// returns detailed error information on failure.
  /// Returns `Ok(1)` on success, or `Err(InsufficientSpace)` with details about
  /// the available space if the buffer is full.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_put_u8(0xFF).is_ok());
  ///
  /// let mut empty = &mut [][..];
  /// let err = empty.try_put_u8(0xFF).unwrap_err();
  /// // err contains details about requested vs available space
  /// ```
  #[inline]
  fn try_put_u8(&mut self, value: u8) -> Result<usize, InsufficientSpace> {
    self.try_put_slice(&[value])
  }

  /// Tries to put `i8` value to the beginning of the buffer without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_i8`](ChunkMut::put_i8) that
  /// returns detailed error information on failure.
  /// Returns `Ok(1)` on success, or `Err(InsufficientSpace)` with details about
  /// the available space if the buffer is full.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 5];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_put_i8(-42).is_ok());
  ///
  /// let mut empty = &mut [][..];
  /// let err = empty.try_put_i8(-42).unwrap_err();
  /// // err contains details about requested vs available space
  /// ```
  #[inline]
  fn try_put_i8(&mut self, value: i8) -> Result<usize, InsufficientSpace> {
    self.try_put_slice(&[value as u8])
  }

  /// Tries to put `u8` value to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_u8_at`](ChunkMut::put_u8_at) that
  /// returns detailed error information on failure.
  /// Returns `Ok(1)` on success, or `Err(TryPutAtError)` with details about
  /// what went wrong (out of bounds offset, etc.).
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_put_u8_at(0xFF, 5).is_ok());
  ///
  /// let err = slice.try_put_u8_at(0xFF, 30).unwrap_err();
  /// // err contains detailed information about the failure
  /// ```
  #[inline]
  fn try_put_u8_at(&mut self, value: u8, offset: usize) -> Result<usize, TryPutAtError> {
    self.try_put_slice_at(&[value], offset)
  }

  /// Tries to put `i8` value to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// This is the non-panicking version of [`put_i8_at`](ChunkMut::put_i8_at) that
  /// returns detailed error information on failure.
  /// Returns `Ok(1)` on success, or `Err(TryPutAtError)` with details about
  /// what went wrong (out of bounds offset, etc.).
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::ChunkMut;
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  ///
  /// assert!(slice.try_put_i8_at(-42, 5).is_ok());
  ///
  /// let err = slice.try_put_i8_at(-42, 30).unwrap_err();
  /// // err contains detailed information about the failure
  /// ```
  #[inline]
  fn try_put_i8_at(&mut self, value: i8, offset: usize) -> Result<usize, TryPutAtError> {
    self.try_put_slice_at(&[value as u8], offset)
  }
}

/// A trait that extends `ChunkMut` with additional methods.
pub trait ChunkMutExt: ChunkMut {
  /// Puts type in LEB128 format to the buffer without advancing the internal cursor.
  ///
  /// Uses the LEB128 encoding format. The number of bytes written depends
  /// on the value being encoded.
  ///
  /// Returns `Ok(bytes_written)` on success, or `Err(EncodeVarintError)` if there's
  /// insufficient space or an encoding error occurs.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::{ChunkMut, ChunkMutExt};
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_varint(&42u32).unwrap();
  /// // written will be 1 for small values like 42
  /// ```
  #[inline]
  fn put_varint<V>(&mut self, value: &V) -> Result<NonZeroUsize, EncodeVarintError>
  where
    V: Varint,
  {
    // Pre-flight the exact encoded length. `V::encode` writes continuation bytes
    // as it iterates and only reports `InsufficientSpace` *after* a partial write,
    // which would clobber the destination. `encoded_len()` is exact, so if the
    // value fits we `encode` (guaranteed to fully succeed); otherwise we report
    // the error without touching the buffer, keeping this all-or-error like every
    // other fallible writer here.
    let needed = value.encoded_len();
    let dest = self.buffer_mut();
    if needed.get() > dest.len() {
      return Err(EncodeVarintError::insufficient_space(needed, dest.len()));
    }
    value.encode(dest)
  }

  /// Puts type in LEB128 format to the buffer at the specified offset without advancing the internal cursor.
  ///
  /// Uses the LEB128 encoding format. The number of bytes written depends
  /// on the value being encoded.
  ///
  /// Returns `Ok(bytes_written)` on success, or `Err(EncodeVarintAtError)` if the offset
  /// is out of bounds, there's insufficient space, or an encoding error occurs.
  ///
  /// # Errors
  ///
  /// A varint always needs at least one byte, so `offset >= remaining_mut()` (including the
  /// boundary `offset == remaining_mut()`) returns `OutOfBounds`, matching the sibling
  /// [`try_put_slice_at`](ChunkMut::try_put_slice_at). This write-side boundary at
  /// `offset == space` is the deliberate opposite of the read-side `peek_*_at`, where
  /// `offset == len` returns `InsufficientData`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::{ChunkMut, ChunkMutExt};
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.put_varint_at(&42u32, 3).unwrap();
  /// // The varint is written starting at offset 3
  ///
  /// // If the offset is out of bounds or there's insufficient space,
  /// // it will return an error.
  /// let err = slice.put_varint_at(&42u32, 30).unwrap_err();
  /// let err = slice.put_varint_at(&8442u32, 23).unwrap_err();
  /// ```
  #[inline]
  fn put_varint_at<V>(
    &mut self,
    value: &V,
    offset: usize,
  ) -> Result<NonZeroUsize, EncodeVarintAtError>
  where
    V: Varint,
  {
    let remaining = self.remaining_mut();
    // A varint always needs at least one byte, so `offset >= remaining` leaves no
    // room — reject it as out of bounds *before* splitting so the boundary matches
    // the sibling `try_put_slice_at`.
    if offset >= remaining {
      return Err(EncodeVarintAtError::out_of_bounds(offset, remaining));
    }

    // Pre-flight the exact encoded length. `V::encode` writes continuation bytes
    // as it iterates and only reports `InsufficientSpace` *after* a partial write,
    // which would clobber the destination. Check up front so the buffer is left
    // untouched on error (all-or-error).
    let needed = value.encoded_len();
    let available = remaining - offset;
    if needed.get() > available {
      return Err(EncodeVarintAtError::insufficient_space(
        needed, available, offset,
      ));
    }

    // The pre-flight guarantees the split succeeds and the suffix holds the value;
    // the inner arms remain as a total, non-panicking fallback.
    match self.split_at_mut_checked(offset) {
      Some((_, suffix)) => match value.encode(suffix) {
        Ok(read) => Ok(read),
        Err(e) => Err(EncodeVarintAtError::from_varint_error(e, offset)),
      },
      None => Err(EncodeVarintAtError::out_of_bounds(offset, remaining)),
    }
  }

  /// Writes type in LEB128 format to the buffer, advancing the internal cursor.
  ///
  /// Uses the LEB128 encoding format. The number of bytes written depends
  /// on the value being encoded.
  ///
  /// Returns `Ok(bytes_written)` on success, or `Err(EncodeVarintError)` if there's
  /// insufficient space or an encoding error occurs.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::{ChunkMut, ChunkMutExt};
  ///
  /// let mut buf = [0u8; 24];
  /// let mut slice = &mut buf[..];
  /// let written = slice.write_varint(&42u32).unwrap();
  /// // written will be 1 for small values like 42
  ///
  /// assert_eq!(slice.remaining_mut(), 24 - written.get());
  /// ```
  #[inline]
  fn write_varint<V>(&mut self, value: &V) -> Result<NonZeroUsize, EncodeVarintError>
  where
    V: Varint,
  {
    // Pre-flight the exact encoded length. `V::encode` writes continuation bytes
    // as it iterates and only reports `InsufficientSpace` *after* a partial write,
    // which would clobber the destination (and still leave the cursor unmoved).
    // Only `encode` (then advance) once we know the value fits — all-or-error.
    let needed = value.encoded_len();
    let remaining = self.remaining_mut();
    if needed.get() > remaining {
      return Err(EncodeVarintError::insufficient_space(needed, remaining));
    }
    value.encode(self.buffer_mut()).inspect(|bytes_written| {
      self.advance_mut(bytes_written.get());
    })
  }
}

impl<T: ChunkMut> ChunkMutExt for T {}

macro_rules! deref_forward_buf_mut {
  () => {
    #[inline]
    fn has_remaining_mut(&self) -> bool {
      (**self).has_remaining_mut()
    }

    #[inline]
    fn remaining_mut(&self) -> usize {
      (**self).remaining_mut()
    }

    #[inline]
    fn advance_mut(&mut self, cnt: usize) {
      (**self).advance_mut(cnt)
    }

    #[inline]
    fn try_advance_mut(&mut self, cnt: usize) -> Result<(), TryAdvanceError> {
      (**self).try_advance_mut(cnt)
    }

    #[inline]
    fn truncate_mut(&mut self, new_len: usize) {
      (**self).truncate_mut(new_len)
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut [u8] {
      (**self).buffer_mut()
    }

    #[inline]
    fn fill(&mut self, value: u8) {
      (**self).fill(value)
    }

    #[inline]
    fn prefix_mut(&mut self, len: usize) -> &mut [u8] {
      (**self).prefix_mut(len)
    }

    #[inline]
    fn prefix_mut_checked(&mut self, len: usize) -> Option<&mut [u8]> {
      (**self).prefix_mut_checked(len)
    }

    #[inline]
    fn suffix_mut(&mut self, len: usize) -> &mut [u8] {
      (**self).suffix_mut(len)
    }

    #[inline]
    fn suffix_mut_checked(&mut self, len: usize) -> Option<&mut [u8]> {
      (**self).suffix_mut_checked(len)
    }

    #[inline]
    fn split_at_mut(&mut self, mid: usize) -> (&mut [u8], &mut [u8]) {
      (**self).split_at_mut(mid)
    }

    #[inline]
    fn split_at_mut_checked(&mut self, mid: usize) -> Option<(&mut [u8], &mut [u8])> {
      (**self).split_at_mut_checked(mid)
    }

    #[inline]
    fn put_slice(&mut self, slice: &[u8]) -> usize {
      (**self).put_slice(slice)
    }

    #[inline]
    fn put_slice_checked(&mut self, slice: &[u8]) -> Option<usize> {
      (**self).put_slice_checked(slice)
    }

    #[inline]
    fn try_put_slice(&mut self, slice: &[u8]) -> Result<usize, InsufficientSpace> {
      (**self).try_put_slice(slice)
    }

    #[inline]
    fn put_slice_at(&mut self, slice: &[u8], offset: usize) -> usize {
      (**self).put_slice_at(slice, offset)
    }

    #[inline]
    fn put_slice_at_checked(&mut self, slice: &[u8], offset: usize) -> Option<usize> {
      (**self).put_slice_at_checked(slice, offset)
    }

    #[inline]
    fn try_put_slice_at(&mut self, slice: &[u8], offset: usize) -> Result<usize, TryPutAtError> {
      (**self).try_put_slice_at(slice, offset)
    }

    #[inline]
    fn put_u8(&mut self, value: u8) -> usize {
      (**self).put_u8(value)
    }

    #[inline]
    fn put_u8_checked(&mut self, value: u8) -> Option<usize> {
      (**self).put_u8_checked(value)
    }

    #[inline]
    fn try_put_u8(&mut self, value: u8) -> Result<usize, InsufficientSpace> {
      (**self).try_put_u8(value)
    }

    #[inline]
    fn put_i8(&mut self, value: i8) -> usize {
      (**self).put_i8(value)
    }

    #[inline]
    fn put_i8_checked(&mut self, value: i8) -> Option<usize> {
      (**self).put_i8_checked(value)
    }

    #[inline]
    fn try_put_i8(&mut self, value: i8) -> Result<usize, InsufficientSpace> {
      (**self).try_put_i8(value)
    }

    #[inline]
    fn put_u8_at(&mut self, value: u8, offset: usize) -> usize {
      (**self).put_u8_at(value, offset)
    }

    #[inline]
    fn put_u8_at_checked(&mut self, value: u8, offset: usize) -> Option<usize> {
      (**self).put_u8_at_checked(value, offset)
    }

    #[inline]
    fn try_put_u8_at(&mut self, value: u8, offset: usize) -> Result<usize, TryPutAtError> {
      (**self).try_put_u8_at(value, offset)
    }

    #[inline]
    fn put_i8_at(&mut self, value: i8, offset: usize) -> usize {
      (**self).put_i8_at(value, offset)
    }

    #[inline]
    fn put_i8_at_checked(&mut self, value: i8, offset: usize) -> Option<usize> {
      (**self).put_i8_at_checked(value, offset)
    }

    #[inline]
    fn try_put_i8_at(&mut self, value: i8, offset: usize) -> Result<usize, TryPutAtError> {
      (**self).try_put_i8_at(value, offset)
    }

    put_fixed! {
      @forward
      u16, u32, u64, u128,
      i16, i32, i64, i128,
      f32, f64
    }

    #[inline]
    fn write_slice(&mut self, slice: &[u8]) -> usize {
      (**self).write_slice(slice)
    }

    #[inline]
    fn write_slice_checked(&mut self, slice: &[u8]) -> Option<usize> {
      (**self).write_slice_checked(slice)
    }

    #[inline]
    fn try_write_slice(&mut self, slice: &[u8]) -> Result<usize, InsufficientSpace> {
      (**self).try_write_slice(slice)
    }

    #[inline]
    fn write_u8(&mut self, value: u8) -> usize {
      (**self).write_u8(value)
    }

    #[inline]
    fn write_u8_checked(&mut self, value: u8) -> Option<usize> {
      (**self).write_u8_checked(value)
    }

    #[inline]
    fn try_write_u8(&mut self, value: u8) -> Result<usize, InsufficientSpace> {
      (**self).try_write_u8(value)
    }

    #[inline]
    fn write_i8(&mut self, value: i8) -> usize {
      (**self).write_i8(value)
    }

    #[inline]
    fn write_i8_checked(&mut self, value: i8) -> Option<usize> {
      (**self).write_i8_checked(value)
    }

    #[inline]
    fn try_write_i8(&mut self, value: i8) -> Result<usize, InsufficientSpace> {
      (**self).try_write_i8(value)
    }

    write_fixed! {
      @forward
      u16, u32, u64, u128,
      i16, i32, i64, i128,
      f32, f64
    }
  };
}

impl<T: ChunkMut + ?Sized> ChunkMut for &mut T {
  deref_forward_buf_mut!();
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  impl<T: ChunkMut + ?Sized> ChunkMut for std::boxed::Box<T> {
    deref_forward_buf_mut!();
  }

  impl<T: EmptyChunk> EmptyChunk for std::boxed::Box<T> {
    /// ```rust
    /// use bufkit::{EmptyChunk, ChunkMut};
    ///
    /// let mut slice = <Box<&mut [u8]>>::empty();
    /// assert_eq!(slice.remaining_mut(), 0);
    /// assert!(!slice.has_remaining_mut());
    /// ```
    #[inline]
    fn empty() -> Self
    where
      Self: Sized,
    {
      std::boxed::Box::new(T::empty())
    }
  }
};

impl EmptyChunk for &mut [u8] {
  /// ```rust
  /// use bufkit::{EmptyChunk, ChunkMut};
  ///
  /// let mut slice = <&mut [u8]>::empty();
  /// assert_eq!(slice.remaining_mut(), 0);
  /// assert!(!slice.has_remaining_mut());
  /// ```
  #[inline]
  fn empty() -> Self
  where
    Self: Sized,
  {
    &mut []
  }
}

impl ChunkMut for &mut [u8] {
  #[inline]
  fn advance_mut(&mut self, cnt: usize) {
    if self.len() < cnt {
      panic_advance(&TryAdvanceError::new(must_non_zero(cnt), self.len()));
    }

    // Lifetime dance taken from `impl Write for &mut [u8]`.
    let (_, b) = core::mem::take(self).split_at_mut(cnt);
    *self = b;
  }

  #[inline]
  fn truncate_mut(&mut self, len: usize) {
    if len >= self.len() {
      return;
    }

    // Lifetime dance taken from `impl Write for &mut [u8]`.
    let (a, _) = core::mem::take(self).split_at_mut(len);
    *self = a;
  }

  #[inline]
  fn buffer_mut(&mut self) -> &mut [u8] {
    self
  }

  #[inline]
  fn remaining_mut(&self) -> usize {
    <[u8]>::len(self)
  }

  #[inline]
  fn has_remaining_mut(&self) -> bool {
    !self.is_empty()
  }
}

/// A wrapper around any type that implements `ChunkMut` for improved ergonomics in generic code.
///
/// `ChunkWriter` provides a unified interface over different buffer types, making it easier to write
/// generic functions that accept various writable buffer types without requiring complex type
/// signatures or awkward reference patterns.
///
/// # Problem Solved
///
/// When writing generic functions that accept `ChunkMut` implementors, you often run into ergonomic
/// issues with reference types. For example, with `&mut [u8]`, callers would need to pass
/// `&mut &mut [u8]` which is awkward and unintuitive.
///
/// # Solution
///
/// By using `impl Into<ChunkWriter<B>>` in your function signatures, callers can pass buffer types
/// naturally without extra reference wrapping:
///
/// ```rust,ignore
/// use bufkit::{ChunkMut, ChunkWriter};
///
/// // Instead of this awkward signature:
/// fn encode_bad<B: ChunkMut>(buf: &mut B) { /* ... */ }
///
/// // Use this ergonomic signature:
/// fn encode_good<B: ChunkMut>(buf: impl Into<ChunkWriter<B>>) { /* ... */ }
///
/// // Now callers can write:
/// let mut buffer = [0u8; 64];
/// encode_good(&mut buffer[..]);        // Natural - no double reference needed
/// encode_bad(&mut &mut buffer[..]);    // Awkward - requires &mut &mut
/// ```
///
/// # Common Usage Pattern
///
/// This type is particularly useful for encoding/serialization APIs where you want to accept
/// various buffer types:
///
/// ```rust,ignore
/// use bufkit::{ChunkMut, ChunkWriter};
///
/// pub trait Encode {
///     fn encode<B: ChunkMut>(&self, buf: impl Into<ChunkWriter<B>>) -> Result<usize, Error>;
/// }
///
/// impl Encode for MyStruct {
///     fn encode<B: ChunkMut>(&self, buf: impl Into<ChunkWriter<B>>) -> Result<usize, Error> {
///         let mut buf = buf.into();
///         // Write to buf...
///         Ok(bytes_written)
///     }
/// }
///
/// // Usage is clean and intuitive:
/// let my_struct = MyStruct::new();
/// let mut array_buf = [0u8; 100];
///
/// my_struct.encode(&mut array_buf[..])?;  // No &mut &mut needed
/// ```
///
/// # Type Flexibility
///
/// `ChunkWriter` can wrap any type that implements `ChunkMut`, including:
/// - `&mut [u8]` - Fixed-size byte arrays
/// - Other `ChunkMut` implementors
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ChunkWriter<B: ?Sized>(B);

impl<B> From<B> for ChunkWriter<B> {
  #[inline]
  fn from(value: B) -> Self {
    Self(value)
  }
}

impl<B: ?Sized> core::ops::Deref for ChunkWriter<B> {
  type Target = B;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<B: ?Sized> core::ops::DerefMut for ChunkWriter<B> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<B: EmptyChunk> EmptyChunk for ChunkWriter<B> {
  /// ```rust
  /// use bufkit::{EmptyChunk, ChunkMut, ChunkWriter};
  ///
  /// let mut slice = <ChunkWriter<&mut [u8]>>::empty();
  /// assert_eq!(slice.remaining_mut(), 0);
  /// assert!(!slice.has_remaining_mut());
  /// ```
  #[inline]
  fn empty() -> Self
  where
    Self: Sized,
  {
    ChunkWriter(B::empty())
  }
}

impl<B: ?Sized + ChunkMut> ChunkMut for ChunkWriter<B> {
  deref_forward_buf_mut!();
}

impl<B: ?Sized> ChunkWriter<B> {
  /// Similar to `Option::as_mut`, converts `&mut ChunkWriter<B>` to `ChunkWriter<&mut B>`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use bufkit::{ChunkMut, ChunkWriter};
  ///
  /// let mut buf = [0u8; 24];
  /// let mut write_buf = ChunkWriter::from(&mut buf[..]);
  /// let _ = write_buf.as_mut();
  /// ```
  #[inline]
  pub const fn as_mut(&mut self) -> ChunkWriter<&mut B> {
    ChunkWriter(&mut self.0)
  }

  /// Similar to `Option::as_ref`, converts `&ChunkWriter<B>` to `ChunkWriter<&B>`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use bufkit::{ChunkMut, ChunkWriter};
  ///
  /// let mut buf = [0u8; 24];
  /// let write_buf = ChunkWriter::from(&mut buf[..]);
  /// let _ = write_buf.as_ref();
  /// ```
  #[inline]
  pub const fn as_ref(&self) -> ChunkWriter<&B> {
    ChunkWriter(&self.0)
  }
}

impl<B> ChunkWriter<B> {
  /// Creates a new `ChunkWriter` from the given `ChunkMut`.
  #[inline]
  pub const fn new(buf: B) -> Self {
    ChunkWriter(buf)
  }

  /// Consumes the `ChunkWriter` and returns the underlying `ChunkMut`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use bufkit::{ChunkMut, ChunkWriter};
  ///
  /// let mut buf = [0u8; 24];
  /// let mut write_buf = ChunkWriter::from(&mut buf[..]);
  /// let _ = write_buf.into_inner();
  /// ```
  #[inline]
  pub fn into_inner(self) -> B {
    self.0
  }
}

const _: () = {
  // The existence of this function makes the compiler catch if the ChunkMut
  // trait is "object-safe" or not.
  fn _assert_trait_object(_b: &dyn ChunkMut) {}
};

#[cfg(test)]
mod tests {
  use super::*;

  struct Wrapper<'a>(&'a mut [u8]);

  impl ChunkMut for Wrapper<'_> {
    fn remaining_mut(&self) -> usize {
      self.0.len()
    }

    fn truncate_mut(&mut self, new_len: usize) {
      self.0.truncate_mut(new_len);
    }

    fn buffer_mut(&mut self) -> &mut [u8] {
      self.0
    }

    fn advance_mut(&mut self, cnt: usize) {
      self.0.advance_mut(cnt);
    }
  }

  #[test]
  #[should_panic]
  fn test_advance_mut_panic() {
    let mut buf = [0u8; 5];
    let mut slice = &mut buf[..];
    slice.advance_mut(10);
  }

  #[test]
  fn test_blanket_has_remaining_mut() {
    let mut buf = [0u8; 5];
    let slice = Wrapper(&mut buf[..]);
    assert!(ChunkMut::has_remaining_mut(&slice));

    let empty_slice = Wrapper(&mut []);
    assert!(!ChunkMut::has_remaining_mut(&empty_slice));
  }

  #[test]
  fn test_blanket_fill() {
    let mut buf = [0u8; 5];
    let mut slice = Wrapper(&mut buf[..]);
    slice.fill(0xFF);
    assert_eq!(slice.0, &[0xFF; 5]);

    slice.fill(0x00);
    assert_eq!(slice.0, &[0x00; 5]);
  }

  #[test]
  fn test_blanket_split_at_mut() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = Wrapper(&mut buf[..]);
    let (left, right) = slice.split_at_mut(2);
    assert_eq!(left, &[1, 2]);
    assert_eq!(right, &[3, 4, 5]);
  }

  #[test]
  fn test_deref_write_buf() {
    let mut buf = [0u8; 10];
    let write_buf = ChunkWriter::from(&mut buf[..]);
    let val = core::ops::Deref::deref(&write_buf);
    assert_eq!(val[0], 0); // Check deref gives the underlying buffer
  }

  #[test]
  fn test_deref_mut_write_buf() {
    let mut buf = [0u8; 10];
    let mut write_buf = ChunkWriter::from(&mut buf[..]);
    let val = core::ops::DerefMut::deref_mut(&mut write_buf);
    val[0] = 42; // Modify through deref_mut
    assert_eq!(buf[0], 42);
  }

  #[test]
  fn test_deref_forward_has_remaining_mut() {
    let mut buf = [0u8; 5];
    let slice = ChunkWriter::from(&mut buf[..]);
    assert!(slice.has_remaining_mut());

    let empty_slice: ChunkWriter<&mut [u8]> = ChunkWriter::from([].as_mut_slice());
    assert!(!empty_slice.has_remaining_mut());
  }

  #[test]
  fn test_deref_forward_advance_mut() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    slice.advance_mut(3);
    assert_eq!(slice.buffer_mut(), &[0, 0]);
    assert_eq!(slice.remaining_mut(), 2); // Remaining space after advancing
  }

  #[test]
  fn test_deref_forward_try_advance_mut() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert!(slice.try_advance_mut(3).is_ok());
    assert_eq!(slice.buffer_mut(), &[0, 0]);
    assert_eq!(slice.remaining_mut(), 2); // Remaining space after advancing

    let err = slice.try_advance_mut(10);
    assert!(err.is_err()); // Should fail since we can't advance beyond available space
  }

  #[test]
  fn test_deref_forward_truncate_mut() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    slice.truncate_mut(3);
    assert_eq!(slice.buffer_mut(), &[0, 0, 0]);
    assert_eq!(slice.remaining_mut(), 3); // Remaining space after truncation
  }

  #[test]
  fn test_deref_forward_buffer_mut() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let buffer = slice.buffer_mut();
    assert_eq!(buffer, &mut [0u8; 5]);
    buffer[0] = 42; // Modify through buffer_mut
    assert_eq!(slice.buffer_mut(), &[42, 0, 0, 0, 0]);
  }

  #[test]
  fn test_deref_forward_fill() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    ChunkMut::fill(&mut slice, 0xFF); // Call fill via ChunkMut trait
    assert_eq!(slice.buffer_mut(), &[0xFF; 5]);
  }

  #[test]
  fn test_deref_forward_prefix_mut() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let prefix = slice.prefix_mut(3);
    assert_eq!(prefix, &mut [1, 2, 3]);
    prefix[0] = 10; // Modify prefix
    assert_eq!(slice.buffer_mut(), &[10, 2, 3, 4, 5]);
  }

  #[test]
  fn test_deref_forward_prefix_mut_checked() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert!(slice.prefix_mut_checked(3).is_some());
    assert!(slice.prefix_mut_checked(10).is_none()); // Out of bounds
  }

  #[test]
  fn test_deref_forward_suffix_mut() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let suffix = slice.suffix_mut(2);
    assert_eq!(suffix, &mut [4, 5]);
  }

  #[test]
  fn test_deref_forward_suffix_mut_checked() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert!(slice.suffix_mut_checked(2).is_some());
    assert!(slice.suffix_mut_checked(10).is_none()); // Out of bounds
  }

  #[test]
  fn test_deref_forward_split_at_mut() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let (left, right) = slice.split_at_mut(2);
    assert_eq!(left, &[1, 2]);
    assert_eq!(right, &[3, 4, 5]);
  }

  #[test]
  fn test_deref_forward_split_at_mut_checked() {
    let mut buf = [1, 2, 3, 4, 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert!(slice.split_at_mut_checked(2).is_some());
    assert!(slice.split_at_mut_checked(10).is_none()); // Out of bounds
  }

  #[test]
  fn test_deref_forward_put_slice() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let written = slice.put_slice(&[1, 2, 3]);
    assert_eq!(written, 3);
    assert_eq!(slice.buffer_mut(), &[1, 2, 3, 0, 0]);
  }

  #[test]
  fn test_deref_forward_put_slice_checked() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert_eq!(slice.put_slice_checked(&[1, 2, 3]), Some(3));
    assert_eq!(slice.put_slice_checked(&[1, 2, 3, 4, 5, 6]), None); // Out of bounds
    assert_eq!(slice.buffer_mut(), &[1, 2, 3, 0, 0]);
  }

  #[test]
  fn test_deref_forward_try_put_slice() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert_eq!(slice.try_put_slice(&[1, 2, 3]), Ok(3));
    assert_eq!(
      slice.try_put_slice(&[1, 2, 3, 4, 5, 6]),
      Err(InsufficientSpace::new(must_non_zero(6), 5))
    ); // Out of bounds
    assert_eq!(slice.buffer_mut(), &[1, 2, 3, 0, 0]);
  }

  #[test]
  fn test_deref_forward_put_slice_at() {
    let mut buf = [0u8; 10];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let written = slice.put_slice_at(&[1, 2, 3], 2);
    assert_eq!(written, 3);
    assert_eq!(slice.buffer_mut(), &[0, 0, 1, 2, 3, 0, 0, 0, 0, 0]);
    assert_eq!(slice.remaining_mut(), 10);
  }

  #[test]
  fn test_deref_forward_put_slice_at_checked() {
    let mut buf = [0u8; 10];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert_eq!(slice.put_slice_at_checked(&[1, 2, 3], 2), Some(3));
    assert_eq!(slice.put_slice_at_checked(&[1, 2, 3], 8), None); // Out of bounds
    assert_eq!(slice.buffer_mut(), &[0, 0, 1, 2, 3, 0, 0, 0, 0, 0]);
    assert_eq!(slice.remaining_mut(), 10);
  }

  #[test]
  fn test_deref_forward_try_put_slice_at() {
    let mut buf = [0u8; 10];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert_eq!(slice.try_put_slice_at(&[1, 2, 3], 2), Ok(3));
    assert_eq!(
      slice.try_put_slice_at(&[1, 2, 3], 8),
      Err(TryPutAtError::insufficient_space(must_non_zero(3), 2, 8))
    );
    assert_eq!(slice.buffer_mut(), &[0, 0, 1, 2, 3, 0, 0, 0, 0, 0]);
    assert_eq!(slice.remaining_mut(), 10);
  }

  macro_rules! deref_forward_put {
    (@one $($ty:ident),+$(,)?) => {
      paste::paste! {
        $(
          #[test]
          fn [< test_deref_forward_put_ $ty >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            let written = slice.[< put_ $ty >](42);
            assert_eq!(written, 1);
            assert_eq!(slice.buffer_mut(), &[42, 0, 0, 0, 0]);
          }

          #[test]
          fn [< test_deref_forward_put_ $ty _checked >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< put_ $ty _checked >](42), Some(1));
            assert_eq!(slice.buffer_mut(), &[42, 0, 0, 0, 0]);

            let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
            assert_eq!(empty.[< put_ $ty _checked >](42), None); // Out of bounds,
            assert_eq!(empty.buffer_mut(), &[]);
          }

          #[test]
          fn [< test_deref_forward_try_put_ $ty >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< try_put_ $ty >](42), Ok(1));
            assert_eq!(slice.buffer_mut(), &[42, 0, 0, 0, 0]);

            let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
            assert_eq!(empty.[< try_put_ $ty >](42), Err(InsufficientSpace::new(crate::NON_ZERO_1, 0)));
            assert_eq!(empty.buffer_mut(), &[]);
          }

          #[test]
          fn [< test_deref_forward_put_ $ty _at >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< put_ $ty _at >](42, 1), 1);
            assert_eq!(slice.buffer_mut(), &[0, 42, 0, 0, 0]);
            assert_eq!(slice.remaining_mut(), 5);
          }

          #[test]
          fn [< test_deref_forward_put_ $ty _at_checked >] () {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< put_ $ty _at_checked >](42, 1), Some(1));
            assert_eq!(slice.buffer_mut(), &[0, 42, 0, 0, 0]);
            assert_eq!(slice.remaining_mut(), 5);

            assert_eq!(slice.[< put_ $ty _at_checked >](42, 5), None); // Out of bounds
            assert_eq!(slice.buffer_mut(), &[0, 42, 0, 0, 0]);
          }

          #[test]
          fn [< test_deref_forward_try_put_ $ty _at >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< try_put_ $ty _at >](42, 1), Ok(1));
            assert_eq!(slice.buffer_mut(), &[0, 42, 0, 0, 0]);
            assert_eq!(slice.remaining_mut(), 5);

            assert_eq!(slice.[< try_put_ $ty _at >](42, 5), Err(TryPutAtError::out_of_bounds(5, 5)));
            assert_eq!(slice.buffer_mut(), &[0, 42, 0, 0, 0]);
          }
        )*
      }
    };
    ($($ty:ident), +$(,)?) => {
      $(deref_forward_put!(le $ty);)*
      $(deref_forward_put!(be $ty);)*
      $(deref_forward_put!(ne $ty);)*
    };
    ($endian:ident $ty:ident) => {
      paste::paste! {
        #[test]
        fn [< test_deref_forward_put_ $ty _ $endian >]() {
          let mut buf = [0u8; size_of::<$ty>()];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          let written = slice.[< put_ $ty _ $endian >](42 as $ty);
          assert_eq!(written, size_of::<$ty>());
          assert_eq!(slice.buffer_mut(), (42 as $ty).[< to_ $endian _bytes >]().as_slice());
        }

        #[test]
        fn [< test_deref_forward_put_ $ty _ $endian _checked >]() {
          let mut buf = [0u8; size_of::<$ty>()];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          assert_eq!(slice.[< put_ $ty _ $endian _checked >](42 as $ty), Some(size_of::<$ty>()));
          assert_eq!(slice.buffer_mut(), (42 as $ty).[< to_ $endian _bytes >]().as_slice());

          let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
          assert_eq!(empty.[< put_ $ty _ $endian _checked >](42 as $ty), None); // Out of bounds
        }

        #[test]
        fn [< test_deref_forward_try_put_ $ty _ $endian >]() {
          let mut buf = [0u8; size_of::<$ty>()];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          assert_eq!(slice.[< try_put_ $ty _ $endian >](42 as $ty), Ok(size_of::<$ty>()));
          assert_eq!(slice.buffer_mut(), (42 as $ty).[< to_ $endian _bytes >]().as_slice());
          let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
          assert_eq!(empty.[< try_put_ $ty _ $endian >](42 as $ty), Err(InsufficientSpace::new(must_non_zero(size_of::<$ty>()), 0)));
          assert_eq!(empty.buffer_mut(), &[]);
        }

        #[test]
        fn [< test_deref_forward_put_ $ty _ $endian _at >]() {
          let mut buf = [0u8; { size_of::<$ty>() + 1 }];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          let written = slice.[< put_ $ty _ $endian _at >](42 as $ty, 1);
          assert_eq!(written, size_of::<$ty>());
          assert_eq!(&slice.buffer_mut()[1..], (42 as $ty).[< to_ $endian _bytes >]().as_slice());
        }

        #[test]
        fn [< test_deref_forward_put_ $ty _ $endian _at_checked >]() {
          let mut buf = [0u8; { size_of::<$ty>() + 1 }];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          assert_eq!(slice.[< put_ $ty _ $endian _at_checked >](42 as $ty, 1), Some(size_of::<$ty>()));
          assert_eq!(&slice.buffer_mut()[1..], (42 as $ty).[< to_ $endian _bytes >]().as_slice());

          let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
          assert_eq!(empty.[< put_ $ty _ $endian _at_checked >](42 as $ty, 0), None); // Out of bounds
        }

        #[test]
        fn [< test_deref_forward_try_put_ $ty _ $endian _at >]() {
          let mut buf = [0u8; { size_of::<$ty>() + 1 }];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          assert_eq!(slice.[< try_put_ $ty _ $endian _at >](42 as $ty, 1), Ok(size_of::<$ty>()));
          assert_eq!(&slice.buffer_mut()[1..], (42 as $ty).[< to_ $endian _bytes >]().as_slice());

          let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
          assert_eq!(empty.[< try_put_ $ty _ $endian _at >](42 as $ty, 0), Err(TryPutAtError::out_of_bounds(0, 0)));
          assert_eq!(empty.buffer_mut(), &[]);
        }
      }
    };
  }

  deref_forward_put! {
    @one u8, i8
  }

  deref_forward_put! {
    u16, u32, u64, u128,
    i16, i32, i64, i128,
    f32, f64
  }

  #[test]
  fn test_deref_forward_write_slice() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    let written = slice.write_slice(&[1, 2, 3]);
    assert_eq!(written, 3);
    assert_eq!(slice.buffer_mut(), &[0, 0]);
    assert_eq!(slice.remaining_mut(), 2); // Remaining space after writing
    assert_eq!(buf, [1, 2, 3, 0, 0]);
  }

  #[test]
  fn test_deref_forward_write_slice_checked() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert_eq!(slice.write_slice_checked(&[1, 2, 3]), Some(3));

    assert_eq!(slice.write_slice_checked(&[1, 2, 3, 4, 5, 6]), None); // Out of bounds
    assert_eq!(slice.buffer_mut(), &[0, 0]);
    assert_eq!(slice.remaining_mut(), 2); // Remaining space after writing

    assert_eq!(buf, [1, 2, 3, 0, 0]);
  }

  #[test]
  fn test_deref_forward_try_write_slice() {
    let mut buf = [0u8; 5];
    let mut slice = ChunkWriter::from(&mut buf[..]);
    assert_eq!(slice.try_write_slice(&[1, 2, 3]), Ok(3));
    assert_eq!(slice.buffer_mut(), &[0, 0]);
    assert_eq!(slice.remaining_mut(), 2); // Remaining space after writing

    let err = slice.try_write_slice(&[1, 2, 3, 4, 5, 6]);
    assert!(err.is_err()); // Should fail since we can't write beyond available space
    assert_eq!(
      err.unwrap_err(),
      InsufficientSpace::new(must_non_zero(6), 2)
    );
    assert_eq!(slice.buffer_mut(), &[0, 0]);
    assert_eq!(slice.remaining_mut(), 2); // Remaining space after writing

    assert_eq!(buf, [1, 2, 3, 0, 0]);
  }

  macro_rules! deref_forward_write {
    (@one $($ty:ident),+$(,)?) => {
      paste::paste! {
        $(
          #[test]
          fn [< test_deref_forward_write_ $ty >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            let written = slice.[< write_ $ty >](42);
            assert_eq!(written, 1);
            assert_eq!(slice.buffer_mut(), &[0, 0, 0, 0]);
          }

          #[test]
          fn [< test_deref_forward_write_ $ty _checked >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< write_ $ty _checked >](42), Some(1));
            assert_eq!(slice.buffer_mut(), &[0, 0, 0, 0]);

            let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
            assert_eq!(empty.[< write_ $ty _checked >](42), None); // Out of bounds,
            assert_eq!(empty.buffer_mut(), &[]);
          }

          #[test]
          fn [< test_deref_forward_try_write_ $ty >]() {
            let mut buf = [0u8; 5];
            let mut slice = ChunkWriter::from(&mut buf[..]);
            assert_eq!(slice.[< try_write_ $ty >](42), Ok(1));
            assert_eq!(slice.buffer_mut(), &[0, 0, 0, 0]);

            let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
            assert_eq!(empty.[< try_write_ $ty >](42), Err(InsufficientSpace::new(crate::NON_ZERO_1, 0)));
            assert_eq!(empty.buffer_mut(), &[]);
          }
        )*
      }
    };
    ($($ty:ident), +$(,)?) => {
      $(deref_forward_write!(le $ty);)*
      $(deref_forward_write!(be $ty);)*
      $(deref_forward_write!(ne $ty);)*
    };
    ($endian:ident $ty:ident) => {
      paste::paste! {
        #[test]
        fn [< test_deref_forward_write_ $ty _ $endian >]() {
          let mut buf = [0u8; size_of::<$ty>()];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          let written = slice.[< write_ $ty _ $endian >](42 as $ty);
          assert_eq!(written, size_of::<$ty>());
          assert_eq!(slice.buffer_mut(), &[]);

          assert_eq!(buf, (42 as $ty).[< to_ $endian _bytes >]().as_slice());
        }

        #[test]
        fn [< test_deref_forward_write_ $ty _ $endian _checked >]() {
          let mut buf = [0u8; size_of::<$ty>()];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          assert_eq!(slice.[< write_ $ty _ $endian _checked >](42 as $ty), Some(size_of::<$ty>()));
          assert_eq!(slice.buffer_mut(), &[]);

          assert_eq!(buf, (42 as $ty).[< to_ $endian _bytes >]().as_slice());

          let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
          assert_eq!(empty.[< write_ $ty _ $endian _checked >](42 as $ty), None); // Out of bounds
          assert_eq!(empty.buffer_mut(), &[]);
        }

        #[test]
        fn [< test_deref_forward_try_write_ $ty _ $endian >]() {
          let mut buf = [0u8; size_of::<$ty>()];
          let mut slice = ChunkWriter::from(&mut buf[..]);
          assert_eq!(slice.[< try_write_ $ty _ $endian >](42 as $ty), Ok(size_of::<$ty>()));
          assert_eq!(slice.buffer_mut(), &[]);
          assert_eq!(buf, (42 as $ty).[< to_ $endian _bytes >]().as_slice());

          let mut empty: ChunkWriter<&mut [u8]> = ChunkWriter::from(&mut [][..]);
          assert_eq!(empty.[< try_write_ $ty _ $endian >](42 as $ty), Err(InsufficientSpace::new(must_non_zero(size_of::<$ty>()), 0)));
          assert_eq!(empty.buffer_mut(), &[]);
        }
      }
    };
  }

  deref_forward_write! {
    @one u8, i8
  }

  deref_forward_write! {
    u16, u32, u64, u128,
    i16, i32, i64, i128,
    f32, f64
  }

  // ===========================================================================
  // Regression tests for the varint encoders (audit fixes P1/P2).
  //
  // `V::encode` (varing 0.14) writes continuation bytes as it iterates and only
  // reports `InsufficientSpace` *after* a partial write. The three encoders must
  // pre-flight `encoded_len()` so that on failure the destination is left
  // bit-identical (no clobber) and the cursor is unmoved — all-or-error.
  // ===========================================================================

  #[test]
  fn test_put_varint_insufficient_space_no_clobber() {
    const SENTINEL: u8 = 0xAB;
    let mut buf = [SENTINEL; 4];
    // `u64::MAX` needs 10 bytes; only 4 are available.
    let needed = u64::MAX.encoded_len();
    assert_eq!(needed.get(), 10);
    {
      let mut slice = &mut buf[..];
      let err = slice.put_varint(&u64::MAX).unwrap_err();
      // (a) Err with correct requested / available.
      assert_eq!(err, EncodeVarintError::insufficient_space(needed, 4));
      // (c) Cursor unmoved (`put_*` never advances).
      assert_eq!(slice.remaining_mut(), 4);
    }
    // (b) Buffer bit-identical to the sentinel — NOT clobbered.
    assert_eq!(buf, [SENTINEL; 4]);
  }

  #[test]
  fn test_write_varint_insufficient_space_no_clobber() {
    const SENTINEL: u8 = 0xAB;
    let mut buf = [SENTINEL; 4];
    let needed = u64::MAX.encoded_len();
    {
      let mut slice = &mut buf[..];
      let err = slice.write_varint(&u64::MAX).unwrap_err();
      assert_eq!(err, EncodeVarintError::insufficient_space(needed, 4));
      // A failed write must not advance the cursor.
      assert_eq!(slice.remaining_mut(), 4);
    }
    assert_eq!(buf, [SENTINEL; 4]);
  }

  #[test]
  fn test_put_varint_at_insufficient_space_no_clobber() {
    const SENTINEL: u8 = 0xAB;
    let mut buf = [SENTINEL; 8];
    // Offset 6 leaves 2 bytes; `u64::MAX` needs 10 -> InsufficientSpace at offset 6.
    let needed = u64::MAX.encoded_len();
    {
      let mut slice = &mut buf[..];
      let err = slice.put_varint_at(&u64::MAX, 6).unwrap_err();
      assert_eq!(err, EncodeVarintAtError::insufficient_space(needed, 2, 6));
      assert_eq!(slice.remaining_mut(), 8);
    }
    assert_eq!(buf, [SENTINEL; 8]);
  }

  #[test]
  fn test_put_varint_boundary_exact_fit() {
    let v = 300u32; // encodes to exactly 2 bytes
    let needed = v.encoded_len();
    assert_eq!(needed.get(), 2);

    // remaining == needed - 1: Err, buffer untouched, cursor unmoved.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 1];
      {
        let mut slice = &mut buf[..];
        let err = slice.put_varint(&v).unwrap_err();
        assert_eq!(err, EncodeVarintError::insufficient_space(needed, 1));
        assert_eq!(slice.remaining_mut(), 1);
      }
      assert_eq!(buf, [S; 1]);
    }

    // remaining == needed: exact fit succeeds.
    {
      let mut buf = [0u8; 2];
      {
        let mut slice = &mut buf[..];
        assert_eq!(slice.put_varint(&v).unwrap().get(), 2);
        assert_eq!(slice.remaining_mut(), 2); // `put_*` does not advance
      }
      let (read, decoded) = u32::decode(&buf).unwrap();
      assert_eq!(read.get(), 2);
      assert_eq!(decoded, v);
    }

    // remaining == needed + 1: succeeds, trailing byte untouched.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 3];
      {
        let mut slice = &mut buf[..];
        assert_eq!(slice.put_varint(&v).unwrap().get(), 2);
      }
      let (read, decoded) = u32::decode(&buf[..2]).unwrap();
      assert_eq!(read.get(), 2);
      assert_eq!(decoded, v);
      assert_eq!(buf[2], S); // the +1 byte is not touched
    }
  }

  #[test]
  fn test_write_varint_boundary_exact_fit() {
    let v = 300u32;
    let needed = v.encoded_len();

    // needed - 1: Err, no clobber, cursor unmoved.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 1];
      {
        let mut slice = &mut buf[..];
        let err = slice.write_varint(&v).unwrap_err();
        assert_eq!(err, EncodeVarintError::insufficient_space(needed, 1));
        assert_eq!(slice.remaining_mut(), 1);
      }
      assert_eq!(buf, [S; 1]);
    }

    // needed: exact fit succeeds and advances to 0.
    {
      let mut buf = [0u8; 2];
      {
        let mut slice = &mut buf[..];
        assert_eq!(slice.write_varint(&v).unwrap().get(), 2);
        assert_eq!(slice.remaining_mut(), 0); // advanced by `needed`
      }
      let (read, decoded) = u32::decode(&buf).unwrap();
      assert_eq!(read.get(), 2);
      assert_eq!(decoded, v);
    }

    // needed + 1: succeeds, advances by `needed`, trailing byte untouched.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 3];
      {
        let mut slice = &mut buf[..];
        assert_eq!(slice.write_varint(&v).unwrap().get(), 2);
        assert_eq!(slice.remaining_mut(), 1);
      }
      let (read, decoded) = u32::decode(&buf[..2]).unwrap();
      assert_eq!(read.get(), 2);
      assert_eq!(decoded, v);
      assert_eq!(buf[2], S);
    }
  }

  #[test]
  fn test_put_varint_at_boundary_exact_fit() {
    let v = 300u32; // 2 bytes
    let needed = v.encoded_len();

    // available == needed - 1 (offset 4 of 5): InsufficientSpace, no clobber.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 5];
      {
        let mut slice = &mut buf[..];
        let err = slice.put_varint_at(&v, 4).unwrap_err();
        assert_eq!(err, EncodeVarintAtError::insufficient_space(needed, 1, 4));
        assert_eq!(slice.remaining_mut(), 5);
      }
      assert_eq!(buf, [S; 5]);
    }

    // available == needed (offset 3 of 5): exact fit succeeds.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 5];
      {
        let mut slice = &mut buf[..];
        assert_eq!(slice.put_varint_at(&v, 3).unwrap().get(), 2);
        assert_eq!(slice.remaining_mut(), 5); // `put_*` does not advance
      }
      let (read, decoded) = u32::decode(&buf[3..]).unwrap();
      assert_eq!(read.get(), 2);
      assert_eq!(decoded, v);
      assert_eq!(&buf[..3], &[S; 3]); // bytes before the offset untouched
    }

    // available == needed + 1 (offset 2 of 5): succeeds, trailing byte untouched.
    {
      const S: u8 = 0x5A;
      let mut buf = [S; 5];
      {
        let mut slice = &mut buf[..];
        assert_eq!(slice.put_varint_at(&v, 2).unwrap().get(), 2);
      }
      let (read, decoded) = u32::decode(&buf[2..4]).unwrap();
      assert_eq!(read.get(), 2);
      assert_eq!(decoded, v);
      assert_eq!(&buf[..2], &[S; 2]);
      assert_eq!(buf[4], S);
    }
  }

  #[test]
  fn test_put_varint_at_out_of_bounds_at_remaining() {
    let mut buf = [0u8; 4];
    {
      let mut slice = &mut buf[..];
      let remaining = slice.remaining_mut(); // 4
                                             // A varint needs >= 1 byte, so `offset == remaining` is OutOfBounds (matching
                                             // `try_put_slice_at`), not InsufficientSpace.
      let err = slice.put_varint_at(&1u32, remaining).unwrap_err();
      assert_eq!(
        err,
        EncodeVarintAtError::out_of_bounds(remaining, remaining)
      );
    }
    assert_eq!(buf, [0u8; 4]);
  }

  // Fix 3b: the `_checked` slice writers must share the `len == 0` early-out of the
  // `try_*` siblings so a zero-length write never indexes `buffer_mut()` and cannot
  // panic on an incoherent impl at the `offset == space` boundary.
  #[test]
  fn test_checked_zero_len_early_out_no_panic() {
    // Deliberately *incoherent*: `remaining_mut()` overstates the real buffer length.
    struct Incoherent([u8; 3]);
    impl ChunkMut for Incoherent {
      fn remaining_mut(&self) -> usize {
        5 // claims 5 but only backs 3
      }
      fn truncate_mut(&mut self, _new_len: usize) {}
      fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.0
      }
      fn advance_mut(&mut self, _cnt: usize) {}
    }

    let mut c = Incoherent([0u8; 3]);
    // Zero-length writes: Some(0), no panic.
    assert_eq!(c.put_slice_checked(&[]), Some(0));
    assert_eq!(c.write_slice_checked(&[]), Some(0));
    // Before the fix this indexed `buffer_mut()[5..5]` on a 3-byte buffer and panicked.
    assert_eq!(c.put_slice_at_checked(&[], 5), Some(0));
    // Offset beyond the claimed space still reports failure (behavior preserved).
    assert_eq!(c.put_slice_at_checked(&[], 6), None);

    // The `try_*` siblings already behaved this way.
    assert_eq!(c.try_put_slice(&[]), Ok(0));
    assert_eq!(c.try_write_slice(&[]), Ok(0));
    assert_eq!(c.try_put_slice_at(&[], 5), Ok(0));
    assert_eq!(
      c.try_put_slice_at(&[], 6),
      Err(TryPutAtError::out_of_bounds(6, 5))
    );
  }
}
