use crate::schema::{LiteralValue, Property, Schema};

pub struct SchemaBuilder;

impl SchemaBuilder {
    pub fn null() -> Schema {
        Schema::Null
    }

    pub fn bool() -> Schema {
        Schema::Bool
    }

    pub fn int8() -> Schema {
        Schema::Int8
    }

    pub fn int16() -> Schema {
        Schema::Int16
    }

    pub fn int32() -> Schema {
        Schema::Int32
    }

    pub fn int64() -> Schema {
        Schema::Int64
    }

    pub fn uint8() -> Schema {
        Schema::UInt8
    }

    pub fn uint16() -> Schema {
        Schema::UInt16
    }

    pub fn uint32() -> Schema {
        Schema::UInt32
    }

    pub fn uint64() -> Schema {
        Schema::UInt64
    }

    pub fn float32() -> Schema {
        Schema::Float32
    }

    pub fn float64() -> Schema {
        Schema::Float64
    }

    pub fn string() -> Schema {
        Schema::String
    }

    pub fn bytes() -> Schema {
        Schema::Bytes
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

// Implement From traits for LiteralValue
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

pub struct ArrayBuilder {
    items: Schema,
    min_items: Option<usize>,
    max_items: Option<usize>,
}

impl ArrayBuilder {
    pub fn new(items: Schema) -> Self {
        Self {
            items,
            min_items: None,
            max_items: None,
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

    pub fn build(self) -> Schema {
        Schema::Array {
            items: Box::new(self.items),
            min_items: self.min_items,
            max_items: self.max_items,
        }
    }
}

pub struct ObjectBuilder {
    properties: Vec<Property>,
    additional_properties: bool,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            properties: vec![],
            additional_properties: false,
        }
    }

    pub fn field(mut self, name: &str, schema: Schema) -> Self {
        self.properties
            .push(Property::from_schema(name, &schema, false, None));
        self
    }

    pub fn optional_field(mut self, name: &str, schema: Schema) -> Self {
        self.properties
            .push(Property::from_schema(name, &schema, true, None));
        self
    }

    pub fn description(mut self, name: &str, desc: &str) -> Self {
        if let Some(prop) = self.properties.iter_mut().find(|p| p.name == name) {
            prop.description = Some(desc.to_string());
        }
        self
    }

    pub fn additional_properties(mut self, allow: bool) -> Self {
        self.additional_properties = allow;
        self
    }

    pub fn build(self) -> Schema {
        Schema::Object {
            properties: self.properties,
            additional_properties: self.additional_properties,
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
            .field("name", SchemaBuilder::string())
            .optional_field("email", SchemaBuilder::string())
            .build();

        assert!(matches!(schema, Schema::Object { .. }));
    }

    #[test]
    fn test_array_with_constraints() {
        let schema = SchemaBuilder::array(SchemaBuilder::string())
            .min_items(1)
            .max_items(10)
            .build();

        match schema {
            Schema::Array {
                min_items,
                max_items,
                ..
            } => {
                assert_eq!(min_items, Some(1));
                assert_eq!(max_items, Some(10));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_optional_type() {
        let schema = SchemaBuilder::optional(SchemaBuilder::string());

        match schema {
            Schema::Union { any_of } => {
                assert_eq!(any_of.len(), 2);
                assert!(matches!(&any_of[0], Schema::String));
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
}
