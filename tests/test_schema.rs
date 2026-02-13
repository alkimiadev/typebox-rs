use typebox::{validate, SchemaBuilder, Value};

#[test]
fn test_basic_schema() {
    let schema = SchemaBuilder::object()
        .field("id", SchemaBuilder::int64())
        .field("name", SchemaBuilder::string().build())
        .optional_field("active", SchemaBuilder::bool())
        .build();

    let valid = Value::object()
        .field("id", Value::Int64(42))
        .field("name", Value::String("test".to_string()))
        .build();

    assert!(validate(&schema, &valid).is_ok());
}

#[test]
fn test_array_schema() {
    let schema = SchemaBuilder::array(SchemaBuilder::int64())
        .min_items(1)
        .max_items(10)
        .build();

    let valid = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(3)]);
    let empty = Value::Array(vec![]);
    let too_many: Value = Value::Array((0..20).map(Value::Int64).collect());

    assert!(validate(&schema, &valid).is_ok());
    assert!(validate(&schema, &empty).is_err());
    assert!(validate(&schema, &too_many).is_err());
}

#[test]
fn test_layout_calculation() {
    let schema = SchemaBuilder::object()
        .field("a", SchemaBuilder::int8())
        .field("b", SchemaBuilder::int64())
        .field("c", SchemaBuilder::int8())
        .build();

    let layout = schema.layout();

    assert_eq!(layout.offsets, vec![0, 8, 16]);
    assert_eq!(layout.align, 8);
    assert_eq!(layout.size, 24);
}

#[cfg(feature = "codegen")]
#[test]
fn test_rust_codegen() {
    use typebox::{RustGenerator, SchemaRegistry};

    let gen = RustGenerator::new();
    let mut registry = SchemaRegistry::new();

    registry.register(
        "Person",
        SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build(),
    );

    let code = gen.generate_module(&registry).unwrap();
    assert!(code.contains("pub struct Person"));
    assert!(code.contains("pub id: i64"));
    assert!(code.contains("pub name: String"));
}

#[cfg(feature = "codegen")]
#[test]
fn test_typescript_codegen() {
    use typebox::{SchemaRegistry, TypeScriptGenerator};

    let gen = TypeScriptGenerator::new();
    let mut registry = SchemaRegistry::new();

    registry.register(
        "Person",
        SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .build(),
    );

    let code = gen.generate_module(&registry).unwrap();
    assert!(code.contains("export interface Person"));
    assert!(code.contains("id: number"));
    assert!(code.contains("name: string"));
}

#[test]
fn test_value_from_json() {
    let json = serde_json::json!({"id": 42, "name": "test"});
    let schema = SchemaBuilder::object()
        .field("id", SchemaBuilder::int64())
        .field("name", SchemaBuilder::string().build())
        .build();

    let value = Value::from_json(json, &schema).unwrap();
    assert!(validate(&schema, &value).is_ok());
}

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
