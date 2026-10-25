# uklad

A compact binary encoder and decoder for Rust. Values are serialized without
field names or other metadata, so the encoded output is typically the same size
as, or smaller than, the value's in-memory representation.

The crate provides:

- `Encode` and `Decode` traits with derive macros.
- No required dependencies.
- `no_std` support with optional `alloc`.
- Configurable byte order, integer encoding, and size limits.
- Optional serde interoperability.

## Installation

```toml
[dependencies]
uklad = "2.0"
```

The minimum supported Rust version is 1.85.0.

## Usage

```rust
use uklad::{config, Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct World(Vec<Entity>);

fn main() {
    let config = config::standard();

    let world = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);

    let encoded: Vec<u8> = uklad::encode_to_vec(&world, config).unwrap();

    let (decoded, len): (World, usize) =
        uklad::decode_from_slice(&encoded, config).unwrap();

    assert_eq!(world, decoded);
    assert_eq!(len, encoded.len());
}
```

Encoding and decoding also work directly with readers and writers, and with
pre-allocated buffers. See the
[documentation](https://docs.rs/uklad/latest/uklad/) for the full API.

## Feature flags

| Feature  | Default | Description                                                        |
| -------- | ------- | ------------------------------------------------------------------ |
| `std`    | yes     | Standard library support, including `std::io` readers and writers. |
| `alloc`  | yes     | Support for `Vec`, `String`, `Box`, and other alloc types.         |
| `derive` | yes     | `#[derive(Encode, Decode)]` macros.                                |
| `serde`  | no      | Adapters and encode/decode functions for serde types.              |

For `no_std` targets with an allocator:

```toml
[dependencies]
uklad = { version = "2.0", default-features = false, features = ["alloc", "derive"] }
```

## Configuration

`config::standard()` selects compact variable-length integers. `config::legacy()`
uses a fixed-length integer encoding. Both can be adjusted for byte order and
size limits:

```rust
let config = uklad::config::standard()
    .with_big_endian()
    .with_limit::<1024>();
```

## Untrusted input

When decoding data from an untrusted source, enable a size limit with
`Configuration::with_limit`. This bounds pre-allocation and causes decoding to
fail with `DecodeError::LimitExceeded` instead of attempting to allocate an
excessive amount of memory.

## Format

The serialization format is specified in [docs/spec.md](docs/spec.md).
