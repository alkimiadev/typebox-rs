//! Diff computation between values.

use crate::value::Value;
use indexmap::IndexMap;

/// A single edit operation in a delta.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Insert a new value at the path.
    Insert {
        /// JSON pointer path.
        path: String,
        /// Value to insert.
        value: Value,
    },
    /// Update the value at the path.
    Update {
        /// JSON pointer path.
        path: String,
        /// New value.
        value: Value,
    },
    /// Delete the value at the path.
    Delete {
        /// JSON pointer path.
        path: String,
    },
}

/// A sequence of edit operations.
pub type Delta = Vec<Edit>;

/// Compute the difference between two values.
///
/// Returns a list of [`Edit`] operations using JSON pointer paths.
/// Use [`patch()`](crate::patch()) to apply the edits.
pub fn delta(current: &Value, next: &Value) -> Delta {
    let mut edits = Vec::new();
    collect_edits("", current, next, &mut edits);
    edits
}

fn collect_edits(path: &str, current: &Value, next: &Value, edits: &mut Delta) {
    match (current, next) {
        (Value::Object(curr_map), Value::Object(next_map)) => {
            diff_objects(path, curr_map, next_map, edits);
        }
        (Value::Array(curr_arr), Value::Array(next_arr)) => {
            diff_arrays(path, curr_arr, next_arr, edits);
        }
        (Value::Float32Array(curr_arr), Value::Float32Array(next_arr)) => {
            diff_typed_arrays(path, "Float32Array", curr_arr, next_arr, edits);
        }
        (Value::Float64Array(curr_arr), Value::Float64Array(next_arr)) => {
            diff_typed_arrays(path, "Float64Array", curr_arr, next_arr, edits);
        }
        (Value::Int32Array(curr_arr), Value::Int32Array(next_arr)) => {
            diff_typed_arrays(path, "Int32Array", curr_arr, next_arr, edits);
        }
        (Value::Int64Array(curr_arr), Value::Int64Array(next_arr)) => {
            diff_typed_arrays(path, "Int64Array", curr_arr, next_arr, edits);
        }
        (Value::UInt8Array(curr_arr), Value::UInt8Array(next_arr)) => {
            diff_typed_arrays(path, "UInt8Array", curr_arr, next_arr, edits);
        }
        (Value::Bytes(curr_arr), Value::Bytes(next_arr)) => {
            diff_typed_arrays(path, "Bytes", curr_arr, next_arr, edits);
        }
        _ => {
            if !values_equal(current, next) {
                edits.push(Edit::Update {
                    path: path.to_string(),
                    value: next.clone(),
                });
            }
        }
    }
}

fn diff_objects(
    path: &str,
    current: &IndexMap<String, Value>,
    next: &IndexMap<String, Value>,
    edits: &mut Delta,
) {
    let curr_keys: std::collections::HashSet<_> = current.keys().collect();
    let next_keys: std::collections::HashSet<_> = next.keys().collect();

    for key in &next_keys {
        if !curr_keys.contains(key) {
            edits.push(Edit::Insert {
                path: format!("{}/{}", path, key),
                value: next[*key].clone(),
            });
        }
    }

    for key in &curr_keys {
        if next_keys.contains(key) {
            let child_path = format!("{}/{}", path, key);
            collect_edits(&child_path, &current[*key], &next[*key], edits);
        }
    }

    for key in &curr_keys {
        if !next_keys.contains(key) {
            edits.push(Edit::Delete {
                path: format!("{}/{}", path, key),
            });
        }
    }
}

fn diff_arrays(path: &str, current: &[Value], next: &[Value], edits: &mut Delta) {
    let min_len = current.len().min(next.len());

    for i in 0..min_len {
        let child_path = format!("{}/{}", path, i);
        collect_edits(&child_path, &current[i], &next[i], edits);
    }

    for (i, val) in next.iter().enumerate().skip(min_len) {
        edits.push(Edit::Insert {
            path: format!("{}/{}", path, i),
            value: val.clone(),
        });
    }

    for i in (min_len..current.len()).rev() {
        edits.push(Edit::Delete {
            path: format!("{}/{}", path, i),
        });
    }
}

fn diff_typed_arrays<T: PartialEq + Clone>(
    path: &str,
    type_name: &str,
    current: &[T],
    next: &[T],
    edits: &mut Delta,
) {
    if current.len() != next.len() || current != next {
        edits.push(Edit::Update {
            path: path.to_string(),
            value: Value::String(format!("{} changed", type_name)),
        });
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    a == b
}

/// Returns a human-readable summary of delta edits.
pub fn diff_summary(edits: &[Edit]) -> String {
    let inserts = edits
        .iter()
        .filter(|e| matches!(e, Edit::Insert { .. }))
        .count();
    let updates = edits
        .iter()
        .filter(|e| matches!(e, Edit::Update { .. }))
        .count();
    let deletes = edits
        .iter()
        .filter(|e| matches!(e, Edit::Delete { .. }))
        .count();
    format!(
        "Delta: {} inserts, {} updates, {} deletes",
        inserts, updates, deletes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_no_change() {
        let a = Value::Int64(42);
        let b = Value::Int64(42);
        assert_eq!(delta(&a, &b), vec![]);
    }

    #[test]
    fn test_delta_update_primitive() {
        let a = Value::Int64(42);
        let b = Value::Int64(43);
        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Update { path, value } if path == "" && *value == Value::Int64(43)
        ));
    }

    #[test]
    fn test_delta_object_insert() {
        let a = Value::object().field("id", Value::Int64(1)).build();
        let b = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("test".to_string()))
            .build();

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Insert { path, .. } if path == "/name"
        ));
    }

    #[test]
    fn test_delta_object_delete() {
        let a = Value::object()
            .field("id", Value::Int64(1))
            .field("name", Value::String("test".to_string()))
            .build();
        let b = Value::object().field("id", Value::Int64(1)).build();

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Delete { path } if path == "/name"
        ));
    }

    #[test]
    fn test_delta_object_update() {
        let a = Value::object().field("id", Value::Int64(1)).build();
        let b = Value::object().field("id", Value::Int64(2)).build();

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Update { path, value } if path == "/id" && *value == Value::Int64(2)
        ));
    }

    #[test]
    fn test_delta_array_insert() {
        let a = Value::Array(vec![Value::Int64(1)]);
        let b = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Insert { path, value } if path == "/1" && *value == Value::Int64(2)
        ));
    }

    #[test]
    fn test_delta_array_delete() {
        let a = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);
        let b = Value::Array(vec![Value::Int64(1)]);

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Delete { path } if path == "/1"
        ));
    }

    #[test]
    fn test_delta_array_update() {
        let a = Value::Array(vec![Value::Int64(1), Value::Int64(2)]);
        let b = Value::Array(vec![Value::Int64(1), Value::Int64(3)]);

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Update { path, value } if path == "/1" && *value == Value::Int64(3)
        ));
    }

    #[test]
    fn test_delta_nested() {
        let inner_a = Value::object().field("x", Value::Int64(1)).build();
        let inner_b = Value::object().field("x", Value::Int64(2)).build();

        let a = Value::object().field("nested", inner_a).build();
        let b = Value::object().field("nested", inner_b).build();

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Update { path, value } if path == "/nested/x" && *value == Value::Int64(2)
        ));
    }

    #[test]
    fn test_delta_type_change() {
        let a = Value::Int64(42);
        let b = Value::String("42".to_string());

        let edits = delta(&a, &b);
        assert_eq!(edits.len(), 1);
        assert!(matches!(
            &edits[0],
            Edit::Update { path, .. } if path == ""
        ));
    }

    #[test]
    fn test_diff_summary() {
        let edits = vec![
            Edit::Insert {
                path: "/a".to_string(),
                value: Value::Int64(1),
            },
            Edit::Update {
                path: "/b".to_string(),
                value: Value::Int64(2),
            },
            Edit::Delete {
                path: "/c".to_string(),
            },
        ];
        assert_eq!(
            diff_summary(&edits),
            "Delta: 1 inserts, 1 updates, 1 deletes"
        );
    }
}
