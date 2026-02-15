use crate::value::Value;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// JSON Schema type definition with metadata support.
///
/// Use [`SchemaBuilder`](crate::SchemaBuilder) for a fluent API to construct schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    /// The type definition.
    #[serde(flatten)]
    pub kind: SchemaKind,

    /// JSON Schema $id field for schema identification.
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// JSON Schema $schema field to specify the schema version.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,

    /// Human-readable title for the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description of the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value for the schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    /// Example values matching this schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<Value>>,

    /// Mark as read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Mark as write-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// Mark as deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
}

/// Schema type variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SchemaKind {
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

    /// Function type with parameters and return type.
    Function {
        parameters: Vec<Schema>,
        returns: Box<Schema>,
    },

    /// Void type - represents no value (function return).
    Void,

    /// Never type - uninhabitable type for exhaustive matching.
    Never,

    /// Any type - escape hatch, validates any value.
    Any,

    /// Unknown type - like Any but semantically requires checking.
    Unknown,

    /// Undefined type - for TypeScript optional unions.
    Undefined,
}

/// String format constraints.
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

/// Literal value types.
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
    /// Creates a new schema with the given kind and no metadata.
    pub fn new(kind: SchemaKind) -> Self {
        Self {
            kind,
            id: None,
            schema_version: None,
            title: None,
            description: None,
            default: None,
            examples: None,
            read_only: None,
            write_only: None,
            deprecated: None,
        }
    }

    /// Returns the kind name of this schema.
    pub fn kind(&self) -> &'static str {
        self.kind.kind_name()
    }

    /// Sets the $id field.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the title field.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description field.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the default value.
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the examples.
    pub fn with_examples(mut self, examples: Vec<Value>) -> Self {
        self.examples = Some(examples);
        self
    }

    /// Sets the read-only flag.
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }

    /// Sets the write-only flag.
    pub fn with_write_only(mut self, write_only: bool) -> Self {
        self.write_only = Some(write_only);
        self
    }

    /// Sets the deprecated flag.
    pub fn with_deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    /// Checks if this schema is optional within the given parent object.
    pub fn is_optional_in(&self, parent: &Schema) -> bool {
        if let SchemaKind::Object { required, .. } = &parent.kind {
            match &self.kind {
                SchemaKind::Named { name, .. } => !required.contains(name),
                _ => false,
            }
        } else {
            false
        }
    }
}

impl SchemaKind {
    /// Returns the kind name for this schema variant.
    pub fn kind_name(&self) -> &'static str {
        match self {
            SchemaKind::Null => "Null",
            SchemaKind::Bool => "Bool",
            SchemaKind::Int8 { .. } => "Int8",
            SchemaKind::Int16 { .. } => "Int16",
            SchemaKind::Int32 { .. } => "Int32",
            SchemaKind::Int64 { .. } => "Int64",
            SchemaKind::UInt8 { .. } => "UInt8",
            SchemaKind::UInt16 { .. } => "UInt16",
            SchemaKind::UInt32 { .. } => "UInt32",
            SchemaKind::UInt64 { .. } => "UInt64",
            SchemaKind::Float32 { .. } => "Float32",
            SchemaKind::Float64 { .. } => "Float64",
            SchemaKind::String { .. } => "String",
            SchemaKind::Bytes { .. } => "Bytes",
            SchemaKind::Array { .. } => "Array",
            SchemaKind::Object { .. } => "Object",
            SchemaKind::Tuple { .. } => "Tuple",
            SchemaKind::Union { .. } => "Union",
            SchemaKind::Literal { .. } => "Literal",
            SchemaKind::Enum { .. } => "Enum",
            SchemaKind::Ref { .. } => "Ref",
            SchemaKind::Named { .. } => "Named",
            SchemaKind::Function { .. } => "Function",
            SchemaKind::Void => "Void",
            SchemaKind::Never => "Never",
            SchemaKind::Any => "Any",
            SchemaKind::Unknown => "Unknown",
            SchemaKind::Undefined => "Undefined",
        }
    }
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::fmt::Display for SchemaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaKind::Null => write!(f, "null"),
            SchemaKind::Bool => write!(f, "boolean"),
            SchemaKind::Int8 { .. } => write!(f, "int8"),
            SchemaKind::Int16 { .. } => write!(f, "int16"),
            SchemaKind::Int32 { .. } => write!(f, "int32"),
            SchemaKind::Int64 { .. } => write!(f, "int64"),
            SchemaKind::UInt8 { .. } => write!(f, "uint8"),
            SchemaKind::UInt16 { .. } => write!(f, "uint16"),
            SchemaKind::UInt32 { .. } => write!(f, "uint32"),
            SchemaKind::UInt64 { .. } => write!(f, "uint64"),
            SchemaKind::Float32 { .. } => write!(f, "float32"),
            SchemaKind::Float64 { .. } => write!(f, "float64"),
            SchemaKind::String { .. } => write!(f, "string"),
            SchemaKind::Bytes { .. } => write!(f, "bytes"),
            SchemaKind::Array { items, .. } => write!(f, "Array<{}>", items),
            SchemaKind::Object {
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
            SchemaKind::Tuple { items } => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            SchemaKind::Union { any_of } => {
                for (i, variant) in any_of.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", variant)?;
                }
                Ok(())
            }
            SchemaKind::Literal { value } => match value {
                LiteralValue::String(s) => write!(f, "\"{}\"", s),
                LiteralValue::Number(n) => write!(f, "{}", n),
                LiteralValue::Float(fl) => write!(f, "{}", fl),
                LiteralValue::Boolean(b) => write!(f, "{}", b),
                LiteralValue::Null => write!(f, "null"),
            },
            SchemaKind::Enum { values } => {
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "\"{}\"", v)?;
                }
                Ok(())
            }
            SchemaKind::Ref { reference } => write!(f, "{}", reference),
            SchemaKind::Named { name, schema } => write!(f, "type {} = {}", name, schema),
            SchemaKind::Function {
                parameters,
                returns,
            } => {
                write!(f, "(")?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") => {}", returns)
            }
            SchemaKind::Void => write!(f, "void"),
            SchemaKind::Never => write!(f, "never"),
            SchemaKind::Any => write!(f, "any"),
            SchemaKind::Unknown => write!(f, "unknown"),
            SchemaKind::Undefined => write!(f, "undefined"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_serialize() {
        let schema = Schema::new(SchemaKind::Object {
            properties: {
                let mut props = IndexMap::new();
                props.insert(
                    "id".to_string(),
                    Schema::new(SchemaKind::Int64 {
                        minimum: None,
                        maximum: None,
                    }),
                );
                props.insert(
                    "name".to_string(),
                    Schema::new(SchemaKind::String {
                        format: None,
                        pattern: None,
                        min_length: Some(1),
                        max_length: None,
                    }),
                );
                props
            },
            required: vec!["id".to_string(), "name".to_string()],
            additional_properties: None,
        });

        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("\"kind\": \"object\""));
        assert!(json.contains("\"id\""));
    }

    #[test]
    fn test_schema_deserialize() {
        let json = r#"{"kind": "object", "properties": {"x": {"kind": "int64", "minimum": null, "maximum": null}}, "required": ["x"]}"#;
        let schema: Schema = serde_json::from_str(json).unwrap();
        assert!(matches!(schema.kind, SchemaKind::Object { .. }));
    }

    #[test]
    fn test_string_format_serialize() {
        let schema = Schema::new(SchemaKind::String {
            format: Some(StringFormat::Email),
            pattern: None,
            min_length: None,
            max_length: None,
        });
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"format\":\"email\""));
    }

    #[test]
    fn test_schema_with_metadata() {
        let schema = Schema::new(SchemaKind::String {
            format: Some(StringFormat::Email),
            pattern: None,
            min_length: None,
            max_length: None,
        })
        .with_id("https://example.com/schemas/email")
        .with_title("Email")
        .with_description("An email address");

        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("\"$id\": \"https://example.com/schemas/email\""));
        assert!(json.contains("\"title\": \"Email\""));
        assert!(json.contains("\"description\": \"An email address\""));
    }

    #[test]
    fn test_function_type() {
        let schema = Schema::new(SchemaKind::Function {
            parameters: vec![Schema::new(SchemaKind::Int64 {
                minimum: None,
                maximum: None,
            })],
            returns: Box::new(Schema::new(SchemaKind::String {
                format: None,
                pattern: None,
                min_length: None,
                max_length: None,
            })),
        });

        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("\"kind\":\"function\""));
        assert!(json.contains("\"parameters\""));
        assert!(json.contains("\"returns\""));
    }

    #[test]
    fn test_void_never_any_unknown_undefined() {
        assert_eq!(Schema::new(SchemaKind::Void).kind(), "Void");
        assert_eq!(Schema::new(SchemaKind::Never).kind(), "Never");
        assert_eq!(Schema::new(SchemaKind::Any).kind(), "Any");
        assert_eq!(Schema::new(SchemaKind::Unknown).kind(), "Unknown");
        assert_eq!(Schema::new(SchemaKind::Undefined).kind(), "Undefined");
    }

    #[test]
    fn test_schema_display() {
        assert_eq!(format!("{}", Schema::new(SchemaKind::Void)), "void");
        assert_eq!(format!("{}", Schema::new(SchemaKind::Never)), "never");
        assert_eq!(format!("{}", Schema::new(SchemaKind::Any)), "any");
        assert_eq!(format!("{}", Schema::new(SchemaKind::Unknown)), "unknown");
        assert_eq!(
            format!("{}", Schema::new(SchemaKind::Undefined)),
            "undefined"
        );
    }
}
