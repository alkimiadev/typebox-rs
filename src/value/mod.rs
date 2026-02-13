pub mod cast;
pub mod check;
pub mod clean;
pub mod clone;
pub mod create;
pub mod delta;
pub mod equal;
pub mod patch;

#[cfg(feature = "fake")]
pub mod fake;

use crate::error::ParseError;
use crate::schema::{LiteralValue, Schema};
use indexmap::IndexMap;
use std::borrow::Cow;

pub use cast::cast;
pub use check::{check, check_with_errors};
pub use clean::clean;
pub use clone::clone;
pub use create::create;
pub use delta::{delta, diff_summary, Delta, Edit};
pub use equal::equal;
pub use patch::patch;

#[cfg(feature = "fake")]
pub use fake::{fake, fake_with_context, FakeContext};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int64(i64),
    Float64(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
    Float32Array(Vec<f32>),
    Float64Array(Vec<f64>),
    Int32Array(Vec<i32>),
    Int64Array(Vec<i64>),
    UInt8Array(Vec<u8>),
}

impl Value {
    pub fn null() -> Self {
        Value::Null
    }

    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }

    pub fn int64(n: i64) -> Self {
        Value::Int64(n)
    }

    pub fn float64(f: f64) -> Self {
        Value::Float64(f)
    }

    pub fn string(s: impl Into<String>) -> Self {
        Value::String(s.into())
    }

    pub fn bytes(b: Vec<u8>) -> Self {
        Value::Bytes(b)
    }

    pub fn array(items: Vec<Value>) -> Self {
        Value::Array(items)
    }

    pub fn object() -> ObjectBuilder {
        ObjectBuilder::new()
    }

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

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int64(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float64(f) => Some(*f),
            Value::Int64(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            Value::UInt8Array(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&IndexMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn from_json(json: serde_json::Value, schema: &Schema) -> Result<Self, ParseError> {
        match (json, schema) {
            (serde_json::Value::Null, Schema::Null) => Ok(Value::Null),
            (serde_json::Value::Null, Schema::Union { any_of }) => {
                if any_of.iter().any(|s| matches!(s, Schema::Null)) {
                    Ok(Value::Null)
                } else {
                    Err(ParseError::TypeMismatch {
                        expected: schema.kind().to_string(),
                        got: "Null".to_string(),
                    })
                }
            }
            (serde_json::Value::Bool(b), Schema::Bool) => Ok(Value::Bool(b)),
            (serde_json::Value::Number(n), Schema::Int64 { .. }) => n
                .as_i64()
                .map(Value::Int64)
                .ok_or_else(|| ParseError::TypeMismatch {
                    expected: "Int64".to_string(),
                    got: "Number".to_string(),
                }),
            (serde_json::Value::Number(n), Schema::Float64 { .. }) => n
                .as_f64()
                .map(Value::Float64)
                .ok_or_else(|| ParseError::TypeMismatch {
                    expected: "Float64".to_string(),
                    got: "Number".to_string(),
                }),
            (serde_json::Value::String(s), Schema::String { .. }) => Ok(Value::String(s)),
            (serde_json::Value::Array(arr), Schema::Array { items, .. }) => {
                let values: Result<Vec<Value>, ParseError> = arr
                    .into_iter()
                    .map(|v| Value::from_json(v, items))
                    .collect();
                Ok(Value::Array(values?))
            }
            (
                serde_json::Value::Object(map),
                Schema::Object {
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
            (serde_json::Value::Array(arr), Schema::Tuple { items }) => {
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
            (value, Schema::Union { any_of }) => {
                for variant in any_of {
                    if let Ok(parsed) = Value::from_json(value.clone(), variant) {
                        return Ok(parsed);
                    }
                }
                Err(ParseError::NoMatchingVariant)
            }
            (value, Schema::Literal { value: lit }) => {
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
            (serde_json::Value::String(s), Schema::Enum { values }) => {
                if values.contains(&s) {
                    Ok(Value::String(s))
                } else {
                    Err(ParseError::EnumMismatch {
                        allowed: values.clone(),
                        got: s,
                    })
                }
            }
            (value, schema) => Err(ParseError::TypeMismatch {
                expected: schema.kind().to_string(),
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

fn base64_encode(data: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.encode(data)
}

pub struct ObjectBuilder {
    properties: IndexMap<String, Value>,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
        }
    }

    pub fn field(mut self, name: &str, value: Value) -> Self {
        self.properties.insert(name.to_string(), value);
        self
    }

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
}
