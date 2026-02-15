//! In-place deep mutation for [`Value`].
//!
//! Mutates a value in-place to match another value's structure while
//! preserving internal references where possible. Useful for UI frameworks
//! and caching scenarios where object identity matters.
//!
//! # Examples
//!
//! ```
//! use typebox::{Value, value::mutate::mutate};
//!
//! let mut current = Value::object()
//!     .field("name", Value::string("Alice"))
//!     .field("age", Value::int64(30))
//!     .build();
//!
//! let next = Value::object()
//!     .field("name", Value::string("Bob"))
//!     .field("email", Value::string("bob@example.com"))
//!     .build();
//!
//! mutate(&mut current, &next).unwrap();
//! // current is now mutated in-place with preserved reference
//! ```
/// use typebox::{Value, value::mutate::mutate};
///
/// let mut current = Value::object()
///     .field("name", Value::string("Alice"))
///     .field("age", Value::int64(30))
///     .build();
///
/// let next = Value::object()
///     .field("name", Value::string("Bob"))
///     .field("email", Value::string("bob@example.com"))
///     .build();
///
/// mutate(&mut current, &next).unwrap();
/// // current is now mutated in-place with preserved reference
/// ```
use crate::error::MutateError;
use crate::value::{clone, Value};

/// Mutates `current` in-place to match `next`.
///
/// This operation preserves the top-level reference of `current` while
/// updating its contents to match `next`. Useful when object identity
/// must be preserved (e.g., for UI frameworks tracking object identity).
///
/// # Behavior
///
/// - **Objects**: Adds missing keys (with null values), removes extra keys,
///   recursively mutates existing keys
/// - **Arrays**: Updates elements in-place, truncates to match `next`'s length
/// - **Typed Arrays**: Updates elements in-place if lengths match, otherwise clones
/// - **Primitives**: Replaces if values differ
///
/// # Errors
///
/// Returns [`MutateError`] if:
/// - Root values are not objects or arrays
/// - Root values have mismatched types (object vs array)
///
/// # Examples
///
/// ```
/// use typebox::Value;
/// use typebox::value::mutate::mutate;
///
/// let mut current = Value::array(vec![
///     Value::int64(1),
///     Value::int64(2),
///     Value::int64(3),
/// ]);
///
/// let next = Value::array(vec![
///     Value::int64(10),
///     Value::int64(20),
/// ]);
///
/// mutate(&mut current, &next).unwrap();
/// // current is now [10, 20], with original array reference preserved
/// ```
pub fn mutate(current: &mut Value, next: &Value) -> Result<(), MutateError> {
    if !is_mutable(current) || !is_mutable(next) {
        return Err(MutateError::NotMutable);
    }

    if is_type_mismatch(current, next) {
        return Err(MutateError::TypeMismatch);
    }

    mutate_value(current, next);
    Ok(())
}

fn is_mutable(value: &Value) -> bool {
    matches!(
        value,
        Value::Object(_)
            | Value::Array(_)
            | Value::Null
            | Value::Float32Array(_)
            | Value::Float64Array(_)
            | Value::Int32Array(_)
            | Value::Int64Array(_)
            | Value::UInt8Array(_)
    )
}

fn is_type_mismatch(current: &Value, next: &Value) -> bool {
    matches!(
        (current, next),
        (Value::Object(_), Value::Array(_)) | (Value::Array(_), Value::Object(_))
    )
}

fn mutate_value(current: &mut Value, next: &Value) {
    match (current, next) {
        (Value::Object(current_map), Value::Object(next_map)) => {
            let current_keys: Vec<_> = current_map.keys().cloned().collect();
            let next_keys: Vec<_> = next_map.keys().cloned().collect();

            for key in &current_keys {
                if !next_map.contains_key(key) {
                    current_map.shift_remove(key);
                }
            }

            for key in &next_keys {
                if !current_map.contains_key(key) {
                    current_map.insert(key.clone(), Value::Null);
                }
            }

            for key in &next_keys {
                if let (Some(current_val), Some(next_val)) =
                    (current_map.get_mut(key), next_map.get(key))
                {
                    mutate_value(current_val, next_val);
                }
            }
        }

        (Value::Array(current_arr), Value::Array(next_arr)) => {
            for (i, next_val) in next_arr.iter().enumerate() {
                if i < current_arr.len() {
                    mutate_value(&mut current_arr[i], next_val);
                }
            }
            current_arr.truncate(next_arr.len());
        }

        (Value::Float32Array(current_arr), Value::Float32Array(next_arr))
            if current_arr.len() == next_arr.len() =>
        {
            current_arr.copy_from_slice(next_arr);
        }

        (Value::Float64Array(current_arr), Value::Float64Array(next_arr))
            if current_arr.len() == next_arr.len() =>
        {
            current_arr.copy_from_slice(next_arr);
        }

        (Value::Int32Array(current_arr), Value::Int32Array(next_arr))
            if current_arr.len() == next_arr.len() =>
        {
            current_arr.copy_from_slice(next_arr);
        }

        (Value::Int64Array(current_arr), Value::Int64Array(next_arr))
            if current_arr.len() == next_arr.len() =>
        {
            current_arr.copy_from_slice(next_arr);
        }

        (Value::UInt8Array(current_arr), Value::UInt8Array(next_arr))
            if current_arr.len() == next_arr.len() =>
        {
            current_arr.copy_from_slice(next_arr);
        }

        (current, next) => {
            if current != next {
                *current = clone(next);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::get_pointer;
    use indexmap::indexmap;

    #[test]
    fn test_mutate_object_update() {
        let mut current = Value::Object(indexmap! {
            "name".to_string() => Value::string("Alice"),
            "age".to_string() => Value::int64(30),
        });

        let next = Value::Object(indexmap! {
            "name".to_string() => Value::string("Bob"),
            "age".to_string() => Value::int64(31),
        });

        mutate(&mut current, &next).unwrap();

        assert_eq!(get_pointer(&current, "/name"), Some(&Value::string("Bob")));
        assert_eq!(get_pointer(&current, "/age"), Some(&Value::int64(31)));
    }

    #[test]
    fn test_mutate_object_add_remove_keys() {
        let mut current = Value::Object(indexmap! {
            "name".to_string() => Value::string("Alice"),
            "age".to_string() => Value::int64(30),
        });

        let next = Value::Object(indexmap! {
            "name".to_string() => Value::string("Bob"),
            "email".to_string() => Value::string("bob@example.com"),
        });

        mutate(&mut current, &next).unwrap();

        let map = current.as_object().unwrap();
        assert_eq!(get_pointer(&current, "/name"), Some(&Value::string("Bob")));
        assert!(map.contains_key("email"));
        assert!(!map.contains_key("age"));
    }

    #[test]
    fn test_mutate_array_update() {
        let mut current = Value::array(vec![Value::int64(1), Value::int64(2), Value::int64(3)]);

        let next = Value::array(vec![Value::int64(10), Value::int64(20)]);

        mutate(&mut current, &next).unwrap();

        assert_eq!(get_pointer(&current, "/0"), Some(&Value::int64(10)));
        assert_eq!(get_pointer(&current, "/1"), Some(&Value::int64(20)));
        assert_eq!(current.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_mutate_array_truncate() {
        let mut current = Value::array(vec![Value::int64(1), Value::int64(2), Value::int64(3)]);
        let next = Value::array(vec![Value::int64(1)]);

        mutate(&mut current, &next).unwrap();

        assert_eq!(current.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_mutate_nested() {
        let mut current = Value::Object(indexmap! {
            "user".to_string() => Value::Object(indexmap! {
                "name".to_string() => Value::string("Alice"),
                "address".to_string() => Value::Object(indexmap! {
                    "city".to_string() => Value::string("NYC"),
                }),
            }),
        });

        let next = Value::Object(indexmap! {
            "user".to_string() => Value::Object(indexmap! {
                "name".to_string() => Value::string("Bob"),
                "address".to_string() => Value::Object(indexmap! {
                    "city".to_string() => Value::string("LA"),
                }),
            }),
        });

        mutate(&mut current, &next).unwrap();

        assert_eq!(
            get_pointer(&current, "/user/name"),
            Some(&Value::string("Bob"))
        );
        assert_eq!(
            get_pointer(&current, "/user/address/city"),
            Some(&Value::string("LA"))
        );
    }

    #[test]
    fn test_mutate_type_mismatch() {
        let mut current = Value::object().build();
        let next = Value::array(vec![Value::int64(1)]);

        assert!(matches!(
            mutate(&mut current, &next),
            Err(MutateError::TypeMismatch)
        ));
    }

    #[test]
    fn test_mutate_primitive_not_mutable() {
        let mut current = Value::int64(42);
        let next = Value::int64(100);

        assert!(matches!(
            mutate(&mut current, &next),
            Err(MutateError::NotMutable)
        ));
    }

    #[test]
    fn test_mutate_typed_array_same_length() {
        let mut current = Value::Float64Array(vec![1.0, 2.0, 3.0]);
        let next = Value::Float64Array(vec![10.0, 20.0, 30.0]);

        mutate(&mut current, &next).unwrap();

        assert_eq!(current, Value::Float64Array(vec![10.0, 20.0, 30.0]));
    }

    #[test]
    fn test_mutate_typed_array_different_length() {
        let mut current = Value::Float64Array(vec![1.0, 2.0, 3.0]);
        let next = Value::Float64Array(vec![10.0, 20.0]);

        mutate(&mut current, &next).unwrap();

        assert_eq!(current, Value::Float64Array(vec![10.0, 20.0]));
    }

    #[test]
    fn test_mutate_preserves_object_reference() {
        let mut current = Value::object()
            .field("name", Value::string("Alice"))
            .build();

        let ptr = &current as *const Value;

        let next = Value::object().field("name", Value::string("Bob")).build();

        mutate(&mut current, &next).unwrap();

        assert_eq!(ptr, &current as *const Value);
    }

    #[test]
    fn test_mutate_null_to_object() {
        let mut current = Value::Null;
        let next = Value::object().field("name", Value::string("Bob")).build();

        mutate(&mut current, &next).unwrap();
    }
}
