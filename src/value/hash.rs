//! FNV-1A 64-bit hash implementation for [`Value`].
//!
//! Provides stable, deterministic hashing for use in HashMaps, caching, and deduplication.

use crate::value::Value;
use std::hash::{Hash, Hasher};

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum TypeMarker {
    Null = 0,
    Bool = 1,
    Int64 = 2,
    Float64 = 3,
    String = 4,
    Bytes = 5,
    Array = 6,
    Object = 7,
    Float32Array = 8,
    Float64Array = 9,
    Int32Array = 10,
    Int64Array = 11,
    UInt8Array = 12,
}

struct Fnv1aHasher {
    state: u64,
}

impl Fnv1aHasher {
    fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    fn write_byte(&mut self, byte: u8) {
        self.state ^= byte as u64;
        self.state = self.state.wrapping_mul(FNV_PRIME);
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_byte(byte);
        }
    }

    fn write_i64(&mut self, value: i64) {
        self.write_u64(value as u64);
    }

    fn write_f64(&mut self, value: f64) {
        let bits = if value.is_nan() {
            u64::MAX
        } else {
            value.to_bits()
        };
        self.write_u64(bits);
    }

    fn write_f32(&mut self, value: f32) {
        let bits = if value.is_nan() {
            u32::MAX as u64
        } else {
            value.to_bits() as u64
        };
        self.write_u64(bits);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_byte(*byte);
        }
    }

    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn write_marker(&mut self, marker: TypeMarker) {
        self.write_byte(marker as u8);
    }

    fn finish(self) -> u64 {
        self.state
    }
}

/// Computes an FNV-1A 64-bit hash of a [`Value`].
///
/// The hash is stable and deterministic:
/// - Object keys are sorted alphabetically
/// - NaN values hash to a consistent value
/// - Different Value types have different type markers
///
/// # Examples
///
/// ```
/// use typebox::{Value, value::hash::hash_fnv1a};
///
/// let v1 = Value::int64(42);
/// let v2 = Value::int64(42);
/// assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));
/// ```
pub fn hash_fnv1a(value: &Value) -> u64 {
    let mut hasher = Fnv1aHasher::new();
    hash_value(value, &mut hasher);
    hasher.finish()
}

fn hash_value(value: &Value, hasher: &mut Fnv1aHasher) {
    match value {
        Value::Null => {
            hasher.write_marker(TypeMarker::Null);
        }
        Value::Bool(b) => {
            hasher.write_marker(TypeMarker::Bool);
            hasher.write_byte(if *b { 1 } else { 0 });
        }
        Value::Int64(n) => {
            hasher.write_marker(TypeMarker::Int64);
            hasher.write_i64(*n);
        }
        Value::Float64(f) => {
            hasher.write_marker(TypeMarker::Float64);
            hasher.write_f64(*f);
        }
        Value::String(s) => {
            hasher.write_marker(TypeMarker::String);
            hasher.write_str(s);
        }
        Value::Bytes(b) => {
            hasher.write_marker(TypeMarker::Bytes);
            hasher.write_bytes(b);
        }
        Value::Array(arr) => {
            hasher.write_marker(TypeMarker::Array);
            hasher.write_u64(arr.len() as u64);
            for item in arr {
                hash_value(item, hasher);
            }
        }
        Value::Object(obj) => {
            hasher.write_marker(TypeMarker::Object);
            hasher.write_u64(obj.len() as u64);
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            for key in keys {
                hasher.write_str(key);
                hash_value(&obj[key], hasher);
            }
        }
        Value::Float32Array(arr) => {
            hasher.write_marker(TypeMarker::Float32Array);
            hasher.write_u64(arr.len() as u64);
            for f in arr {
                hasher.write_f32(*f);
            }
        }
        Value::Float64Array(arr) => {
            hasher.write_marker(TypeMarker::Float64Array);
            hasher.write_u64(arr.len() as u64);
            for f in arr {
                hasher.write_f64(*f);
            }
        }
        Value::Int32Array(arr) => {
            hasher.write_marker(TypeMarker::Int32Array);
            hasher.write_u64(arr.len() as u64);
            for n in arr {
                hasher.write_u64(*n as u64);
            }
        }
        Value::Int64Array(arr) => {
            hasher.write_marker(TypeMarker::Int64Array);
            hasher.write_u64(arr.len() as u64);
            for n in arr {
                hasher.write_i64(*n);
            }
        }
        Value::UInt8Array(arr) => {
            hasher.write_marker(TypeMarker::UInt8Array);
            hasher.write_u64(arr.len() as u64);
            hasher.write_bytes(arr);
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(hash_fnv1a(self));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;

    #[test]
    fn test_hash_primitives() {
        let v1 = Value::Null;
        let v2 = Value::Null;
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));

        let v1 = Value::bool(true);
        let v2 = Value::bool(true);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));

        let v1 = Value::int64(42);
        let v2 = Value::int64(42);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }

    #[test]
    fn test_hash_different_types() {
        let v1 = Value::int64(42);
        let v2 = Value::float64(42.0);
        assert_ne!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }

    #[test]
    fn test_hash_string() {
        let v1 = Value::string("hello");
        let v2 = Value::string("hello");
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));

        let v3 = Value::string("world");
        assert_ne!(hash_fnv1a(&v1), hash_fnv1a(&v3));
    }

    #[test]
    fn test_hash_array() {
        let v1 = Value::array(vec![Value::int64(1), Value::int64(2)]);
        let v2 = Value::array(vec![Value::int64(1), Value::int64(2)]);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));

        let v3 = Value::array(vec![Value::int64(2), Value::int64(1)]);
        assert_ne!(hash_fnv1a(&v1), hash_fnv1a(&v3));
    }

    #[test]
    fn test_hash_object_key_order() {
        let v1 = Value::Object(indexmap! {
            "a".to_string() => Value::int64(1),
            "b".to_string() => Value::int64(2),
        });
        let v2 = Value::Object(indexmap! {
            "b".to_string() => Value::int64(2),
            "a".to_string() => Value::int64(1),
        });
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }

    #[test]
    fn test_hash_nan() {
        let v1 = Value::float64(f64::NAN);
        let v2 = Value::float64(f64::NAN);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }

    #[test]
    fn test_hash_negative_zero() {
        let v1 = Value::float64(0.0);
        let v2 = Value::float64(-0.0);
        assert_ne!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }

    #[test]
    fn test_hash_bytes() {
        let v1 = Value::bytes(vec![1, 2, 3]);
        let v2 = Value::bytes(vec![1, 2, 3]);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));

        let v3 = Value::bytes(vec![3, 2, 1]);
        assert_ne!(hash_fnv1a(&v1), hash_fnv1a(&v3));
    }

    #[test]
    fn test_hash_typed_arrays() {
        let v1 = Value::Float32Array(vec![1.0, 2.0, 3.0]);
        let v2 = Value::Float32Array(vec![1.0, 2.0, 3.0]);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));

        let v1 = Value::Int64Array(vec![1, 2, 3]);
        let v2 = Value::Int64Array(vec![1, 2, 3]);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }

    #[test]
    fn test_std_hash_trait() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let v1 = Value::int64(42);
        let v2 = Value::int64(42);
        let v3 = Value::int64(43);

        set.insert(v1.clone());
        assert!(set.contains(&v2));
        assert!(!set.contains(&v3));
    }

    #[test]
    fn test_nested_value() {
        let inner = Value::Object(indexmap! {
            "x".to_string() => Value::int64(10),
            "y".to_string() => Value::int64(20),
        });
        let v1 = Value::array(vec![inner.clone(), Value::string("test")]);
        let v2 = Value::array(vec![inner, Value::string("test")]);
        assert_eq!(hash_fnv1a(&v1), hash_fnv1a(&v2));
    }
}
