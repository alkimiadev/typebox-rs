use crate::value::Value;

pub fn equal(a: &Value, b: &Value) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_equal_primitives() {
        assert!(equal(&Value::Null, &Value::Null));
        assert!(equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(!equal(&Value::Bool(true), &Value::Bool(false)));
        assert!(equal(&Value::Int64(42), &Value::Int64(42)));
        assert!(!equal(&Value::Int64(42), &Value::Int64(43)));
        assert!(equal(&Value::Float64(3.14), &Value::Float64(3.14)));
        assert!(equal(
            &Value::String("hello".to_string()),
            &Value::String("hello".to_string())
        ));
        assert!(!equal(
            &Value::String("hello".to_string()),
            &Value::String("world".to_string())
        ));
    }

    #[test]
    fn test_equal_type_mismatch() {
        assert!(!equal(&Value::Int64(1), &Value::Float64(1.0)));
        assert!(!equal(&Value::Int64(1), &Value::String("1".to_string())));
        assert!(!equal(&Value::Bool(true), &Value::Int64(1)));
    }

    #[test]
    fn test_equal_arrays() {
        let a = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(3)]);
        let b = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(3)]);
        assert!(equal(&a, &b));

        let c = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);
        assert!(!equal(&a, &c));

        let d = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(4)]);
        assert!(!equal(&a, &d));
    }

    #[test]
    fn test_equal_objects() {
        let mut map_a = IndexMap::new();
        map_a.insert("id".to_string(), Value::Int64(1));
        map_a.insert("name".to_string(), Value::String("test".to_string()));
        let a = Value::Object(map_a);

        let mut map_b = IndexMap::new();
        map_b.insert("id".to_string(), Value::Int64(1));
        map_b.insert("name".to_string(), Value::String("test".to_string()));
        let b = Value::Object(map_b);

        assert!(equal(&a, &b));

        let mut map_c = IndexMap::new();
        map_c.insert("id".to_string(), Value::Int64(2));
        let c = Value::Object(map_c);
        assert!(!equal(&a, &c));
    }

    #[test]
    fn test_equal_typed_arrays() {
        assert!(equal(
            &Value::Float32Array(vec![1.0, 2.0, 3.0]),
            &Value::Float32Array(vec![1.0, 2.0, 3.0])
        ));
        assert!(!equal(
            &Value::Float32Array(vec![1.0, 2.0, 3.0]),
            &Value::Float64Array(vec![1.0, 2.0, 3.0])
        ));
        assert!(equal(
            &Value::Bytes(vec![1, 2, 3]),
            &Value::Bytes(vec![1, 2, 3])
        ));
        assert!(!equal(
            &Value::Bytes(vec![1, 2, 3]),
            &Value::Bytes(vec![1, 2])
        ));
    }

    #[test]
    fn test_equal_nested() {
        let mut inner = IndexMap::new();
        inner.insert("nested".to_string(), Value::Int64(42));

        let mut outer_a = IndexMap::new();
        outer_a.insert("inner".to_string(), Value::Object(inner.clone()));
        let a = Value::Object(outer_a);

        let mut outer_b = IndexMap::new();
        outer_b.insert("inner".to_string(), Value::Object(inner));
        let b = Value::Object(outer_b);

        assert!(equal(&a, &b));
    }
}
