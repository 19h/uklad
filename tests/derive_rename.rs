#![cfg(feature = "derive")]

extern crate uklad as uklad_new;

// Make sure that the `uklad` crate exists, just symlink it to `core.
extern crate core as uklad;

#[derive(uklad_new::Encode)]
#[uklad(crate = "uklad_new")]
#[allow(dead_code)]
struct DeriveRenameTest {
    a: u32,
    b: u32,
}
