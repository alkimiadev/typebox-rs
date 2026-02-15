# typebox-rs

[![crates.io](https://img.shields.io/crates/v/typebox.svg)](https://crates.io/crates/typebox)
[![docs.rs](https://img.shields.io/docsrs/typebox)](https://docs.rs/typebox)

JSON Schema type construction with validation, code generation, and binary layout. Inspired by [TypeBox](https://github.com/sinclairzx81/typebox).

**[Documentation](https://docs.rs/typebox)** | **[Crate](https://crates.io/crates/typebox)** | **[Repository](https://github.com/alkimiadev/typebox-rs)**

## Example

```rust
use typebox::{SchemaBuilder, Value, check, create, delta, patch};

// Define a schema
let person = SchemaBuilder::object()
    .field("id", SchemaBuilder::int64())
    .field("name", SchemaBuilder::string().build())
    .optional_field("email", SchemaBuilder::string().build())
    .named("Person");

// Create a default value
let value = create(&person)?;
assert!(check(&person, &value));

// Work with values
let a = Value::object()
    .field("id", Value::Int64(1))
    .field("name", Value::String("Alice".to_string()))
    .build();

let b = Value::object()
    .field("id", Value::Int64(1))
    .field("name", Value::String("Bob".to_string()))
    .build();

// Compute and apply diffs
let edits = delta(&a, &b);
let restored = patch(&a, &edits)?;
assert_eq!(restored, b);
```

## Usage

```toml
[dependencies]
typebox = "0.1"
```

## Feature Flags

| Flag | Description |
|------|-------------|
| `codegen` | Generate Rust/TypeScript code from schemas |
| `fake` | Generate random test data (`fake` + `rand` crates) |
| `pattern` | Regex pattern validation for strings |

Default: none (minimal by default)

## License

MIT
