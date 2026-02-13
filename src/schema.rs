use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// JSON Schema type definition.
///
/// Use [`SchemaBuilder`](crate::SchemaBuilder) for a fluent API to construct schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Schema {
    /// Null type.
    Null,
    /// Boolean type.
    Bool,

    Int8 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<i8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<i8>,
    },
    Int16 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<i16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<i16>,
    },
    Int32 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<i32>,
    },
    Int64 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<i64>,
    },
    UInt8 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<u8>,
    },
    UInt16 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<u16>,
    },
    UInt32 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<u32>,
    },
    UInt64 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<u64>,
    },
    Float32 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<f32>,
    },
    Float64 {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        minimum: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        maximum: Option<f64>,
    },

    String {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<StringFormat>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },

    Bytes {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },

    Array {
        items: Box<Schema>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_items: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_items: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unique_items: Option<bool>,
    },

    Object {
        #[serde(default)]
        properties: IndexMap<String, Schema>,
        #[serde(default)]
        required: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_properties: Option<Box<Schema>>,
    },

    Tuple {
        items: Vec<Schema>,
    },

    Union {
        any_of: Vec<Schema>,
    },

    Literal {
        value: LiteralValue,
    },

    Enum {
        values: Vec<String>,
    },

    Ref {
        #[serde(rename = "$ref")]
        reference: String,
    },

    Named {
        name: String,
        schema: Box<Schema>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StringFormat {
    Email,
    Uuid,
    Uri,
    DateTime,
    Date,
    Time,
    Hostname,
    Ipv4,
    Ipv6,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LiteralValue {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl Schema {
    pub fn kind(&self) -> &'static str {
        match self {
            Schema::Null => "Null",
            Schema::Bool => "Bool",
            Schema::Int8 { .. } => "Int8",
            Schema::Int16 { .. } => "Int16",
            Schema::Int32 { .. } => "Int32",
            Schema::Int64 { .. } => "Int64",
            Schema::UInt8 { .. } => "UInt8",
            Schema::UInt16 { .. } => "UInt16",
            Schema::UInt32 { .. } => "UInt32",
            Schema::UInt64 { .. } => "UInt64",
            Schema::Float32 { .. } => "Float32",
            Schema::Float64 { .. } => "Float64",
            Schema::String { .. } => "String",
            Schema::Bytes { .. } => "Bytes",
            Schema::Array { .. } => "Array",
            Schema::Object { .. } => "Object",
            Schema::Tuple { .. } => "Tuple",
            Schema::Union { .. } => "Union",
            Schema::Literal { .. } => "Literal",
            Schema::Enum { .. } => "Enum",
            Schema::Ref { .. } => "Ref",
            Schema::Named { .. } => "Named",
        }
    }

    pub fn is_optional_in(&self, parent: &Schema) -> bool {
        if let Schema::Object { required, .. } = parent {
            match self {
                Schema::Named { name, .. } => !required.contains(name),
                _ => false,
            }
        } else {
            false
        }
    }
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Null => write!(f, "null"),
            Schema::Bool => write!(f, "boolean"),
            Schema::Int8 { .. } => write!(f, "int8"),
            Schema::Int16 { .. } => write!(f, "int16"),
            Schema::Int32 { .. } => write!(f, "int32"),
            Schema::Int64 { .. } => write!(f, "int64"),
            Schema::UInt8 { .. } => write!(f, "uint8"),
            Schema::UInt16 { .. } => write!(f, "uint16"),
            Schema::UInt32 { .. } => write!(f, "uint32"),
            Schema::UInt64 { .. } => write!(f, "uint64"),
            Schema::Float32 { .. } => write!(f, "float32"),
            Schema::Float64 { .. } => write!(f, "float64"),
            Schema::String { .. } => write!(f, "string"),
            Schema::Bytes { .. } => write!(f, "bytes"),
            Schema::Array { items, .. } => write!(f, "Array<{}>", items),
            Schema::Object {
                properties,
                required,
                ..
            } => {
                write!(f, "{{")?;
                for (i, (name, schema)) in properties.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if required.contains(name) {
                        write!(f, "{}: {}", name, schema)?;
                    } else {
                        write!(f, "{}?: {}", name, schema)?;
                    }
                }
                write!(f, "}}")
            }
            Schema::Tuple { items } => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Schema::Union { any_of } => {
                for (i, variant) in any_of.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", variant)?;
                }
                Ok(())
            }
            Schema::Literal { value } => match value {
                LiteralValue::String(s) => write!(f, "\"{}\"", s),
                LiteralValue::Number(n) => write!(f, "{}", n),
                LiteralValue::Float(fl) => write!(f, "{}", fl),
                LiteralValue::Boolean(b) => write!(f, "{}", b),
                LiteralValue::Null => write!(f, "null"),
            },
            Schema::Enum { values } => {
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "\"{}\"", v)?;
                }
                Ok(())
            }
            Schema::Ref { reference } => write!(f, "{}", reference),
            Schema::Named { name, schema } => write!(f, "type {} = {}", name, schema),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_serialize() {
        let mut properties = IndexMap::new();
        properties.insert(
            "id".to_string(),
            Schema::Int64 {
                minimum: None,
                maximum: None,
            },
        );
        properties.insert(
            "name".to_string(),
            Schema::String {
                format: None,
                pattern: None,
                min_length: Some(1),
                max_length: None,
            },
        );

        let schema = Schema::Object {
            properties,
            required: vec!["id".to_string(), "name".to_string()],
            additional_properties: None,
        };

        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("\"kind\": \"object\""));
        assert!(json.contains("\"id\""));
    }

    #[test]
    fn test_schema_deserialize() {
        let json = r#"{"kind": "object", "properties": {"x": {"kind": "int64", "minimum": null, "maximum": null}}, "required": ["x"]}"#;
        let schema: Schema = serde_json::from_str(json).unwrap();
        assert!(matches!(schema, Schema::Object { .. }));
    }

    #[test]
    fn test_string_format_serialize() {
        let schema = Schema::String {
            format: Some(StringFormat::Email),
            pattern: None,
            min_length: None,
            max_length: None,
        };
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"format\":\"email\""));
    }
}
