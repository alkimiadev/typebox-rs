use crate::error::CleanError;
use crate::schema::Schema;
use crate::value::Value;
use indexmap::IndexMap;

pub fn clean(schema: &Schema, value: &Value) -> Result<Value, CleanError> {
    match (schema, value) {
        (Schema::Null, Value::Null) => Ok(Value::Null),
        (Schema::Bool, Value::Bool(b)) => Ok(Value::Bool(*b)),

        (Schema::Int8 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::Int16 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::Int32 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::Int64 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::UInt8 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::UInt16 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::UInt32 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::UInt64 { .. }, Value::Int64(n)) => Ok(Value::Int64(*n)),
        (Schema::Float32 { .. }, Value::Float64(f)) => Ok(Value::Float64(*f)),
        (Schema::Float64 { .. }, Value::Float64(f)) => Ok(Value::Float64(*f)),

        (Schema::String { .. }, Value::String(s)) => Ok(Value::String(s.clone())),
        (Schema::Bytes { .. }, Value::Bytes(b)) => Ok(Value::Bytes(b.clone())),
        (Schema::Bytes { .. }, Value::UInt8Array(b)) => Ok(Value::UInt8Array(b.clone())),

        (Schema::Array { items, .. }, Value::Array(arr)) => {
            let cleaned: Result<Vec<Value>, CleanError> =
                arr.iter().map(|v| clean(items, v)).collect();
            Ok(Value::Array(cleaned?))
        }

        (
            Schema::Object {
                properties,
                additional_properties,
                ..
            },
            Value::Object(map),
        ) => {
            let mut result = IndexMap::new();

            for (key, val) in map {
                if let Some(prop_schema) = properties.get(key) {
                    result.insert(key.clone(), clean(prop_schema, val)?);
                } else if let Some(ref additional_schema) = additional_properties {
                    if super::check::check(additional_schema, val) {
                        result.insert(key.clone(), clean(additional_schema, val)?);
                    }
                }
            }

            Ok(Value::Object(result))
        }

        (Schema::Tuple { items }, Value::Array(arr)) => {
            let len = items.len().min(arr.len());
            let mut result = Vec::with_capacity(len);
            for i in 0..len {
                result.push(clean(&items[i], &arr[i])?);
            }
            Ok(Value::Array(result))
        }

        (Schema::Union { any_of }, value) => {
            for variant in any_of {
                if super::check::check(variant, value) {
                    return clean(variant, value);
                }
            }
            Ok(value.clone())
        }

        (Schema::Literal { .. }, val) => Ok(val.clone()),

        (Schema::Enum { .. }, Value::String(s)) => Ok(Value::String(s.clone())),

        (Schema::Ref { reference }, _) => Err(CleanError::CannotClean(format!(
            "unresolved ref: {}",
            reference
        ))),

        (Schema::Named { schema, .. }, value) => clean(schema, value),

        _ => Ok(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;

    #[test]
    fn test_clean_primitives() {
        assert_eq!(clean(&Schema::Null, &Value::Null).unwrap(), Value::Null);
        assert_eq!(
            clean(&Schema::Bool, &Value::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            clean(&SchemaBuilder::int64(), &Value::Int64(42)).unwrap(),
            Value::Int64(42)
        );
        assert_eq!(
            clean(
                &SchemaBuilder::string().build(),
                &Value::String("hello".to_string())
            )
            .unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_clean_object_removes_extra() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build();

        let value = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("test".to_string()))
            .field("extra", Value::String("should be removed".to_string()))
            .build();

        let cleaned = clean(&schema, &value).unwrap();
        let map = cleaned.as_object().unwrap();

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("id"));
        assert!(map.contains_key("name"));
        assert!(!map.contains_key("extra"));
    }

    #[test]
    fn test_clean_object_keeps_additional() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .additional_properties(Some(SchemaBuilder::string().build()))
            .build();

        let value = Value::object()
            .field("id", Value::Int64(1))
            .field("extra", Value::String("kept".to_string()))
            .field("bad_extra", Value::Int64(42))
            .build();

        let cleaned = clean(&schema, &value).unwrap();
        let map = cleaned.as_object().unwrap();

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("id"));
        assert!(map.contains_key("extra"));
        assert!(!map.contains_key("bad_extra"));
    }

    #[test]
    fn test_clean_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64()).build();

        let value = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);
        let cleaned = clean(&schema, &value).unwrap();

        assert_eq!(cleaned, value);
    }

    #[test]
    fn test_clean_tuple_truncates() {
        let schema = Schema::Tuple {
            items: vec![SchemaBuilder::int64(), SchemaBuilder::string().build()],
        };

        let value = Value::Array(vec![
            Value::Int64(1),
            Value::String("two".to_string()),
            Value::Int64(3),
        ]);

        let cleaned = clean(&schema, &value).unwrap();
        assert_eq!(cleaned.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_clean_nested_object() {
        let inner_schema = SchemaBuilder::object()
            .field("x", SchemaBuilder::int64())
            .build();

        let schema = SchemaBuilder::object()
            .field("nested", inner_schema.clone())
            .build();

        let inner_value = Value::object()
            .field("x", Value::Int64(1))
            .field("y", Value::Int64(2))
            .build();

        let value = Value::object().field("nested", inner_value).build();

        let cleaned = clean(&schema, &value).unwrap();
        let outer_map = cleaned.as_object().unwrap();
        let inner_cleaned = outer_map.get("nested").unwrap().as_object().unwrap();

        assert!(inner_cleaned.contains_key("x"));
        assert!(!inner_cleaned.contains_key("y"));
    }

    #[test]
    fn test_clean_union() {
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::int64(),
            SchemaBuilder::string().build(),
        ]);

        let int_val = Value::Int64(42);
        let cleaned = clean(&schema, &int_val).unwrap();
        assert_eq!(cleaned, int_val);

        let str_val = Value::String("hello".to_string());
        let cleaned = clean(&schema, &str_val).unwrap();
        assert_eq!(cleaned, str_val);
    }

    #[test]
    fn test_clean_literal() {
        let schema = Schema::Literal {
            value: LiteralValue::String("hello".to_string()),
        };

        let value = Value::String("hello".to_string());
        let cleaned = clean(&schema, &value).unwrap();
        assert_eq!(cleaned, value);
    }

    #[test]
    fn test_clean_enum() {
        let schema = Schema::Enum {
            values: vec!["one".to_string(), "two".to_string()],
        };

        let value = Value::String("one".to_string());
        let cleaned = clean(&schema, &value).unwrap();
        assert_eq!(cleaned, value);
    }

    #[test]
    fn test_clean_returns_same_if_no_match() {
        let schema = SchemaBuilder::int64();
        let value = Value::String("not an int".to_string());
        let cleaned = clean(&schema, &value).unwrap();
        assert_eq!(cleaned, value);
    }
}
