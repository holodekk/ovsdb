//! A Rust implementation of the `OVSDB` schema and wire format.
//!
//! [`ovsdb`] provides a Rust interface to an `OVSDB` server. It utilizes [`serde`]
//! for protocol processing and [`tokio`] for asynchronous io. Its features
//! include:
//!
//! - automated generation of Rust models via [`ovsdb-build`]
//! - strongly typed interfaces to OVSDB data structures
//! - automatic conversion to/from OVSDB protocol types
//!
//! ## Overview
//!
//! Interacting with a database server is a 3-step process.
//!
//! 1. Load a copy of the [Schema][schema::Schema] for the database
//! 1. Build `rust` modules represent the [Table][schema::Table]'s in the schema
//! 1. Connect to the database via a [Client][client::Client] and execute methods
//!
//! Steps 1 and 2 above are handled by [`ovsdb-build`].
//!
//! ## Requirements
//!
//! To use the models generated by [`ovsdb-build`], it is also necessary to install [`serde`]
//! with the `derive` feature flag enabled.
//!
//! ```sh
//! $ cargo add serde --features derive
//! ```
//!
//! As [`ovsdb`] also utilizes [`tokio`] for asynchronous io and process management, an async
//! runtime is required.  See the `tokio` documentation for more details.
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::path::Path;
//!
//! use ovsdb::{Client, protocol::method::EchoResult};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), std::io::Error> {
//!     let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
//!         .await
//!         .unwrap();
//!
//!     let result: EchoResult = client.echo(vec!["Hello", "OVSDB"]).await.unwrap();
//!     assert_eq!(*result, vec!["Hello".to_string(), "OVSDB".to_string()]);
//!
//!     client.stop().await.unwrap();
//!
//!     Ok(())
//! }
//! ```
//!
//! [`ovsdb`]: https://docs.rs/ovsdb
//! [`ovsdb-build`]: https://docs.rs/ovsdb-build
//! [`serde`]: https://docs.rs/serde
//! [`tokio`]: https://docs.rs/tokio

// Built-in Lints
#![warn(
    unreachable_pub,
    missing_debug_implementations,
    missing_copy_implementations,
    elided_lifetimes_in_paths,
    missing_docs
)]
// Clippy lints
#![allow(
    clippy::match_same_arms,
    clippy::needless_doctest_main,
    clippy::map_unwrap_or,
    clippy::redundant_field_names,
    clippy::type_complexity
)]
#![warn(
    clippy::unwrap_used,
    // clippy::print_stdout,
    clippy::mut_mut,
    clippy::non_ascii_literal,
    clippy::similar_names,
    clippy::unicode_not_nfc,
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::used_underscore_binding
)]
#![deny(unsafe_code)]

mod macros;
mod result;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "protocol")]
pub mod protocol;
#[cfg(feature = "schema")]
pub mod schema;

#[cfg(feature = "client")]
pub use client::Client;
pub use result::*;

/// An entity that can be retrieved from OVSDB.
///
/// This represents a single row of data retrieved from a table.
pub trait Entity {
    /// The name of the OVSDB table associated with this entity.
    ///
    /// Aids in generating transact queries.
    fn table_name() -> &'static str;
}
