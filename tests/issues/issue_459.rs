#![cfg(all(feature = "std", feature = "derive"))]

extern crate std;

use std::collections::BTreeMap;

#[derive(uklad::Encode)]
struct AllTypes(BTreeMap<u8, AllTypes>);

#[test]
fn test_issue_459() {
    let _result = uklad::encode_to_vec(AllTypes(BTreeMap::new()), uklad::config::standard());
}
