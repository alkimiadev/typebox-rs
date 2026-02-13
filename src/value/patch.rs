use crate::error::PatchError;
use crate::value::{clone, delta::Edit, Value};

pub fn patch(value: &Value, edits: &[Edit]) -> Result<Value, PatchError> {
    if edits.is_empty() {
        return Ok(clone(value));
    }

    if is_root_update(edits) {
        if let Edit::Update {
            value: new_value, ..
        } = &edits[0]
        {
            return Ok(clone(new_value));
        }
    }

    let mut result = clone(value);
    for edit in edits {
        apply_edit(&mut result, edit)?;
    }
    Ok(result)
}

fn is_root_update(edits: &[Edit]) -> bool {
    edits.len() == 1
        && matches!(
            &edits[0],
            Edit::Update { path, .. } if path.is_empty()
        )
}

fn apply_edit(value: &mut Value, edit: &Edit) -> Result<(), PatchError> {
    match edit {
        Edit::Insert {
            path,
            value: new_value,
        } => apply_insert(value, path, new_value.clone()),
        Edit::Update {
            path,
            value: new_value,
        } => apply_update(value, path, new_value.clone()),
        Edit::Delete { path } => apply_delete(value, path),
    }
}

fn apply_insert(value: &mut Value, path: &str, new_value: Value) -> Result<(), PatchError> {
    if path.is_empty() {
        return Err(PatchError::InvalidPath("cannot insert at root".to_string()));
    }

    let (parent_path, key) = split_path(path)?;
    let parent = get_value_mut(value, parent_path)?;

    match parent {
        Value::Object(map) => {
            map.insert(key.to_string(), new_value);
            Ok(())
        }
        Value::Array(arr) => {
            let idx: usize = key
                .parse()
                .map_err(|_| PatchError::InvalidPath(format!("invalid array index: {}", key)))?;
            if idx <= arr.len() {
                arr.insert(idx, new_value);
                Ok(())
            } else {
                Err(PatchError::InvalidPath(format!(
                    "array index {} out of bounds",
                    idx
                )))
            }
        }
        _ => Err(PatchError::TypeMismatch {
            path: parent_path.to_string(),
            message: "expected object or array".to_string(),
        }),
    }
}

fn apply_update(value: &mut Value, path: &str, new_value: Value) -> Result<(), PatchError> {
    if path.is_empty() {
        return Ok(());
    }

    let (parent_path, key) = split_path(path)?;
    let parent = get_value_mut(value, parent_path)?;

    match parent {
        Value::Object(map) => {
            if map.contains_key(key) {
                map.insert(key.to_string(), new_value);
                Ok(())
            } else {
                Err(PatchError::InvalidPath(format!(
                    "key '{}' not found in object",
                    key
                )))
            }
        }
        Value::Array(arr) => {
            let idx: usize = key
                .parse()
                .map_err(|_| PatchError::InvalidPath(format!("invalid array index: {}", key)))?;
            if idx < arr.len() {
                arr[idx] = new_value;
                Ok(())
            } else {
                Err(PatchError::InvalidPath(format!(
                    "array index {} out of bounds",
                    idx
                )))
            }
        }
        _ => Err(PatchError::TypeMismatch {
            path: parent_path.to_string(),
            message: "expected object or array".to_string(),
        }),
    }
}

fn apply_delete(value: &mut Value, path: &str) -> Result<(), PatchError> {
    if path.is_empty() {
        return Err(PatchError::InvalidPath("cannot delete root".to_string()));
    }

    let (parent_path, key) = split_path(path)?;
    let parent = get_value_mut(value, parent_path)?;

    match parent {
        Value::Object(map) => {
            map.shift_remove(key);
            Ok(())
        }
        Value::Array(arr) => {
            let idx: usize = key
                .parse()
                .map_err(|_| PatchError::InvalidPath(format!("invalid array index: {}", key)))?;
            if idx < arr.len() {
                arr.remove(idx);
                Ok(())
            } else {
                Err(PatchError::InvalidPath(format!(
                    "array index {} out of bounds",
                    idx
                )))
            }
        }
        _ => Err(PatchError::TypeMismatch {
            path: parent_path.to_string(),
            message: "expected object or array".to_string(),
        }),
    }
}

fn split_path(path: &str) -> Result<(&str, &str), PatchError> {
    let path = path.trim_start_matches('/');
    let last_slash = path.rfind('/');
    match last_slash {
        Some(pos) => {
            let parent = &path[..pos];
            let key = &path[pos + 1..];
            let parent = if parent.is_empty() { "" } else { parent };
            Ok((parent, key))
        }
        None => Ok(("", path)),
    }
}

fn get_value_mut<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Value, PatchError> {
    if path.is_empty() {
        return Ok(value);
    }

    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let mut current = value;

    for part in parts {
        current = match current {
            Value::Object(map) => map
                .get_mut(part)
                .ok_or_else(|| PatchError::InvalidPath(format!("key '{}' not found", part)))?,
            Value::Array(arr) => {
                let idx: usize = part.parse().map_err(|_| {
                    PatchError::InvalidPath(format!("invalid array index: {}", part))
                })?;
                arr.get_mut(idx).ok_or_else(|| {
                    PatchError::InvalidPath(format!("array index {} out of bounds", idx))
                })?
            }
            _ => {
                return Err(PatchError::TypeMismatch {
                    path: part.to_string(),
                    message: "expected object or array".to_string(),
                })
            }
        };
    }

    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::delta::{delta, Edit};

    #[test]
    fn test_patch_no_edits() {
        let value = Value::Int64(42);
        let result = patch(&value, &[]).unwrap();
        assert_eq!(result, Value::Int64(42));
    }

    #[test]
    fn test_patch_root_update() {
        let value = Value::Int64(42);
        let edits = vec![Edit::Update {
            path: "".to_string(),
            value: Value::Int64(43),
        }];
        let result = patch(&value, &edits).unwrap();
        assert_eq!(result, Value::Int64(43));
    }

    #[test]
    fn test_patch_object_insert() {
        let value = Value::object().field("id", Value::Int64(1)).build();
        let edits = vec![Edit::Insert {
            path: "/name".to_string(),
            value: Value::String("test".to_string()),
        }];
        let result = patch(&value, &edits).unwrap();
        assert_eq!(
            result.as_object().unwrap().get("name"),
            Some(&Value::String("test".to_string()))
        );
    }

    #[test]
    fn test_patch_object_update() {
        let value = Value::object().field("id", Value::Int64(1)).build();
        let edits = vec![Edit::Update {
            path: "/id".to_string(),
            value: Value::Int64(2),
        }];
        let result = patch(&value, &edits).unwrap();
        assert_eq!(
            result.as_object().unwrap().get("id"),
            Some(&Value::Int64(2))
        );
    }

    #[test]
    fn test_patch_object_delete() {
        let value = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("test".to_string()))
            .build();
        let edits = vec![Edit::Delete {
            path: "/name".to_string(),
        }];
        let result = patch(&value, &edits).unwrap();
        assert!(!result.as_object().unwrap().contains_key("name"));
    }

    #[test]
    fn test_patch_array_insert() {
        let value = Value::Array(vec![Value::Int64(1)]);
        let edits = vec![Edit::Insert {
            path: "/1".to_string(),
            value: Value::Int64(2),
        }];
        let result = patch(&value, &edits).unwrap();
        assert_eq!(result, Value::Array(vec![Value::Int64(1), Value::Int64(2)]));
    }

    #[test]
    fn test_patch_array_update() {
        let value = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);
        let edits = vec![Edit::Update {
            path: "/1".to_string(),
            value: Value::Int64(3),
        }];
        let result = patch(&value, &edits).unwrap();
        assert_eq!(result, Value::Array(vec![Value::Int64(1), Value::Int64(3)]));
    }

    #[test]
    fn test_patch_array_delete() {
        let value = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);
        let edits = vec![Edit::Delete {
            path: "/1".to_string(),
        }];
        let result = patch(&value, &edits).unwrap();
        assert_eq!(result, Value::Array(vec![Value::Int64(1)]));
    }

    #[test]
    fn test_patch_nested() {
        let inner = Value::object().field("x", Value::Int64(1)).build();
        let value = Value::object().field("nested", inner).build();

        let edits = vec![Edit::Update {
            path: "/nested/x".to_string(),
            value: Value::Int64(2),
        }];
        let result = patch(&value, &edits).unwrap();

        let nested = result.as_object().unwrap().get("nested").unwrap();
        assert_eq!(nested.as_object().unwrap().get("x"), Some(&Value::Int64(2)));
    }

    #[test]
    fn test_patch_roundtrip() {
        let a = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("test".to_string()))
            .field(
                "items",
                Value::Array(vec![Value::Int64(1), Value::Int64(2)]),
            )
            .build();

        let b = Value::object()
            .field("id", Value::Int64(2))
            .field("name", Value::String("updated".to_string()))
            .field(
                "items",
                Value::Array(vec![Value::Int64(1), Value::Int64(3), Value::Int64(4)]),
            )
            .build();

        let edits = delta(&a, &b);
        let result = patch(&a, &edits).unwrap();

        assert_eq!(result, b);
    }

    #[test]
    fn test_patch_invalid_path() {
        let value = Value::Int64(42);
        let edits = vec![Edit::Insert {
            path: "/foo".to_string(),
            value: Value::String("test".to_string()),
        }];
        assert!(patch(&value, &edits).is_err());
    }
}
