# Uklad-derive

The derive crate for uklad. Implements `uklad::Encode` and `uklad::Decode`.

# Structure

- `src/attribute.rs` parses the `#[uklad(...)]` container and field attributes.
- `src/derive_struct.rs` generates the `Encode`/`Decode` implementations for structs.
- `src/derive_enum.rs` generates the `Encode`/`Decode` implementations for enums.
- `src/lib.rs` wires the derive macros together.

For additional derive testing, see the test cases in `../tests`.

For testing purposes, all generated code is outputted to the current `target/generated/uklad` folder, under file name `<struct/enum name>_Encode.rs` and `<struct/enum name>_Decode.rs`. This can help with debugging.
