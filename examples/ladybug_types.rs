use typebox::{RustGenerator, SchemaBuilder, SchemaRegistry, TypeScriptGenerator};

fn main() {
    let cypher_request = SchemaBuilder::object()
        .field("query", SchemaBuilder::string())
        .optional_field(
            "params",
            SchemaBuilder::object().additional_properties(true).build(),
        )
        .named("CypherRequest");

    let cypher_response = SchemaBuilder::object()
        .field(
            "rows",
            SchemaBuilder::array(SchemaBuilder::object().additional_properties(true).build())
                .build(),
        )
        .optional_field(
            "dataTypes",
            SchemaBuilder::object().additional_properties(true).build(),
        )
        .field("isSchemaChanged", SchemaBuilder::bool())
        .named("CypherResponse");

    let property_info = SchemaBuilder::object()
        .field("name", SchemaBuilder::string())
        .field("type", SchemaBuilder::string())
        .field("isPrimaryKey", SchemaBuilder::bool())
        .named("PropertyInfo");

    let table_info = SchemaBuilder::object()
        .field("name", SchemaBuilder::string())
        .field("comment", SchemaBuilder::string())
        .field(
            "properties",
            SchemaBuilder::array(SchemaBuilder::r#ref("PropertyInfo")).build(),
        )
        .optional_field("isPrimaryKey", SchemaBuilder::bool())
        .optional_field("src", SchemaBuilder::string())
        .optional_field("dst", SchemaBuilder::string())
        .optional_field(
            "connectivity",
            SchemaBuilder::array(
                SchemaBuilder::object()
                    .field("src", SchemaBuilder::string())
                    .field("dst", SchemaBuilder::string())
                    .build(),
            )
            .build(),
        )
        .named("TableInfo");

    let schema_response = SchemaBuilder::object()
        .field(
            "nodeTables",
            SchemaBuilder::array(SchemaBuilder::r#ref("TableInfo")).build(),
        )
        .field(
            "relTables",
            SchemaBuilder::array(SchemaBuilder::r#ref("TableInfo")).build(),
        )
        .field(
            "relGroups",
            SchemaBuilder::array(
                SchemaBuilder::object()
                    .field("name", SchemaBuilder::string())
                    .field(
                        "relTables",
                        SchemaBuilder::array(SchemaBuilder::string()).build(),
                    )
                    .build(),
            )
            .build(),
        )
        .field(
            "rdf",
            SchemaBuilder::array(
                SchemaBuilder::object()
                    .field("prefix", SchemaBuilder::string())
                    .field("iri", SchemaBuilder::string())
                    .build(),
            )
            .build(),
        )
        .named("SchemaResponse");

    let mut registry = SchemaRegistry::new();
    registry.register("PropertyInfo", property_info);
    registry.register("TableInfo", table_info);
    registry.register("CypherRequest", cypher_request);
    registry.register("CypherResponse", cypher_response);
    registry.register("SchemaResponse", schema_response);

    println!("=== Rust Types ===\n");
    let rust_gen = RustGenerator::new();
    let rust_code = rust_gen.generate_module(&registry).unwrap();
    println!("{}", rust_code);

    println!("\n=== TypeScript Types ===\n");
    let ts_gen = TypeScriptGenerator::new();
    let ts_code = ts_gen.generate_module(&registry).unwrap();
    println!("{}", ts_code);
}
