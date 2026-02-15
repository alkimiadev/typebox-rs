//! Schema registry for `$ref` resolution and named schema lookup.
//!
//! The registry stores named schemas and resolves `$ref` pointers during
//! validation. This enables:
//! - Schema reuse via references
//! - Recursive type definitions
//! - Cross-schema validation
//!
//! # Examples
//!
//! ```
//! use typebox::{SchemaBuilder, SchemaRegistry, Value, validate_with_registry};
//!
//! let person_schema = SchemaBuilder::object()
//!     .field("name", SchemaBuilder::string().build())
//!     .field("age", SchemaBuilder::int64())
//!     .named("Person");
//!
//! let mut registry = SchemaRegistry::new();
//! registry.register("Person", person_schema);
//!
//! let ref_schema = SchemaBuilder::r#ref("Person");
//! let value = Value::object()
//!     .field("name", Value::string("Alice"))
//!     .field("age", Value::int64(30))
//!     .build();
//!
//! assert!(validate_with_registry(&ref_schema, &value, Some(&registry)).is_ok());
//! ```
use crate::error::RegistryError;
use crate::schema::{Schema, SchemaKind};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct SchemaRegistry {
    schemas: HashMap<String, Schema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, schema: Schema) {
        self.schemas.insert(name.into(), schema);
    }

    pub fn get(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    pub fn schemas(&self) -> impl Iterator<Item = (&String, &Schema)> {
        self.schemas.iter()
    }

    pub fn len(&self) -> usize {
        self.schemas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.schemas.is_empty()
    }

    pub fn resolve<'a>(&'a self, schema: &'a Schema) -> Result<&'a Schema, RegistryError> {
        self.resolve_with_visited(schema, &mut HashSet::new())
    }

    fn resolve_with_visited<'a>(
        &'a self,
        schema: &'a Schema,
        visited: &mut HashSet<String>,
    ) -> Result<&'a Schema, RegistryError> {
        match &schema.kind {
            SchemaKind::Ref { reference } => {
                let name = reference
                    .strip_prefix("#/definitions/")
                    .unwrap_or(reference);

                if visited.contains(name) {
                    return Err(RegistryError::CircularRef(name.to_string()));
                }
                visited.insert(name.to_string());

                let resolved = self
                    .schemas
                    .get(name)
                    .ok_or_else(|| RegistryError::SchemaNotFound(reference.clone()))?;

                self.resolve_with_visited(resolved, visited)
            }
            _ => Ok(schema),
        }
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;
    use crate::schema::SchemaKind;

    #[test]
    fn test_register_and_get() {
        let mut registry = SchemaRegistry::new();
        let schema = SchemaBuilder::int64();
        registry.register("Age", schema.clone());

        assert!(registry.contains("Age"));
        assert!(registry.get("Age").is_some());
        assert!(registry.get("Unknown").is_none());
    }

    #[test]
    fn test_resolve_ref() {
        let mut registry = SchemaRegistry::new();
        let person = SchemaBuilder::object()
            .field("name", SchemaBuilder::string().build())
            .named("Person");
        registry.register("Person", person);

        let ref_schema = SchemaBuilder::r#ref("Person");
        let resolved = registry.resolve(&ref_schema).unwrap();

        assert!(matches!(resolved.kind, SchemaKind::Named { .. }));
    }

    #[test]
    fn test_resolve_not_found() {
        let registry = SchemaRegistry::new();
        let ref_schema = SchemaBuilder::r#ref("Unknown");

        assert!(matches!(
            registry.resolve(&ref_schema),
            Err(RegistryError::SchemaNotFound(_))
        ));
    }

    #[test]
    fn test_resolve_circular() {
        let mut registry = SchemaRegistry::new();

        let a = SchemaBuilder::r#ref("B");
        let b = SchemaBuilder::r#ref("A");

        registry.register("A", a);
        registry.register("B", b);

        let ref_a = SchemaBuilder::r#ref("A");
        assert!(matches!(
            registry.resolve(&ref_a),
            Err(RegistryError::CircularRef(_))
        ));
    }

    #[test]
    fn test_resolve_non_ref() {
        let registry = SchemaRegistry::new();
        let schema = SchemaBuilder::int64();

        let resolved = registry.resolve(&schema).unwrap();
        assert!(matches!(resolved.kind, SchemaKind::Int64 { .. }));
    }
}
