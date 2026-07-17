use core::ops::{Bound, RangeBounds};

use crate::{error::TryAdvanceError, EmptyChunk};

use super::{
  super::{must_non_zero, panic_advance},
  ChunkMut, ChunkWriter,
};

/// A putter for writing to a buffer without modifying the original buffer's cursor.
///
/// `Putter` provides a non-destructive way to write buffer contents by maintaining
/// its own independent cursor position. This is particularly useful when you need to:
/// - Preview write operations before committing them
/// - Write data that might need to be rolled back
/// - Implement transactional write operations
/// - Share write access to the same buffer data from different positions
/// - Test serialization without modifying the original buffer
///
/// The putter can be constrained to a specific range within the buffer, making it
/// safe for write operations that should not exceed certain boundaries.
///
/// # Examples
///
/// ```rust
/// use buffo::{ChunkMut, Putter};
///
/// let mut data = [0u8; 10];
/// let mut putter = Putter::new(&mut data[..]);  // No need for &mut &mut
///
/// // Write without affecting the original buffer's cursor
/// putter.put_u8(0x42);
/// putter.put_u16_le(0x1234);
///
/// // Original buffer's state is unchanged until we access it
/// // The writes are staged in the putter's view
/// ```
#[derive(Debug)]
pub struct Putter<B: ?Sized> {
  /// Current cursor position relative to the buffer's current position
  cursor: usize,
  /// The original start bound of the putter, used for resetting.
  start: Bound<usize>,
  /// The original end bound of the putter, used for resetting.
  end: Bound<usize>,
  /// Current limit bound of the putter
  limit: Bound<usize>,
  /// The underlying buffer wrapped in ChunkWriter for ergonomic access
  buf: ChunkWriter<B>,
}

impl<B: ChunkMut> From<B> for Putter<B> {
  #[inline]
  fn from(buf: B) -> Self {
    Self::new(buf)
  }
}

impl<B: EmptyChunk> EmptyChunk for Putter<B> {
  /// ```rust
  /// use buffo::{EmptyChunk, ChunkMut, Putter};
  ///
  /// let mut slice = <Putter<&mut [u8]>>::empty();
  /// assert_eq!(slice.remaining_mut(), 0);
  /// assert!(!slice.has_remaining_mut());
  /// ```
  #[inline]
  fn empty() -> Self {
    Self::new(B::empty())
  }
}

impl<B> Putter<B> {
  /// Creates a new `Putter` instance with the given buffer.
  ///
  /// The putter starts at the beginning of the buffer's current position
  /// and can write to all remaining space in the buffer.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let putter = Putter::new(&mut data[..]);
  /// assert_eq!(putter.remaining_mut(), 10);
  /// ```
  #[inline]
  pub fn new(buf: impl Into<ChunkWriter<B>>) -> Self {
    Self::with_cursor_and_bounds_inner(buf.into(), 0, Bound::Included(0), Bound::Unbounded)
  }

  /// Creates a new `Putter` instance with the given buffer.
  ///
  /// The putter starts at the beginning of the buffer's current position
  /// and can write to all remaining space in the buffer.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let mut slice: &mut [u8] = &mut data[..];
  /// let putter = Putter::const_new(&mut slice);
  /// assert_eq!(putter.remaining_mut(), 10);
  /// ```
  #[inline]
  pub const fn const_new(buf: B) -> Self {
    let buf = ChunkWriter::new(buf);
    Self::with_cursor_and_bounds_inner(buf, 0, Bound::Included(0), Bound::Unbounded)
  }

  /// Creates a new `Putter` constrained to a specific length.
  ///
  /// This is useful when you want to ensure the putter cannot write beyond
  /// a certain number of bytes, providing additional safety for serialization operations.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let putter = Putter::with_limit(&mut data[..], 5); // Only write first 5 bytes
  /// assert_eq!(putter.remaining_mut(), 5);
  /// ```
  #[inline]
  pub fn with_limit(buf: impl Into<ChunkWriter<B>>, limit: usize) -> Self {
    let write_buf = buf.into();
    Self::with_cursor_and_bounds_inner(write_buf, 0, Bound::Included(0), Bound::Excluded(limit))
  }

  /// Creates a new `Putter` constrained to a specific length.
  ///
  /// This is useful when you want to ensure the putter cannot write beyond
  /// a certain number of bytes, providing additional safety for serialization operations.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let putter = Putter::const_with_limit((&mut data[..]).into(), 5); // Only write first 5 bytes
  /// assert_eq!(putter.remaining_mut(), 5);
  /// ```
  #[inline]
  pub const fn const_with_limit(buf: ChunkWriter<B>, limit: usize) -> Self {
    Self::with_cursor_and_bounds_inner(buf, 0, Bound::Included(0), Bound::Excluded(limit))
  }

  /// Creates a new `Putter` with specific start and end bounds.
  ///
  /// This provides maximum flexibility in defining the putter's range,
  /// allowing for more complex write scenarios.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use core::ops::Bound;
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  ///
  /// // Write from index 2 to 7 (exclusive)
  /// let putter = Putter::with_range(&mut data[..], 2..7);
  /// assert_eq!(putter.remaining_mut(), 5);
  /// ```
  #[inline]
  pub fn with_range(buf: impl Into<ChunkWriter<B>>, range: impl RangeBounds<usize>) -> Self
  where
    B: ChunkMut,
  {
    let start = range.start_bound().cloned();
    let end = range.end_bound().cloned();
    let write_buf = buf.into();
    let start_pos = Self::resolve_start_bound(start, &write_buf);
    Self::with_cursor_and_bounds_inner(write_buf, start_pos, Bound::Included(start_pos), end)
  }

  /// Returns the current position of the internal cursor relative to the start of the buffer.
  ///
  /// This represents how far the putter's cursor has advanced from its starting position.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let mut putter = Putter::new(&mut data[..]);
  ///
  /// assert_eq!(putter.position(), 0);
  /// putter.write_u8(0x42);
  /// assert_eq!(putter.position(), 1);
  /// putter.advance_mut(2);
  /// assert_eq!(putter.position(), 3);
  /// ```
  #[inline]
  pub const fn position(&self) -> usize {
    let start_pos = Self::resolve_start_bound_without_check(self.start);
    self.cursor.saturating_sub(start_pos)
  }

  /// Returns the absolute position of the putter's cursor in the underlying buffer.
  ///
  /// This is the position relative to the start of the buffer, not just the putter's starting point.
  /// This is useful for understanding where the putter is writing in the context of the entire buffer.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  /// use core::ops::Bound;
  ///
  /// let mut data = [0u8; 10];
  /// let mut putter = Putter::with_range(&mut data[..], 3..7);
  ///
  /// putter.write_u8(0x42);
  /// assert_eq!(putter.absolute_position(), 4); // 3 (offset) + 1 (written)
  /// ```
  #[inline]
  pub const fn absolute_position(&self) -> usize {
    self.cursor
  }

  /// Resets the putter's to the initial state.
  ///
  /// After calling this method, the putter will start writing from the same
  /// position and the same limit where it was initially created.
  ///
  /// This method will not clean the dirty state of the putter, which means
  /// any previously written data will still be present in the buffer. See also [`reset`](Putter::reset).
  ///
  /// The restored limit is the **construction-time** limit, so calling this (or `reset`)
  /// after `truncate_mut` re-widens the writable window past that truncation. This is a
  /// deliberate divergence from `Peeker` / `RefPeeker`, whose `truncate` re-anchors the
  /// reset target so a later `reset` stays within the truncated window.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let mut putter = Putter::new(&mut data[..]);
  ///
  /// putter.advance_mut(3);
  /// assert_eq!(putter.position(), 3);
  ///
  /// putter.reset_position();
  /// assert_eq!(putter.position(), 0);
  /// assert_eq!(putter.remaining_mut(), 10);
  /// ```
  #[inline]
  pub const fn reset_position(&mut self) {
    self.cursor = Self::resolve_start_bound_without_check(self.start);
    self.limit = self.end;
  }

  /// Resets the putter's buffer to its initial state, filling the written area with zeros.
  ///
  /// This method clears the buffer's contents from the start position to the limit position,
  /// effectively resetting the buffer to a clean state.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let mut putter = Putter::new(&mut data[..]);
  ///
  /// putter.put_u8(0x42);
  /// assert_eq!(putter.buffer_mut()[0], 0x42);
  ///
  /// putter.reset();
  /// assert_eq!(putter.buffer_mut()[0], 0x00); // The first byte is reset to zero
  /// ```
  pub fn reset(&mut self)
  where
    B: ChunkMut,
  {
    self.reset_position();
    self.fill(0);
  }

  /// Consumes the putter and returns the underlying buffer.
  ///
  /// Any writes made through the putter will be reflected in the returned buffer.
  /// This is useful when you want to finalize the writes and retrieve the modified buffer.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{ChunkMut, Putter};
  ///
  /// let mut data = [0u8; 10];
  /// let mut putter = Putter::new(&mut data[..]);
  /// putter.put_u8(0x42);
  /// let buf = putter.into_inner();
  /// assert_eq!(buf[0], 0x42);
  /// ```
  #[inline]
  pub fn into_inner(self) -> B {
    self.buf.0
  }

  #[inline]
  const fn with_cursor_and_bounds_inner(
    buf: ChunkWriter<B>,
    cursor: usize,
    start: Bound<usize>,
    end: Bound<usize>,
  ) -> Self {
    Self {
      buf,
      cursor,
      start,
      end,
      limit: end,
    }
  }

  #[inline]
  const fn resolve_start_bound_without_check(bound: Bound<usize>) -> usize {
    match bound {
      Bound::Included(n) => n,
      Bound::Excluded(n) => n.saturating_add(1),
      Bound::Unbounded => 0,
    }
  }
}

impl<B: ?Sized> Putter<B> {
  #[inline]
  fn resolve_start_bound(bound: Bound<usize>, buf: &ChunkWriter<B>) -> usize
  where
    B: ChunkMut,
  {
    let pos = match bound {
      Bound::Included(n) => n,
      Bound::Excluded(n) => n.saturating_add(1),
      Bound::Unbounded => 0,
    };
    pos.min(buf.remaining_mut())
  }

  #[inline]
  fn resolve_end_bound(&self, bound: Bound<usize>) -> usize
  where
    B: ChunkMut,
  {
    match bound {
      Bound::Included(n) => (n.saturating_add(1)).min(self.buf.remaining_mut()),
      Bound::Excluded(n) => n.min(self.buf.remaining_mut()),
      Bound::Unbounded => self.buf.remaining_mut(),
    }
  }

  /// Computes the clamped half-open `[start, end)` view into the backing buffer
  /// that is shared by both [`remaining_mut`](ChunkMut::remaining_mut) and
  /// [`buffer_mut`](ChunkMut::buffer_mut).
  ///
  /// The end is clamped to the backing buffer's writable length and the start is
  /// clamped to that end, guaranteeing `start <= end <= self.buf.remaining_mut()`.
  /// This keeps `buffer_mut()` panic-free and upholds the
  /// `buffer_mut().len() == remaining_mut()` invariant for every `(limit, range)`,
  /// including inverted ranges. Returning owned `usize`s (rather than borrowing)
  /// lets `buffer_mut` compute the bounds before taking its `&mut` slice.
  #[inline]
  fn view_bounds(&self) -> (usize, usize)
  where
    B: ChunkMut,
  {
    let inner = self.buf.remaining_mut();
    let end = self.resolve_end_bound(self.limit).min(inner);
    let start = self.cursor.min(end);
    (start, end)
  }
}

impl<B: ChunkMut + ?Sized> ChunkMut for Putter<B> {
  #[inline]
  fn remaining_mut(&self) -> usize {
    let (start, end) = self.view_bounds();
    end - start
  }

  #[inline]
  fn buffer_mut(&mut self) -> &mut [u8] {
    // Compute the clamped bounds before taking the `&mut` slice to satisfy the
    // borrow checker (`view_bounds` borrows `&self`, then the borrow is released).
    let (start, end) = self.view_bounds();
    &mut self.buf.buffer_mut()[start..end]
  }

  #[inline]
  fn advance_mut(&mut self, cnt: usize) {
    let remaining = self.remaining_mut();
    if cnt > remaining {
      panic_advance(&TryAdvanceError::new(must_non_zero(cnt), remaining));
    }
    self.cursor += cnt;
  }

  #[inline]
  fn try_advance_mut(&mut self, cnt: usize) -> Result<(), TryAdvanceError> {
    let remaining = self.remaining_mut();
    if cnt > remaining {
      return Err(TryAdvanceError::new(must_non_zero(cnt), remaining));
    }
    self.cursor += cnt;
    Ok(())
  }

  /// Narrows the writable window to at most `new_len` bytes from the current cursor.
  ///
  /// Only the current limit is tightened; the construction-time bound is left intact, so a
  /// later `reset_position` / `reset` re-widens the window past this truncation. This is the
  /// intentional divergence from `Peeker` / `RefPeeker`, whose `truncate` re-anchors the
  /// reset target so a later `reset` stays within the truncated window.
  #[inline]
  fn truncate_mut(&mut self, new_len: usize) {
    let current_remaining = self.remaining_mut();
    if new_len >= current_remaining {
      return; // No truncation needed
    }

    let new_end_pos = self.cursor + new_len;
    let current_end_pos = self.resolve_end_bound(self.limit);

    // Only truncate if the new limit is more restrictive than the current one
    if new_end_pos < current_end_pos {
      self.limit = Bound::Excluded(new_end_pos);
    }
  }
}

#[cfg(test)]
#[allow(clippy::reversed_empty_ranges)]
mod tests {
  use super::*;

  #[test]
  fn test_putter_basic_functionality() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    assert_eq!(putter.remaining_mut(), 10);
    assert_eq!(putter.position(), 0);

    // Write a byte
    putter.write_u8(0x42);
    assert_eq!(putter.remaining_mut(), 9);
    assert_eq!(putter.position(), 1);

    // Check that the data was written
    assert_eq!(data[0], 0x42);
  }

  #[test]
  fn test_putter_ergonomic_api() {
    // Test the improved ergonomics - no need for &mut &mut
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]); // Clean!

    putter.put_u8(0x11);
    assert_eq!(data[0], 0x11);
  }

  #[test]
  fn test_putter_with_limit() {
    let mut data = [0u8; 10];
    let mut putter = Putter::with_limit(&mut data[..], 3);

    assert_eq!(putter.remaining_mut(), 3);

    // Write within limit
    putter.write_u8(0x11);
    putter.write_u8(0x22);
    putter.write_u8(0x33);

    // Should be at limit now
    assert_eq!(putter.remaining_mut(), 0);

    // Check that data was written correctly
    assert_eq!(data[0], 0x11);
    assert_eq!(data[1], 0x22);
    assert_eq!(data[2], 0x33);
    assert_eq!(data[3], 0x00); // Unchanged
  }

  #[test]
  fn test_putter_reset_position() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    putter.advance_mut(3);
    assert_eq!(putter.position(), 3);
    assert_eq!(putter.remaining_mut(), 7);

    putter.reset_position();
    assert_eq!(putter.position(), 0);
    assert_eq!(putter.remaining_mut(), 10);
  }

  #[test]
  fn test_putter_truncate() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    putter.truncate_mut(3);
    assert_eq!(putter.remaining_mut(), 3);

    // Should only be able to write 3 bytes
    putter.write_u8(0x11);
    putter.write_u8(0x22);
    putter.write_u8(0x33);
    assert_eq!(putter.remaining_mut(), 0);

    // Check data
    assert_eq!(data[0], 0x11);
    assert_eq!(data[1], 0x22);
    assert_eq!(data[2], 0x33);
    assert_eq!(data[3], 0x00); // Unchanged
  }

  #[test]
  fn test_putter_try_advance() {
    let mut data = [0u8; 5];
    let mut putter = Putter::new(&mut data[..]);

    assert!(putter.try_advance_mut(2).is_ok());
    assert_eq!(putter.position(), 2);

    // Should fail when trying to advance beyond available space
    assert!(putter.try_advance_mut(5).is_err());
    assert_eq!(putter.position(), 2); // Should remain unchanged
  }

  #[test]
  #[should_panic(expected = "advance")]
  fn test_putter_advance_panic() {
    let mut data = [0u8; 3];
    let mut putter = Putter::new(&mut data[..]);

    putter.advance_mut(5); // Should panic
  }

  #[test]
  fn test_putter_with_different_integer_types() {
    let mut data = [0u8; 20];
    let mut putter = Putter::new(&mut data[..]);

    // Test little-endian writes
    putter.write_u16_le(0x1234);
    putter.write_u32_le(0x56789ABC);
    assert_eq!(putter.position(), 6);

    // Verify the data was written correctly
    assert_eq!(&data[0..2], &[0x34, 0x12]); // u16 LE
    assert_eq!(&data[2..6], &[0xBC, 0x9A, 0x78, 0x56]); // u32 LE
  }

  #[test]
  fn test_putter_write_slice() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    let test_data = [0x11, 0x22, 0x33, 0x44];
    putter.write_slice(&test_data);

    assert_eq!(putter.position(), 4);
    assert_eq!(&data[0..4], &test_data);
    assert_eq!(data[4], 0x00); // Unchanged
  }

  #[test]
  fn test_putter_with_advanced_buffer() {
    let mut data = [0u8; 10];
    let mut buf = &mut data[..];

    // Advance the original buffer
    buf.advance_mut(3);
    assert_eq!(buf.remaining_mut(), 7);

    // Putter should work with the advanced buffer
    let mut putter = Putter::new(buf);
    assert_eq!(putter.remaining_mut(), 7);
    putter.put_u8(0x42);
    // Should write to the correct position
    assert_eq!(data[3], 0x42); // Position 3 in original array
  }

  #[test]
  fn test_putter_buffer_mut_access() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    // Write some data first
    putter.write_u8(0x11);
    putter.write_u8(0x22);

    // Access the remaining buffer
    let remaining_buffer = putter.buffer_mut();
    remaining_buffer[0] = 0x99; // Should write to position 2 in original

    assert_eq!(data[0], 0x11);
    assert_eq!(data[1], 0x22);
    assert_eq!(data[2], 0x99);
  }

  #[test]
  fn test_putter_state_consistency() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    // Verify invariants are maintained through operations
    let initial_remaining = putter.remaining_mut();
    let initial_written = putter.position();

    // After advance
    putter.advance_mut(3);
    assert_eq!(
      putter.remaining_mut() + putter.position(),
      initial_remaining
    );

    // After truncate
    putter.truncate_mut(5);
    assert_eq!(putter.remaining_mut(), 5);
    assert_eq!(putter.position(), 3);

    // After reset_position
    putter.reset_position();
    assert_eq!(putter.position(), initial_written);
    assert_eq!(putter.remaining_mut(), initial_remaining); // back to initial state
  }

  #[test]
  fn test_putter_exhaustive_write() {
    let mut data = [0u8; 3];
    let mut putter = Putter::new(&mut data[..]);

    // Write all bytes one by one
    assert_eq!(putter.write_u8(0x11), 1);
    assert_eq!(putter.remaining_mut(), 2);

    assert_eq!(putter.write_u8(0x22), 1);
    assert_eq!(putter.remaining_mut(), 1);

    assert_eq!(putter.write_u8(0x33), 1);
    assert_eq!(putter.remaining_mut(), 0);

    // Verify data
    assert_eq!(data, [0x11, 0x22, 0x33]);
  }

  #[test]
  fn test_putter_from_trait() {
    let mut data = [0u8; 10];

    // Test From trait implementation
    let putter: Putter<_> = (&mut data[..]).into();
    assert_eq!(putter.remaining_mut(), 10);
  }

  #[test]
  fn test_putter_endianness() {
    let mut data = [0u8; 16];
    let mut putter = Putter::new(&mut data[..]);

    // Test different endianness
    putter.write_u16_le(0x1234);
    putter.write_u16_be(0x1234);
    putter.write_u32_le(0x12345678);
    putter.write_u32_be(0x12345678);

    // Verify little-endian
    assert_eq!(&data[0..2], &[0x34, 0x12]);
    // Verify big-endian
    assert_eq!(&data[2..4], &[0x12, 0x34]);
    // Verify little-endian u32
    assert_eq!(&data[4..8], &[0x78, 0x56, 0x34, 0x12]);
    // Verify big-endian u32
    assert_eq!(&data[8..12], &[0x12, 0x34, 0x56, 0x78]);
  }

  #[test]
  fn test_putter_signed_values() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    // Test signed values
    putter.write_i8(-1);
    putter.write_i16_le(-1);
    putter.write_i32_be(-1);

    // Verify i8
    assert_eq!(data[0], 0xFF);
    // Verify i16 LE
    assert_eq!(&data[1..3], &[0xFF, 0xFF]);
    // Verify i32 BE
    assert_eq!(&data[3..7], &[0xFF, 0xFF, 0xFF, 0xFF]);
  }

  #[test]
  fn test_putter_reset_position_preserves_limits() {
    let mut data = [0u8; 10];
    let mut putter = Putter::with_limit(&mut data[..], 5);

    putter.advance_mut(3);
    assert_eq!(putter.position(), 3);
    assert_eq!(putter.remaining_mut(), 2);

    putter.reset_position();
    assert_eq!(putter.position(), 0);
    assert_eq!(putter.remaining_mut(), 5); // Should still be limited to 5
  }

  #[test]
  fn test_putter_truncate_after_advance() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    putter.advance_mut(3);
    assert_eq!(putter.remaining_mut(), 7);

    putter.truncate_mut(4);
    assert_eq!(putter.remaining_mut(), 4);

    // Write to verify the truncation works
    putter.write_slice(&[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(putter.remaining_mut(), 0);

    // Verify data was written starting from position 3
    assert_eq!(&data[3..7], &[0x11, 0x22, 0x33, 0x44]);
  }

  #[test]
  fn test_putter_truncate_limited_putter() {
    let mut data = [0u8; 10];
    let mut putter = Putter::with_limit(&mut data[..], 6);

    assert_eq!(putter.remaining_mut(), 6);

    // Truncate within the limit
    putter.truncate_mut(4);
    assert_eq!(putter.remaining_mut(), 4);

    // Further truncation
    putter.truncate_mut(2);
    assert_eq!(putter.remaining_mut(), 2);
  }

  #[test]
  fn test_error_details() {
    let mut data = [0u8; 3];
    let mut putter = Putter::new(&mut data[..]);

    // Test TryAdvanceError
    let advance_err = putter.try_advance_mut(5).unwrap_err();
    assert_eq!(advance_err.requested().get(), 5);
    assert_eq!(advance_err.available(), 3);
  }

  #[test]
  fn test_putter_large_data() {
    // Test with larger buffer to ensure no performance issues
    let mut large_data = [0u8; 1000];
    let mut putter = Putter::new(&mut large_data[..]);

    assert_eq!(putter.remaining_mut(), 1000);

    // Write some data
    for i in 0..100 {
      putter.write_u8(i as u8);
    }

    assert_eq!(putter.remaining_mut(), 900);
    assert_eq!(putter.position(), 100);

    // Verify data
    #[allow(clippy::needless_range_loop)]
    for i in 0..100 {
      assert_eq!(large_data[i], i as u8);
    }
  }

  #[test]
  fn test_putter_boundary_conditions() {
    // Test with single byte buffer
    let mut single_byte = [0u8; 1];
    let mut putter = Putter::new(&mut single_byte[..]);

    assert_eq!(putter.remaining_mut(), 1);
    putter.write_u8(0x42);
    assert_eq!(putter.remaining_mut(), 0);

    // Operations on exhausted putter
    assert!(putter.try_advance_mut(1).is_err());
    assert_eq!(putter.put_u8_checked(0x99), None);
    assert!(putter.try_put_u8(0x99).is_err());

    // Reset should work
    putter.reset_position();
    assert_eq!(putter.remaining_mut(), 1);
  }

  #[test]
  fn test_putter_with_limit_larger_than_buffer() {
    let mut data = [0u8; 3];
    let putter = Putter::with_limit(&mut data[..], 10);

    // Should be limited by buffer size, not the requested limit
    assert_eq!(putter.remaining_mut(), 3);
  }

  #[test]
  fn test_putter_with_limit_zero() {
    let mut data = [0u8; 5];
    let putter = Putter::with_limit(&mut data[..], 0);

    assert_eq!(putter.remaining_mut(), 0);
  }

  #[test]
  fn test_written_calculation() {
    let mut data = [0u8; 10];
    let mut putter = Putter::new(&mut data[..]);

    // Test written calculation with different operations
    assert_eq!(putter.position(), 0);

    putter.advance_mut(2);
    assert_eq!(putter.position(), 2);

    putter.write_u8(0x42);
    assert_eq!(putter.position(), 3);

    putter.write_slice(&[1, 2, 3]);
    assert_eq!(putter.position(), 6);

    // Reset and verify
    putter.reset_position();
    assert_eq!(putter.position(), 0);
  }

  #[test]
  fn test_putter_ergonomic_comparison() {
    let mut data = [0u8; 10];

    // Before: awkward double reference
    // let mut buf = &mut data[..];
    // let mut putter = Putter::new(&mut buf);  // &mut &mut [u8]

    // After: clean and natural
    let mut putter = Putter::new(&mut data[..]); // Just &mut [u8]

    putter.write_u8(0x42);
    assert_eq!(putter.position(), 1);

    assert_eq!(data[0], 0x42);
  }

  #[test]
  fn test_putter_with_range() {
    let mut data = [0u8; 10];
    let mut putter = Putter::with_range(&mut data[..], 2..=8);

    assert_eq!(putter.remaining_mut(), 7);

    putter.write_u8(0x42);
    assert_eq!(putter.position(), 1);
    putter.reset();
    assert_eq!(data[2], 0);

    let mut putter = Putter::with_range(&mut data[..], ..7);
    assert_eq!(putter.remaining_mut(), 7);
    putter.write_u8(0x99);
    assert_eq!(putter.position(), 1);
    putter.reset();
    assert_eq!(data[0], 0);

    let mut putter = Putter::with_range(&mut data[..], (Bound::Excluded(1), Bound::Unbounded));
    assert_eq!(putter.remaining_mut(), 8);
    putter.write_u8(0x77);
    assert_eq!(putter.position(), 1);
    putter.reset();
    assert_eq!(data[2], 0);
  }

  // Regression: `buffer_mut()` clamped only the start and sliced to the raw end,
  // so inverted ranges panicked (`&mut buf[5..2]`) even though `remaining_mut()`
  // reported 0. The clamped shared view must keep
  // `buffer_mut().len() == remaining_mut()` and stay panic-free.
  #[test]
  fn test_putter_panic_safety_out_of_range_bounds() {
    use crate::chunk_mut::ChunkMutExt; // brings `put_varint` into scope

    let mut data = [0u8; 8];
    let len = data.len(); // 8

    // `with_limit` past the length (already clamped by `resolve_end_bound`, but
    // assert the shared-view invariant regardless).
    {
      let mut p = Putter::with_limit(&mut data[..], len + 3);
      assert_eq!(p.buffer_mut().len(), p.remaining_mut());
      assert_eq!(p.remaining_mut(), len);
      assert_eq!(p.put_slice_checked(&[]), Some(0));
      assert!(p.try_advance_mut(p.remaining_mut()).is_ok());
      assert!(p.try_advance_mut(1).is_err());
    }

    // Inverted range: empty, non-panicking view (previously panicked in
    // `buffer_mut()` slicing `&mut buf[5..2]`).
    {
      let mut p = Putter::with_range(&mut data[..], 5..2);
      assert_eq!(p.buffer_mut().len(), p.remaining_mut());
      assert_eq!(p.remaining_mut(), 0);
      assert_eq!(p.put_slice_checked(&[]), Some(0));
      assert!(p.try_advance_mut(0).is_ok());
      assert!(p.try_advance_mut(1).is_err());
    }

    // `..=usize::MAX`: end clamped to the backing writable length.
    {
      let mut p = Putter::with_range(&mut data[..], ..=usize::MAX);
      assert_eq!(p.buffer_mut().len(), p.remaining_mut());
      assert_eq!(p.remaining_mut(), len);
      assert_eq!(p.put_slice_checked(&[]), Some(0));
      assert!(p.try_advance_mut(len).is_ok());
    }

    // Explicit end past the backing length.
    {
      let mut p = Putter::with_range(&mut data[..], 0..(len + 5));
      assert_eq!(p.buffer_mut().len(), p.remaining_mut());
      assert_eq!(p.remaining_mut(), len);
      assert_eq!(p.put_slice_checked(&[]), Some(0));
      assert!(p.try_advance_mut(len).is_ok());
    }

    // Inverted range `5..3`: the writer methods that route through
    // `buffer_mut()` (reset/fill/put_varint) must not panic on the empty view.
    {
      let mut p = Putter::with_range(&mut data[..], 5..3);
      assert_eq!(p.remaining_mut(), 0);
      assert_eq!(p.buffer_mut().len(), 0);
      p.reset(); // reset_position() + fill(0) over the empty view
      p.fill(0);
      // `put_varint` needs >= 1 byte; on the empty view it must error, not panic.
      assert!(p.put_varint(&1u8).is_err());
      assert_eq!(p.put_slice_checked(&[]), Some(0));
    }
  }
}
