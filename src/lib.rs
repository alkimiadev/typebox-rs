#![recursion_limit = "1024"]

//! JSON Schema type construction with validation, code generation, and binary layout.
//!
//! Inspired by [TypeBox](https://github.com/sinclairzx81/typebox) (TypeScript).
//!
//! # Example
//!
//! ```
//! use typebox::{SchemaBuilder, Value, check, create};
//!
//! // Define a schema
//! let person = SchemaBuilder::object()
//!     .field("id", SchemaBuilder::int64())
//!     .field("name", SchemaBuilder::string().build())
//!     .optional_field("email", SchemaBuilder::string().build())
//!     .named("Person");
//!
//! // Create a default value
//! let value = create(&person).unwrap();
//! assert!(check(&person, &value));
//!
//! // Build values manually
//! let alice = Value::object()
//!     .field("id", Value::Int64(1))
//!     .field("name", Value::String("Alice".to_string()))
//!     .build();
//! assert!(check(&person, &alice));
//! ```
//!
//! # Feature Flags
//!
//! - `codegen` - Generate Rust/TypeScript code from schemas
//! - `fake` - Generate random test data (requires `fake` and `rand` crates)
//! - `safetensor` - SafeTensor file support
//! - `ffi` - C-compatible FFI types

pub mod builder;
pub mod error;
pub mod layout;
pub mod schema;
pub mod validate;
pub mod value;

#[cfg(feature = "codegen")]
pub mod codegen;

pub use builder::SchemaBuilder;
pub use error::{CastError, CleanError, CreateError, Error, PatchError};
pub use layout::Layout;
pub use schema::{LiteralValue, Schema, StringFormat};
pub use validate::validate;
pub use value::Value;
pub use value::{
    cast, check, check_with_errors, clean, clone, create, delta, diff_summary, equal, patch, Delta,
    Edit,
};

#[cfg(feature = "fake")]
pub use error::FakeError;
#[cfg(feature = "fake")]
pub use value::{fake, fake_with_context, FakeContext};

#[cfg(feature = "codegen")]
pub use codegen::{RustGenerator, SchemaRegistry, TypeScriptGenerator};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
