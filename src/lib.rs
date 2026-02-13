#![recursion_limit = "1024"]

pub mod builder;
pub mod error;
pub mod layout;
pub mod schema;
pub mod validate;
pub mod value;

#[cfg(feature = "codegen")]
pub mod codegen;

// TODO: Implement safetensor module
// #[cfg(feature = "safetensor")]
// pub mod safetensor;

pub use builder::SchemaBuilder;
pub use error::Error;
pub use layout::Layout;
pub use schema::{LiteralValue, Schema, StringFormat};
pub use validate::validate;
pub use value::Value;
pub use value::{check, check_with_errors, clone};

#[cfg(feature = "codegen")]
pub use codegen::{RustGenerator, SchemaRegistry, TypeScriptGenerator};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
