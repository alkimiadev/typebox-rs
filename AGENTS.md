# AGENTS.md

Context for AI agents working on typebox-rs.

## Project Overview

typebox-rs is a Rust library for JSON Schema type construction with validation, code generation, and binary layout calculation. Inspired by [TypeBox](https://github.com/sinclairzx81/typebox) (TypeScript).

**The schema is the source of truth** - from a single schema definition we generate:
- Runtime validation
- Code (Rust, TypeScript, Python, C)
- Binary layouts for structs and tensors
- Cross-language FFI types

**Key insight:** Schemas are not limited to JSON. They describe typed data that can be:
- Serialized as JSON for interchange
- Laid out as packed binary structs (like Arrow)
- Stored in SafeTensor files with self-describing metadata
- Used via FFI from any language

## Directory Structure

This project follows the alkimiadev pattern for separating code from development artifacts:

```
/workspace/@alkimiadev/
├── typebox-rs/           # This repo - code and end-user docs only
├── plans/typebox-rs/     # Implementation plans, milestones, specs
└── research/typebox-rs/  # Design research, patterns, use cases
```

**Important:** Do not create task summaries, code views, or development planning docs in this repo. Place them in `/workspace/@alkimiadev/plans/typebox-rs/` instead.

## Use Cases

1. **LadybugDB client** - Schema-driven types for graph database client with FFI for Deno/Python/Node
2. **SafeTensor/Metatensor** - Self-describing tensor files with schema in `__metadata__`
3. **Self-hosting** - TypeScript types → ts2typebox → TypeBox JSON → typebox-rs → Rust types

## Current Status

**Milestone M1:** ✅ Complete - Schema enum with inline constraints, custom Value enum

**Milestone M2:** ✅ Complete - Value operations (check, clone, equal, create, cast, delta, patch, clean, fake)

**Milestone M2.1:** 🔲 In Progress - Core value operations (hash, pointer, mutate) + validation completeness

**See:** 
- `/workspace/@alkimiadev/plans/typebox-rs/decisions.md` for architecture decisions.
- `/workspace/@alkimiadev/plans/typebox-rs/v0.1-implementation-checklist.md` for current work.

**Remaining milestones** (see plans):
- M3: SafeTensor reader with schema metadata
- M4: FFI layer for cross-language clients
- M5: Ladybug integration

## Value Operations Roadmap

Following TypeBox's `value` module structure:

| Operation | Status | Description |
|-----------|--------|-------------|
| `check` | ✅ | Validate value against schema |
| `clone` | ✅ | Deep clone values |
| `equal` | ✅ | Structural equality comparison |
| `create` | ✅ | Generate default value from schema |
| `fake` | ✅ | Generate random test data from schema |
| `cast` | ✅ | Coerce value to match schema |
| `delta` | ✅ | Compute diff (insert/update/delete) |
| `patch` | ✅ | Apply delta edits |
| `clean` | ✅ | Remove extraneous properties |
| `hash` | 🔲 | FNV-1A hash for HashMap/caching |
| `pointer` | 🔲 | JSON Pointer (RFC6901) access |
| `mutate` | 🔲 | In-place deep mutation |

Reference: `/workspace/typebox-schema-faker/` for `fake` implementation pattern.

## Testing Standards

**Coverage Target:** 80% line coverage (soft requirement - aim for meaningful coverage over raw numbers)

### Test Categories Required

1. **Happy Path Tests** - Normal usage with valid inputs
2. **Error Path Tests** - Invalid inputs, boundary conditions, edge cases
3. **Roundtrip Tests** - Reversible operations (`delta` + `patch`, `to_json` + `from_json`)
4. **Integration Tests** - End-to-end scenarios, feature combinations

### Test Organization

```rust
#[cfg(test)]
mod tests {
    mod validation {
        #[test] fn test_valid_input() { /* ... */ }
        #[test] fn test_invalid_input() { /* ... */ }
    }
    mod edge_cases {
        #[test] fn test_empty() { /* ... */ }
        #[test] fn test_nan_handling() { /* ... */ }
    }
}
```

### Coverage Commands

```bash
cargo llvm-cov --all-features          # Run with coverage
cargo llvm-cov --all-features --html   # Generate HTML report
```

## Documentation Standards

Every public item must have:

1. **Summary Line** - One sentence starting with a verb
2. **Examples** - For non-trivial functions
3. **Errors Section** - `# Errors` listing possible failures
4. **Panics Section** - `# Panics` if applicable (avoid panics in public API)

```rust
/// Validates a value against a schema.
///
/// # Examples
/// ```
/// use typebox::{SchemaBuilder, validate};
/// let schema = SchemaBuilder::int64();
/// let value = Value::int64(42);
/// assert!(validate(&schema, &value).is_ok());
/// ```
///
/// # Errors
/// Returns `ValidationError` on type mismatch or constraint violation.
pub fn validate(schema: &Schema, value: &Value) -> Result<(), ValidationError> { ... }
```

### Doc Commands

```bash
cargo doc --all-features --no-deps     # Build docs
cargo test --doc --all-features        # Run doc tests
```

**Note:** We will add `#![warn(missing_docs)]` after filling documentation gaps.

## Module Structure

```
src/
├── lib.rs             # Crate entry point, re-exports
├── schema.rs          # Core Schema enum
├── builder.rs         # SchemaBuilder API
├── validate.rs        # Validation logic
├── layout.rs          # Binary layout calculation
├── error.rs           # Error types
├── registry.rs        # SchemaRegistry for $ref resolution (TODO)
├── value/             # Value type and operations
│   ├── mod.rs         # Value enum
│   ├── check.rs       # Validation
│   ├── clone.rs       # Deep clone
│   ├── equal.rs       # Equality comparison
│   ├── create.rs      # Default generation
│   ├── fake.rs        # Random test data (feature: fake)
│   ├── cast.rs        # Value coercion
│   ├── delta.rs       # Diff computation
│   ├── patch.rs       # Apply diffs
│   ├── clean.rs       # Remove extraneous properties
│   ├── hash.rs        # FNV-1A hashing (TODO)
│   ├── pointer.rs     # JSON Pointer RFC6901 (TODO)
│   └── mutate.rs      # In-place mutation (TODO)
└── codegen/           # Code generation (feature: codegen)
```

## Build & Test Commands

```bash
cargo build
cargo test
cargo test --all-features
cargo run --example ladybug_types
cargo clippy --all-features
cargo fmt --check
```

## Code Conventions

- Rust edition 2021
- Use `thiserror` for error types
- Handlebars templates in `src/codegen/templates/`
- Feature flags: `codegen`, `fake`, `safetensor`, `ffi`
- Serde with `#[serde]` attributes for JSON compatibility with TypeBox

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate entry point, re-exports |
| `src/schema.rs` | Core Schema enum |
| `src/builder.rs` | SchemaBuilder API |
| `src/validate.rs` | Validation logic |
| `src/value/mod.rs` | Value enum with JSON conversion |
| `src/value/*.rs` | Value operations (check, clone, cast, etc.) |
| `src/layout.rs` | Binary layout calculation |
| `src/codegen/` | Code generators (Rust, TypeScript) |
| `tests/test_schema.rs` | Integration tests |
| `examples/ladybug_types.rs` | Full example for Ladybug use case |

## External Resources

- **Implementation Checklist:** `/workspace/@alkimiadev/plans/typebox-rs/v0.1-implementation-checklist.md`
- **Pre-Release Review:** `/workspace/@alkimiadev/plans/typebox-rs/pre-release-review.md`
- **Research:** `/workspace/@alkimiadev/research/typebox-rs/`
- **TypeBox Legacy:** `/workspace/typebox-legacy/` (reference implementation)
- **ts2typebox Example:** `/workspace/lbugdev/tmp/lbug_typebox.ts`
