# uklad

[![crates.io](https://img.shields.io/crates/v/uklad.svg)](https://crates.io/crates/uklad)
[![docs.rs](https://img.shields.io/docsrs/uklad)](https://docs.rs/uklad)
[![license](https://img.shields.io/crates/l/uklad.svg)](LICENSE-MIT)

`uklad` is a compact binary serialization format and Rust implementation. It
encodes values without field names, tags, or other metadata, so the output is
typically no larger than the in-memory representation of the value.

## Features

- `Encode` and `Decode` traits with derive macros.
- Compact variable-length integer encoding, with an alternative fixed-width
  encoding for predictable sizes.
- Configurable byte order and decode size limits.
- `no_std` support with an optional `alloc` feature.
- Optional [serde](https://serde.rs) interoperability.
- No required runtime dependencies.

## Installation

```toml
[dependencies]
uklad = "1.0"
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

    let encoded = uklad::encode_to_vec(&world, config).unwrap();
    let (decoded, len): (World, usize) =
        uklad::decode_from_slice(&encoded, config).unwrap();

    assert_eq!(world, decoded);
    assert_eq!(len, encoded.len());
}
```

The same types work with files, network streams, custom readers and writers, and
pre-allocated buffers:

| Situation              | Encode                  | Decode                  |
| ---------------------- | ----------------------- | ----------------------- |
| Files, streams         | `encode_into_std_write` | `decode_from_std_read`  |
| In-memory buffers      | `encode_to_vec`         | `decode_from_slice`     |
| Custom reader/writer   | `encode_into_writer`    | `decode_from_reader`    |
| Pre-allocated buffers  | `encode_into_slice`     | `decode_from_slice`     |

See the [API documentation](https://docs.rs/uklad) for the full API.

## Feature flags

| Feature  | Default | Description                                                              |
| -------- | ------- | ------------------------------------------------------------------------ |
| `std`    | yes     | Standard library support, including `std::io` readers and writers.       |
| `alloc`  | yes     | Support for `Vec`, `String`, `Box`, and other allocation-backed types.   |
| `derive` | yes     | `#[derive(Encode, Decode, BorrowDecode)]` macros.                        |
| `serde`  | no      | Adapters and encode/decode functions for serde types.                    |

For `no_std` targets with an allocator:

```toml
[dependencies]
uklad = { version = "1.0", default-features = false, features = ["alloc", "derive"] }
```

## Configuration

`config::standard()` enables compact variable-length integer encoding.
`config::legacy()` uses fixed-width integer encoding. Both can be adjusted for
byte order and size limits:

```rust
let config = uklad::config::standard()
    .with_big_endian()
    .with_limit::<1024>();
```

## Untrusted input

Decoding does not trust length prefixes. When the input may come from an
untrusted source, set a limit with `Configuration::with_limit`. That bounds
pre-allocation and makes decoding fail with `DecodeError::LimitExceeded` instead
of attempting to allocate an excessive amount of memory.

## Wire format

The format is specified in [docs/spec.md](docs/spec.md). It is stable across
releases of the 1.x series.

## License

Licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT), at your option.
