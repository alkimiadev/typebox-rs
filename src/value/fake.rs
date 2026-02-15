use crate::error::FakeError;
use crate::schema::{LiteralValue, Schema, SchemaKind, StringFormat};
use crate::value::Value;
use indexmap::IndexMap;

#[cfg(feature = "fake")]
use fake::Fake;
#[cfg(feature = "fake")]
use rand::Rng;
#[cfg(feature = "fake")]
use uuid::Uuid;

pub struct FakeContext {
    pub max_depth: usize,
    pub current_depth: usize,
}

impl Default for FakeContext {
    fn default() -> Self {
        Self {
            max_depth: 3,
            current_depth: 0,
        }
    }
}

impl FakeContext {
    pub fn with_depth(&self, depth: usize) -> Self {
        Self {
            max_depth: self.max_depth,
            current_depth: depth,
        }
    }

    pub fn child(&self) -> Self {
        self.with_depth(self.current_depth + 1)
    }
}

#[cfg(feature = "fake")]
pub fn fake(schema: &Schema) -> Result<Value, FakeError> {
    fake_with_context(schema, &FakeContext::default())
}

#[cfg(feature = "fake")]
pub fn fake_with_context(schema: &Schema, ctx: &FakeContext) -> Result<Value, FakeError> {
    if ctx.current_depth > ctx.max_depth {
        return Err(FakeError::MaxDepthExceeded);
    }

    match &schema.kind {
        SchemaKind::Null => Ok(Value::Null),

        SchemaKind::Bool => {
            let val: bool = rand::rng().random_bool(0.5);
            Ok(Value::Bool(val))
        }

        SchemaKind::Int8 { minimum, maximum } => {
            let min = minimum.unwrap_or(i8::MIN) as i64;
            let max = maximum.unwrap_or(i8::MAX) as i64;
            let val: i64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val))
        }

        SchemaKind::Int16 { minimum, maximum } => {
            let min = minimum.unwrap_or(i16::MIN) as i64;
            let max = maximum.unwrap_or(i16::MAX) as i64;
            let val: i64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val))
        }

        SchemaKind::Int32 { minimum, maximum } => {
            let min = minimum.unwrap_or(i32::MIN) as i64;
            let max = maximum.unwrap_or(i32::MAX) as i64;
            let val: i64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val))
        }

        SchemaKind::Int64 { minimum, maximum } => {
            let min = minimum.unwrap_or(i64::MIN);
            let max = maximum.unwrap_or(i64::MAX);
            let val: i64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val))
        }

        SchemaKind::UInt8 { minimum, maximum } => {
            let min = minimum.unwrap_or(u8::MIN) as u64;
            let max = maximum.unwrap_or(u8::MAX) as u64;
            let val: u64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val as i64))
        }

        SchemaKind::UInt16 { minimum, maximum } => {
            let min = minimum.unwrap_or(u16::MIN) as u64;
            let max = maximum.unwrap_or(u16::MAX) as u64;
            let val: u64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val as i64))
        }

        SchemaKind::UInt32 { minimum, maximum } => {
            let min = minimum.unwrap_or(u32::MIN) as u64;
            let max = maximum.unwrap_or(u32::MAX) as u64;
            let val: u64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val as i64))
        }

        SchemaKind::UInt64 { minimum, maximum } => {
            let min = minimum.unwrap_or(u64::MIN);
            let max = maximum.unwrap_or(u64::MAX);
            let val: u64 = rand::rng().random_range(min..=max);
            Ok(Value::Int64(val as i64))
        }

        SchemaKind::Float32 { minimum, maximum } => {
            let min = minimum.unwrap_or(0.0);
            let max = maximum.unwrap_or(1000000.0);
            let val: f32 = rand::rng().random_range(min..=max);
            Ok(Value::Float64(val as f64))
        }

        SchemaKind::Float64 { minimum, maximum } => {
            let min = minimum.unwrap_or(0.0);
            let max = maximum.unwrap_or(1000000.0);
            let val: f64 = rand::rng().random_range(min..=max);
            Ok(Value::Float64(val))
        }

        SchemaKind::String {
            format,
            min_length,
            max_length,
            ..
        } => fake_string(format.as_ref(), *min_length, *max_length),

        SchemaKind::Bytes {
            min_length,
            max_length,
        } => {
            let min = min_length.unwrap_or(0);
            let max = max_length.unwrap_or(min + 16);
            let len = rand::rng().random_range(min..=max);
            let mut bytes = vec![0u8; len];
            rand::rng().fill(&mut bytes[..]);
            Ok(Value::Bytes(bytes))
        }

        SchemaKind::Array {
            items,
            min_items,
            max_items,
            ..
        } => {
            let min = min_items.unwrap_or(0);
            let max = max_items.unwrap_or(min + 3);
            let len = rand::rng().random_range(min..=max);
            let child_ctx = ctx.child();
            let mut arr = Vec::with_capacity(len);
            for _ in 0..len {
                arr.push(fake_with_context(items, &child_ctx)?);
            }
            Ok(Value::Array(arr))
        }

        SchemaKind::Object {
            properties,
            required,
            additional_properties,
        } => {
            if ctx.current_depth >= ctx.max_depth {
                return Ok(Value::Object(IndexMap::new()));
            }

            let child_ctx = ctx.child();
            let mut obj = IndexMap::new();

            for field_name in required {
                if let Some(field_schema) = properties.get(field_name) {
                    obj.insert(
                        field_name.clone(),
                        fake_with_context(field_schema, &child_ctx)?,
                    );
                }
            }

            if let Some(additional_schema) = additional_properties {
                let should_add: bool = rand::rng().random_bool(0.3);
                if should_add {
                    obj.insert(
                        "extra".to_string(),
                        fake_with_context(additional_schema, &child_ctx)?,
                    );
                }
            }

            Ok(Value::Object(obj))
        }

        SchemaKind::Tuple { items } => {
            let child_ctx = ctx.child();
            let mut arr = Vec::with_capacity(items.len());
            for item_schema in items {
                arr.push(fake_with_context(item_schema, &child_ctx)?);
            }
            Ok(Value::Array(arr))
        }

        SchemaKind::Union { any_of } => {
            if any_of.is_empty() {
                return Ok(Value::Null);
            }
            let idx = rand::rng().random_range(0..any_of.len());
            fake_with_context(&any_of[idx], ctx)
        }

        SchemaKind::Literal { value } => Ok(match value {
            LiteralValue::Null => Value::Null,
            LiteralValue::Boolean(b) => Value::Bool(*b),
            LiteralValue::Number(n) => Value::Int64(*n),
            LiteralValue::Float(f) => Value::Float64(*f),
            LiteralValue::String(s) => Value::String(s.clone()),
        }),

        SchemaKind::Enum { values } => {
            if values.is_empty() {
                return Err(FakeError::UnsupportedSchema("empty enum".to_string()));
            }
            let idx = rand::rng().random_range(0..values.len());
            Ok(Value::String(values[idx].clone()))
        }

        SchemaKind::Ref { reference } => Err(FakeError::UnsupportedSchema(format!(
            "unresolved ref: {}",
            reference
        ))),

        SchemaKind::Named { schema, .. } => fake_with_context(schema, ctx),

        SchemaKind::Function { .. } => Ok(Value::Null),
        SchemaKind::Void => Ok(Value::Null),
        SchemaKind::Never => Err(FakeError::UnsupportedSchema("never type".to_string())),
        SchemaKind::Any => Ok(Value::Null),
        SchemaKind::Unknown => Ok(Value::Null),
        SchemaKind::Undefined => Ok(Value::Null),
    }
}

#[cfg(feature = "fake")]
fn fake_string(
    format: Option<&StringFormat>,
    min_length: Option<usize>,
    max_length: Option<usize>,
) -> Result<Value, FakeError> {
    use fake::faker::internet::en::{DomainSuffix, IPv4, IPv6, SafeEmail};

    match format {
        Some(StringFormat::Email) => {
            let email: String = SafeEmail().fake();
            Ok(Value::String(email))
        }
        Some(StringFormat::Uuid) => Ok(Value::String(Uuid::new_v4().to_string())),
        Some(StringFormat::Uri) => {
            let domain: String = DomainSuffix().fake();
            Ok(Value::String(format!("https://example.{}", domain)))
        }
        Some(StringFormat::DateTime) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let offset: i64 = rand::rng().random_range(-86400..86400);
            let ts = (now as i64) + offset;
            Ok(Value::String(format!("{}Z", ts)))
        }
        Some(StringFormat::Date) => {
            let year: u32 = rand::rng().random_range(2000..2030);
            let month: u32 = rand::rng().random_range(1..=12);
            let day: u32 = rand::rng().random_range(1..=28);
            Ok(Value::String(format!(
                "{:04}-{:02}-{:02}",
                year, month, day
            )))
        }
        Some(StringFormat::Time) => {
            let hour: u32 = rand::rng().random_range(0..23);
            let min: u32 = rand::rng().random_range(0..59);
            let sec: u32 = rand::rng().random_range(0..59);
            Ok(Value::String(format!("{:02}:{:02}:{:02}", hour, min, sec)))
        }
        Some(StringFormat::Hostname) => {
            let domain: String = DomainSuffix().fake();
            Ok(Value::String(format!("host-{}", domain)))
        }
        Some(StringFormat::Ipv4) => {
            let ip: String = IPv4().fake();
            Ok(Value::String(ip))
        }
        Some(StringFormat::Ipv6) => {
            let ip: String = IPv6().fake();
            Ok(Value::String(ip))
        }
        Some(StringFormat::Custom(_)) | None => {
            let min = min_length.unwrap_or(1);
            let max = max_length.unwrap_or(min + 10);
            let len = rand::rng().random_range(min..=max);
            let s: String = fake::faker::lorem::en::Word().fake();
            let s = if s.len() >= len {
                s[..len].to_string()
            } else {
                let mut result = s;
                while result.len() < len {
                    result.push_str(&fake::faker::lorem::en::Word().fake::<String>());
                }
                result[..len].to_string()
            };
            Ok(Value::String(s))
        }
    }
}

#[cfg(all(test, feature = "fake"))]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;
    use crate::schema::Schema;

    #[test]
    fn test_fake_primitives() {
        assert!(fake(&Schema::new(SchemaKind::Null)).unwrap().is_null());

        let bool_val = fake(&Schema::new(SchemaKind::Bool)).unwrap();
        assert!(matches!(bool_val, Value::Bool(_)));

        let int_val = fake(&SchemaBuilder::int64()).unwrap();
        assert!(matches!(int_val, Value::Int64(_)));

        let float_val = fake(&SchemaBuilder::float64()).unwrap();
        assert!(matches!(float_val, Value::Float64(_)));
    }

    #[test]
    fn test_fake_string() {
        let s = fake(&SchemaBuilder::string().build()).unwrap();
        assert!(matches!(s, Value::String(_)));
    }

    #[test]
    fn test_fake_string_with_format() {
        let schema = SchemaBuilder::string().format(StringFormat::Email).build();
        let s = fake(&schema).unwrap();
        if let Value::String(email) = s {
            assert!(email.contains('@'));
        } else {
            panic!("Expected string value");
        }

        let schema = SchemaBuilder::string().format(StringFormat::Uuid).build();
        let s = fake(&schema).unwrap();
        if let Value::String(uuid) = s {
            assert_eq!(uuid.len(), 36);
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_fake_numeric_bounds() {
        let schema = Schema::new(SchemaKind::Int64 {
            minimum: Some(10),
            maximum: Some(20),
        });
        for _ in 0..10 {
            let val = fake(&schema).unwrap();
            if let Value::Int64(n) = val {
                assert!((10..=20).contains(&n));
            } else {
                panic!("Expected Int64");
            }
        }
    }

    #[test]
    fn test_fake_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64())
            .min_items(2)
            .max_items(5)
            .build();

        let arr = fake(&schema).unwrap();
        if let Value::Array(items) = arr {
            assert!(items.len() >= 2 && items.len() <= 5);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_fake_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build();

        let obj = fake(&schema).unwrap();
        if let Value::Object(map) = obj {
            assert!(map.contains_key("id"));
            assert!(map.contains_key("name"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_fake_union() {
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::string().build(),
            SchemaBuilder::int64(),
        ]);

        let val = fake(&schema).unwrap();
        assert!(matches!(val, Value::String(_) | Value::Int64(_)));
    }

    #[test]
    fn test_fake_enum() {
        let schema = Schema::new(SchemaKind::Enum {
            values: vec!["one".to_string(), "two".to_string(), "three".to_string()],
        });

        for _ in 0..10 {
            let val = fake(&schema).unwrap();
            if let Value::String(s) = val {
                assert!(["one", "two", "three"].contains(&s.as_str()));
            } else {
                panic!("Expected string");
            }
        }
    }

    #[test]
    fn test_fake_literal() {
        let schema = Schema::new(SchemaKind::Literal {
            value: LiteralValue::String("hello".to_string()),
        });
        assert_eq!(fake(&schema).unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_fake_max_depth() {
        let ctx = FakeContext {
            max_depth: 2,
            current_depth: 3,
        };
        let result = fake_with_context(&SchemaBuilder::int64(), &ctx);
        assert!(matches!(result, Err(FakeError::MaxDepthExceeded)));
    }
}
