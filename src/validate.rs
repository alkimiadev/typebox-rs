use crate::error::ValidationError;
use crate::format::FormatRegistry;
use crate::registry::SchemaRegistry;
use crate::schema::{LiteralValue, Schema, SchemaKind, StringFormat};
use crate::value::{hash_fnv1a, Value};
use std::collections::HashSet;

pub fn validate(schema: &Schema, value: &Value) -> Result<(), ValidationError> {
    validate_full(schema, value, None, None)
}

pub fn validate_with_registry(
    schema: &Schema,
    value: &Value,
    registry: Option<&SchemaRegistry>,
) -> Result<(), ValidationError> {
    validate_full(schema, value, registry, None)
}

pub fn validate_with_format(
    schema: &Schema,
    value: &Value,
    registry: Option<&SchemaRegistry>,
    formats: Option<&FormatRegistry>,
) -> Result<(), ValidationError> {
    validate_full(schema, value, registry, formats)
}

fn validate_full(
    schema: &Schema,
    value: &Value,
    registry: Option<&SchemaRegistry>,
    formats: Option<&FormatRegistry>,
) -> Result<(), ValidationError> {
    match (&schema.kind, value) {
        (SchemaKind::Null, Value::Null) => Ok(()),

        (SchemaKind::Bool, Value::Bool(_)) => Ok(()),

        (SchemaKind::Int8 { minimum, maximum }, Value::Int64(n)) => {
            let n8 = i8::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "int8".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                f64::from(n8),
                minimum.map(f64::from),
                maximum.map(f64::from),
            )?;
            Ok(())
        }

        (SchemaKind::Int16 { minimum, maximum }, Value::Int64(n)) => {
            let n16 = i16::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "int16".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                f64::from(n16),
                minimum.map(f64::from),
                maximum.map(f64::from),
            )?;
            Ok(())
        }

        (SchemaKind::Int32 { minimum, maximum }, Value::Int64(n)) => {
            let n32 = i32::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "int32".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                f64::from(n32),
                minimum.map(f64::from),
                maximum.map(f64::from),
            )?;
            Ok(())
        }

        (SchemaKind::Int64 { minimum, maximum }, Value::Int64(n)) => {
            check_numeric_bounds(
                *n as f64,
                minimum.map(|m| m as f64),
                maximum.map(|m| m as f64),
            )?;
            Ok(())
        }

        (SchemaKind::UInt8 { minimum, maximum }, Value::Int64(n)) => {
            let n8 = u8::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "uint8".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                f64::from(n8),
                minimum.map(f64::from),
                maximum.map(f64::from),
            )?;
            Ok(())
        }

        (SchemaKind::UInt16 { minimum, maximum }, Value::Int64(n)) => {
            let n16 = u16::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "uint16".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                f64::from(n16),
                minimum.map(f64::from),
                maximum.map(f64::from),
            )?;
            Ok(())
        }

        (SchemaKind::UInt32 { minimum, maximum }, Value::Int64(n)) => {
            let n32 = u32::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "uint32".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                f64::from(n32),
                minimum.map(f64::from),
                maximum.map(f64::from),
            )?;
            Ok(())
        }

        (SchemaKind::UInt64 { minimum, maximum }, Value::Int64(n)) => {
            u64::try_from(*n).map_err(|_| ValidationError::TypeMismatch {
                expected: "uint64".to_string(),
                actual: value_type(value),
            })?;
            check_numeric_bounds(
                *n as f64,
                minimum.map(|m| m as f64),
                maximum.map(|m| m as f64),
            )?;
            Ok(())
        }

        (SchemaKind::Float32 { minimum, maximum }, Value::Float64(f)) => {
            check_numeric_bounds(*f, minimum.map(f64::from), maximum.map(f64::from))?;
            Ok(())
        }

        (SchemaKind::Float64 { minimum, maximum }, Value::Float64(f)) => {
            check_numeric_bounds(*f, *minimum, *maximum)?;
            Ok(())
        }

        (
            SchemaKind::String {
                format,
                min_length,
                max_length,
                ..
            },
            Value::String(s),
        ) => {
            if let Some(min) = min_length {
                if s.len() < *min {
                    return Err(ValidationError::MinLength {
                        min: *min,
                        actual: s.len(),
                    });
                }
            }
            if let Some(max) = max_length {
                if s.len() > *max {
                    return Err(ValidationError::MaxLength {
                        max: *max,
                        actual: s.len(),
                    });
                }
            }
            if let Some(fmt) = format {
                let format_name = match fmt {
                    StringFormat::Email => "email",
                    StringFormat::Uuid => "uuid",
                    StringFormat::Uri => "uri",
                    StringFormat::DateTime => "date-time",
                    StringFormat::Date => "date",
                    StringFormat::Time => "time",
                    StringFormat::Hostname => "hostname",
                    StringFormat::Ipv4 => "ipv4",
                    StringFormat::Ipv6 => "ipv6",
                    StringFormat::Custom(name) => name.as_str(),
                };
                if let Some(fmt_registry) = formats {
                    if let Some(result) = fmt_registry.validate(format_name, s) {
                        if !result {
                            return Err(ValidationError::InvalidFormat {
                                format: format_name.to_string(),
                                value: s.clone(),
                            });
                        }
                    }
                }
            }
            Ok(())
        }

        (SchemaKind::Bytes { .. }, Value::Bytes(_)) => Ok(()),
        (SchemaKind::Bytes { .. }, Value::UInt8Array(_)) => Ok(()),

        (
            SchemaKind::Array {
                items,
                min_items,
                max_items,
                unique_items,
            },
            Value::Array(arr),
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
            if unique_items.unwrap_or(false) {
                let mut seen = HashSet::new();
                for item in arr {
                    let hash = hash_fnv1a(item);
                    if !seen.insert(hash) {
                        return Err(ValidationError::DuplicateItem);
                    }
                }
            }
            for (i, item) in arr.iter().enumerate() {
                validate_full(items, item, registry, formats)
                    .map_err(|e| e.with_path(i.to_string()))?;
            }
            Ok(())
        }

        (
            SchemaKind::Array {
                items: _,
                min_items,
                max_items,
                ..
            },
            typed_arr,
        ) if is_typed_array(typed_arr) => {
            let len = typed_array_len(typed_arr);
            if let Some(min) = min_items {
                if len < *min {
                    return Err(ValidationError::MinItems {
                        min: *min,
                        actual: len,
                    });
                }
            }
            if let Some(max) = max_items {
                if len > *max {
                    return Err(ValidationError::MaxItems {
                        max: *max,
                        actual: len,
                    });
                }
            }
            Ok(())
        }

        (
            SchemaKind::Object {
                properties,
                required,
                additional_properties,
            },
            Value::Object(map),
        ) => {
            for name in required {
                if !map.contains_key(name) {
                    return Err(ValidationError::MissingField {
                        field: name.clone(),
                    });
                }
            }

            for (name, val) in map {
                if let Some(prop_schema) = properties.get(name) {
                    validate_full(prop_schema, val, registry, formats)
                        .map_err(|e| e.with_path(name))?;
                } else if let Some(ref additional) = additional_properties {
                    validate_full(additional, val, registry, formats)
                        .map_err(|e| e.with_path(name))?;
                } else {
                    return Err(ValidationError::UnknownField {
                        field: name.clone(),
                    });
                }
            }

            Ok(())
        }

        (SchemaKind::Tuple { items }, Value::Array(arr)) => {
            if arr.len() != items.len() {
                return Err(ValidationError::TypeMismatch {
                    expected: format!("tuple of {} elements", items.len()),
                    actual: format!("array of {} elements", arr.len()),
                });
            }
            for (i, (item_schema, item_val)) in items.iter().zip(arr.iter()).enumerate() {
                validate_full(item_schema, item_val, registry, formats)
                    .map_err(|e| e.with_path(i.to_string()))?;
            }
            Ok(())
        }

        (SchemaKind::Union { any_of }, value) => {
            for variant in any_of {
                if validate_full(variant, value, registry, formats).is_ok() {
                    return Ok(());
                }
            }
            Err(ValidationError::NoMatchingVariant)
        }

        (SchemaKind::Literal { value: lit }, val) => match (lit, val) {
            (LiteralValue::Null, Value::Null) => Ok(()),
            (LiteralValue::Boolean(b), Value::Bool(v)) if b == v => Ok(()),
            (LiteralValue::String(s), Value::String(v)) if s == v => Ok(()),
            (LiteralValue::Number(n), Value::Int64(v)) if n == v => Ok(()),
            (LiteralValue::Float(f), Value::Float64(v)) if (f - v).abs() < f64::EPSILON => Ok(()),
            _ => Err(ValidationError::InvalidLiteral),
        },

        (SchemaKind::Enum { values }, Value::String(s)) => {
            if values.contains(s) {
                Ok(())
            } else {
                Err(ValidationError::NotInEnum(s.clone()))
            }
        }

        (SchemaKind::Ref { reference }, value) => {
            let registry = registry.ok_or_else(|| ValidationError::TypeMismatch {
                expected: format!("resolved ref {}", reference),
                actual: "no registry".to_string(),
            })?;
            let resolved = registry
                .resolve(schema)
                .map_err(|_| ValidationError::TypeMismatch {
                    expected: format!("resolved ref {}", reference),
                    actual: "unresolved".to_string(),
                })?;
            validate_full(resolved, value, Some(registry), formats)
        }

        (SchemaKind::Named { schema, .. }, value) => {
            validate_full(schema, value, registry, formats)
        }

        (SchemaKind::Function { .. }, _) => Ok(()),

        (SchemaKind::Void, Value::Null) => Ok(()),

        (SchemaKind::Never, _) => Err(ValidationError::TypeMismatch {
            expected: "never".to_string(),
            actual: value_type(value),
        }),

        (SchemaKind::Any, _) => Ok(()),

        (SchemaKind::Unknown, _) => Ok(()),

        (SchemaKind::Undefined, Value::Null) => Ok(()),

        (SchemaKind::Recursive { schema: inner }, value) => {
            let mut temp_registry = registry.cloned().unwrap_or_default();
            if let Some(ref id) = schema.id {
                temp_registry.register(id, (**inner).clone());
            }
            validate_full(inner, value, Some(&temp_registry), formats)
        }

        (SchemaKind::Intersect { all_of }, value) => {
            for s in all_of {
                validate_full(s, value, registry, formats)?;
            }
            Ok(())
        }

        _ => Err(ValidationError::TypeMismatch {
            expected: schema.kind().to_string(),
            actual: value_type(value),
        }),
    }
}

fn check_numeric_bounds(
    value: f64,
    minimum: Option<f64>,
    maximum: Option<f64>,
) -> Result<(), ValidationError> {
    if let Some(min) = minimum {
        if value < min {
            return Err(ValidationError::BelowMinimum {
                value,
                minimum: min,
            });
        }
    }
    if let Some(max) = maximum {
        if value > max {
            return Err(ValidationError::AboveMaximum {
                value,
                maximum: max,
            });
        }
    }
    Ok(())
}

fn is_typed_array(value: &Value) -> bool {
    matches!(
        value,
        Value::Float32Array(_)
            | Value::Float64Array(_)
            | Value::Int32Array(_)
            | Value::Int64Array(_)
            | Value::UInt8Array(_)
    )
}

fn typed_array_len(value: &Value) -> usize {
    match value {
        Value::Float32Array(arr) => arr.len(),
        Value::Float64Array(arr) => arr.len(),
        Value::Int32Array(arr) => arr.len(),
        Value::Int64Array(arr) => arr.len(),
        Value::UInt8Array(arr) => arr.len(),
        _ => 0,
    }
}

fn value_type(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Int64(_) => "number".to_string(),
        Value::Float64(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Bytes(_) => "bytes".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
        Value::Float32Array(_) => "Float32Array".to_string(),
        Value::Float64Array(_) => "Float64Array".to_string(),
        Value::Int32Array(_) => "Int32Array".to_string(),
        Value::Int64Array(_) => "Int64Array".to_string(),
        Value::UInt8Array(_) => "UInt8Array".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;
    use crate::schema::SchemaKind;

    #[test]
    fn test_validate_primitives() {
        assert!(validate(&SchemaBuilder::null(), &Value::Null).is_ok());
        assert!(validate(&SchemaBuilder::bool(), &Value::Bool(true)).is_ok());
        assert!(validate(&SchemaBuilder::int64(), &Value::Int64(42)).is_ok());
        assert!(validate(
            &SchemaBuilder::string().build(),
            &Value::String("hello".to_string())
        )
        .is_ok());
        assert!(validate(&SchemaBuilder::float64(), &Value::Float64(3.14)).is_ok());
    }

    #[test]
    fn test_validate_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .optional_field("email", SchemaBuilder::string().build())
            .build();

        let valid = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("Alice".to_string()))
            .build();
        assert!(validate(&schema, &valid).is_ok());

        let with_email = Value::object()
            .field("id", Value::Int64(2))
            .field("name", Value::String("Bob".to_string()))
            .field("email", Value::String("bob@example.com".to_string()))
            .build();
        assert!(validate(&schema, &with_email).is_ok());

        let missing_required = Value::object()
            .field("name", Value::String("Charlie".to_string()))
            .build();
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

        assert!(validate(
            &schema,
            &Value::Array(vec![Value::Int64(1), Value::Int64(2)])
        )
        .is_ok());
        assert!(matches!(
            validate(&schema, &Value::Array(vec![])),
            Err(ValidationError::MinItems { .. })
        ));
        assert!(matches!(
            validate(
                &schema,
                &Value::Array(vec![
                    Value::Int64(1),
                    Value::Int64(2),
                    Value::Int64(3),
                    Value::Int64(4)
                ])
            ),
            Err(ValidationError::MaxItems { .. })
        ));
    }

    #[test]
    fn test_validate_union() {
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::string().build(),
            SchemaBuilder::int64(),
        ]);

        assert!(validate(&schema, &Value::String("hello".to_string())).is_ok());
        assert!(validate(&schema, &Value::Int64(42)).is_ok());
        assert!(matches!(
            validate(&schema, &Value::Bool(true)),
            Err(ValidationError::NoMatchingVariant)
        ));
    }

    #[test]
    fn test_validate_optional() {
        let schema = SchemaBuilder::optional(SchemaBuilder::string().build());

        assert!(validate(&schema, &Value::String("hello".to_string())).is_ok());
        assert!(validate(&schema, &Value::Null).is_ok());
    }

    #[test]
    fn test_validate_numeric_bounds() {
        let schema = Schema::new(SchemaKind::Int64 {
            minimum: Some(10),
            maximum: Some(100),
        });

        assert!(validate(&schema, &Value::Int64(50)).is_ok());
        assert!(matches!(
            validate(&schema, &Value::Int64(5)),
            Err(ValidationError::BelowMinimum { .. })
        ));
        assert!(matches!(
            validate(&schema, &Value::Int64(150)),
            Err(ValidationError::AboveMaximum { .. })
        ));
    }

    #[test]
    fn test_validate_string_length() {
        let schema = SchemaBuilder::string().min_length(2).max_length(10).build();

        assert!(validate(&schema, &Value::String("hello".to_string())).is_ok());
        assert!(matches!(
            validate(&schema, &Value::String("a".to_string())),
            Err(ValidationError::MinLength { .. })
        ));
        assert!(matches!(
            validate(&schema, &Value::String("this is too long".to_string())),
            Err(ValidationError::MaxLength { .. })
        ));
    }

    #[test]
    fn test_validate_int8_bounds() {
        let schema = Schema::new(SchemaKind::Int8 {
            minimum: None,
            maximum: None,
        });

        assert!(validate(&schema, &Value::Int64(0)).is_ok());
        assert!(validate(&schema, &Value::Int64(127)).is_ok());
        assert!(validate(&schema, &Value::Int64(-128)).is_ok());
        assert!(matches!(
            validate(&schema, &Value::Int64(128)),
            Err(ValidationError::TypeMismatch { .. })
        ));
        assert!(matches!(
            validate(&schema, &Value::Int64(-129)),
            Err(ValidationError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn test_validate_uint8_bounds() {
        let schema = Schema::new(SchemaKind::UInt8 {
            minimum: None,
            maximum: None,
        });

        assert!(validate(&schema, &Value::Int64(0)).is_ok());
        assert!(validate(&schema, &Value::Int64(255)).is_ok());
        assert!(matches!(
            validate(&schema, &Value::Int64(-1)),
            Err(ValidationError::TypeMismatch { .. })
        ));
        assert!(matches!(
            validate(&schema, &Value::Int64(256)),
            Err(ValidationError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn test_validate_uint64_negative() {
        let schema = Schema::new(SchemaKind::UInt64 {
            minimum: None,
            maximum: None,
        });

        assert!(validate(&schema, &Value::Int64(0)).is_ok());
        assert!(validate(&schema, &Value::Int64(i64::MAX)).is_ok());
        assert!(matches!(
            validate(&schema, &Value::Int64(-1)),
            Err(ValidationError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn test_validate_with_registry_ref() {
        use crate::registry::SchemaRegistry;

        let person = SchemaBuilder::object()
            .field("name", SchemaBuilder::string().build())
            .field("age", SchemaBuilder::int64())
            .named("Person");

        let mut registry = SchemaRegistry::new();
        registry.register("Person", person);

        let ref_schema = SchemaBuilder::r#ref("Person");

        let valid = Value::object()
            .field("name", Value::string("Alice"))
            .field("age", Value::int64(30))
            .build();

        assert!(validate_with_registry(&ref_schema, &valid, Some(&registry)).is_ok());

        let invalid = Value::object().field("name", Value::string("Bob")).build();

        assert!(validate_with_registry(&ref_schema, &invalid, Some(&registry)).is_err());
    }

    #[test]
    fn test_validate_ref_without_registry() {
        let ref_schema = SchemaBuilder::r#ref("Person");
        let value = Value::object().build();

        assert!(validate(&ref_schema, &value).is_err());
    }

    #[test]
    fn test_validate_function() {
        let schema = SchemaBuilder::function(
            vec![SchemaBuilder::int64()],
            SchemaBuilder::string().build(),
        );
        assert!(validate(&schema, &Value::Null).is_ok());
    }

    #[test]
    fn test_validate_void() {
        let schema = SchemaBuilder::void();
        assert!(validate(&schema, &Value::Null).is_ok());
        assert!(validate(&schema, &Value::Int64(42)).is_err());
    }

    #[test]
    fn test_validate_never() {
        let schema = SchemaBuilder::never();
        assert!(validate(&schema, &Value::Null).is_err());
        assert!(validate(&schema, &Value::Int64(42)).is_err());
    }

    #[test]
    fn test_validate_any() {
        let schema = SchemaBuilder::any();
        assert!(validate(&schema, &Value::Null).is_ok());
        assert!(validate(&schema, &Value::Int64(42)).is_ok());
        assert!(validate(&schema, &Value::String("hello".to_string())).is_ok());
    }

    #[test]
    fn test_validate_unknown() {
        let schema = SchemaBuilder::unknown();
        assert!(validate(&schema, &Value::Null).is_ok());
        assert!(validate(&schema, &Value::Int64(42)).is_ok());
    }

    #[test]
    fn test_validate_undefined() {
        let schema = SchemaBuilder::undefined();
        assert!(validate(&schema, &Value::Null).is_ok());
        assert!(validate(&schema, &Value::Int64(42)).is_err());
    }

    #[test]
    fn test_validate_recursive() {
        let schema = SchemaBuilder::recursive("JsonTree", |this| {
            SchemaBuilder::union(vec![
                SchemaBuilder::null(),
                SchemaBuilder::bool(),
                SchemaBuilder::int64(),
                SchemaBuilder::string().build(),
                SchemaBuilder::array(this).build(),
            ])
        });

        assert!(validate(&schema, &Value::Null).is_ok());
        assert!(validate(&schema, &Value::Bool(true)).is_ok());
        assert!(validate(&schema, &Value::Int64(42)).is_ok());
        assert!(validate(&schema, &Value::String("hello".to_string())).is_ok());
        assert!(validate(&schema, &Value::Array(vec![Value::Int64(1)])).is_ok());
        assert!(validate(
            &schema,
            &Value::Array(vec![Value::Array(vec![Value::Int64(1)])])
        )
        .is_ok());
    }

    #[test]
    fn test_validate_intersect() {
        let node = SchemaBuilder::object()
            .field("type", SchemaBuilder::string().build())
            .additional_properties(Some(SchemaBuilder::any()))
            .build();

        let literal_extra = SchemaBuilder::object()
            .field("value", SchemaBuilder::any())
            .additional_properties(Some(SchemaBuilder::any()))
            .build();

        let literal = SchemaBuilder::intersect(vec![node, literal_extra]);

        let value = Value::object()
            .field("type", Value::string("text"))
            .field("value", Value::Int64(42))
            .build();

        assert!(validate(&literal, &value).is_ok());

        let missing_field = Value::object().field("type", Value::string("text")).build();
        assert!(validate(&literal, &missing_field).is_err());
    }

    #[test]
    fn test_validate_format() {
        use crate::format::FormatRegistry;
        use crate::schema::StringFormat;

        let mut formats = FormatRegistry::new();
        formats.register("email", |s| s.contains('@'));

        let schema = SchemaBuilder::string().format(StringFormat::Email).build();

        let valid = Value::String("test@example.com".to_string());
        let invalid = Value::String("invalid-email".to_string());

        assert!(validate_with_format(&schema, &valid, None, Some(&formats)).is_ok());
        assert!(validate_with_format(&schema, &invalid, None, Some(&formats)).is_err());

        assert!(validate(&schema, &invalid).is_ok());
    }

    #[test]
    fn test_validate_unique_items() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64())
            .unique_items(true)
            .build();

        let unique = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(3)]);
        let duplicate = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(1)]);

        assert!(validate(&schema, &unique).is_ok());
        assert!(matches!(
            validate(&schema, &duplicate),
            Err(ValidationError::DuplicateItem)
        ));
    }
}
