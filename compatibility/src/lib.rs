#![cfg(test)]

use ::rand::Rng;
use uklad_1::Options;

mod membership;
mod misc;
mod rand;
mod sway;

pub fn test_same_with_config<T, C, O>(t: &T, uklad_1_options: O, uklad_2_config: C)
where
    T: uklad_2::Encode
        + uklad_2::Decode<()>
        + serde::Serialize
        + serde::de::DeserializeOwned
        + core::fmt::Debug
        + PartialEq,
    C: uklad_2::config::Config,
    O: uklad_1::Options + Copy,
{
    // This is what uklad 1 serializes to. This will be our comparison value.
    let encoded = uklad_1_options.serialize(t).unwrap();

    println!("Encoded {t:?} as {encoded:?}");

    // Test uklad 2 encode
    let uklad_2_output = uklad_2::encode_to_vec(t, uklad_2_config).unwrap();
    assert_eq!(
        encoded,
        uklad_2_output,
        "{t:?} serializes differently\nuklad 2 config {:?}",
        core::any::type_name::<C>(),
    );

    // Test uklad 2 serde serialize
    let uklad_2_serde_output = uklad_2::serde::encode_to_vec(t, uklad_2_config).unwrap();
    assert_eq!(
        encoded, uklad_2_serde_output,
        "{t:?} serializes differently"
    );

    // Test uklad 1 deserialize
    let decoded: T = uklad_1_options.deserialize(&encoded).unwrap();
    assert_eq!(&decoded, t);

    // Test uklad 2 decode
    let decoded: T = uklad_2::decode_from_slice(&encoded, uklad_2_config)
        .unwrap()
        .0;
    assert_eq!(&decoded, t);

    // Test uklad 2 serde deserialize
    let decoded: T = uklad_2::serde::decode_from_slice(&encoded, uklad_2_config)
        .unwrap()
        .0;
    assert_eq!(&decoded, t);
}

pub fn test_same<T>(t: T)
where
    T: uklad_2::Encode
        + uklad_2::Decode<()>
        + serde::Serialize
        + serde::de::DeserializeOwned
        + core::fmt::Debug
        + PartialEq,
{
    test_same_with_config(
        &t,
        // This is the config used internally by uklad 1
        uklad_1::options().with_fixint_encoding(),
        // Should match `::legacy()`
        uklad_2::config::legacy(),
    );

    // Check a bunch of different configs:
    test_same_with_config(
        &t,
        uklad_1::options()
            .with_big_endian()
            .with_varint_encoding(),
        uklad_2::config::legacy()
            .with_big_endian()
            .with_variable_int_encoding(),
    );
    test_same_with_config(
        &t,
        uklad_1::options()
            .with_little_endian()
            .with_varint_encoding(),
        uklad_2::config::legacy()
            .with_little_endian()
            .with_variable_int_encoding(),
    );
    test_same_with_config(
        &t,
        uklad_1::options()
            .with_big_endian()
            .with_fixint_encoding(),
        uklad_2::config::legacy()
            .with_big_endian()
            .with_fixed_int_encoding(),
    );
    test_same_with_config(
        &t,
        uklad_1::options()
            .with_little_endian()
            .with_fixint_encoding(),
        uklad_2::config::legacy()
            .with_little_endian()
            .with_fixed_int_encoding(),
    );
}

pub fn gen_string(rng: &mut impl Rng) -> String {
    let len = rng.gen_range(0..100usize);
    let mut result = String::with_capacity(len * 4);
    for _ in 0..len {
        result.push(rng.gen_range('\0'..char::MAX));
    }
    result
}
