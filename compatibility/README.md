# Uklad compatibility tests

This crate verifies that uklad 2 produces the same output as uklad 1.3 for equivalent configurations. It encodes types with a range of settings, compares the results, and checks that decoding round-trips.

## Adding a test case

To add a test case for a project, follow these steps:

- [ ] Create a new file `compatibility/src/<name>.rs`.
- [ ] Add a link to the project.
- [ ] Add `License: MIT OR Apache-2.0` if the code is distributed under that license.
- [ ] Add a `mod <name>;` in `lib.rs`, keeping the list alphabetically ordered.
- [ ] Add the structs.
  - Prefer defining types locally rather than referencing external libraries, since those libraries will not implement uklad 2's encoding system.
  - If a library type is required, consider adding a test case for that library and referencing it instead.
- [ ] Derive the following traits:
  - [ ] `serde::Serialize` and `serde::Deserialize`.
  - [ ] `uklad_2::Encode` and `uklad_2::Decode`.
  - [ ] `Debug` and `PartialEq`.
  - [ ] Because the crate is renamed to `uklad_2`, add `#[uklad(crate = "uklad_2")]`.

```rs
#[derive(Serialize, Deserialize, uklad_2::Encode, uklad_2::Decode, Debug, PartialEq)]
#[uklad(crate = "uklad_2")]
pub struct YourStruct {
}
```

- [ ] Use `rand` to generate random test data.
  - [ ] For strings, use the helper `gen_string(rng: &mut impl Rng) -> String`.
- [ ] Add the following test:

```rs
#[test]
pub fn test() {
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        crate::test_same(<your_rand_function>(&mut rng));
    }
}
```

See the existing cases in `compatibility/src/` for examples.

- [ ] Open a [pull request](https://github.com/bgw/uklad/pulls) titled `Uklad 1 compatibility: <name of your project>`.
