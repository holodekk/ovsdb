//! # ovsdb
//!
//! [![Gitlab pipeline][pipeline-badge]][pipeline-url]
//! [![crates.io][cratesio-badge]][cratesio-url]
//! [![docs.rs][docsrs-badge]][docsrs-url]
//! [![license][license-badge]][license-url]
//! [![issues][issues-badge]][issues-url]
//!
//! A Rust implementation of the [OVSDB][1] schema and wire format.
//!
//! ## What is OVSDB?
//!
//! OVSDB is the database protocol behind [Open vSwitch][2] and [OVN][3], documented in
//! [RFC 7047][4]. If you don't know what either of those technologies are, you're
//! probably in the wrong place.
//!
//! ## Overview
//!
//! At the core of OVSDB are the schema and protocol. Together they describe both
//! the structure of data in the database, and the manner in which that data can be
//! manipulated. Both are described in detail in the RFC mentioned above (which
//! you really should read), so instead of restating them here, we'll discuss how
//! to interact with them.
//!
//! For instance, let's assume you want to interact with the Open vSwitch database.
//! The first thing you'll need to do is obtain a copy of the schema (usually stored
//! with a `.ovsschema` extension). The schema file can be found in the [repository](https://github.com/openvswitch/ovs/blob/master/vswitchd/vswitch.ovsschema), or obtained by connecting directly to an OVSDB instance. If you're running Open vSwitch, then a copy of OVSDB is already running on your machine (usually listening on a local socket).
//! For example, on my local machine, I can connect and download a copy of the schema with the following command:
//!
//! ```sh
//! # ovsdb-client get-schema /var/run/openvswitch/db.sock > vswitch.ovsschema
//! ```
//!
//! Note that connecting to the OVSDB socket requires root priveleges. Before proceeding, it is probably worth taking a look at the schema file itself, as the OVSDB schema introduces a few concepts not present in other database systems.
//!
//! With the schema file present, you can now proceed to build your first client application. The first step is to build built structs to represent the diffrerent tables in the database, as well as proxy objects to handle mapping from the esoteric format of the OVSDB protocol into more sane Rust representations. Next, a TCP or Unix socket client is needed depending on how you intend to interface with the database itself. Thankfully, both of those tasks have already been handled for you.
//!
//! ## Model generation
//!
//! First, we need to use `ovsdb-build` to generate our models and proxies. Add it as a build dependency of your project, either via shell command:
//!
//! ```sh
//! $ cargo add --build ovsdb-build
//! ```
//!
//! or by adding it directly to `Cargo.toml`
//!
//! ```toml
//! # Cargo.toml
//! # ...
//! [build-dependencies]
//! ovsdb-build = { version = "0.0.4" }
//! ```
//!
//! Next, add a build script to your project, passing it the path to the schema file we downloaded previously:
//!
//! ```rust,ignore
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     ovsdb_build::configure().compile("/path/to/vswitch.ovsschema", "vswitch")?;
//!     Ok(())
//! }
//! ```
//!
//! ## Database access
//!
//! Now that we have our build process established, its time to move on to the actual application code. There are three main aspects to the OVSDB integration:
//!
//! - importing the generated models
//! - connecting to the database
//! - issuing methods to the database and receiving responses
//!
//! As an example, we're going to write just enough code to query the list of Open vSwitch bridges. This should be enough to ensure a functional implementation. For more, check out the available [examples](examples/). Before proceeding, make sure that `ovsdb` has been added as a dependency, with the `client` feature enabled.
//!
//! ```sh
//! $ cargo add ovsdb --features client
//! ```
//!
//! The `ovsdb` models require the `serde` framework for encoding/decoding the OVSDB protocol data:
//!
//! ```sh
//! $ cargo add serde --features derive
//! ```
//!
//! and the TCP/Unix socket client requires `tokio`:
//!
//! ```sh
//! $ cargo add tokio
//! ```
//!
//! With the requirements in place, we can build our first client. In `main.rs`:
//!
//! ```rust,ignore
//! use std::path::Path;
//!
//! use ovsdb::{
//!     client::Client,
//!     protocol::{method::Operation, ListResult},
//!     Entity,
//! };
//!
//! mod vswitch {
//!     ovsdb::include_schema!("vswitch");
//! }
//!
//! use vswitch::Bridge;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), std::io::Error> {
//!     let client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
//!         .await
//!         .unwrap();
//!
//!     let op = Operation::Select {
//!         table: Bridge::table_name().to_string(),
//!         clauses: vec![],
//!     };
//!
//!     let bridges: &Vec<ListResult<Bridge>> =
//!         &client.transact("Open_vSwitch", vec![op]).await.unwrap();
//!     println!("Got some bridges: {:#?}", bridges);
//!
//!     client.stop().await.unwrap();
//!
//!     Ok(())
//! }
//! ```
//!
//! Build and run the above code, and you should see output similar to the following:
//!
//! ```text
//! Got some bridges: [
//!     ListResult {
//!         rows: [
//!             Bridge {
//!                 auto_attach: None,
//!                 controller: [],
//!
//!                 ...
//!
//!                 sflow: None,
//!                 status: {},
//!                 stp_enable: false,
//!             },
//!         ],
//!     },
//! ]
//! ```
//!
//! Congratulations. You've just had your first Rust conversation with OVSDB. For further reading, check out the [examples](https://git.dubzland.com/holodekk/ovsdb/-/tree/main/examples/) directory in the `ovsdb` repository, or the [docs][docsrs-url].
//!

//! [pipeline-badge]: https://img.shields.io/gitlab/pipeline-status/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&branch=main&style=flat-square&logo=gitlab
//! [pipeline-url]: https://git.dubzland.com/holodekk/ovsdb/pipelines?scope=all&page=1&ref=main
//! [cratesio-badge]: https://img.shields.io/crates/v/ovsdb?style=flat-square&logo=rust
//! [cratesio-url]: https://crates.io/crates/ovsdb
//! [docsrs-badge]: https://img.shields.io/badge/docs.rs-ovsdb-blue?style=flat-square&logo=docsdotrs
//! [docsrs-url]: https://docs.rs/ovsdb/latest/ovsdb/
//! [license-badge]: https://img.shields.io/gitlab/license/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&style=flat-square
//! [license-url]: https://git.dubzland.com/holodekk/ovsdb/-/blob/main/LICENSE.md
//! [issues-badge]: https://img.shields.io/gitlab/issues/open/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&style=flat-square&logo=gitlab
//! [issues-url]: https://git.dubzland.com/holodekk/ovsdb/-/issues
//! [1]: https://docs.openvswitch.org/en/latest/ref/ovsdb.7/
//! [2]: https://www.openvswitch.org/
//! [3]: https://docs.ovn.org/en/latest/contents.html
//! [4]: https://datatracker.ietf.org/doc/html/rfc7047

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
pub mod protocol;
pub mod schema;

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
