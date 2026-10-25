# uklad_derive

Derive macros for [`uklad`](https://crates.io/crates/uklad): `Encode`, `Decode`, and `BorrowDecode`.

## Structure

- `src/attrs.rs` parses `#[uklad(...)]` container, field, and variant attributes.
- `src/expand.rs` generates the trait implementations.
- `src/lib.rs` exposes the proc macros.

## Attributes

Container options: `crate`, `bounds`, `encode_bounds`, `decode_bounds`, `borrow_decode_bounds`, `decode_context`.

Field options: `skip`, `default`, `with`, `with_serde`.

Variant options: `with`.

See the `uklad` documentation for usage.
