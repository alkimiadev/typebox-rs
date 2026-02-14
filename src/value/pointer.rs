//! JSON Pointer (RFC6901) operations for [`Value`].
//!
//! Provides path-based access to nested values using JSON Pointer syntax.
//!
//! # Examples
//!
//! ```
//! use typebox::Value;
//! use typebox::value::pointer::{get_pointer, set_pointer, has_pointer};
//!
//! let mut value = Value::object()
//!     .field("user", Value::object()
//!         .field("name", Value::string("Alice"))
//!         .build())
//!     .build();
//!
//! assert_eq!(get_pointer(&value, "/user/name"), Some(&Value::string("Alice")));
//! assert!(has_pointer(&value, "/user/name"));
//! ```

use crate::error::PointerError;
use crate::value::Value;

/// Parses a JSON Pointer path into components.
///
/// RFC6901 escape sequences are decoded:
/// - `~0` → `~`
/// - `~1` → `/`
fn parse_pointer(pointer: &str) -> Vec<String> {
    if pointer.is_empty() {
        return vec![];
    }

    let pointer = pointer.strip_prefix('/').unwrap_or(pointer);

    pointer
        .split('/')
        .map(|component| component.replace("~1", "/").replace("~0", "~"))
        .collect()
}

/// Gets a reference to the value at the given JSON Pointer path.
///
/// Returns `None` if the path doesn't exist or the value structure
/// doesn't match the path (e.g., trying to access a field on an array).
///
/// # Examples
///
/// ```
/// use typebox::Value;
/// use typebox::value::pointer::get_pointer;
///
/// let value = Value::array(vec![
///     Value::int64(1),
///     Value::int64(2),
/// ]);
///
/// assert_eq!(get_pointer(&value, "/0"), Some(&Value::int64(1)));
/// assert_eq!(get_pointer(&value, "/2"), None);
/// ```
pub fn get_pointer<'a>(value: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        return Some(value);
    }

    let components = parse_pointer(pointer);
    let mut current = value;

    for component in components {
        current = match current {
            Value::Object(map) => map.get(&component)?,
            Value::Array(arr) => {
                let index: usize = component.parse().ok()?;
                arr.get(index)?
            }
            _ => return None,
        };
    }

    Some(current)
}

/// Gets a mutable reference to the value at the given JSON Pointer path.
///
/// Returns `None` if the path doesn't exist or the value structure
/// doesn't match the path.
pub fn get_pointer_mut<'a>(value: &'a mut Value, pointer: &str) -> Option<&'a mut Value> {
    if pointer.is_empty() {
        return Some(value);
    }

    let components = parse_pointer(pointer);
    let mut current = value;

    for component in components {
        current = match current {
            Value::Object(map) => map.get_mut(&component)?,
            Value::Array(arr) => {
                let index: usize = component.parse().ok()?;
                arr.get_mut(index)?
            }
            _ => return None,
        };
    }

    Some(current)
}

/// Sets the value at the given JSON Pointer path.
///
/// Creates intermediate objects as needed. Returns an error if:
/// - The pointer is empty (cannot replace root)
/// - A non-object/array is encountered in the path
/// - An invalid array index is specified
///
/// # Examples
///
/// ```
/// use typebox::Value;
/// use typebox::value::pointer::{get_pointer, set_pointer};
///
/// let mut value = Value::object().build();
/// set_pointer(&mut value, "/user/name", Value::string("Bob")).unwrap();
/// assert_eq!(get_pointer(&value, "/user/name"), Some(&Value::string("Bob")));
/// ```
///
/// # Errors
///
/// Returns [`PointerError::EmptyPointer`] if pointer is empty.
/// Returns [`PointerError::InvalidPath`] if the path is invalid.
pub fn set_pointer(value: &mut Value, pointer: &str, new_value: Value) -> Result<(), PointerError> {
    if pointer.is_empty() {
        return Err(PointerError::EmptyPointer);
    }

    let components = parse_pointer(pointer);
    if components.is_empty() {
        return Err(PointerError::EmptyPointer);
    }

    let (last, path) = components.split_last().ok_or(PointerError::EmptyPointer)?;

    let mut current = value;

    for component in path {
        current = match current {
            Value::Object(map) => {
                if !map.contains_key(component) {
                    map.insert(component.clone(), Value::object().build());
                }
                map.get_mut(component).unwrap()
            }
            Value::Array(arr) => {
                let index: usize = component.parse().map_err(|_| {
                    PointerError::InvalidPath(format!("Invalid array index: {}", component))
                })?;
                if index >= arr.len() {
                    return Err(PointerError::InvalidPath(format!(
                        "Array index {} out of bounds (length {})",
                        index,
                        arr.len()
                    )));
                }
                &mut arr[index]
            }
            _ => {
                return Err(PointerError::InvalidPath(format!(
                    "Cannot navigate through {}",
                    current.kind()
                )));
            }
        };
    }

    match current {
        Value::Object(map) => {
            map.insert(last.clone(), new_value);
        }
        Value::Array(arr) => {
            let index: usize = last
                .parse()
                .map_err(|_| PointerError::InvalidPath(format!("Invalid array index: {}", last)))?;
            if index >= arr.len() {
                return Err(PointerError::InvalidPath(format!(
                    "Array index {} out of bounds (length {})",
                    index,
                    arr.len()
                )));
            }
            arr[index] = new_value;
        }
        _ => {
            return Err(PointerError::InvalidPath(format!(
                "Cannot set field on {}",
                current.kind()
            )));
        }
    }

    Ok(())
}

/// Deletes the value at the given JSON Pointer path.
///
/// For arrays, removes the element (indices shift). For objects, removes the key.
/// Returns an error if:
/// - The pointer is empty (cannot delete root)
/// - The path doesn't exist
/// - An invalid array index is specified
///
/// # Examples
///
/// ```
/// use typebox::Value;
/// use typebox::value::pointer::{delete_pointer, has_pointer};
///
/// let mut value = Value::object()
///     .field("name", Value::string("Alice"))
///     .field("age", Value::int64(30))
///     .build();
///
/// delete_pointer(&mut value, "/age").unwrap();
/// assert!(!has_pointer(&value, "/age"));
/// ```
///
/// # Errors
///
/// Returns [`PointerError::EmptyPointer`] if pointer is empty.
/// Returns [`PointerError::NotFound`] if the path doesn't exist.
pub fn delete_pointer(value: &mut Value, pointer: &str) -> Result<(), PointerError> {
    if pointer.is_empty() {
        return Err(PointerError::EmptyPointer);
    }

    let components = parse_pointer(pointer);
    if components.is_empty() {
        return Err(PointerError::EmptyPointer);
    }

    let (last, path) = components.split_last().ok_or(PointerError::EmptyPointer)?;

    let current = if path.is_empty() {
        value
    } else {
        let mut current = value;
        for component in path {
            current = match current {
                Value::Object(map) => map
                    .get_mut(component)
                    .ok_or_else(|| PointerError::NotFound(format!("/{}", component)))?,
                Value::Array(arr) => {
                    let index: usize = component.parse().map_err(|_| {
                        PointerError::InvalidPath(format!("Invalid array index: {}", component))
                    })?;
                    arr.get_mut(index)
                        .ok_or_else(|| PointerError::NotFound(format!("/{}", component)))?
                }
                _ => {
                    return Err(PointerError::InvalidPath(format!(
                        "Cannot navigate through {}",
                        current.kind()
                    )));
                }
            };
        }
        current
    };

    match current {
        Value::Object(map) => {
            if map.shift_remove(last).is_none() {
                return Err(PointerError::NotFound(format!("/{}", last)));
            }
        }
        Value::Array(arr) => {
            let index: usize = last
                .parse()
                .map_err(|_| PointerError::InvalidPath(format!("Invalid array index: {}", last)))?;
            if index >= arr.len() {
                return Err(PointerError::NotFound(format!("/{}", last)));
            }
            arr.remove(index);
        }
        _ => {
            return Err(PointerError::InvalidPath(format!(
                "Cannot delete from {}",
                current.kind()
            )));
        }
    }

    Ok(())
}

/// Returns `true` if a value exists at the given JSON Pointer path.
///
/// # Examples
///
/// ```
/// use typebox::Value;
/// use typebox::value::pointer::has_pointer;
///
/// let value = Value::object()
///     .field("name", Value::string("Alice"))
///     .build();
///
/// assert!(has_pointer(&value, "/name"));
/// assert!(!has_pointer(&value, "/age"));
/// assert!(has_pointer(&value, "")); // Root always exists
/// ```
pub fn has_pointer(value: &Value, pointer: &str) -> bool {
    if pointer.is_empty() {
        return true;
    }
    get_pointer(value, pointer).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;

    #[test]
    fn test_parse_pointer() {
        assert_eq!(parse_pointer(""), Vec::<String>::new());
        assert_eq!(parse_pointer("/"), vec![""]);
        assert_eq!(parse_pointer("/a"), vec!["a"]);
        assert_eq!(parse_pointer("/a/b"), vec!["a", "b"]);
        assert_eq!(parse_pointer("/a/b/c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_pointer_escape() {
        assert_eq!(parse_pointer("/~0"), vec!["~"]);
        assert_eq!(parse_pointer("/~1"), vec!["/"]);
        assert_eq!(parse_pointer("/a~0b"), vec!["a~b"]);
        assert_eq!(parse_pointer("/a~1b"), vec!["a/b"]);
        assert_eq!(parse_pointer("/~0~1"), vec!["~/"]);
    }

    #[test]
    fn test_get_pointer_root() {
        let value = Value::int64(42);
        assert_eq!(get_pointer(&value, ""), Some(&Value::int64(42)));
    }

    #[test]
    fn test_get_pointer_object() {
        let value = Value::Object(indexmap! {
            "name".to_string() => Value::string("Alice"),
            "age".to_string() => Value::int64(30),
        });

        assert_eq!(get_pointer(&value, "/name"), Some(&Value::string("Alice")));
        assert_eq!(get_pointer(&value, "/age"), Some(&Value::int64(30)));
        assert_eq!(get_pointer(&value, "/missing"), None);
    }

    #[test]
    fn test_get_pointer_array() {
        let value = Value::array(vec![Value::int64(1), Value::int64(2), Value::int64(3)]);

        assert_eq!(get_pointer(&value, "/0"), Some(&Value::int64(1)));
        assert_eq!(get_pointer(&value, "/1"), Some(&Value::int64(2)));
        assert_eq!(get_pointer(&value, "/2"), Some(&Value::int64(3)));
        assert_eq!(get_pointer(&value, "/3"), None);
        assert_eq!(get_pointer(&value, "/invalid"), None);
    }

    #[test]
    fn test_get_pointer_nested() {
        let inner = Value::Object(indexmap! {
            "city".to_string() => Value::string("NYC"),
        });
        let value = Value::Object(indexmap! {
            "user".to_string() => Value::Object(indexmap! {
                "name".to_string() => Value::string("Bob"),
                "address".to_string() => inner,
            }),
        });

        assert_eq!(
            get_pointer(&value, "/user/name"),
            Some(&Value::string("Bob"))
        );
        assert_eq!(
            get_pointer(&value, "/user/address/city"),
            Some(&Value::string("NYC"))
        );
    }

    #[test]
    fn test_set_pointer_object() {
        let mut value = Value::object().build();
        set_pointer(&mut value, "/name", Value::string("Alice")).unwrap();
        assert_eq!(get_pointer(&value, "/name"), Some(&Value::string("Alice")));
    }

    #[test]
    fn test_set_pointer_nested() {
        let mut value = Value::object().build();
        set_pointer(&mut value, "/user/name", Value::string("Bob")).unwrap();
        assert_eq!(
            get_pointer(&value, "/user/name"),
            Some(&Value::string("Bob"))
        );
    }

    #[test]
    fn test_set_pointer_array() {
        let mut value = Value::array(vec![Value::int64(1), Value::int64(2)]);
        set_pointer(&mut value, "/0", Value::int64(10)).unwrap();
        assert_eq!(get_pointer(&value, "/0"), Some(&Value::int64(10)));
        assert_eq!(get_pointer(&value, "/1"), Some(&Value::int64(2)));
    }

    #[test]
    fn test_set_pointer_empty_error() {
        let mut value = Value::int64(42);
        assert!(matches!(
            set_pointer(&mut value, "", Value::int64(0)),
            Err(PointerError::EmptyPointer)
        ));
    }

    #[test]
    fn test_delete_pointer_object() {
        let mut value = Value::object()
            .field("name", Value::string("Alice"))
            .field("age", Value::int64(30))
            .build();

        delete_pointer(&mut value, "/age").unwrap();
        assert!(!has_pointer(&value, "/age"));
    }

    #[test]
    fn test_delete_pointer_array() {
        let mut value = Value::array(vec![Value::int64(1), Value::int64(2), Value::int64(3)]);
        delete_pointer(&mut value, "/1").unwrap();
        assert_eq!(get_pointer(&value, "/0"), Some(&Value::int64(1)));
        assert_eq!(get_pointer(&value, "/1"), Some(&Value::int64(3)));
    }

    #[test]
    fn test_delete_pointer_not_found() {
        let mut value = Value::object()
            .field("name", Value::string("Alice"))
            .build();
        assert!(matches!(
            delete_pointer(&mut value, "/missing"),
            Err(PointerError::NotFound(_))
        ));
    }

    #[test]
    fn test_has_pointer() {
        let value = Value::object()
            .field("name", Value::string("Alice"))
            .build();

        assert!(has_pointer(&value, ""));
        assert!(has_pointer(&value, "/name"));
        assert!(!has_pointer(&value, "/missing"));
    }

    #[test]
    fn test_get_pointer_mut() {
        let mut value = Value::object().field("count", Value::int64(0)).build();

        if let Some(count) = get_pointer_mut(&mut value, "/count") {
            *count = Value::int64(42);
        }

        assert_eq!(get_pointer(&value, "/count"), Some(&Value::int64(42)));
    }

    #[test]
    fn test_set_pointer_creates_intermediate() {
        let mut value = Value::object().build();
        set_pointer(&mut value, "/a/b/c", Value::int64(42)).unwrap();
        assert_eq!(get_pointer(&value, "/a/b/c"), Some(&Value::int64(42)));
    }

    #[test]
    fn test_set_pointer_invalid_path_through_primitive() {
        let mut value = Value::object().field("name", Value::string("test")).build();
        assert!(matches!(
            set_pointer(&mut value, "/name/invalid", Value::int64(42)),
            Err(PointerError::InvalidPath(_))
        ));
    }
}
