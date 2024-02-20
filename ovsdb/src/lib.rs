//! A rust implementation of the [OVSDB][1] schema and protocol.
//!
//! [1]: <http://tools.ietf.org/html/rfc7047>

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
