use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Schema {
    Null,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    String,
    Bytes,

    Array {
        items: Box<Schema>,
        #[serde(default)]
        min_items: Option<usize>,
        #[serde(default)]
        max_items: Option<usize>,
    },
    Object {
        #[serde(default)]
        properties: Vec<Property>,
        #[serde(default)]
        additional_properties: bool,
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
pub struct Property {
    pub name: String,
    #[serde(rename = "type")]
    pub type_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<Property>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<LiteralValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Property {
    pub fn schema(&self) -> Schema {
        match self.type_kind.as_str() {
            "Null" => Schema::Null,
            "Bool" => Schema::Bool,
            "Int8" => Schema::Int8,
            "Int16" => Schema::Int16,
            "Int32" => Schema::Int32,
            "Int64" => Schema::Int64,
            "UInt8" => Schema::UInt8,
            "UInt16" => Schema::UInt16,
            "UInt32" => Schema::UInt32,
            "UInt64" => Schema::UInt64,
            "Float32" => Schema::Float32,
            "Float64" => Schema::Float64,
            "String" => Schema::String,
            "Bytes" => Schema::Bytes,
            "Array" => Schema::Array {
                items: self.items.clone().unwrap_or_else(|| Box::new(Schema::Null)),
                min_items: self.min_items,
                max_items: self.max_items,
            },
            "Object" => Schema::Object {
                properties: self.properties.clone().unwrap_or_default(),
                additional_properties: self.additional_properties.unwrap_or(false),
            },
            "Tuple" => Schema::Tuple {
                items: vec![], // Not fully supported in flattened form
            },
            "Union" => Schema::Union {
                any_of: self.any_of.clone().unwrap_or_default(),
            },
            "Literal" => Schema::Literal {
                value: self.value.clone().unwrap_or(LiteralValue::Null),
            },
            "Enum" => Schema::Enum {
                values: self.values.clone().unwrap_or_default(),
            },
            "Ref" => Schema::Ref {
                reference: self.reference.clone().unwrap_or_default(),
            },
            _ => Schema::Null,
        }
    }

    pub fn from_schema(
        name: &str,
        schema: &Schema,
        optional: bool,
        description: Option<String>,
    ) -> Self {
        let type_kind = schema.kind().to_string();

        let (
            items,
            properties,
            any_of,
            value,
            values,
            reference,
            min_items,
            max_items,
            additional_properties,
        ) = match schema {
            Schema::Array {
                items,
                min_items,
                max_items,
            } => (
                Some(items.clone()),
                None,
                None,
                None,
                None,
                None,
                *min_items,
                *max_items,
                None,
            ),
            Schema::Object {
                properties,
                additional_properties,
            } => (
                None,
                Some(properties.clone()),
                None,
                None,
                None,
                None,
                None,
                None,
                Some(*additional_properties),
            ),
            Schema::Union { any_of } => (
                None,
                None,
                Some(any_of.clone()),
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            Schema::Literal { value } => (
                None,
                None,
                None,
                Some(value.clone()),
                None,
                None,
                None,
                None,
                None,
            ),
            Schema::Enum { values } => (
                None,
                None,
                None,
                None,
                Some(values.clone()),
                None,
                None,
                None,
                None,
            ),
            Schema::Ref { reference } => (
                None,
                None,
                None,
                None,
                None,
                Some(reference.clone()),
                None,
                None,
                None,
            ),
            _ => (None, None, None, None, None, None, None, None, None),
        };

        Self {
            name: name.to_string(),
            type_kind,
            items,
            properties,
            any_of,
            value,
            values,
            reference,
            min_items,
            max_items,
            additional_properties,
            optional,
            description,
        }
    }
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
            Schema::Int8 => "Int8",
            Schema::Int16 => "Int16",
            Schema::Int32 => "Int32",
            Schema::Int64 => "Int64",
            Schema::UInt8 => "UInt8",
            Schema::UInt16 => "UInt16",
            Schema::UInt32 => "UInt32",
            Schema::UInt64 => "UInt64",
            Schema::Float32 => "Float32",
            Schema::Float64 => "Float64",
            Schema::String => "String",
            Schema::Bytes => "Bytes",
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
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Null => write!(f, "null"),
            Schema::Bool => write!(f, "boolean"),
            Schema::Int8 => write!(f, "int8"),
            Schema::Int16 => write!(f, "int16"),
            Schema::Int32 => write!(f, "int32"),
            Schema::Int64 => write!(f, "int64"),
            Schema::UInt8 => write!(f, "uint8"),
            Schema::UInt16 => write!(f, "uint16"),
            Schema::UInt32 => write!(f, "uint32"),
            Schema::UInt64 => write!(f, "uint64"),
            Schema::Float32 => write!(f, "float32"),
            Schema::Float64 => write!(f, "float64"),
            Schema::String => write!(f, "string"),
            Schema::Bytes => write!(f, "bytes"),
            Schema::Array { items, .. } => write!(f, "Array<{}>", items),
            Schema::Object { properties, .. } => {
                write!(f, "{{")?;
                for (i, prop) in properties.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", prop.name, prop.schema())?;
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
            Schema::Literal { value } => write!(f, "{:?}", value),
            Schema::Enum { values } => {
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", v)?;
                }
                Ok(())
            }
            Schema::Ref { reference } => write!(f, "{}", reference),
            Schema::Named { name, schema } => write!(f, "{} = {}", name, schema),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_serialize() {
        let schema = Schema::Object {
            properties: vec![
                Property::from_schema("id", &Schema::Int64, false, None),
                Property::from_schema("name", &Schema::String, false, Some("The name".to_string())),
            ],
            additional_properties: false,
        };

        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("\"kind\": \"Object\""));
        assert!(json.contains("\"id\""));
    }

    #[test]
    fn test_schema_deserialize() {
        let json = r#"{"kind": "Object", "properties": [{"name": "x", "type": "Int64"}]}"#;
        let schema: Schema = serde_json::from_str(json).unwrap();
        assert!(matches!(schema, Schema::Object { .. }));
    }
}
