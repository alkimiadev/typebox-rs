use crate::error::ValidationError;
use crate::schema::{LiteralValue, Schema};
use std::collections::HashSet;

pub fn validate(schema: &Schema, value: &serde_json::Value) -> Result<(), ValidationError> {
    match (schema, value) {
        (Schema::Null, serde_json::Value::Null) => Ok(()),

        (Schema::Bool, serde_json::Value::Bool(_)) => Ok(()),

        (Schema::Int8, serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                    return Ok(());
                }
            }
            Err(ValidationError::TypeMismatch {
                expected: "int8".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::Int16, serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                    return Ok(());
                }
            }
            Err(ValidationError::TypeMismatch {
                expected: "int16".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::Int32, serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    return Ok(());
                }
            }
            Err(ValidationError::TypeMismatch {
                expected: "int32".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::Int64, serde_json::Value::Number(n)) => {
            if n.is_i64() {
                return Ok(());
            }
            Err(ValidationError::TypeMismatch {
                expected: "int64".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::UInt8, serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_u64() {
                if i <= u8::MAX as u64 {
                    return Ok(());
                }
            }
            Err(ValidationError::TypeMismatch {
                expected: "uint8".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::UInt16, serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_u64() {
                if i <= u16::MAX as u64 {
                    return Ok(());
                }
            }
            Err(ValidationError::TypeMismatch {
                expected: "uint16".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::UInt32, serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_u64() {
                if i <= u32::MAX as u64 {
                    return Ok(());
                }
            }
            Err(ValidationError::TypeMismatch {
                expected: "uint32".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::UInt64, serde_json::Value::Number(n)) => {
            if n.is_u64() {
                return Ok(());
            }
            Err(ValidationError::TypeMismatch {
                expected: "uint64".to_string(),
                actual: value_type(value),
            })
        }

        (Schema::Float32 | Schema::Float64, serde_json::Value::Number(_)) => Ok(()),

        (Schema::String, serde_json::Value::String(_)) => Ok(()),

        (Schema::Bytes, serde_json::Value::Array(arr)) => {
            for item in arr {
                if !item.is_number() {
                    return Err(ValidationError::TypeMismatch {
                        expected: "bytes".to_string(),
                        actual: value_type(value),
                    });
                }
            }
            Ok(())
        }

        (
            Schema::Array {
                items,
                min_items,
                max_items,
            },
            serde_json::Value::Array(arr),
        ) => {
            if let Some(min) = min_items {
                if arr.len() < *min {
                    return Err(ValidationError::MinItems {
                        min: *min,
                        actual: arr.len(),
                    });
                }
            }
            if let Some(max) = max_items {
                if arr.len() > *max {
                    return Err(ValidationError::MaxItems {
                        max: *max,
                        actual: arr.len(),
                    });
                }
            }
            for (i, item) in arr.iter().enumerate() {
                validate(items, item).map_err(|e| e.with_path(i.to_string()))?;
            }
            Ok(())
        }

        (
            Schema::Object {
                properties,
                additional_properties,
            },
            serde_json::Value::Object(map),
        ) => {
            let schema_fields: HashSet<_> = properties.iter().map(|p| &p.name).collect();

            // Check required fields
            for prop in properties {
                if let Some(val) = map.get(&prop.name) {
                    validate(&prop.schema(), val).map_err(|e| e.with_path(&prop.name))?;
                } else if !prop.optional {
                    return Err(ValidationError::MissingField {
                        field: prop.name.clone(),
                    });
                }
            }

            // Check for unknown fields
            if !additional_properties {
                for key in map.keys() {
                    if !schema_fields.contains(key) {
                        return Err(ValidationError::UnknownField { field: key.clone() });
                    }
                }
            }

            Ok(())
        }

        (Schema::Tuple { items }, serde_json::Value::Array(arr)) => {
            if arr.len() != items.len() {
                return Err(ValidationError::TypeMismatch {
                    expected: format!("tuple of {} elements", items.len()),
                    actual: format!("array of {} elements", arr.len()),
                });
            }
            for (i, (item_schema, item_val)) in items.iter().zip(arr.iter()).enumerate() {
                validate(item_schema, item_val).map_err(|e| e.with_path(i.to_string()))?;
            }
            Ok(())
        }

        (Schema::Union { any_of }, value) => {
            for variant in any_of {
                if validate(variant, value).is_ok() {
                    return Ok(());
                }
            }
            Err(ValidationError::NoMatchingVariant)
        }

        (Schema::Literal { value: lit }, val) => match (lit, val) {
            (LiteralValue::Null, serde_json::Value::Null) => Ok(()),
            (LiteralValue::Boolean(b), serde_json::Value::Bool(v)) if b == v => Ok(()),
            (LiteralValue::String(s), serde_json::Value::String(v)) if s == v => Ok(()),
            (LiteralValue::Number(n), serde_json::Value::Number(v)) if v.as_i64() == Some(*n) => {
                Ok(())
            }
            (LiteralValue::Float(f), serde_json::Value::Number(v)) if v.as_f64() == Some(*f) => {
                Ok(())
            }
            _ => Err(ValidationError::InvalidLiteral),
        },

        (Schema::Enum { values }, serde_json::Value::String(s)) => {
            if values.contains(s) {
                Ok(())
            } else {
                Err(ValidationError::NotInEnum(s.clone()))
            }
        }

        (Schema::Ref { reference }, value) => {
            // TODO: Resolve reference from registry
            Err(ValidationError::TypeMismatch {
                expected: reference.clone(),
                actual: value_type(value),
            })
        }

        (Schema::Named { schema, .. }, value) => validate(schema, value),

        _ => Err(ValidationError::TypeMismatch {
            expected: schema.kind().to_string(),
            actual: value_type(value),
        }),
    }
}

fn value_type(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;

    #[test]
    fn test_validate_primitives() {
        assert!(validate(&SchemaBuilder::null(), &serde_json::json!(null)).is_ok());
        assert!(validate(&SchemaBuilder::bool(), &serde_json::json!(true)).is_ok());
        assert!(validate(&SchemaBuilder::int64(), &serde_json::json!(42)).is_ok());
        assert!(validate(&SchemaBuilder::string(), &serde_json::json!("hello")).is_ok());
        assert!(validate(&SchemaBuilder::float64(), &serde_json::json!(3.14)).is_ok());
    }

    #[test]
    fn test_validate_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string())
            .optional_field("email", SchemaBuilder::string())
            .build();

        let valid = serde_json::json!({
            "id": 1,
            "name": "Alice"
        });
        assert!(validate(&schema, &valid).is_ok());

        let with_email = serde_json::json!({
            "id": 2,
            "name": "Bob",
            "email": "bob@example.com"
        });
        assert!(validate(&schema, &with_email).is_ok());

        let missing_required = serde_json::json!({
            "name": "Charlie"
        });
        assert!(matches!(
            validate(&schema, &missing_required),
            Err(ValidationError::MissingField { .. })
        ));
    }

    #[test]
    fn test_validate_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64())
            .min_items(1)
            .max_items(3)
            .build();

        assert!(validate(&schema, &serde_json::json!([1, 2])).is_ok());
        assert!(matches!(
            validate(&schema, &serde_json::json!([])),
            Err(ValidationError::MinItems { .. })
        ));
        assert!(matches!(
            validate(&schema, &serde_json::json!([1, 2, 3, 4])),
            Err(ValidationError::MaxItems { .. })
        ));
    }

    #[test]
    fn test_validate_union() {
        let schema = SchemaBuilder::union(vec![SchemaBuilder::string(), SchemaBuilder::int64()]);

        assert!(validate(&schema, &serde_json::json!("hello")).is_ok());
        assert!(validate(&schema, &serde_json::json!(42)).is_ok());
        assert!(matches!(
            validate(&schema, &serde_json::json!(true)),
            Err(ValidationError::NoMatchingVariant)
        ));
    }

    #[test]
    fn test_validate_optional() {
        let schema = SchemaBuilder::optional(SchemaBuilder::string());

        assert!(validate(&schema, &serde_json::json!("hello")).is_ok());
        assert!(validate(&schema, &serde_json::json!(null)).is_ok());
    }
}
