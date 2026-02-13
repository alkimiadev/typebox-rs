use crate::schema::{LiteralValue, Schema, StringFormat};
use indexmap::IndexMap;

pub struct SchemaBuilder;

impl SchemaBuilder {
    pub fn null() -> Schema {
        Schema::Null
    }

    pub fn bool() -> Schema {
        Schema::Bool
    }

    pub fn int8() -> Schema {
        Schema::Int8 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn int16() -> Schema {
        Schema::Int16 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn int32() -> Schema {
        Schema::Int32 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn int64() -> Schema {
        Schema::Int64 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn uint8() -> Schema {
        Schema::UInt8 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn uint16() -> Schema {
        Schema::UInt16 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn uint32() -> Schema {
        Schema::UInt32 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn uint64() -> Schema {
        Schema::UInt64 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn float32() -> Schema {
        Schema::Float32 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn float64() -> Schema {
        Schema::Float64 {
            minimum: None,
            maximum: None,
        }
    }

    pub fn string() -> StringBuilder {
        StringBuilder::new()
    }

    pub fn bytes() -> Schema {
        Schema::Bytes {
            min_length: None,
            max_length: None,
        }
    }

    pub fn array(items: Schema) -> ArrayBuilder {
        ArrayBuilder::new(items)
    }

    pub fn object() -> ObjectBuilder {
        ObjectBuilder::new()
    }

    pub fn tuple(items: Vec<Schema>) -> Schema {
        Schema::Tuple { items }
    }

    pub fn union(variants: Vec<Schema>) -> Schema {
        Schema::Union { any_of: variants }
    }

    pub fn optional(schema: Schema) -> Schema {
        Schema::Union {
            any_of: vec![schema, Schema::Null],
        }
    }

    pub fn literal(value: impl Into<LiteralValue>) -> Schema {
        Schema::Literal {
            value: value.into(),
        }
    }

    pub fn enum_values(values: Vec<&str>) -> Schema {
        Schema::Enum {
            values: values.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn r#ref(name: &str) -> Schema {
        Schema::Ref {
            reference: format!("#/definitions/{}", name),
        }
    }

    pub fn named(name: &str, schema: Schema) -> Schema {
        Schema::Named {
            name: name.to_string(),
            schema: Box::new(schema),
        }
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
        Schema::String {
            format: self.format,
            pattern: self.pattern,
            min_length: self.min_length,
            max_length: self.max_length,
        }
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
        Schema::Array {
            items: Box::new(self.items),
            min_items: self.min_items,
            max_items: self.max_items,
            unique_items: self.unique_items,
        }
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
        Schema::Object {
            properties: self.properties,
            required: self.required,
            additional_properties: self.additional_properties.map(Box::new),
        }
    }

    pub fn named(self, name: &str) -> Schema {
        Schema::Named {
            name: name.to_string(),
            schema: Box::new(self.build()),
        }
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

        match schema {
            Schema::Object {
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

        match schema {
            Schema::Array {
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

        match schema {
            Schema::Union { any_of } => {
                assert_eq!(any_of.len(), 2);
                assert!(matches!(&any_of[0], Schema::String { .. }));
                assert!(matches!(&any_of[1], Schema::Null));
            }
            _ => panic!("Expected Union"),
        }
    }

    #[test]
    fn test_named_schema() {
        let schema = SchemaBuilder::object()
            .field("x", SchemaBuilder::int64())
            .named("Point");

        match schema {
            Schema::Named { name, schema } => {
                assert_eq!(name, "Point");
                assert!(matches!(*schema, Schema::Object { .. }));
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

        match schema {
            Schema::String {
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
}
