//! Default value generation from schemas.

use crate::error::CreateError;
use crate::schema::{LiteralValue, Schema, SchemaKind};
use crate::value::Value;
use indexmap::IndexMap;

/// Create a default value conforming to the schema.
///
/// Uses minimum bounds for numbers, empty strings, empty arrays,
/// and required object fields.
pub fn create(schema: &Schema) -> Result<Value, CreateError> {
    match &schema.kind {
        SchemaKind::Null => Ok(Value::Null),

        SchemaKind::Bool => Ok(Value::Bool(false)),

        SchemaKind::Int8 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),
        SchemaKind::Int16 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),
        SchemaKind::Int32 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),
        SchemaKind::Int64 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0))),

        SchemaKind::UInt8 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),
        SchemaKind::UInt16 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),
        SchemaKind::UInt32 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),
        SchemaKind::UInt64 { minimum, .. } => Ok(Value::Int64(minimum.unwrap_or(0) as i64)),

        SchemaKind::Float32 { minimum, .. } => Ok(Value::Float64(minimum.unwrap_or(0.0) as f64)),
        SchemaKind::Float64 { minimum, .. } => Ok(Value::Float64(minimum.unwrap_or(0.0))),

        SchemaKind::String { .. } => Ok(Value::String(String::new())),

        SchemaKind::Bytes { .. } => Ok(Value::Bytes(Vec::new())),

        SchemaKind::Array {
            items, min_items, ..
        } => {
            let count = min_items.unwrap_or(0);
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count {
                arr.push(create(items)?);
            }
            Ok(Value::Array(arr))
        }

        SchemaKind::Object {
            properties,
            required,
            ..
        } => {
            let mut obj = IndexMap::new();
            for field_name in required {
                if let Some(field_schema) = properties.get(field_name) {
                    obj.insert(field_name.clone(), create(field_schema)?);
                }
            }
            Ok(Value::Object(obj))
        }

        SchemaKind::Tuple { items } => {
            let mut arr = Vec::with_capacity(items.len());
            for item_schema in items {
                arr.push(create(item_schema)?);
            }
            Ok(Value::Array(arr))
        }

        SchemaKind::Union { any_of } => any_of
            .first()
            .map(create)
            .unwrap_or_else(|| Ok(Value::Null)),

        SchemaKind::Literal { value } => Ok(match value {
            LiteralValue::Null => Value::Null,
            LiteralValue::Boolean(b) => Value::Bool(*b),
            LiteralValue::Number(n) => Value::Int64(*n),
            LiteralValue::Float(f) => Value::Float64(*f),
            LiteralValue::String(s) => Value::String(s.clone()),
        }),

        SchemaKind::Enum { values } => values
            .first()
            .map(|s| Value::String(s.clone()))
            .ok_or_else(|| CreateError::UnsupportedSchema("empty enum".to_string())),

        SchemaKind::Ref { reference } => Err(CreateError::UnsupportedSchema(format!(
            "unresolved ref: {}",
            reference
        ))),

        SchemaKind::Named { schema, .. } => create(schema),

        SchemaKind::Function { .. } => Ok(Value::Null),

        SchemaKind::Void => Ok(Value::Null),

        SchemaKind::Never => Err(CreateError::UnsupportedSchema("never type".to_string())),

        SchemaKind::Any => Ok(Value::Null),

        SchemaKind::Unknown => Ok(Value::Null),

        SchemaKind::Undefined => Ok(Value::Null),

        SchemaKind::Recursive { schema } => create(schema),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;
    use crate::schema::Schema;

    #[test]
    fn test_create_primitives() {
        assert_eq!(create(&Schema::new(SchemaKind::Null)).unwrap(), Value::Null);
        assert_eq!(
            create(&Schema::new(SchemaKind::Bool)).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(create(&SchemaBuilder::int64()).unwrap(), Value::Int64(0));
        assert_eq!(
            create(&SchemaBuilder::float64()).unwrap(),
            Value::Float64(0.0)
        );
        assert_eq!(
            create(&SchemaBuilder::string().build()).unwrap(),
            Value::String(String::new())
        );
    }

    #[test]
    fn test_create_numeric_with_minimum() {
        let schema = Schema::new(SchemaKind::Int64 {
            minimum: Some(10),
            maximum: None,
        });
        assert_eq!(create(&schema).unwrap(), Value::Int64(10));

        let schema = Schema::new(SchemaKind::Float64 {
            minimum: Some(1.5),
            maximum: None,
        });
        assert_eq!(create(&schema).unwrap(), Value::Float64(1.5));
    }

    #[test]
    fn test_create_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64()).build();
        assert_eq!(create(&schema).unwrap(), Value::Array(vec![]));
    }

    #[test]
    fn test_create_array_with_min_items() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64())
            .min_items(3)
            .build();
        let result = create(&schema).unwrap();
        assert_eq!(
            result,
            Value::Array(vec![Value::Int64(0), Value::Int64(0), Value::Int64(0)])
        );
    }

    #[test]
    fn test_create_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build();

        let result = create(&schema).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("id"), Some(&Value::Int64(0)));
        assert_eq!(obj.get("name"), Some(&Value::String(String::new())));
    }

    #[test]
    fn test_create_tuple() {
        let schema = Schema::new(SchemaKind::Tuple {
            items: vec![SchemaBuilder::int64(), SchemaBuilder::string().build()],
        });

        let result = create(&schema).unwrap();
        assert_eq!(
            result,
            Value::Array(vec![Value::Int64(0), Value::String(String::new())])
        );
    }

    #[test]
    fn test_create_union() {
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::string().build(),
            SchemaBuilder::int64(),
        ]);

        let result = create(&schema).unwrap();
        assert_eq!(result, Value::String(String::new()));
    }

    #[test]
    fn test_create_literal() {
        let schema = Schema::new(SchemaKind::Literal {
            value: LiteralValue::String("hello".to_string()),
        });
        assert_eq!(create(&schema).unwrap(), Value::String("hello".to_string()));

        let schema = Schema::new(SchemaKind::Literal {
            value: LiteralValue::Number(42),
        });
        assert_eq!(create(&schema).unwrap(), Value::Int64(42));
    }

    #[test]
    fn test_create_enum() {
        let schema = Schema::new(SchemaKind::Enum {
            values: vec!["one".to_string(), "two".to_string()],
        });
        assert_eq!(create(&schema).unwrap(), Value::String("one".to_string()));
    }

    #[test]
    fn test_create_ref_fails() {
        let schema = Schema::new(SchemaKind::Ref {
            reference: "#/definitions/MyType".to_string(),
        });
        assert!(create(&schema).is_err());
    }

    #[test]
    fn test_create_function() {
        let schema = SchemaBuilder::function(vec![SchemaBuilder::int64()], SchemaBuilder::void());
        assert_eq!(create(&schema).unwrap(), Value::Null);
    }

    #[test]
    fn test_create_void() {
        assert_eq!(create(&SchemaBuilder::void()).unwrap(), Value::Null);
    }

    #[test]
    fn test_create_never_fails() {
        assert!(create(&SchemaBuilder::never()).is_err());
    }

    #[test]
    fn test_create_any() {
        assert_eq!(create(&SchemaBuilder::any()).unwrap(), Value::Null);
    }

    #[test]
    fn test_create_unknown() {
        assert_eq!(create(&SchemaBuilder::unknown()).unwrap(), Value::Null);
    }

    #[test]
    fn test_create_undefined() {
        assert_eq!(create(&SchemaBuilder::undefined()).unwrap(), Value::Null);
    }
}
