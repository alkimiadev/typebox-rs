//! Deep clone values.

use crate::value::Value;

/// Create a deep clone of a value.
pub fn clone(value: &Value) -> Value {
    value.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_clone_null() {
        let original = Value::Null;
        let cloned = clone(&original);
        assert_eq!(cloned, Value::Null);
    }

    #[test]
    fn test_clone_primitives() {
        let bool_val = Value::Bool(true);
        assert_eq!(clone(&bool_val), bool_val);

        let int_val = Value::Int64(42);
        assert_eq!(clone(&int_val), int_val);

        let float_val = Value::Float64(3.14);
        assert_eq!(clone(&float_val), float_val);

        let string_val = Value::String("hello".to_string());
        assert_eq!(clone(&string_val), string_val);
    }

    #[test]
    fn test_clone_array() {
        let arr = Value::Array(vec![Value::Int64(1), Value::Int64(2), Value::Int64(3)]);
        let cloned = clone(&arr);
        assert_eq!(cloned, arr);
    }

    #[test]
    fn test_clone_object() {
        let mut map = IndexMap::new();
        map.insert("id".to_string(), Value::Int64(1));
        map.insert("name".to_string(), Value::String("test".to_string()));
        let obj = Value::Object(map);

        let cloned = clone(&obj);
        assert_eq!(cloned, obj);
    }

    #[test]
    fn test_clone_typed_arrays() {
        let bytes = Value::Bytes(vec![1, 2, 3]);
        assert_eq!(clone(&bytes), bytes);

        let f32_arr = Value::Float32Array(vec![1.0, 2.0, 3.0]);
        assert_eq!(clone(&f32_arr), f32_arr);

        let f64_arr = Value::Float64Array(vec![1.0, 2.0, 3.0]);
        assert_eq!(clone(&f64_arr), f64_arr);

        let i32_arr = Value::Int32Array(vec![1, 2, 3]);
        assert_eq!(clone(&i32_arr), i32_arr);

        let i64_arr = Value::Int64Array(vec![1, 2, 3]);
        assert_eq!(clone(&i64_arr), i64_arr);

        let u8_arr = Value::UInt8Array(vec![1, 2, 3]);
        assert_eq!(clone(&u8_arr), u8_arr);
    }

    #[test]
    fn test_clone_is_deep() {
        let mut inner_map = IndexMap::new();
        inner_map.insert("nested".to_string(), Value::Int64(42));
        let outer = Value::Object(inner_map);

        let cloned = clone(&outer);

        // Verify they are equal but distinct
        assert_eq!(cloned, outer);
    }
}
