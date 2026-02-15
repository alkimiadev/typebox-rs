//! Schema builder API for constructing JSON Schema types.
//!
//! Provides a fluent API for building schemas with constraints and metadata.
//!
//! # Example
//!
//! ```
//! use typebox::{SchemaBuilder, StringFormat};
//!
//! let person = SchemaBuilder::object()
//!     .field("id", SchemaBuilder::int64())
//!     .field("name", SchemaBuilder::string().min_length(1).build())
//!     .optional_field("email", SchemaBuilder::string()
//!         .format(StringFormat::Email)
//!         .build())
//!     .named("Person");
//! ```

use crate::schema::{LiteralValue, Schema, SchemaKind, StringFormat};
use indexmap::IndexMap;

/// Builder for constructing JSON Schema types.
///
/// Provides static methods for creating all schema types with a fluent API.
/// Use this as the primary entry point for schema construction.
///
/// # Example
///
/// ```
/// use typebox::SchemaBuilder;
///
/// let schema = SchemaBuilder::string().max_length(100).build();
/// ```
pub struct SchemaBuilder;

impl SchemaBuilder {
    /// Creates a null type schema.
    pub fn null() -> Schema {
        Schema::new(SchemaKind::Null)
    }

    /// Creates a boolean type schema.
    pub fn bool() -> Schema {
        Schema::new(SchemaKind::Bool)
    }

    /// Creates a signed 8-bit integer schema.
    pub fn int8() -> Schema {
        Schema::new(SchemaKind::Int8 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates a signed 16-bit integer schema.
    pub fn int16() -> Schema {
        Schema::new(SchemaKind::Int16 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates a signed 32-bit integer schema.
    pub fn int32() -> Schema {
        Schema::new(SchemaKind::Int32 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates a signed 64-bit integer schema.
    pub fn int64() -> Schema {
        Schema::new(SchemaKind::Int64 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates an unsigned 8-bit integer schema.
    pub fn uint8() -> Schema {
        Schema::new(SchemaKind::UInt8 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates an unsigned 16-bit integer schema.
    pub fn uint16() -> Schema {
        Schema::new(SchemaKind::UInt16 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates an unsigned 32-bit integer schema.
    pub fn uint32() -> Schema {
        Schema::new(SchemaKind::UInt32 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates an unsigned 64-bit integer schema.
    pub fn uint64() -> Schema {
        Schema::new(SchemaKind::UInt64 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates a 32-bit floating point schema.
    pub fn float32() -> Schema {
        Schema::new(SchemaKind::Float32 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates a 64-bit floating point schema.
    pub fn float64() -> Schema {
        Schema::new(SchemaKind::Float64 {
            minimum: None,
            maximum: None,
        })
    }

    /// Creates a string schema builder for adding constraints.
    ///
    /// # Example
    ///
    /// ```
    /// use typebox::{SchemaBuilder, StringFormat};
    ///
    /// let email = SchemaBuilder::string()
    ///     .format(StringFormat::Email)
    ///     .min_length(5)
    ///     .max_length(100)
    ///     .build();
    /// ```
    pub fn string() -> StringBuilder {
        StringBuilder::new()
    }

    /// Creates a bytes schema.
    pub fn bytes() -> Schema {
        Schema::new(SchemaKind::Bytes {
            min_length: None,
            max_length: None,
        })
    }

    /// Creates an array schema builder with the given item schema.
    ///
    /// # Example
    ///
    /// ```
    /// use typebox::SchemaBuilder;
    ///
    /// let numbers = SchemaBuilder::array(SchemaBuilder::int64())
    ///     .min_items(1)
    ///     .max_items(100)
    ///     .build();
    /// ```
    pub fn array(items: Schema) -> ArrayBuilder {
        ArrayBuilder::new(items)
    }

    /// Creates an object schema builder.
    ///
    /// # Example
    ///
    /// ```
    /// use typebox::SchemaBuilder;
    ///
    /// let person = SchemaBuilder::object()
    ///     .field("name", SchemaBuilder::string().build())
    ///     .optional_field("age", SchemaBuilder::int64())
    ///     .build();
    /// ```
    pub fn object() -> ObjectBuilder {
        ObjectBuilder::new()
    }

    /// Creates a tuple schema with fixed-position items.
    pub fn tuple(items: Vec<Schema>) -> Schema {
        Schema::new(SchemaKind::Tuple { items })
    }

    /// Creates a union schema matching any of the variants.
    pub fn union(variants: Vec<Schema>) -> Schema {
        Schema::new(SchemaKind::Union { any_of: variants })
    }

    /// Wraps a schema to make it optional (union with null).
    pub fn optional(schema: Schema) -> Schema {
        Schema::new(SchemaKind::Union {
            any_of: vec![schema, Schema::new(SchemaKind::Null)],
        })
    }

    /// Creates a literal schema matching an exact value.
    ///
    /// # Example
    ///
    /// ```
    /// use typebox::SchemaBuilder;
    ///
    /// let status = SchemaBuilder::literal("active");
    /// let count = SchemaBuilder::literal(42);
    /// ```
    pub fn literal(value: impl Into<LiteralValue>) -> Schema {
        Schema::new(SchemaKind::Literal {
            value: value.into(),
        })
    }

    /// Creates an enum schema from string values.
    pub fn enum_values(values: Vec<&str>) -> Schema {
        Schema::new(SchemaKind::Enum {
            values: values.iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Creates a reference to a schema by name.
    ///
    /// Used with [`SchemaRegistry`](crate::SchemaRegistry) for resolving
    /// shared or recursive schemas.
    pub fn r#ref(name: &str) -> Schema {
        Schema::new(SchemaKind::Ref {
            reference: format!("#/definitions/{}", name),
        })
    }

    /// Wraps a schema with a name for code generation.
    pub fn named(name: &str, schema: Schema) -> Schema {
        Schema::new(SchemaKind::Named {
            name: name.to_string(),
            schema: Box::new(schema),
        })
    }

    /// Creates a function type schema.
    pub fn function(parameters: Vec<Schema>, returns: Schema) -> Schema {
        Schema::new(SchemaKind::Function {
            parameters,
            returns: Box::new(returns),
        })
    }

    /// Creates a void type (no value).
    pub fn void() -> Schema {
        Schema::new(SchemaKind::Void)
    }

    /// Creates a never type (uninhabitable).
    pub fn never() -> Schema {
        Schema::new(SchemaKind::Never)
    }

    /// Creates an any type (accepts any value).
    pub fn any() -> Schema {
        Schema::new(SchemaKind::Any)
    }

    /// Creates an unknown type (requires type checking).
    pub fn unknown() -> Schema {
        Schema::new(SchemaKind::Unknown)
    }

    /// Creates an undefined type.
    pub fn undefined() -> Schema {
        Schema::new(SchemaKind::Undefined)
    }

    /// Creates a recursive type schema.
    ///
    /// The callback receives a reference schema that can be used for self-reference.
    ///
    /// # Example
    ///
    /// ```
    /// use typebox::SchemaBuilder;
    ///
    /// let json_tree = SchemaBuilder::recursive("JsonTree", |this| {
    ///     SchemaBuilder::union(vec![
    ///         SchemaBuilder::null(),
    ///         SchemaBuilder::bool(),
    ///         SchemaBuilder::int64(),
    ///         SchemaBuilder::string().build(),
    ///         SchemaBuilder::array(this.clone()).build(),
    ///     ])
    /// });
    /// ```
    pub fn recursive<F>(id: &str, callback: F) -> Schema
    where
        F: FnOnce(Schema) -> Schema,
    {
        let this_ref = Schema::new(SchemaKind::Ref {
            reference: id.to_string(),
        });
        let inner = callback(this_ref);
        Schema::new(SchemaKind::Recursive {
            schema: Box::new(inner),
        })
        .with_id(id)
    }

    /// Creates an intersection schema requiring all schemas to match.
    ///
    /// Equivalent to JSON Schema's `allOf` constraint.
    pub fn intersect(schemas: Vec<Schema>) -> Schema {
        Schema::new(SchemaKind::Intersect { all_of: schemas })
    }
}

impl From<&str> for LiteralValue {
    fn from(s: &str) -> Self {
        LiteralValue::String(s.to_string())
    }
}

impl From<String> for LiteralValue {
    fn from(s: String) -> Self {
        LiteralValue::String(s)
    }
}

impl From<i64> for LiteralValue {
    fn from(n: i64) -> Self {
        LiteralValue::Number(n)
    }
}

impl From<i32> for LiteralValue {
    fn from(n: i32) -> Self {
        LiteralValue::Number(n as i64)
    }
}

impl From<f64> for LiteralValue {
    fn from(n: f64) -> Self {
        LiteralValue::Float(n)
    }
}

impl From<bool> for LiteralValue {
    fn from(b: bool) -> Self {
        LiteralValue::Boolean(b)
    }
}

/// Builder for string schemas with constraints.
pub struct StringBuilder {
    format: Option<StringFormat>,
    pattern: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl StringBuilder {
    /// Creates a new string builder with no constraints.
    pub fn new() -> Self {
        Self {
            format: None,
            pattern: None,
            min_length: None,
            max_length: None,
        }
    }

    /// Sets the string format constraint.
    pub fn format(mut self, format: StringFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets a regex pattern constraint (requires `pattern` feature).
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    /// Sets the minimum string length.
    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Sets the maximum string length.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Builds the string schema.
    pub fn build(self) -> Schema {
        Schema::new(SchemaKind::String {
            format: self.format,
            pattern: self.pattern,
            min_length: self.min_length,
            max_length: self.max_length,
        })
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for array schemas with constraints.
pub struct ArrayBuilder {
    items: Schema,
    min_items: Option<usize>,
    max_items: Option<usize>,
    unique_items: Option<bool>,
}

impl ArrayBuilder {
    /// Creates a new array builder for the given item schema.
    pub fn new(items: Schema) -> Self {
        Self {
            items,
            min_items: None,
            max_items: None,
            unique_items: None,
        }
    }

    /// Sets the minimum number of items.
    pub fn min_items(mut self, min: usize) -> Self {
        self.min_items = Some(min);
        self
    }

    /// Sets the maximum number of items.
    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    /// Requires all items to be unique.
    pub fn unique_items(mut self, unique: bool) -> Self {
        self.unique_items = Some(unique);
        self
    }

    /// Builds the array schema.
    pub fn build(self) -> Schema {
        Schema::new(SchemaKind::Array {
            items: Box::new(self.items),
            min_items: self.min_items,
            max_items: self.max_items,
            unique_items: self.unique_items,
        })
    }
}

/// Builder for object schemas with properties.
pub struct ObjectBuilder {
    properties: IndexMap<String, Schema>,
    required: Vec<String>,
    additional_properties: Option<Schema>,
}

impl ObjectBuilder {
    /// Creates a new object builder with no properties.
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
            required: Vec::new(),
            additional_properties: None,
        }
    }

    /// Adds a required field.
    pub fn field(mut self, name: &str, schema: Schema) -> Self {
        self.properties.insert(name.to_string(), schema);
        self.required.push(name.to_string());
        self
    }

    /// Adds an optional field.
    pub fn optional_field(mut self, name: &str, schema: Schema) -> Self {
        self.properties.insert(name.to_string(), schema);
        self
    }

    /// Sets the schema for additional properties.
    pub fn additional_properties(mut self, schema: Option<Schema>) -> Self {
        self.additional_properties = schema;
        self
    }

    /// Builds the object schema.
    pub fn build(self) -> Schema {
        Schema::new(SchemaKind::Object {
            properties: self.properties,
            required: self.required,
            additional_properties: self.additional_properties.map(Box::new),
        })
    }

    /// Builds and wraps with a name for code generation.
    pub fn named(self, name: &str) -> Schema {
        Schema::new(SchemaKind::Named {
            name: name.to_string(),
            schema: Box::new(self.build()),
        })
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .optional_field("email", SchemaBuilder::string().build())
            .build();

        match schema.kind {
            SchemaKind::Object {
                properties,
                required,
                ..
            } => {
                assert_eq!(properties.len(), 3);
                assert_eq!(required.len(), 2);
                assert!(required.contains(&"id".to_string()));
                assert!(required.contains(&"name".to_string()));
                assert!(!required.contains(&"email".to_string()));
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_array_with_constraints() {
        let schema = SchemaBuilder::array(SchemaBuilder::string().build())
            .min_items(1)
            .max_items(10)
            .unique_items(true)
            .build();

        match schema.kind {
            SchemaKind::Array {
                min_items,
                max_items,
                unique_items,
                ..
            } => {
                assert_eq!(min_items, Some(1));
                assert_eq!(max_items, Some(10));
                assert_eq!(unique_items, Some(true));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_optional_type() {
        let schema = SchemaBuilder::optional(SchemaBuilder::string().build());

        match schema.kind {
            SchemaKind::Union { any_of } => {
                assert_eq!(any_of.len(), 2);
                assert!(matches!(&any_of[0].kind, SchemaKind::String { .. }));
                assert!(matches!(&any_of[1].kind, SchemaKind::Null));
            }
            _ => panic!("Expected Union"),
        }
    }

    #[test]
    fn test_named_schema() {
        let schema = SchemaBuilder::object()
            .field("x", SchemaBuilder::int64())
            .named("Point");

        match schema.kind {
            SchemaKind::Named { name, schema } => {
                assert_eq!(name, "Point");
                assert!(matches!(schema.kind, SchemaKind::Object { .. }));
            }
            _ => panic!("Expected Named"),
        }
    }

    #[test]
    fn test_string_with_format() {
        let schema = SchemaBuilder::string()
            .format(StringFormat::Email)
            .min_length(5)
            .max_length(100)
            .build();

        match schema.kind {
            SchemaKind::String {
                format,
                min_length,
                max_length,
                ..
            } => {
                assert_eq!(format, Some(StringFormat::Email));
                assert_eq!(min_length, Some(5));
                assert_eq!(max_length, Some(100));
            }
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_function_builder() {
        let schema = SchemaBuilder::function(
            vec![SchemaBuilder::int64(), SchemaBuilder::string().build()],
            SchemaBuilder::void(),
        );

        match schema.kind {
            SchemaKind::Function {
                parameters,
                returns,
            } => {
                assert_eq!(parameters.len(), 2);
                assert!(matches!(&returns.kind, SchemaKind::Void));
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_special_types() {
        assert!(matches!(SchemaBuilder::void().kind, SchemaKind::Void));
        assert!(matches!(SchemaBuilder::never().kind, SchemaKind::Never));
        assert!(matches!(SchemaBuilder::any().kind, SchemaKind::Any));
        assert!(matches!(SchemaBuilder::unknown().kind, SchemaKind::Unknown));
        assert!(matches!(
            SchemaBuilder::undefined().kind,
            SchemaKind::Undefined
        ));
    }

    #[test]
    fn test_metadata_methods() {
        let schema = SchemaBuilder::string()
            .build()
            .with_id("https://example.com/schemas/email")
            .with_title("Email")
            .with_description("An email address");

        assert_eq!(
            schema.id,
            Some("https://example.com/schemas/email".to_string())
        );
        assert_eq!(schema.title, Some("Email".to_string()));
        assert_eq!(schema.description, Some("An email address".to_string()));
    }

    #[test]
    fn test_recursive_type() {
        let schema = SchemaBuilder::recursive("JsonTree", |this| {
            SchemaBuilder::union(vec![
                SchemaBuilder::null(),
                SchemaBuilder::bool(),
                SchemaBuilder::int64(),
                SchemaBuilder::string().build(),
                SchemaBuilder::array(this.clone()).build(),
            ])
        });

        assert!(matches!(schema.kind, SchemaKind::Recursive { .. }));
        assert_eq!(schema.id, Some("JsonTree".to_string()));
    }

    #[test]
    fn test_intersect_type() {
        let node = SchemaBuilder::object()
            .field("type", SchemaBuilder::string().build())
            .build();

        let with_value = SchemaBuilder::object()
            .field("value", SchemaBuilder::any())
            .build();

        let literal = SchemaBuilder::intersect(vec![node, with_value]);

        match literal.kind {
            SchemaKind::Intersect { all_of } => {
                assert_eq!(all_of.len(), 2);
            }
            _ => panic!("Expected Intersect"),
        }
    }
}
