# RELEASED

## 0.8.0 (Jul 17th, 2026)

Renamed the crate from `bufkit` to `buffo`, bumped `varing` to `0.14`, added an
optional `smol-bytes` integration, and raised the MSRV to 1.85.

### Added

- Optional `smol_bytes_01` feature integrating
  [`smol-bytes`](https://crates.io/crates/smol-bytes), read-only like the
  `bytes_1` integration (owned buffers get `Chunk`, not `ChunkMut`): `Chunk`
  for `smol_bytes::Bytes`, `compact::Bytes`, `BytesMut`, and the inline
  `Buffer` (the latter usable even in `no_std` + no-alloc; its `Chunk` impl
  is pure-core); `to_smol_bytes`/`to_smol_bytes_mut` conversions; and
  `From<smol_bytes::TryGetError>` conversions. The UTF-8 types are
  intentionally not integrated — their invariant is incompatible with
  raw-byte `ChunkMut` access; reach the byte-level type via their
  `as_inner()`/`into_inner()`. Raises the MSRV to 1.85 (required by
  `smol-bytes`).

### Fixed (soundness / panic-freedom)

- The varint encoders (`put_varint`/`put_varint_at`/`write_varint`) now pre-flight
  `encoded_len()` and fail without calling `encode`, so a value that does not fit no
  longer partially overwrites the destination buffer on the error path (all-or-error).
- `Peeker`, `RefPeeker`, and `Putter` clamp the view end bound, so an out-of-range,
  inverted, or unbounded limit no longer makes `remaining()` over-report or
  `buffer()`/`buffer_mut()` panic inside the non-panicking (`_checked`/`try_*`) APIs.
- `From<bytes::TryGetError>` is now total instead of panicking in release inside
  error-conversion paths.

### Changed

- `io::ErrorKind` mapping is now variant-explicit and consistent across read and write
  (`OutOfBounds` -> `InvalidInput`, `InsufficientSpace` -> `WriteZero`).
- `try_peek_u8_at`/`try_peek_i8_at` report `InsufficientData` (not `OutOfBounds`) at
  `offset == len`, matching the other `*_at` methods.
- `OutOfBounds::excess()` saturates instead of overflowing.
- The untyped varint scan (`try_scan_varint`/`try_scan_varint_at`/`try_consume_varint`)
  is documented as a structural skip; it does not bound the value to a type width. Use
  the typed `read_varint`/`peek_varint` for type-bounded decoding.

### Added

- `DecodeVarintAtError::Overflow`, so the varint overflow signal is preserved rather
  than flattened into an opaque error.

### Fixed (build)

- Corrected the `bytes` dependency floor to `1.9` (`try_get_*`/`TryGetError`) and the
  `test_to_bytes` cfg gate (`--no-default-features --features bytes_1` now compiles).

## 0.5.1 (Aug 21st, 2025)

- Fix typo in README

## 0.5.0 (Aug 14th, 2025)

- Add consume and scan varint related methods for `ChunkExt`
- Change type of `requested` from `usize` to `NonZeroUsize`
- Bumpup `varing` to `0.10`
- Change feature `varing` to feature `varint`

## 0.4.0 (Aug 11st, 2025)

- Rename `buffo::{Buf, BufMut}` to `buffo::{Chunk, ChunkMut}` to avoid collisions with `bytes::{Buf, BufMut}`.

## 0.3.0 (Aug 8th, 2025)

- Add `RefPeeker`, `Peeker` and `Putter`

## 0.2.2 (Aug 6th, 2025)

- Add `WriteBuf` for convenient trait API design

## 0.2.0 (Aug 6th, 2025)

- Remove `BufMut` implementation for `BytesMut` and `Vec<u8>`
- Add `advance_mut` and `try_advance_mut` for `BufMut`
- Add `write_*` and `try_write_*` APIs

## 0.1.4 (Aug 5th, 2025)

- Add `buffer_from` and `buffer_from_checked`

## 0.1.3 (Aug 3rd, 2025)

- Change `requested: usize` to `requested: NonZeroUsize`

## 0.1.2 (Aug 2nd, 2025)

- Finish basic implementation
