use crate::error::ValidationError;
use crate::schema::Schema;
use crate::value::Value;

pub fn check(schema: &Schema, value: &Value) -> bool {
    crate::validate::validate(schema, value).is_ok()
}

pub fn check_with_errors(schema: &Schema, value: &Value) -> Result<(), ValidationError> {
    crate::validate::validate(schema, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::SchemaBuilder;

    #[test]
    fn test_check_primitives() {
        assert!(check(&SchemaBuilder::null(), &Value::Null));
        assert!(check(&SchemaBuilder::bool(), &Value::Bool(true)));
        assert!(check(&SchemaBuilder::int64(), &Value::Int64(42)));
        assert!(check(
            &SchemaBuilder::string().build(),
            &Value::String("hello".to_string())
        ));
        assert!(check(&SchemaBuilder::float64(), &Value::Float64(3.14)));
    }

    #[test]
    fn test_check_type_mismatch() {
        assert!(!check(
            &SchemaBuilder::int64(),
            &Value::String("not a number".to_string())
        ));
        assert!(!check(&SchemaBuilder::string().build(), &Value::Int64(42)));
        assert!(!check(&SchemaBuilder::bool(), &Value::Null));
    }

    #[test]
    fn test_check_object() {
        let schema = SchemaBuilder::object()
            .field("id", SchemaBuilder::int64())
            .field("name", SchemaBuilder::string().build())
            .optional_field("email", SchemaBuilder::string().build())
            .build();

        let valid = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("Alice".to_string()))
            .build();
        assert!(check(&schema, &valid));

        let missing_required = Value::object()
            .field("name", Value::String("Charlie".to_string()))
            .build();
        assert!(!check(&schema, &missing_required));
    }

    #[test]
    fn test_check_array() {
        let schema = SchemaBuilder::array(SchemaBuilder::int64())
            .min_items(1)
            .max_items(3)
            .build();

        assert!(check(
            &schema,
            &Value::Array(vec![Value::Int64(1), Value::Int64(2)])
        ));
        assert!(!check(&schema, &Value::Array(vec![])));
        assert!(!check(
            &schema,
            &Value::Array(vec![
                Value::Int64(1),
                Value::Int64(2),
                Value::Int64(3),
                Value::Int64(4)
            ])
        ));
    }

    #[test]
    fn test_check_union() {
        let schema = SchemaBuilder::union(vec![
            SchemaBuilder::string().build(),
            SchemaBuilder::int64(),
        ]);

        assert!(check(&schema, &Value::String("hello".to_string())));
        assert!(check(&schema, &Value::Int64(42)));
        assert!(!check(&schema, &Value::Bool(true)));
    }

    #[test]
    fn test_check_with_errors() {
        let schema = SchemaBuilder::int64();
        let result = check_with_errors(&schema, &Value::String("not a number".to_string()));
        assert!(result.is_err());
        if let Err(ValidationError::TypeMismatch { expected, actual }) = result {
            assert_eq!(expected, "Int64");
            assert_eq!(actual, "string");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
}
