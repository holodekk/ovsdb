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
//! [RFC 7047][4].
//!
//! ## License
//!
//! This project is licensed under the [MIT license](LICENSE.md).
//!
//! ## Author
//!
//! - [Josh Williams](https://dubzland.com)
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

#[cfg(feature = "client")]
pub mod client;

pub mod schema;

mod macros;
pub mod protocol;
mod result;
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
