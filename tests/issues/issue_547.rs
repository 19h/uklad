#![cfg(all(feature = "serde", feature = "std"))]

use glam::vec3;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Instance {
    position: glam::Vec3,
}

#[test]
fn test() {
    let instance = Instance {
        position: vec3(2.0, 2.0, 2.0),
    };

    let m = uklad::serde::encode_to_vec(&instance, uklad::config::standard()).unwrap();
    let instance2: Instance = uklad::serde::decode_from_slice(&m, uklad::config::standard())
        .unwrap()
        .0;

    assert_eq!(instance, instance2);
}
