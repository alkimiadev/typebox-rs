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

**Milestone M1:** Schema enum + builder + Rust/TypeScript codegen (needs refactoring)

Current implementation uses Property-based design. Planned refactoring:
- Add inline constraints to Schema variants (min/max, format, pattern)
- Change `Object.properties` to `IndexMap<String, Schema>` + `required: Vec<String>`
- Create custom `Value` enum (not serde_json::Value) to support bytes/typed arrays

**See:** `/workspace/@alkimiadev/plans/typebox-rs/decisions.md` for architecture decisions.

**Remaining milestones** (see plans):
- M1 (refactor): Schema + Value refactoring
- M2: Value operations (fake, create, clone, equal, cast, delta, patch, clean)
- M3: SafeTensor reader with schema metadata
- M4: FFI layer for cross-language clients
- M5: Ladybug integration

## Value Operations Roadmap

Following TypeBox's `value` module structure:

| Operation | Status | Description |
|-----------|--------|-------------|
| `check` | ✅ (needs migrate) | Validate value against schema |
| `fake` | TODO | Generate random test data from schema |
| `create` | TODO | Generate default value from schema |
| `clone` | TODO | Deep clone values |
| `equal` | TODO | Structural equality comparison |
| `cast` | TODO | Coerce value to match schema |
| `delta` | TODO | Compute diff (insert/update/delete) |
| `patch` | TODO | Apply delta edits |
| `clean` | TODO | Remove extraneous properties |

Reference: `/workspace/typebox-schema-faker/` for `fake` implementation pattern.

## Module Structure (Target)

Following TypeBox's pattern:

```
src/
├── type/              # Schema type definitions
│   ├── mod.rs
│   ├── schema.rs      # Core Schema trait/enum
│   ├── primitives.rs  # Null, Bool, numeric types
│   ├── array.rs
│   ├── object.rs
│   ├── union.rs
│   ├── record.rs      # Map types
│   └── registry.rs    # Custom type registry
├── value/             # Value operations
│   ├── mod.rs
│   ├── check.rs       # ✅ Validation
│   ├── create.rs      # Default generation
│   ├── clone.rs
│   ├── equal.rs
│   ├── cast.rs
│   ├── delta.rs
│   └── patch.rs
├── codegen/           # Code generation
├── layout.rs          # Binary layout
└── ffi/               # C-compatible FFI
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
- Feature flags: `codegen`, `safetensor`, `ffi`
- Serde with `#[serde]` attributes for JSON compatibility with TypeBox

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate entry point, re-exports |
| `src/schema.rs` | Core Schema enum |
| `src/builder.rs` | SchemaBuilder API |
| `src/validate.rs` | Validation logic |
| `src/layout.rs` | Binary layout calculation |
| `src/codegen/` | Code generators (Rust, TypeScript) |
| `tests/test_schema.rs` | Integration tests |
| `examples/ladybug_types.rs` | Full example for Ladybug use case |

## External Resources

- **Implementation Plan:** `/workspace/@alkimiadev/plans/typebox-rs/implementation-plan.md`
- **Rust Codegen Design:** `/workspace/@alkimiadev/plans/typebox-rs/rust-codegen.md`
- **Research:** `/workspace/@alkimiadev/research/typebox-rs/`
- **TypeBox Legacy:** `/workspace/typebox-legacy/` (reference implementation)
- **ts2typebox Example:** `/workspace/lbugdev/tmp/lbug_typebox.ts`
