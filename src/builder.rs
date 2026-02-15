use crate::schema::{LiteralValue, Schema, SchemaKind, StringFormat};
use indexmap::IndexMap;

pub struct SchemaBuilder;

impl SchemaBuilder {
    pub fn null() -> Schema {
        Schema::new(SchemaKind::Null)
    }

    pub fn bool() -> Schema {
        Schema::new(SchemaKind::Bool)
    }

    pub fn int8() -> Schema {
        Schema::new(SchemaKind::Int8 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn int16() -> Schema {
        Schema::new(SchemaKind::Int16 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn int32() -> Schema {
        Schema::new(SchemaKind::Int32 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn int64() -> Schema {
        Schema::new(SchemaKind::Int64 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn uint8() -> Schema {
        Schema::new(SchemaKind::UInt8 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn uint16() -> Schema {
        Schema::new(SchemaKind::UInt16 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn uint32() -> Schema {
        Schema::new(SchemaKind::UInt32 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn uint64() -> Schema {
        Schema::new(SchemaKind::UInt64 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn float32() -> Schema {
        Schema::new(SchemaKind::Float32 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn float64() -> Schema {
        Schema::new(SchemaKind::Float64 {
            minimum: None,
            maximum: None,
        })
    }

    pub fn string() -> StringBuilder {
        StringBuilder::new()
    }

    pub fn bytes() -> Schema {
        Schema::new(SchemaKind::Bytes {
            min_length: None,
            max_length: None,
        })
    }

    pub fn array(items: Schema) -> ArrayBuilder {
        ArrayBuilder::new(items)
    }

    pub fn object() -> ObjectBuilder {
        ObjectBuilder::new()
    }

    pub fn tuple(items: Vec<Schema>) -> Schema {
        Schema::new(SchemaKind::Tuple { items })
    }

    pub fn union(variants: Vec<Schema>) -> Schema {
        Schema::new(SchemaKind::Union { any_of: variants })
    }

    pub fn optional(schema: Schema) -> Schema {
        Schema::new(SchemaKind::Union {
            any_of: vec![schema, Schema::new(SchemaKind::Null)],
        })
    }

    pub fn literal(value: impl Into<LiteralValue>) -> Schema {
        Schema::new(SchemaKind::Literal {
            value: value.into(),
        })
    }

    pub fn enum_values(values: Vec<&str>) -> Schema {
        Schema::new(SchemaKind::Enum {
            values: values.iter().map(|s| s.to_string()).collect(),
        })
    }

    pub fn r#ref(name: &str) -> Schema {
        Schema::new(SchemaKind::Ref {
            reference: format!("#/definitions/{}", name),
        })
    }

    pub fn named(name: &str, schema: Schema) -> Schema {
        Schema::new(SchemaKind::Named {
            name: name.to_string(),
            schema: Box::new(schema),
        })
    }

    pub fn function(parameters: Vec<Schema>, returns: Schema) -> Schema {
        Schema::new(SchemaKind::Function {
            parameters,
            returns: Box::new(returns),
        })
    }

    pub fn void() -> Schema {
        Schema::new(SchemaKind::Void)
    }

    pub fn never() -> Schema {
        Schema::new(SchemaKind::Never)
    }

    pub fn any() -> Schema {
        Schema::new(SchemaKind::Any)
    }

    pub fn unknown() -> Schema {
        Schema::new(SchemaKind::Unknown)
    }

    pub fn undefined() -> Schema {
        Schema::new(SchemaKind::Undefined)
    }

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

pub struct StringBuilder {
    format: Option<StringFormat>,
    pattern: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self {
            format: None,
            pattern: None,
            min_length: None,
            max_length: None,
        }
    }

    pub fn format(mut self, format: StringFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

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

pub struct ArrayBuilder {
    items: Schema,
    min_items: Option<usize>,
    max_items: Option<usize>,
    unique_items: Option<bool>,
}

impl ArrayBuilder {
    pub fn new(items: Schema) -> Self {
        Self {
            items,
            min_items: None,
            max_items: None,
            unique_items: None,
        }
    }

    pub fn min_items(mut self, min: usize) -> Self {
        self.min_items = Some(min);
        self
    }

    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    pub fn unique_items(mut self, unique: bool) -> Self {
        self.unique_items = Some(unique);
        self
    }

    pub fn build(self) -> Schema {
        Schema::new(SchemaKind::Array {
            items: Box::new(self.items),
            min_items: self.min_items,
            max_items: self.max_items,
            unique_items: self.unique_items,
        })
    }
}

pub struct ObjectBuilder {
    properties: IndexMap<String, Schema>,
    required: Vec<String>,
    additional_properties: Option<Schema>,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
            required: Vec::new(),
            additional_properties: None,
        }
    }

    pub fn field(mut self, name: &str, schema: Schema) -> Self {
        self.properties.insert(name.to_string(), schema);
        self.required.push(name.to_string());
        self
    }

    pub fn optional_field(mut self, name: &str, schema: Schema) -> Self {
        self.properties.insert(name.to_string(), schema);
        self
    }

    pub fn additional_properties(mut self, schema: Option<Schema>) -> Self {
        self.additional_properties = schema;
        self
    }

    pub fn build(self) -> Schema {
        Schema::new(SchemaKind::Object {
            properties: self.properties,
            required: self.required,
            additional_properties: self.additional_properties.map(Box::new),
        })
    }

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
