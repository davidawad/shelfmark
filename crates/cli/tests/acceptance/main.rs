//! Acceptance examples AE1-AE8 (plan U13), run end to end through the
//! `shelfmark` binary against a fresh store each.
//!
//! The scheme is the bundled LCC outline (U2) and the identifier table is
//! the U3 fixture table. AE8 (the sharing rule and scoped kbs,
//! R26) and the kb half of AE5 belong to the kb core resource (plan L4),
//! which runs them against this binary; shelfmark itself knows nothing of
//! kbs (R14).

mod common;

mod ae1;
mod ae2;
mod ae3;
mod ae4;
mod ae5;
mod ae6;
mod ae7;
mod grpc;
