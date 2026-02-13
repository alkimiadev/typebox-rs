#![recursion_limit = "1024"]

pub mod schema;
pub mod builder;
pub mod error;
pub mod validate;
pub mod layout;

#[cfg(feature = "codegen")]
pub mod codegen;

#[cfg(feature = "safetensor")]
pub mod safetensor;

pub use schema::{Schema, Property, LiteralValue};
pub use builder::SchemaBuilder;
pub use error::Error;
pub use validate::validate;
pub use layout::Layout;

#[cfg(feature = "codegen")]
pub use codegen::{RustGenerator, TypeScriptGenerator, SchemaRegistry};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
