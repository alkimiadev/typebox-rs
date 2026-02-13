# typebox-rs

JSON Schema type construction with validation, code generation, and binary layout.

## Features

- **Schema Builder**: Construct schemas programmatically with a fluent API
- **Validation**: Runtime validation against JSON values
- **Code Generation**: Generate Rust, TypeScript, Python, and C types from schemas
- **Binary Layout**: Calculate memory layout for structs and tensors
- **FFI Ready**: C-compatible types for cross-language interop

## Example

```rust
use typebox::{SchemaBuilder, RustGenerator, TypeScriptGenerator, validate};

// Define a schema
let person_schema = SchemaBuilder::object()
    .field("id", SchemaBuilder::int64())
    .field("name", SchemaBuilder::string())
    .optional_field("email", SchemaBuilder::string())
    .named("Person");

// Validate JSON
let json = serde_json::json!({
    "id": 1,
    "name": "Alice"
});
assert!(validate(&person_schema, &json).is_ok());

// Generate Rust types
let rust_gen = RustGenerator::new();
let rust_code = rust_gen.generate("Person", &person_schema)?;
// pub struct Person { pub id: i64, pub name: String, pub email: Option<String> }

// Generate TypeScript types
let ts_gen = TypeScriptGenerator::new();
let ts_code = ts_gen.generate("Person", &person_schema)?;
// export interface Person { id: number; name: string; email?: string; }
```

## Usage

```toml
[dependencies]
typebox = "0.1"
```

### Features

- `codegen` (default): Enable code generation
- `safetensor`: SafeTensor file reading support
- `ffi`: C-compatible FFI types

## Inspiration

This project is inspired by [TypeBox](https://github.com/sinclairzx81/typebox), a TypeScript library for JSON Schema type construction.

## License

MIT
