//! Value type and operations.
//!
//! This module provides the [`Value`] enum and operations for working with
//! schema-constrained values.

pub mod cast;
pub mod check;
pub mod clean;
pub mod clone;
pub mod create;
pub mod delta;
pub mod equal;
pub mod hash;
pub mod mutate;
pub mod patch;
pub mod pointer;

#[cfg(feature = "fake")]
#[doc(hidden)]
pub mod fake;

use crate::error::ParseError;
use crate::schema::{LiteralValue, Schema, SchemaKind};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub use cast::cast;
pub use check::{check, check_with_errors};
pub use clean::clean;
pub use clone::clone;
pub use create::create;
pub use delta::{delta, diff_summary, Delta, Edit};
pub use equal::equal;
pub use hash::hash_fnv1a;
pub use mutate::mutate;
pub use patch::patch;
pub use pointer::{delete_pointer, get_pointer, get_pointer_mut, has_pointer, set_pointer};

#[cfg(feature = "fake")]
pub use fake::{fake, fake_with_context, FakeContext};

/// A dynamically-typed value with schema-aware operations.
///
/// Supports JSON-compatible types plus typed arrays for binary/tensor data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// 64-bit signed integer (used for all integer types).
    Int64(i64),
    /// 64-bit floating point (used for all float types).
    Float64(f64),
    /// String value.
    String(String),
    /// Byte array.
    Bytes(Vec<u8>),
    /// Array of values.
    Array(Vec<Value>),
    /// Object with named fields.
    Object(IndexMap<String, Value>),
    /// Typed array of 32-bit floats.
    Float32Array(Vec<f32>),
    /// Typed array of 64-bit floats.
    Float64Array(Vec<f64>),
    /// Typed array of 32-bit integers.
    Int32Array(Vec<i32>),
    /// Typed array of 64-bit integers.
    Int64Array(Vec<i64>),
    /// Typed array of 8-bit unsigned integers.
    UInt8Array(Vec<u8>),
}

impl Eq for Value {}

impl Value {
    /// Create a null value.
    pub fn null() -> Self {
        Value::Null
    }

    /// Create a boolean value.
    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }

    /// Create a 64-bit integer value.
    pub fn int64(n: i64) -> Self {
        Value::Int64(n)
    }

    /// Create a 64-bit floating point value.
    pub fn float64(f: f64) -> Self {
        Value::Float64(f)
    }

    /// Create a string value.
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(s.into())
    }

    /// Create a byte array value.
    pub fn bytes(b: Vec<u8>) -> Self {
        Value::Bytes(b)
    }

    /// Create an array value.
    pub fn array(items: Vec<Value>) -> Self {
        Value::Array(items)
    }

    /// Create an object builder.
    pub fn object() -> ObjectBuilder {
        ObjectBuilder::new()
    }

    /// Returns the kind name of this value.
    pub fn kind(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Int64(_) => "Int64",
            Value::Float64(_) => "Float64",
            Value::String(_) => "String",
            Value::Bytes(_) => "Bytes",
            Value::Array(_) => "Array",
            Value::Object(_) => "Object",
            Value::Float32Array(_) => "Float32Array",
            Value::Float64Array(_) => "Float64Array",
            Value::Int32Array(_) => "Int32Array",
            Value::Int64Array(_) => "Int64Array",
            Value::UInt8Array(_) => "UInt8Array",
        }
    }

    /// Returns true if this is a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns the boolean value if this is a Bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the i64 value if this is an Int64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int64(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the f64 value if this is a Float64 or Int64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float64(f) => Some(*f),
            Value::Int64(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Returns the string value if this is a String.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the byte slice if this is Bytes or UInt8Array.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            Value::UInt8Array(b) => Some(b),
            _ => None,
        }
    }

    /// Returns the array if this is an Array.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Returns the object if this is an Object.
    pub fn as_object(&self) -> Option<&IndexMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Parses a JSON value according to a schema.
    pub fn from_json(json: serde_json::Value, schema: &Schema) -> Result<Self, ParseError> {
        match (json, &schema.kind) {
            (serde_json::Value::Null, SchemaKind::Null) => Ok(Value::Null),
            (serde_json::Value::Null, SchemaKind::Union { any_of }) => {
                if any_of.iter().any(|s| matches!(&s.kind, SchemaKind::Null)) {
                    Ok(Value::Null)
                } else {
                    Err(ParseError::TypeMismatch {
                        expected: schema.kind().to_string(),
                        got: "Null".to_string(),
                    })
                }
            }
            (serde_json::Value::Bool(b), SchemaKind::Bool) => Ok(Value::Bool(b)),
            (serde_json::Value::Number(n), SchemaKind::Int64 { .. }) => n
                .as_i64()
                .map(Value::Int64)
                .ok_or_else(|| ParseError::TypeMismatch {
                    expected: "Int64".to_string(),
                    got: "Number".to_string(),
                }),
            (serde_json::Value::Number(n), SchemaKind::Float64 { .. }) => n
                .as_f64()
                .map(Value::Float64)
                .ok_or_else(|| ParseError::TypeMismatch {
                    expected: "Float64".to_string(),
                    got: "Number".to_string(),
                }),
            (serde_json::Value::String(s), SchemaKind::String { .. }) => Ok(Value::String(s)),
            (serde_json::Value::Array(arr), SchemaKind::Array { items, .. }) => {
                let values: Result<Vec<Value>, ParseError> = arr
                    .into_iter()
                    .map(|v| Value::from_json(v, items))
                    .collect();
                Ok(Value::Array(values?))
            }
            (
                serde_json::Value::Object(map),
                SchemaKind::Object {
                    properties,
                    required,
                    ..
                },
            ) => {
                let mut result = IndexMap::new();
                for (name, schema) in properties {
                    if let Some(json_val) = map.get(name) {
                        result.insert(name.clone(), Value::from_json(json_val.clone(), schema)?);
                    } else if required.contains(name) {
                        return Err(ParseError::MissingField {
                            field: name.clone(),
                        });
                    }
                }
                Ok(Value::Object(result))
            }
            (serde_json::Value::Array(arr), SchemaKind::Tuple { items }) => {
                if arr.len() != items.len() {
                    return Err(ParseError::InvalidLength {
                        expected: items.len(),
                        got: arr.len(),
                    });
                }
                let values: Result<Vec<Value>, ParseError> = arr
                    .into_iter()
                    .zip(items.iter())
                    .map(|(v, s)| Value::from_json(v, s))
                    .collect();
                Ok(Value::Array(values?))
            }
            (value, SchemaKind::Union { any_of }) => {
                for variant in any_of {
                    if let Ok(parsed) = Value::from_json(value.clone(), variant) {
                        return Ok(parsed);
                    }
                }
                Err(ParseError::NoMatchingVariant)
            }
            (value, SchemaKind::Literal { value: lit }) => {
                let matches = match (&value, lit) {
                    (serde_json::Value::Null, LiteralValue::Null) => true,
                    (serde_json::Value::Bool(b), LiteralValue::Boolean(lit_b)) => *b == *lit_b,
                    (serde_json::Value::Number(n), LiteralValue::Number(lit_n)) => {
                        n.as_i64() == Some(*lit_n)
                    }
                    (serde_json::Value::Number(n), LiteralValue::Float(lit_f)) => {
                        n.as_f64() == Some(*lit_f)
                    }
                    (serde_json::Value::String(s), LiteralValue::String(lit_s)) => s == lit_s,
                    _ => false,
                };
                if matches {
                    match lit {
                        LiteralValue::Null => Ok(Value::Null),
                        LiteralValue::Boolean(b) => Ok(Value::Bool(*b)),
                        LiteralValue::Number(n) => Ok(Value::Int64(*n)),
                        LiteralValue::Float(f) => Ok(Value::Float64(*f)),
                        LiteralValue::String(s) => Ok(Value::String(s.clone())),
                    }
                } else {
                    Err(ParseError::LiteralMismatch)
                }
            }
            (serde_json::Value::String(s), SchemaKind::Enum { values }) => {
                if values.contains(&s) {
                    Ok(Value::String(s))
                } else {
                    Err(ParseError::EnumMismatch {
                        allowed: values.clone(),
                        got: s,
                    })
                }
            }
            (serde_json::Value::Null, SchemaKind::Void) => Ok(Value::Null),
            (serde_json::Value::Null, SchemaKind::Undefined) => Ok(Value::Null),
            (value, SchemaKind::Any) | (value, SchemaKind::Unknown) => Ok(value_to_untyped(value)),
            (value, schema_kind) => Err(ParseError::TypeMismatch {
                expected: schema_kind.kind_name().to_string(),
                got: match value {
                    serde_json::Value::Null => "Null".to_string(),
                    serde_json::Value::Bool(_) => "Bool".to_string(),
                    serde_json::Value::Number(_) => "Number".to_string(),
                    serde_json::Value::String(_) => "String".to_string(),
                    serde_json::Value::Array(_) => "Array".to_string(),
                    serde_json::Value::Object(_) => "Object".to_string(),
                },
            }),
        }
    }

    /// Converts this value to a JSON value.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int64(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
            Value::Float64(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Bytes(b) => {
                let encoded = base64_encode(b);
                serde_json::Value::String(encoded)
            }
            Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect())
            }
            Value::Object(map) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in map {
                    obj.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(obj)
            }
            Value::Float32Array(arr) => serde_json::Value::Array(
                arr.iter()
                    .map(|f| {
                        serde_json::Number::from_f64(*f as f64)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    })
                    .collect(),
            ),
            Value::Float64Array(arr) => serde_json::Value::Array(
                arr.iter()
                    .map(|f| {
                        serde_json::Number::from_f64(*f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    })
                    .collect(),
            ),
            Value::Int32Array(arr) => serde_json::Value::Array(
                arr.iter()
                    .map(|n| serde_json::Value::Number(serde_json::Number::from(*n)))
                    .collect(),
            ),
            Value::Int64Array(arr) => serde_json::Value::Array(
                arr.iter()
                    .map(|n| serde_json::Value::Number(serde_json::Number::from(*n)))
                    .collect(),
            ),
            Value::UInt8Array(arr) => {
                let encoded = base64_encode(arr);
                serde_json::Value::String(encoded)
            }
        }
    }

    /// Returns this value as a byte slice, converting typed arrays.
    pub fn as_bytes_ref(&self) -> Cow<'_, [u8]> {
        match self {
            Value::Bytes(b) => Cow::Borrowed(b),
            Value::UInt8Array(b) => Cow::Borrowed(b),
            Value::String(s) => Cow::Owned(s.as_bytes().to_vec()),
            Value::Int64Array(arr) => {
                let bytes: Vec<u8> = arr.iter().flat_map(|n| n.to_le_bytes()).collect();
                Cow::Owned(bytes)
            }
            Value::Int32Array(arr) => {
                let bytes: Vec<u8> = arr.iter().flat_map(|n| n.to_le_bytes()).collect();
                Cow::Owned(bytes)
            }
            Value::Float32Array(arr) => {
                let bytes: Vec<u8> = arr.iter().flat_map(|f| f.to_le_bytes()).collect();
                Cow::Owned(bytes)
            }
            Value::Float64Array(arr) => {
                let bytes: Vec<u8> = arr.iter().flat_map(|f| f.to_le_bytes()).collect();
                Cow::Owned(bytes)
            }
            _ => Cow::Owned(vec![]),
        }
    }
}

fn value_to_untyped(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(Value::Int64)
            .or_else(|| n.as_f64().map(Value::Float64))
            .unwrap_or(Value::Null),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(value_to_untyped).collect())
        }
        serde_json::Value::Object(map) => {
            let mut result = IndexMap::new();
            for (k, v) in map {
                result.insert(k, value_to_untyped(v));
            }
            Value::Object(result)
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.encode(data)
}

/// Builder for constructing object values.
pub struct ObjectBuilder {
    properties: IndexMap<String, Value>,
}

impl ObjectBuilder {
    /// Creates a new object builder.
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
        }
    }

    /// Adds a field to the object.
    pub fn field(mut self, name: &str, value: Value) -> Self {
        self.properties.insert(name.to_string(), value);
        self
    }

    /// Builds the object value.
    pub fn build(self) -> Value {
        Value::Object(self.properties)
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int64(n)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float64(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Self {
        Value::Array(items.into_iter().map(|v| v.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SchemaBuilder;

    #[test]
    fn test_value_to_json() {
        let value = Value::object()
            .field("id", Value::Int64(42))
            .field("name", Value::String("test".to_string()))
            .build();

        let json = value.to_json();
        assert_eq!(json["id"], 42);
        assert_eq!(json["name"], "test");
    }

    #[test]
    fn test_value_from_json() {
        let json = serde_json::json!({"id": 42, "name": "test"});
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build();

        let value = Value::from_json(json, &schema).unwrap();
        assert!(matches!(value.as_object(), Some(_)));
    }

    #[test]
    fn test_missing_required_field() {
        let json = serde_json::json!({"id": 42});
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build();

        let result = Value::from_json(json, &schema);
        assert!(matches!(result, Err(ParseError::MissingField { .. })));
    }

    #[test]
    fn test_from_json_type_mismatch() {
        let json = serde_json::json!("not a number");
        let schema = SchemaBuilder::int64();
        let result = Value::from_json(json, &schema);
        assert!(matches!(result, Err(ParseError::TypeMismatch { .. })));
    }

    #[test]
    fn test_from_json_tuple_length_mismatch() {
        let json = serde_json::json!([1, 2]);
        let schema = SchemaBuilder::tuple(vec![
            SchemaBuilder::int64(),
            SchemaBuilder::int64(),
            SchemaBuilder::int64(),
        ]);
        let result = Value::from_json(json, &schema);
        assert!(matches!(result, Err(ParseError::InvalidLength { .. })));
    }

    #[test]
    fn test_from_json_union_mismatch() {
        let json = serde_json::Value::Null;
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::int64(),
            SchemaBuilder::string().build(),
        ]);
        let result = Value::from_json(json, &schema);
        assert!(matches!(result, Err(ParseError::TypeMismatch { .. })));
    }

    #[test]
    fn test_from_json_literal_mismatch() {
        let json = serde_json::json!("wrong");
        let schema = SchemaBuilder::literal("correct");
        let result = Value::from_json(json, &schema);
        assert!(matches!(result, Err(ParseError::LiteralMismatch)));
    }

    #[test]
    fn test_to_json_typed_arrays() {
        let float32 = Value::Float32Array(vec![1.0, 2.0, 3.0]);
        let float64 = Value::Float64Array(vec![1.0, 2.0, 3.0]);
        let int32 = Value::Int32Array(vec![1, 2, 3]);
        let int64 = Value::Int64Array(vec![1, 2, 3]);
        let uint8 = Value::UInt8Array(vec![1, 2, 3]);
        let bytes = Value::Bytes(vec![1, 2, 3]);

        assert!(float32.to_json().is_array());
        assert!(float64.to_json().is_array());
        assert!(int32.to_json().is_array());
        assert!(int64.to_json().is_array());
        assert!(uint8.to_json().is_string());
        assert!(bytes.to_json().is_string());
    }

    #[test]
    fn test_as_bytes_ref_conversions() {
        let bytes = Value::Bytes(vec![1, 2, 3]);
        let uint8 = Value::UInt8Array(vec![1, 2, 3]);
        let string = Value::String("abc".to_string());
        let int32 = Value::Int32Array(vec![1, 2, 3]);
        let int64 = Value::Int64Array(vec![1, 2, 3]);
        let float32 = Value::Float32Array(vec![1.0, 2.0, 3.0]);
        let float64 = Value::Float64Array(vec![1.0, 2.0, 3.0]);

        assert_eq!(bytes.as_bytes_ref().len(), 3);
        assert_eq!(uint8.as_bytes_ref().len(), 3);
        assert_eq!(string.as_bytes_ref().len(), 3);
        assert_eq!(int32.as_bytes_ref().len(), 12);
        assert_eq!(int64.as_bytes_ref().len(), 24);
        assert_eq!(float32.as_bytes_ref().len(), 12);
        assert_eq!(float64.as_bytes_ref().len(), 24);
    }

    #[test]
    fn test_value_kind() {
        assert_eq!(Value::Null.kind(), "Null");
        assert_eq!(Value::Bool(true).kind(), "Bool");
        assert_eq!(Value::Int64(42).kind(), "Int64");
        assert_eq!(Value::Float64(3.14).kind(), "Float64");
        assert_eq!(Value::String("test".to_string()).kind(), "String");
        assert_eq!(Value::Bytes(vec![]).kind(), "Bytes");
        assert_eq!(Value::Array(vec![]).kind(), "Array");
        assert_eq!(Value::Object(IndexMap::new()).kind(), "Object");
    }

    #[test]
    fn test_value_accessors() {
        assert!(Value::Null.is_null());
        assert!(!Value::Bool(true).is_null());

        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int64(42).as_bool(), None);

        assert_eq!(Value::Int64(42).as_i64(), Some(42));
        assert_eq!(Value::Bool(true).as_i64(), None);

        assert_eq!(Value::Float64(3.14).as_f64(), Some(3.14));
        assert_eq!(Value::Int64(42).as_f64(), Some(42.0));
        assert_eq!(Value::Bool(true).as_f64(), None);

        assert_eq!(Value::String("test".to_string()).as_str(), Some("test"));
        assert_eq!(Value::Int64(42).as_str(), None);

        assert_eq!(Value::Bytes(vec![1, 2, 3]).as_bytes(), Some(&[1, 2, 3][..]));
        assert_eq!(
            Value::UInt8Array(vec![1, 2, 3]).as_bytes(),
            Some(&[1, 2, 3][..])
        );
        assert_eq!(Value::Int64(42).as_bytes(), None);

        assert!(Value::Array(vec![]).as_array().is_some());
        assert!(Value::Object(IndexMap::new()).as_object().is_some());
    }
}
