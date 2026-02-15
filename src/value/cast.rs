//! Value coercion to match schemas.

use crate::error::CastError;
use crate::schema::{LiteralValue, Schema, SchemaKind};
use crate::value::Value;
use indexmap::IndexMap;

/// Coerce a value to conform to a schema.
///
/// Performs type conversions (string to int, int to bool, etc.),
/// clamps numeric values to bounds, and fills in missing object fields.
pub fn cast(schema: &Schema, value: &Value) -> Result<Value, CastError> {
    match (&schema.kind, value) {
        (SchemaKind::Null, Value::Null) => Ok(Value::Null),
        (SchemaKind::Null, _) => Ok(Value::Null),

        (SchemaKind::Bool, Value::Bool(b)) => Ok(Value::Bool(*b)),
        (SchemaKind::Bool, v) => coerce_to_bool(v),

        (SchemaKind::Int8 { minimum, maximum }, Value::Int64(n)) => {
            let val = clamp_i64_to_i8(*n);
            let min = minimum.unwrap_or(i8::MIN);
            let max = maximum.unwrap_or(i8::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }
        (SchemaKind::Int8 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = clamp_i64_to_i8(n);
            let min = minimum.unwrap_or(i8::MIN);
            let max = maximum.unwrap_or(i8::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::Int16 { minimum, maximum }, Value::Int64(n)) => {
            let val = clamp_i64_to_i16(*n);
            let min = minimum.unwrap_or(i16::MIN);
            let max = maximum.unwrap_or(i16::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }
        (SchemaKind::Int16 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = clamp_i64_to_i16(n);
            let min = minimum.unwrap_or(i16::MIN);
            let max = maximum.unwrap_or(i16::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::Int32 { minimum, maximum }, Value::Int64(n)) => {
            let val = clamp_i64_to_i32(*n);
            let min = minimum.unwrap_or(i32::MIN);
            let max = maximum.unwrap_or(i32::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }
        (SchemaKind::Int32 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = clamp_i64_to_i32(n);
            let min = minimum.unwrap_or(i32::MIN);
            let max = maximum.unwrap_or(i32::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::Int64 { minimum, maximum }, Value::Int64(n)) => {
            let min = minimum.unwrap_or(i64::MIN);
            let max = maximum.unwrap_or(i64::MAX);
            Ok(Value::Int64(clamp(*n, min, max)))
        }
        (SchemaKind::Int64 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let min = minimum.unwrap_or(i64::MIN);
            let max = maximum.unwrap_or(i64::MAX);
            Ok(Value::Int64(clamp(n, min, max)))
        }

        (SchemaKind::UInt8 { minimum, maximum }, Value::Int64(n)) => {
            let val = clamp_i64_to_u8(*n);
            let min = minimum.unwrap_or(u8::MIN);
            let max = maximum.unwrap_or(u8::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }
        (SchemaKind::UInt8 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = clamp_i64_to_u8(n);
            let min = minimum.unwrap_or(u8::MIN);
            let max = maximum.unwrap_or(u8::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::UInt16 { minimum, maximum }, Value::Int64(n)) => {
            let val = clamp_i64_to_u16(*n);
            let min = minimum.unwrap_or(u16::MIN);
            let max = maximum.unwrap_or(u16::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }
        (SchemaKind::UInt16 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = clamp_i64_to_u16(n);
            let min = minimum.unwrap_or(u16::MIN);
            let max = maximum.unwrap_or(u16::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::UInt32 { minimum, maximum }, Value::Int64(n)) => {
            let val = clamp_i64_to_u32(*n);
            let min = minimum.unwrap_or(u32::MIN);
            let max = maximum.unwrap_or(u32::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }
        (SchemaKind::UInt32 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = clamp_i64_to_u32(n);
            let min = minimum.unwrap_or(u32::MIN);
            let max = maximum.unwrap_or(u32::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::UInt64 { minimum, maximum }, Value::Int64(n)) => {
            let val = (*n).max(0);
            let min = minimum.unwrap_or(u64::MIN);
            let max = maximum.unwrap_or(u64::MAX);
            Ok(Value::Int64(clamp(val as u64, min, max) as i64))
        }
        (SchemaKind::UInt64 { minimum, maximum }, v) => {
            let n = coerce_to_i64(v)?;
            let val = n.max(0) as u64;
            let min = minimum.unwrap_or(u64::MIN);
            let max = maximum.unwrap_or(u64::MAX);
            Ok(Value::Int64(clamp(val, min, max) as i64))
        }

        (SchemaKind::Float32 { minimum, maximum }, Value::Float64(f)) => {
            let val = *f as f32;
            let min = minimum.unwrap_or(f32::MIN);
            let max = maximum.unwrap_or(f32::MAX);
            Ok(Value::Float64(clamp_f32(val, min, max) as f64))
        }
        (SchemaKind::Float32 { minimum, maximum }, Value::Int64(n)) => {
            let val = *n as f32;
            let min = minimum.unwrap_or(f32::MIN);
            let max = maximum.unwrap_or(f32::MAX);
            Ok(Value::Float64(clamp_f32(val, min, max) as f64))
        }
        (SchemaKind::Float32 { minimum, maximum }, v) => {
            let f = coerce_to_f64(v)?;
            let val = f as f32;
            let min = minimum.unwrap_or(f32::MIN);
            let max = maximum.unwrap_or(f32::MAX);
            Ok(Value::Float64(clamp_f32(val, min, max) as f64))
        }

        (SchemaKind::Float64 { minimum, maximum }, Value::Float64(f)) => {
            let min = minimum.unwrap_or(f64::MIN);
            let max = maximum.unwrap_or(f64::MAX);
            Ok(Value::Float64(clamp_f64(*f, min, max)))
        }
        (SchemaKind::Float64 { minimum, maximum }, Value::Int64(n)) => {
            let val = *n as f64;
            let min = minimum.unwrap_or(f64::MIN);
            let max = maximum.unwrap_or(f64::MAX);
            Ok(Value::Float64(clamp_f64(val, min, max)))
        }
        (SchemaKind::Float64 { minimum, maximum }, v) => {
            let f = coerce_to_f64(v)?;
            let min = minimum.unwrap_or(f64::MIN);
            let max = maximum.unwrap_or(f64::MAX);
            Ok(Value::Float64(clamp_f64(f, min, max)))
        }

        (SchemaKind::String { .. }, Value::String(s)) => Ok(Value::String(s.clone())),
        (SchemaKind::String { .. }, v) => Ok(Value::String(value_to_string(v))),

        (SchemaKind::Bytes { .. }, Value::Bytes(b)) => Ok(Value::Bytes(b.clone())),
        (SchemaKind::Bytes { .. }, Value::UInt8Array(b)) => Ok(Value::Bytes(b.clone())),
        (SchemaKind::Bytes { .. }, Value::String(s)) => Ok(Value::Bytes(s.as_bytes().to_vec())),
        (SchemaKind::Bytes { .. }, v) => Err(CastError::CannotCast(format!(
            "cannot cast {:?} to bytes",
            v.kind()
        ))),

        (
            SchemaKind::Array {
                items,
                min_items,
                max_items,
                ..
            },
            Value::Array(arr),
        ) => {
            let min = min_items.unwrap_or(0);
            let max = max_items.unwrap_or(usize::MAX);

            let mut result: Vec<Value> = arr
                .iter()
                .take(max)
                .map(|v| cast(items, v))
                .collect::<Result<Vec<_>, _>>()?;

            while result.len() < min {
                result.push(super::create::create(items).map_err(|e| {
                    CastError::CannotCast(format!("cannot create default item: {}", e))
                })?);
            }

            Ok(Value::Array(result))
        }
        (
            SchemaKind::Array {
                items, min_items, ..
            },
            v,
        ) => {
            let min = min_items.unwrap_or(0);
            let single = cast(items, v)?;
            let mut arr = vec![single];
            while arr.len() < min {
                arr.push(super::create::create(items).map_err(|e| {
                    CastError::CannotCast(format!("cannot create default item: {}", e))
                })?);
            }
            Ok(Value::Array(arr))
        }

        (
            SchemaKind::Object {
                properties,
                required,
                additional_properties,
            },
            Value::Object(map),
        ) => {
            let mut result = IndexMap::new();

            for field_name in required {
                if let Some(field_schema) = properties.get(field_name) {
                    if let Some(val) = map.get(field_name) {
                        result.insert(field_name.clone(), cast(field_schema, val)?);
                    } else {
                        result.insert(
                            field_name.clone(),
                            super::create::create(field_schema).map_err(|e| {
                                CastError::CannotCast(format!("cannot create default field: {}", e))
                            })?,
                        );
                    }
                }
            }

            if let Some(additional_schema) = additional_properties {
                for (key, val) in map {
                    if !properties.contains_key(key) {
                        result.insert(key.clone(), cast(additional_schema, val)?);
                    }
                }
            }

            Ok(Value::Object(result))
        }
        (
            SchemaKind::Object {
                properties,
                required,
                ..
            },
            _,
        ) => {
            let mut result = IndexMap::new();
            for field_name in required {
                if let Some(field_schema) = properties.get(field_name) {
                    result.insert(
                        field_name.clone(),
                        super::create::create(field_schema).map_err(|e| {
                            CastError::CannotCast(format!("cannot create default field: {}", e))
                        })?,
                    );
                }
            }
            Ok(Value::Object(result))
        }

        (SchemaKind::Tuple { items }, Value::Array(arr)) => {
            let mut result = Vec::with_capacity(items.len());
            for (i, item_schema) in items.iter().enumerate() {
                if let Some(val) = arr.get(i) {
                    result.push(cast(item_schema, val)?);
                } else {
                    result.push(super::create::create(item_schema).map_err(|e| {
                        CastError::CannotCast(format!("cannot create tuple element: {}", e))
                    })?);
                }
            }
            Ok(Value::Array(result))
        }
        (SchemaKind::Tuple { items }, _) => {
            let mut result = Vec::with_capacity(items.len());
            for item_schema in items {
                result.push(super::create::create(item_schema).map_err(|e| {
                    CastError::CannotCast(format!("cannot create tuple element: {}", e))
                })?);
            }
            Ok(Value::Array(result))
        }

        (SchemaKind::Union { any_of }, value) => {
            for variant in any_of {
                let casted = cast(variant, value)?;
                if super::check::check(variant, &casted) {
                    return Ok(casted);
                }
            }
            if let Some(first) = any_of.first() {
                return cast(first, value);
            }
            Err(CastError::CannotCast("empty union".to_string()))
        }

        (SchemaKind::Literal { value: lit }, val) => {
            let matches = match (lit, val) {
                (LiteralValue::Null, Value::Null) => true,
                (LiteralValue::Boolean(b), Value::Bool(v)) => *b == *v,
                (LiteralValue::Number(n), Value::Int64(v)) => *n == *v,
                (LiteralValue::Float(f), Value::Float64(v)) => (f - v).abs() < f64::EPSILON,
                (LiteralValue::String(s), Value::String(v)) => s == v,
                _ => false,
            };
            if matches {
                Ok(val.clone())
            } else {
                match lit {
                    LiteralValue::Null => Ok(Value::Null),
                    LiteralValue::Boolean(b) => Ok(Value::Bool(*b)),
                    LiteralValue::Number(n) => Ok(Value::Int64(*n)),
                    LiteralValue::Float(f) => Ok(Value::Float64(*f)),
                    LiteralValue::String(s) => Ok(Value::String(s.clone())),
                }
            }
        }

        (SchemaKind::Enum { values }, Value::String(s)) => {
            if values.contains(s) {
                Ok(Value::String(s.clone()))
            } else if let Some(first) = values.first() {
                Ok(Value::String(first.clone()))
            } else {
                Err(CastError::CannotCast("empty enum".to_string()))
            }
        }
        (SchemaKind::Enum { values }, _) => {
            if let Some(first) = values.first() {
                Ok(Value::String(first.clone()))
            } else {
                Err(CastError::CannotCast("empty enum".to_string()))
            }
        }

        (SchemaKind::Ref { reference }, _) => Err(CastError::CannotCast(format!(
            "unresolved ref: {}",
            reference
        ))),

        (SchemaKind::Named { schema, .. }, value) => cast(schema, value),

        (SchemaKind::Function { .. }, val) => Ok(val.clone()),
        (SchemaKind::Void, _) => Ok(Value::Null),
        (SchemaKind::Never, _) => Err(CastError::CannotCast("never type".to_string())),
        (SchemaKind::Any, val) => Ok(val.clone()),
        (SchemaKind::Unknown, val) => Ok(val.clone()),
        (SchemaKind::Undefined, _) => Ok(Value::Null),
    }
}

fn coerce_to_bool(value: &Value) -> Result<Value, CastError> {
    match value {
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Int64(n) => Ok(Value::Bool(*n != 0)),
        Value::Float64(f) => Ok(Value::Bool(*f != 0.0)),
        Value::String(s) => {
            let lower = s.to_lowercase();
            Ok(Value::Bool(
                lower == "true" || lower == "1" || lower == "yes",
            ))
        }
        Value::Null => Ok(Value::Bool(false)),
        _ => Err(CastError::CannotCast(format!(
            "cannot cast {:?} to bool",
            value.kind()
        ))),
    }
}

fn coerce_to_i64(value: &Value) -> Result<i64, CastError> {
    match value {
        Value::Int64(n) => Ok(*n),
        Value::Float64(f) => Ok(*f as i64),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        Value::String(s) => s
            .parse::<i64>()
            .or_else(|_| s.parse::<f64>().map(|f| f as i64))
            .map_err(|_| CastError::CannotCast(format!("cannot parse '{}' as number", s))),
        _ => Err(CastError::CannotCast(format!(
            "cannot cast {:?} to integer",
            value.kind()
        ))),
    }
}

fn coerce_to_f64(value: &Value) -> Result<f64, CastError> {
    match value {
        Value::Float64(f) => Ok(*f),
        Value::Int64(n) => Ok(*n as f64),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::String(s) => s
            .parse::<f64>()
            .map_err(|_| CastError::CannotCast(format!("cannot parse '{}' as float", s))),
        _ => Err(CastError::CannotCast(format!(
            "cannot cast {:?} to float",
            value.kind()
        ))),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int64(n) => n.to_string(),
        Value::Float64(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(map) => {
            let items: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        _ => value.kind().to_string(),
    }
}

fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn clamp_i64_to_i8(n: i64) -> i8 {
    clamp(n, i8::MIN as i64, i8::MAX as i64) as i8
}

fn clamp_i64_to_i16(n: i64) -> i16 {
    clamp(n, i16::MIN as i64, i16::MAX as i64) as i16
}

fn clamp_i64_to_i32(n: i64) -> i32 {
    clamp(n, i32::MIN as i64, i32::MAX as i64) as i32
}

fn clamp_i64_to_u8(n: i64) -> u8 {
    clamp(n, 0, u8::MAX as i64) as u8
}

fn clamp_i64_to_u16(n: i64) -> u16 {
    clamp(n, 0, u16::MAX as i64) as u16
}

fn clamp_i64_to_u32(n: i64) -> u32 {
    clamp(n, 0, u32::MAX as i64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;
    use crate::schema::{Schema, SchemaKind};

    #[test]
    fn test_cast_null() {
        assert_eq!(
            cast(&Schema::new(SchemaKind::Null), &Value::Null).unwrap(),
            Value::Null
        );
        assert_eq!(
            cast(&Schema::new(SchemaKind::Null), &Value::Int64(42)).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn test_cast_bool() {
        assert_eq!(
            cast(&Schema::new(SchemaKind::Bool), &Value::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            cast(&Schema::new(SchemaKind::Bool), &Value::Int64(1)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            cast(&Schema::new(SchemaKind::Bool), &Value::Int64(0)).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            cast(
                &Schema::new(SchemaKind::Bool),
                &Value::String("true".to_string())
            )
            .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            cast(&Schema::new(SchemaKind::Bool), &Value::Null).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_cast_int_with_bounds() {
        let schema = Schema::new(SchemaKind::Int64 {
            minimum: Some(10),
            maximum: Some(20),
        });
        assert_eq!(cast(&schema, &Value::Int64(15)).unwrap(), Value::Int64(15));
        assert_eq!(cast(&schema, &Value::Int64(5)).unwrap(), Value::Int64(10));
        assert_eq!(cast(&schema, &Value::Int64(25)).unwrap(), Value::Int64(20));
    }

    #[test]
    fn test_cast_string_to_int() {
        assert_eq!(
            cast(&SchemaBuilder::int64(), &Value::String("42".to_string())).unwrap(),
            Value::Int64(42)
        );
        assert!(cast(
            &SchemaBuilder::int64(),
            &Value::String("not a number".to_string())
        )
        .is_err());
    }

    #[test]
    fn test_cast_to_string() {
        assert_eq!(
            cast(&SchemaBuilder::string().build(), &Value::Int64(42)).unwrap(),
            Value::String("42".to_string())
        );
        assert_eq!(
            cast(&SchemaBuilder::string().build(), &Value::Bool(true)).unwrap(),
            Value::String("true".to_string())
        );
    }

    #[test]
    fn test_cast_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64())
            .min_items(2)
            .max_items(3)
            .build();

        let arr = cast(&schema, &Value::Array(vec![Value::Int64(1)])).unwrap();
        if let Value::Array(items) = arr {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Int64(1));
            assert_eq!(items[1], Value::Int64(0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_cast_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build();

        let input = Value::object()
            .field("id", Value::String("42".to_string()))
            .build();

        let result = cast(&schema, &input).unwrap();
        if let Value::Object(map) = result {
            assert_eq!(map.get("id"), Some(&Value::Int64(42)));
            assert_eq!(map.get("name"), Some(&Value::String(String::new())));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_cast_tuple() {
        let schema = Schema::new(SchemaKind::Tuple {
            items: vec![SchemaBuilder::int64(), SchemaBuilder::string().build()],
        });

        let input = Value::Array(vec![Value::String("42".to_string())]);
        let result = cast(&schema, &input).unwrap();
        assert_eq!(
            result,
            Value::Array(vec![Value::Int64(42), Value::String(String::new())])
        );
    }

    #[test]
    fn test_cast_union() {
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::int64(),
            SchemaBuilder::string().build(),
        ]);

        let result = cast(&schema, &Value::Int64(42)).unwrap();
        assert_eq!(result, Value::Int64(42));

        let schema_string_first = SchemaBuilder::union(vec![
            SchemaBuilder::string().build(),
            SchemaBuilder::int64(),
        ]);

        let result2 = cast(&schema_string_first, &Value::Int64(42)).unwrap();
        assert_eq!(result2, Value::String("42".to_string()));
    }

    #[test]
    fn test_cast_literal() {
        let schema = Schema::new(SchemaKind::Literal {
            value: LiteralValue::String("hello".to_string()),
        });

        assert_eq!(
            cast(&schema, &Value::String("hello".to_string())).unwrap(),
            Value::String("hello".to_string())
        );
        assert_eq!(
            cast(&schema, &Value::String("world".to_string())).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_cast_enum() {
        let schema = Schema::new(SchemaKind::Enum {
            values: vec!["one".to_string(), "two".to_string()],
        });

        assert_eq!(
            cast(&schema, &Value::String("one".to_string())).unwrap(),
            Value::String("one".to_string())
        );
        assert_eq!(
            cast(&schema, &Value::String("other".to_string())).unwrap(),
            Value::String("one".to_string())
        );
    }

    #[test]
    fn test_cast_float_truncation() {
        assert_eq!(
            cast(&SchemaBuilder::int64(), &Value::Float64(3.7)).unwrap(),
            Value::Int64(3)
        );
    }
}
