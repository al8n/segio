use core::ops::{Bound, RangeBounds};

use crate::error::TryAdvanceError;

use super::{check_out_of_bounds, must_non_zero, panic_advance, Chunk};

/// A peeker for reading from a buffer without advancing the original buffer's cursor.
///
/// `RefPeeker` provides a non-destructive way to examine buffer contents by maintaining
/// its own independent cursor position. This is particularly useful when you need to:
/// - Look ahead in a buffer before deciding how to process the data
/// - Parse data that might need to be rolled back
/// - Implement backtracking algorithms
/// - Share read access to the same buffer data from different positions
///
/// The peeker can be constrained to a specific range within the buffer, making it
/// safe for parsing operations that should not read beyond certain boundaries.
///
/// # Examples
///
/// ```rust
/// use buffo::{Chunk, RefPeeker};
///
/// let data = b"Hello, World!";
/// let buf = &data[..];
/// let mut peeker = RefPeeker::new(&buf);
///
/// // Read without affecting the original buffer
/// assert_eq!(peeker.read_u8(), b'H');
/// assert_eq!(peeker.read_u8(), b'e');
/// assert_eq!(buf.remaining(), 13); // Original buffer unchanged
///
/// // Create a constrained peeker for safe parsing
/// let mut word_peeker = peeker.segment(0..5); // "Hello"
/// assert_eq!(word_peeker.remaining(), 5);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RefPeeker<'a, B: ?Sized> {
  buf: &'a B,
  cursor: usize,
  /// The original start bound of the peeker, used for resetting.
  start: Bound<usize>,
  /// The end bound that [`reset`](RefPeeker::reset) restores to. Initialized to the
  /// construction-time end bound, but re-anchored by `truncate` (and `split_off` /
  /// `split_to`) so that a later `reset` stays within a narrowed window.
  end: Bound<usize>,
  /// Current limit bound of the peeker
  limit: Bound<usize>,
}

impl<'a, B: 'a + ?Sized> From<&'a B> for RefPeeker<'a, B> {
  #[inline]
  fn from(buf: &'a B) -> Self {
    Self::new(buf)
  }
}

impl<'a, B: 'a + ?Sized> Clone for RefPeeker<'a, B> {
  #[inline]
  fn clone(&self) -> Self {
    *self
  }
}

impl<'a, B: 'a + ?Sized> Copy for RefPeeker<'a, B> {}

impl<'a, B: 'a + ?Sized> RefPeeker<'a, B> {
  /// Creates a new `RefPeeker` instance with the given buffer.
  ///
  /// The peeker starts at the beginning of the buffer's current position
  /// and can read all remaining bytes in the buffer.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  /// let peeker = RefPeeker::new(&buf);
  /// assert_eq!(peeker.remaining(), 5);
  /// ```
  #[inline]
  pub const fn new(buf: &'a B) -> Self {
    Self::with_cursor_and_bounds_inner(buf, 0, Bound::Unbounded, Bound::Unbounded)
  }

  /// Creates a new `RefPeeker` constrained to a specific length.
  ///
  /// This is useful when you want to ensure the peeker cannot read beyond
  /// a certain number of bytes, providing additional safety for parsing operations.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = b"Hello, World!";
  /// let buf = &data[..];
  /// let peeker = RefPeeker::with_limit(&buf, 5); // Only peek first 5 bytes
  /// assert_eq!(peeker.remaining(), 5);
  /// ```
  #[inline]
  pub const fn with_limit(buf: &'a B, limit: usize) -> Self {
    Self::with_cursor_and_bounds_inner(buf, 0, Bound::Unbounded, Bound::Excluded(limit))
  }

  /// Creates a new `RefPeeker` with specific start and end bounds.
  ///
  /// This provides maximum flexibility in defining the peeker's range,
  /// allowing for more complex parsing scenarios.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use core::ops::Bound;
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = b"Hello, World!";
  /// let buf = &data[..];
  ///
  /// // Peek from index 2 to 7 (exclusive)
  /// let peeker = RefPeeker::with_range(&buf, 2..7);
  /// assert_eq!(peeker.remaining(), 5);
  /// ```
  #[inline]
  pub fn with_range(buf: &'a B, range: impl RangeBounds<usize>) -> Self
  where
    B: Chunk,
  {
    let start = range.start_bound().cloned();
    let end = range.end_bound().cloned();
    let start_pos = Self::resolve_start_bound(start, buf);
    Self::with_cursor_and_bounds_inner(buf, start_pos, Bound::Included(start_pos), end)
  }

  /// Returns the bounds of the peeker.
  ///
  /// This is useful for understanding the range within which the peeker can read.
  /// The bounds are represented as a tuple of `Bound<usize>`, indicating the start
  /// and end positions.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use core::ops::Bound;
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  /// let peeker = RefPeeker::with_range(&buf, 1..4);
  /// assert_eq!(peeker.bounds(), (Bound::Included(1), Bound::Excluded(4)));
  /// ```
  #[inline]
  pub const fn bounds(&self) -> (Bound<usize>, Bound<usize>) {
    (self.start, self.end)
  }

  /// Returns the current position of the internal cursor relative to the start of the buffer.
  ///
  /// This represents how far the peeker's cursor has advanced from its starting position.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  /// let mut peeker = RefPeeker::new(&buf);
  ///
  /// assert_eq!(peeker.position(), 0);
  /// peeker.read_u8();
  /// assert_eq!(peeker.position(), 1);
  /// peeker.advance(2);
  /// assert_eq!(peeker.position(), 3);
  /// ```
  #[inline]
  pub const fn position(&self) -> usize {
    let start_pos = Self::resolve_start_bound_without_check(self.start);
    self.cursor.saturating_sub(start_pos)
  }

  /// Returns the absolute position of the peeker's cursor in the original buffer.
  ///
  /// This is useful for understanding where the peeker is currently reading
  /// in relation to the entire buffer, especially when the peeker has been advanced
  /// or segmented.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  /// let mut peeker = RefPeeker::new(&buf);
  ///
  /// let mut seg = peeker.segment(1..4);
  /// seg.advance(2);
  /// assert_eq!(seg.absolute_position(), 3); // 1 (start) + 2 (advanced)
  ///
  /// // Resetting the peeker will bring it back to the start position
  /// seg.reset();
  /// assert_eq!(seg.absolute_position(), 1); // Back to the start
  /// ```
  #[inline]
  pub const fn absolute_position(&self) -> usize {
    self.cursor
  }

  /// Resets the peeker's cursor to the beginning.
  ///
  /// After calling this method, the peeker will start reading from the same
  /// position where it was initially created.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use buffo::{Chunk, RefPeeker};
  ///
  /// let data = [1, 2, 3, 4, 5];
  /// let buf = &data[..];
  /// let mut peeker = RefPeeker::new(&buf);
  ///
  /// peeker.advance(3);
  /// assert_eq!(peeker.position(), 3);
  ///
  /// peeker.reset();
  /// assert_eq!(peeker.position(), 0);
  /// assert_eq!(peeker.remaining(), 5);
  /// ```
  #[inline]
  pub const fn reset(&mut self) {
    self.cursor = Self::resolve_start_bound_without_check(self.start);
    self.limit = self.end;
  }

  #[inline]
  const fn with_cursor_and_bounds_inner(
    buf: &'a B,
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

  #[inline]
  fn resolve_start_bound(bound: Bound<usize>, buf: &B) -> usize
  where
    B: Chunk,
  {
    let pos = match bound {
      Bound::Included(n) => n,
      Bound::Excluded(n) => n.saturating_add(1),
      Bound::Unbounded => 0,
    };
    pos.min(buf.remaining())
  }

  #[inline]
  fn resolve_end_bound(&self, bound: Bound<usize>) -> usize
  where
    B: Chunk,
  {
    match bound {
      Bound::Included(n) => n.saturating_add(1),
      Bound::Excluded(n) => n,
      Bound::Unbounded => self.buf.remaining(),
    }
  }

  /// Computes the clamped half-open `[start, end)` view into the backing buffer
  /// that is shared by both [`remaining`](Chunk::remaining) and
  /// [`buffer`](Chunk::buffer).
  ///
  /// The end is clamped to the backing buffer's length and the start is clamped
  /// to that end, guaranteeing `start <= end <= self.buf.remaining()`. This keeps
  /// `buffer()` panic-free and upholds the `buffer().len() == remaining()`
  /// invariant for every `(limit, range)`, including out-of-range limits,
  /// inverted ranges, and `..=usize::MAX`.
  #[inline]
  fn view_bounds(&self) -> (usize, usize)
  where
    B: Chunk,
  {
    let inner = self.buf.remaining();
    let end = self.resolve_end_bound(self.limit).min(inner);
    let start = self.cursor.min(end);
    (start, end)
  }
}

impl<'a, B: 'a + Chunk + ?Sized> Chunk for RefPeeker<'a, B> {
  #[inline]
  fn remaining(&self) -> usize {
    let (start, end) = self.view_bounds();
    end - start
  }

  #[inline]
  fn buffer(&self) -> &[u8] {
    let (start, end) = self.view_bounds();
    &self.buf.buffer()[start..end]
  }

  #[inline]
  fn advance(&mut self, cnt: usize) {
    let remaining = self.remaining();
    if cnt > remaining {
      panic_advance(&TryAdvanceError::new(must_non_zero(cnt), remaining));
    }
    self.cursor += cnt;
  }

  #[inline]
  fn try_advance(&mut self, cnt: usize) -> Result<(), TryAdvanceError> {
    let remaining = self.remaining();
    if cnt > remaining {
      return Err(TryAdvanceError::new(must_non_zero(cnt), remaining));
    }
    self.cursor += cnt;
    Ok(())
  }

  #[inline]
  fn segment(&self, range: impl core::ops::RangeBounds<usize>) -> Self
  where
    Self: Sized,
  {
    let current_remaining = self.remaining();

    let begin = match range.start_bound() {
      Bound::Included(&n) => n,
      Bound::Excluded(&n) => n.checked_add(1).expect("range start overflow"),
      Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
      Bound::Included(&n) => n.checked_add(1).expect("range end overflow"),
      Bound::Excluded(&n) => n,
      Bound::Unbounded => current_remaining,
    };

    super::check_segment_range_bounds(begin, end, current_remaining);

    let start = self.cursor + begin;
    let start_bound = Bound::Included(start);
    if end == begin {
      let end_bound = Bound::Excluded(start);
      return Self::with_cursor_and_bounds_inner(self.buf, start, start_bound, end_bound);
    }

    let end_bound = Bound::Excluded(self.cursor + end);
    Self::with_cursor_and_bounds_inner(self.buf, start, start_bound, end_bound)
  }

  /// Narrows the view to at most `len` bytes from the current cursor.
  ///
  /// This also re-anchors the reset target (`end`), so a later [`reset`](RefPeeker::reset)
  /// stays within the truncated window instead of widening back to the original bound.
  /// This diverges from `Putter`, whose `reset` / `reset_position` restores the
  /// construction-time limit and re-widens past a prior `truncate_mut`.
  #[inline]
  fn truncate(&mut self, len: usize) {
    let current_remaining = self.remaining();
    if len >= current_remaining {
      return; // No truncation needed
    }

    let new_end_pos = self.cursor + len;
    let current_end_pos = self.resolve_end_bound(self.limit);

    // Only truncate if the new limit is more restrictive than the current one
    if new_end_pos < current_end_pos {
      let new_end = Bound::Excluded(new_end_pos);
      self.limit = new_end;
      // Re-anchor the reset target too, so a later `reset()` cannot widen the
      // view back past this truncation. This mirrors `split_off`/`split_to`,
      // which also update `end` to keep `reset()` bounded.
      self.end = new_end;
    }
  }

  #[inline]
  fn split_off(&mut self, at: usize) -> Self
  where
    Self: Sized,
  {
    let remaining = self.remaining();
    check_out_of_bounds("split_off", at, remaining);

    let new_cursor = self.cursor + at;
    let old_limit = self.limit;

    // Truncate self to [0, at) and update end so reset() stays bounded
    let self_end = Bound::Excluded(new_cursor);
    self.limit = self_end;
    self.end = self_end;

    // Return the right part [at, end) with start at the split point
    let start = Bound::Included(new_cursor);
    Self::with_cursor_and_bounds_inner(self.buf, new_cursor, start, old_limit)
  }

  #[inline]
  fn split_to(&mut self, at: usize) -> Self
  where
    Self: Sized,
  {
    let remaining = self.remaining();
    check_out_of_bounds("split_to", at, remaining);

    let old_cursor = self.cursor;
    let new_end = self.cursor + at;

    // Advance self to [at, end) and update start/end so reset() stays bounded
    self.cursor += at;
    self.start = Bound::Included(self.cursor);
    self.end = self.limit;

    let start = Bound::Included(old_cursor);
    let end = Bound::Excluded(new_end);
    // Return the left part [0, at)
    Self::with_cursor_and_bounds_inner(self.buf, old_cursor, start, end)
  }
}

#[cfg(test)]
#[allow(clippy::reversed_empty_ranges, clippy::clone_on_copy)]
mod tests {
  use super::*;

  #[test]
  fn test_peeker_basic_functionality() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    assert_eq!(peeker.remaining(), 5);
    assert_eq!(peeker.position(), 0);

    // Read a byte
    assert_eq!(peeker.read_u8(), 1);
    assert_eq!(peeker.remaining(), 4);
    assert_eq!(peeker.position(), 1);

    // Original buffer should be unchanged
    assert_eq!(buf.remaining(), 5);
  }

  #[test]
  fn test_peeker_with_limit() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::with_limit(&buf, 3);

    assert_eq!(peeker.remaining(), 3);

    // Read within limit
    assert_eq!(peeker.read_u8(), 1);
    assert_eq!(peeker.read_u8(), 2);
    assert_eq!(peeker.read_u8(), 3);

    // Should be at limit now
    assert_eq!(peeker.remaining(), 0);
  }

  #[test]
  fn test_peeker_reset() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    peeker.advance(3);
    assert_eq!(peeker.position(), 3);
    assert_eq!(peeker.remaining(), 2);

    peeker.reset();
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.remaining(), 5);
  }

  #[test]
  fn test_peeker_segment() {
    let data = b"Hello, World!";
    let buf = &data[..];
    let peeker = RefPeeker::new(&buf);

    // Segment "Hello"
    let mut hello_peeker = peeker.segment(0..5);
    assert_eq!(hello_peeker.remaining(), 5);
    assert_eq!(hello_peeker.read_u8(), b'H');
    assert_eq!(hello_peeker.read_u8(), b'e');

    // Segment "World"
    let mut world_peeker = peeker.segment(7..12);
    assert_eq!(world_peeker.remaining(), 5);
    assert_eq!(world_peeker.read_u8(), b'W');
    assert_eq!(world_peeker.read_u8(), b'o');
  }

  #[test]
  fn test_peeker_segment_edge_1() {
    let buf = [1, 2, 3, 4, 5];
    let slice = &buf[..];
    let slice = RefPeeker::from(&slice);
    let output = slice.segment((Bound::Excluded(1), Bound::Included(3)));
    assert_eq!(output.buffer(), &[3, 4]);
  }

  #[test]
  fn test_peeker_segment_edge_2() {
    let buf = [1, 2, 3, 4, 5];
    let slice = &buf[..];
    let slice = RefPeeker::from(&slice);
    let output = slice.segment(2..);
    assert_eq!(output.buffer(), &[3, 4, 5]);
  }

  #[test]
  fn test_peeker_truncate() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::from(&buf);

    peeker.truncate(3);
    assert_eq!(peeker.remaining(), 3);

    // Should only be able to read 3 bytes
    assert_eq!(peeker.read_u8(), 1);
    assert_eq!(peeker.read_u8(), 2);
    assert_eq!(peeker.read_u8(), 3);
    assert_eq!(peeker.remaining(), 0);

    let mut peeker = RefPeeker::with_limit(&buf, 3);
    assert_eq!(peeker.remaining(), 3);
    peeker.truncate(2);

    assert_eq!(peeker.remaining(), 2);
  }

  #[test]
  #[should_panic]
  fn test_peeker_split_off_panic() {
    let data = [1u8, 2, 3];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    // This should panic because we are trying to split_off more than available
    let _ = peeker.split_off(5);
  }

  #[test]
  fn test_peeker_split_off() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    let right_peeker = peeker.split_off(2);

    // Left part should have first 2 bytes
    assert_eq!(peeker.remaining(), 2);
    assert_eq!(peeker.read_u8(), 1);
    assert_eq!(peeker.read_u8(), 2);

    // Right part should have remaining bytes
    assert_eq!(right_peeker.remaining(), 3);
    let mut right = right_peeker;
    assert_eq!(right.position(), 0);
    assert_eq!(right.read_u8(), 3);
    assert_eq!(right.read_u8(), 4);
    assert_eq!(right.read_u8(), 5);
  }

  #[test]
  fn test_peeker_split_off_reset() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    let mut right = peeker.split_off(2);

    // Right part: read some then reset
    assert_eq!(right.position(), 0);
    assert_eq!(right.read_u8(), 3);
    assert_eq!(right.position(), 1);
    right.reset();
    assert_eq!(right.position(), 0);
    assert_eq!(right.remaining(), 3);
    assert_eq!(right.read_u8(), 3); // reads from split point, not original start

    // Left part: read some then reset
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.read_u8(), 1);
    peeker.reset();
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.remaining(), 2); // bounded to [0, 2), not full buffer
    assert_eq!(peeker.read_u8(), 1);
  }

  #[test]
  #[should_panic]
  fn test_peeker_split_to_panic() {
    let data = [1u8, 2, 3];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    // This should panic because we are trying to split_to more than available
    let _ = peeker.split_to(5);
  }

  #[test]
  fn test_peeker_split_to() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    let left_peeker = peeker.split_to(2);

    // Left part should have first 2 bytes
    assert_eq!(left_peeker.remaining(), 2);
    let mut left = left_peeker;
    assert_eq!(left.read_u8(), 1);
    assert_eq!(left.read_u8(), 2);

    // Remaining peeker should have rest
    assert_eq!(peeker.remaining(), 3);
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.read_u8(), 3);
    assert_eq!(peeker.read_u8(), 4);
    assert_eq!(peeker.read_u8(), 5);
  }

  #[test]
  fn test_peeker_split_to_reset() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    let mut left = peeker.split_to(2);

    // Left part: read some then reset
    assert_eq!(left.read_u8(), 1);
    left.reset();
    assert_eq!(left.position(), 0);
    assert_eq!(left.remaining(), 2);
    assert_eq!(left.read_u8(), 1);

    // Remaining part: read some then reset
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.read_u8(), 3);
    assert_eq!(peeker.position(), 1);
    peeker.reset();
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.remaining(), 3); // bounded to [2, 5), not full buffer
    assert_eq!(peeker.read_u8(), 3); // reads from split point, not original start
  }

  #[test]
  fn test_peeker_try_advance() {
    let data = [1u8, 2, 3];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    assert!(peeker.try_advance(2).is_ok());
    assert_eq!(peeker.position(), 2);

    // Should fail when trying to advance beyond available data
    assert!(peeker.try_advance(5).is_err());
    assert_eq!(peeker.position(), 2); // Should remain unchanged
  }

  #[test]
  #[should_panic(expected = "advance")]
  fn test_peeker_advance_panic() {
    let data = [1u8, 2, 3];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    peeker.advance(5); // Should panic
  }

  #[test]
  fn test_peeker_empty_segment() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let peeker = RefPeeker::new(&buf);

    let empty_peeker = peeker.segment(2..2);
    assert_eq!(empty_peeker.remaining(), 0);
  }

  #[test]
  fn test_peeker_clone_and_copy() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let mut peeker = RefPeeker::new(&buf);

    peeker.advance(2);

    let peeker_clone = peeker.clone();
    let peeker_copy = peeker;

    assert_eq!(peeker_clone.position(), 2);
    assert_eq!(peeker_copy.position(), 2);
    assert_eq!(peeker_clone.remaining(), peeker_copy.remaining());
  }

  #[test]
  fn test_peeker_with_advanced_buffer() {
    let data = [1u8, 2, 3, 4, 5];
    let mut buf = &data[..];

    // Advance the original buffer
    buf.advance(2);
    assert_eq!(buf.remaining(), 3);

    // RefPeeker should work with the advanced buffer
    let mut peeker = RefPeeker::new(&buf);
    assert_eq!(peeker.remaining(), 3);
    assert_eq!(peeker.read_u8(), 3); // Should read the 3rd byte
  }

  #[test]
  #[should_panic]
  fn test_peeker_segment_panic_1() {
    let data = b"Hello, World!";
    let buf = &data[..];
    let peeker = RefPeeker::new(&buf);

    // This should panic because the segment end is out of bounds
    let _ = peeker.segment(3..1);
  }

  #[test]
  #[should_panic]
  fn test_peeker_segment_panic_2() {
    let data = b"Hello, World!";
    let buf = &data[..];
    let peeker = RefPeeker::new(&buf);

    // This should panic because the segment end is out of bounds
    let _ = peeker.segment(..20);
  }

  #[test]
  fn test_peeker_multi_level_segments() {
    let data = b"Hello, World!";
    let buf = &data[..];
    let peeker = RefPeeker::new(&buf);

    // Create a segment, then segment that
    let hello_comma = peeker.segment(0..6); // "Hello,"
    let hello = hello_comma.segment(0..5); // "Hello"

    assert_eq!(hello.remaining(), 5);
    let mut hello_mut = hello;
    assert_eq!(hello_mut.read_u8(), b'H');
    assert_eq!(hello_mut.read_u8(), b'e');
  }

  #[test]
  fn test_peeker_range() {
    let data = b"Hello, World!";
    let buf = &data[..];

    // Peek from index 2 to 7 (inclusive)
    let mut peeker = RefPeeker::with_range(&buf, 2..=7);
    assert_eq!(peeker.remaining(), 6);
    assert_eq!(peeker.buffer(), b"llo, W");
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.absolute_position(), 2);

    peeker.read_u8();
    assert_eq!(peeker.position(), 1);
    assert_eq!(peeker.absolute_position(), 3);
    peeker.reset();
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.absolute_position(), 2);

    let mut peeker = RefPeeker::with_range(&buf, ..5);
    assert_eq!(peeker.remaining(), 5);
    assert_eq!(peeker.buffer(), b"Hello");
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.absolute_position(), 0);

    peeker.read_u8();
    assert_eq!(peeker.position(), 1);
    assert_eq!(peeker.absolute_position(), 1);
    peeker.reset();
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.absolute_position(), 0);

    let mut peeker = RefPeeker::with_range(&buf, (Bound::Excluded(1), Bound::Unbounded));
    assert_eq!(peeker.remaining(), 11);
    assert_eq!(peeker.buffer(), b"llo, World!");
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.absolute_position(), 2);

    peeker.read_u8();
    assert_eq!(peeker.position(), 1);
    assert_eq!(peeker.absolute_position(), 3);
    peeker.reset();
    assert_eq!(peeker.position(), 0);
    assert_eq!(peeker.absolute_position(), 2);
  }

  // Regression: end bound was stored raw, so `remaining()` overreported and
  // `buffer()` panicked for out-of-range limits and inverted ranges. The clamped
  // shared view must keep `buffer().len() == remaining()` and stay panic-free.
  #[test]
  fn test_peeker_panic_safety_out_of_range_bounds() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];
    let len = data.len(); // 5

    // `with_limit` past the buffer length: previously `remaining() == len + 3`
    // (a lie) and `buffer()` panicked slicing `&buf[0..len + 3]`.
    let mut p = RefPeeker::with_limit(&buf, len + 3);
    assert_eq!(p.buffer().len(), p.remaining());
    assert_eq!(p.remaining(), len);
    assert_eq!(p.peek_u8_checked(), Some(1));
    assert!(p.try_advance(p.remaining()).is_ok());
    assert_eq!(p.remaining(), 0);
    assert!(p.try_advance(1).is_err());

    // Inverted range: an empty, non-panicking view (previously panicked
    // slicing `&buf[5..2]`).
    let mut p = RefPeeker::with_range(&buf, 5..2);
    assert_eq!(p.buffer().len(), p.remaining());
    assert_eq!(p.remaining(), 0);
    assert_eq!(p.peek_u8_checked(), None);
    assert!(p.try_advance(0).is_ok());
    assert!(p.try_advance(1).is_err());

    // `..=usize::MAX`: end clamped to the backing length (previously
    // `remaining() == usize::MAX`, an allocation oracle).
    let mut p = RefPeeker::with_range(&buf, ..=usize::MAX);
    assert_eq!(p.buffer().len(), p.remaining());
    assert_eq!(p.remaining(), len);
    assert_eq!(p.peek_u8_checked(), Some(1));
    assert!(p.try_advance(len).is_ok());

    // Explicit end past the backing length.
    let mut p = RefPeeker::with_range(&buf, 0..(len + 5));
    assert_eq!(p.buffer().len(), p.remaining());
    assert_eq!(p.remaining(), len);
    assert_eq!(p.peek_u8_checked(), Some(1));
    assert!(p.try_advance(len).is_ok());
  }

  // Regression: `truncate()` narrowed `limit` but not `end`, so a later `reset()`
  // widened the view back past the truncation. `truncate()` must now re-anchor
  // `end` like the split methods.
  #[test]
  fn test_peeker_truncate_then_reset_stays_bounded() {
    let data = [1u8, 2, 3, 4, 5];
    let buf = &data[..];

    let mut peeker = RefPeeker::new(&buf);
    peeker.truncate(2);
    assert_eq!(peeker.remaining(), 2);
    peeker.reset();
    assert!(peeker.remaining() <= 2);
    assert_eq!(peeker.remaining(), 2);
    assert_eq!(peeker.buffer().len(), peeker.remaining());

    // truncate -> split -> reset interactions stay bounded to the truncated window.
    let mut peeker = RefPeeker::new(&buf);
    peeker.truncate(4);
    let right = peeker.split_off(2);
    assert_eq!(peeker.remaining(), 2);
    assert_eq!(right.remaining(), 2); // [2, 4) after truncation, not [2, 5)
    let mut left = peeker;
    left.reset();
    assert!(left.remaining() <= 2);
  }
}
