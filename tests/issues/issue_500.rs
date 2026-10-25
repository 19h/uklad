#![cfg(all(feature = "serde", feature = "derive", feature = "std"))]

extern crate std;

type NodeId = u64;

use std::collections::BTreeSet;

#[derive(
    uklad::Encode, uklad::Decode, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq,
)]
pub struct Membership {
    /// learners set
    learners: BTreeSet<NodeId>,
}

#[test]
fn test() {
    let mut start = Membership {
        learners: BTreeSet::new(),
    };
    start.learners.insert(1);

    let config = uklad::config::legacy();
    let encoded = uklad::encode_to_vec(&start, config).unwrap();
    std::dbg!(&encoded);
    let decoded: Membership = uklad::serde::decode_from_slice(&encoded, config).unwrap().0;
    assert_eq!(start, decoded);
}
