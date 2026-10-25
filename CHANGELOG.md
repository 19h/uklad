# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-10-25

### Added

- Initial release of `uklad` and `uklad_derive`.
- `Encode`, `Decode`, and `BorrowDecode` traits with derive macros.
- Variable-length and fixed-width integer encodings.
- Configurable byte order and decode size limits.
- `no_std` support with an optional `alloc` feature.
- Optional serde interoperability.
- Serialization format specification in `docs/spec.md`.

[1.0.0]: https://github.com/19h/uklad/releases/tag/v1.0.0
