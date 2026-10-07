//! gRPC transport for shelfmark (R19, plan PD8).
//!
//! A tonic server and client for `proto/shelfmark/v1/shelfmark.proto`.
//! `Call` and the streaming `Batch` delegate to any
//! [`shelfmark_core::Service`]; payloads are the same JSON the CLI prints,
//! so the two surfaces cannot drift.

pub mod client;
pub mod convert;
pub mod server;

/// Generated protobuf messages and tonic stubs for `shelfmark.v1`.
#[allow(clippy::all, clippy::pedantic, missing_docs)]
pub mod pb {
    tonic::include_proto!("shelfmark.v1");
}

pub use client::{BatchItem, Client};
pub use server::{serve, serve_with_incoming, GrpcService};
