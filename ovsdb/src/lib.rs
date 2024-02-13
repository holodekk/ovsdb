mod macros;
pub use macros::*;
pub mod protocol;
mod result;
pub use result::*;
pub mod schema;

pub trait Entity {
    fn table_name() -> &'static str;
}
