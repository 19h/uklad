#![cfg(all(feature = "std", feature = "derive"))]

extern crate std;

use std::collections::BTreeMap;

#[derive(uklad::Decode, uklad::Encode)]
struct AllTypes(BTreeMap<u8, AllTypes>);

#[test]
fn test_issue_467() {
    let _result: Result<(AllTypes, _), _> =
        uklad::decode_from_slice(&[], uklad::config::standard().with_limit::<1024>());
}
