//! Acceptance examples AE1-AE8 (plan U13), run end to end through the
//! `shelfmark` binary against a fresh store each.
//!
//! The bundled scheme is the mini-LCC fixture until the LCC outline (U2)
//! lands, and the identifier table is the U3 fixture table. The kb half of
//! AE5 and all of AE8 need the kb core resource (U12) and are `#[ignore]`d
//! as pending; running them with `--ignored` fails on purpose.

mod common;

mod ae1;
mod ae2;
mod ae3;
mod ae4;
mod ae5;
mod ae6;
mod ae7;
mod ae8;
mod grpc;
