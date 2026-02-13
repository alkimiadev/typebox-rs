use typebox::{validate, SchemaBuilder};

#[test]
fn test_basic_schema() {
    let schema = SchemaBuilder::object()
        .field("id", SchemaBuilder::int64())
        .field("name", SchemaBuilder::string())
        .optional_field("active", SchemaBuilder::bool())
        .build();

    let valid = serde_json::json!({
        "id": 42,
        "name": "test"
    });

    assert!(validate(&schema, &valid).is_ok());
}

#[test]
fn test_array_schema() {
    let schema = SchemaBuilder::array(SchemaBuilder::int64())
        .min_items(1)
        .max_items(10)
        .build();

    let valid = serde_json::json!([1, 2, 3]);
    let empty = serde_json::json!([]);
    let too_many: Vec<i64> = (0..20).collect();

    assert!(validate(&schema, &valid).is_ok());
    assert!(validate(&schema, &empty).is_err());
    assert!(validate(&schema, &serde_json::json!(too_many)).is_err());
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
            .field("name", SchemaBuilder::string())
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
            .field("name", SchemaBuilder::string())
            .build(),
    );

    let code = gen.generate_module(&registry).unwrap();
    assert!(code.contains("export interface Person"));
    assert!(code.contains("id: number"));
    assert!(code.contains("name: string"));
}
