# Migrating from uklad 1 to 2

Uklad 2 now has an optional dependency on `serde`. You can either use `serde`, or use uklad's own `derive` feature and macros.

## From `Options` to `Configuration`

Uklad 1 had the [Options](https://docs.rs/uklad/1/uklad/config/trait.Options.html) trait. This has been replaced with the [Configuration](https://docs.rs/uklad/2/uklad/config/struct.Configuration.html) struct.

If you're using `Options`, you can change it like this:

```rust,ignore
# old
uklad_1::DefaultOptions::new().with_varint_encoding()

# new
uklad_2::config::legacy().with_variable_int_encoding()
```

If you want to be compatible with uklad 1, use the following table:

| Uklad 1                                                              | Uklad 2                                       |
| ---------------------------------------------------------------------- | ----------------------------------------------- |
| version 1.0 - 1.2 with `uklad_1::DefaultOptions::new().serialize(T)` | `config::legacy()`                              |
| version 1.3+ with `uklad_1::DefaultOptions::new().serialize(T)`      | `config::legacy().with_variable_int_encoding()` |
| No explicit `Options`, e.g. `uklad::serialize(T)`                    | `config::legacy()`                              |

If you do not care about compatibility with uklad 1, we recommend using `config::standard()`

The following changes have been made:

- `.with_limit(n)` has been changed to `.with_limit::<n>()`.
- `.with_native_endian()` has been removed. Use `.with_big_endian()` or `with_little_endian()` instead.
- `.with_varint_encoding()` has been renamed to `.with_variable_int_encoding()`.
- `.with_fixint_encoding()` has been renamed to `.with_fixed_int_encoding()`.
- `.reject_trailing_bytes()` has been removed.
- `.allow_trailing_bytes()` has been removed.
- You can no longer (de)serialize from the `Options` trait directly. Use one of the `encode_` or `decode_` methods.

Because of confusion with `Options` defaults in uklad 1, we have made `Configuration` mandatory in all calls in uklad 2.

## Migrating with `serde`

You may wish to stick with `serde` when migrating to uklad 2, for example if you are using serde-exclusive derive features such as `#[serde(deserialize_with)]`.

If so, make sure to include uklad 2 with the `serde` feature enabled, and use the `uklad::serde::*` functions instead of `uklad::*` as described below:

```toml
[dependencies]
uklad = { version = "2.0", features = ["serde"] }

# Optionally you can disable the `derive` feature:
# uklad = { version = "2.0", default-features = false, features = ["std", "serde"] }
```

Then replace the following functions: (`Configuration` is `uklad::config::legacy()` by default)

| Uklad 1                                       | Uklad 2                                                                                                                       |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `uklad::deserialize(&[u8])`                   | `uklad::serde::decode_from_slice(&[u8], Configuration)`<br />`uklad::serde::borrow_decode_from_slice(&[u8], Configuration)` |
| `uklad::deserialize_from(std::io::Read)`      | `uklad::serde::decode_from_std_read(std::io::Read, Configuration)`                                                            |
| `uklad::deserialize_from_custom(UkladRead)` | `uklad::serde::decode_from_reader(Reader, Configuration)`                                                                     |
|                                                 |                                                                                                                                 |
| `uklad::serialize(T)`                         | `uklad::serde::encode_to_vec(T, Configuration)`<br />`uklad::serde::encode_into_slice(T, &mut [u8], Configuration)`         |
| `uklad::serialize_into(std::io::Write, T)`    | `uklad::serde::encode_into_std_write(T, std::io::Write, Configuration)`                                                       |
| `uklad::serialized_size(T)`                   | Currently not implemented                                                                                                       |

## Migrating from `serde` to `uklad-derive`

`uklad-derive` is enabled by default. If you're using `default-features = false`, make sure to add `features = ["derive"]` to your `Cargo.toml`.

```toml,ignore
[dependencies]
uklad = "2.0"

# If you need `no_std` with `alloc`:
# uklad = { version = "2.0", default-features = false, features = ["derive", "alloc"] }

# If you need `no_std` and no `alloc`:
# uklad = { version = "2.0", default-features = false, features = ["derive"] }
```

Replace or add the following attributes. You are able to use both `serde-derive` and `uklad-derive` side-by-side.

| serde-derive                    | uklad-derive               |
| ------------------------------- | ---------------------------- |
| `#[derive(serde::Serialize)]`   | `#[derive(uklad::Encode)]` |
| `#[derive(serde::Deserialize)]` | `#[derive(uklad::Decode)]` |

**note:** To implement these traits manually, see the documentation of [Encode](https://docs.rs/uklad/2/uklad/enc/trait.Encode.html) and [Decode](https://docs.rs/uklad/2/uklad/de/trait.Decode.html).

**note:** For more information on using `uklad-derive` with external libraries, see [below](#uklad-derive-and-libraries).

Then replace the following functions: (`Configuration` is `uklad::config::legacy()` by default)

| Uklad 1                                       | Uklad 2                                                                                                          |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `uklad::deserialize(&[u8])`                   | `uklad::decode_from_slice(&bytes, Configuration)`<br />`uklad::borrow_decode_from_slice(&[u8], Configuration)` |
| `uklad::deserialize_from(std::io::Read)`      | `uklad::decode_from_std_read(std::io::Read, Configuration)`                                                      |
| `uklad::deserialize_from_custom(UkladRead)` | `uklad::decode_from_reader(Reader, Configuration)`                                                               |
|                                                 |                                                                                                                    |
| `uklad::serialize(T)`                         | `uklad::encode_to_vec(T, Configuration)`<br />`uklad::encode_into_slice(t: T, &mut [u8], Configuration)`       |
| `uklad::serialize_into(std::io::Write, T)`    | `uklad::encode_into_std_write(T, std::io::Write, Configuration)`                                                 |
| `uklad::serialized_size(T)`                   | Currently not implemented                                                                                          |

### Uklad derive and libraries

Currently not many libraries support the traits `Encode` and `Decode`. There are a couple of options if you want to use `#[derive(uklad::Encode, uklad::Decode)]`:

- Enable the `serde` feature and add a `#[uklad(with_serde)]` above each field that implements `serde::Serialize/Deserialize` but not `Encode/Decode`
- Enable the `serde` feature and wrap your field in [uklad::serde::Compat](https://docs.rs/uklad/2/uklad/serde/struct.Compat.html) or [uklad::serde::BorrowCompat](https://docs.rs/uklad/2/uklad/serde/struct.BorrowCompat.html)
- Make a pull request to the library:
  - Make sure to be respectful, most of the developers are doing this in their free time.
  - Add a dependency `uklad = { version = "2.0", default-features = false, optional = true }` to the `Cargo.toml`
  - Implement [Encode](https://docs.rs/uklad/2/uklad/enc/trait.Encode.html)
  - Implement [Decode](https://docs.rs/uklad/2/uklad/de/trait.Decode.html)
  - Make sure both of these implementations have a `#[cfg(feature = "uklad")]` attribute.
