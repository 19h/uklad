#![cfg(feature = "std")]

extern crate std;

use std::ffi::CString;

#[test]
fn test_issue_498() {
    let bytes = [1, 0, 0, 0, 0, 0, 0, 0, 0];
    let out: Result<(CString, _), _> =
        uklad::decode_from_slice(&bytes, uklad::config::legacy().with_limit::<1024>());

    match out.unwrap_err() {
        uklad::error::DecodeError::CStringNulError { position: _ } => {}
        err => panic!("Expected CStringNullErr, found {:?}", err),
    }
}
