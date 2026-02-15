# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-02-15

### Added

- Schema types with JSON Schema compatibility (`Schema`, `SchemaKind`, `SchemaBuilder`)
- Value types with JSON conversion (`Value` enum)
- Validation with detailed error reporting (`validate`, `validate_with_registry`, `validate_with_format`)
- Value operations:
  - `check` - Validate value against schema
  - `clone` - Deep clone values
  - `equal` - Structural equality comparison
  - `create` - Generate default value from schema
  - `cast` - Coerce value to match schema
  - `delta` - Compute diff (insert/update/delete)
  - `patch` - Apply delta edits
  - `clean` - Remove extraneous properties
  - `hash` - FNV-1A hash for HashMap/caching
  - `pointer` - JSON Pointer (RFC6901) access
  - `mutate` - In-place deep mutation
- Recursive type support via `SchemaBuilder::recursive()`
- Intersection type support via `SchemaBuilder::intersect()`
- Format validation via user-extensible `FormatRegistry`
- Unique items validation for arrays
- Pattern validation for strings (feature-gated: `pattern`)
- Schema registry for `$ref` resolution
- Binary layout calculation (`Layout`)
- Code generation for Rust and TypeScript (feature-gated: `codegen`)
- Random test data generation (feature-gated: `fake`)

### Feature Flags

- `codegen` - Generate Rust/TypeScript code from schemas
- `fake` - Generate random test data
- `pattern` - Regex pattern validation for strings
- `safetensor` - SafeTensor file support (planned)
- `ffi` - C-compatible FFI types (planned)
