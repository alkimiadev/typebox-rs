//! Format registry for custom string format validation.
//!
//! TypeBox does not validate formats by default. Users must register
//! validation closures for each format they want to support.
//!
//! # Examples
//!
//! ```
//! use typebox::FormatRegistry;
//!
//! let mut registry = FormatRegistry::new();
//!
//! // Register a custom email validator
//! registry.register("email", |s| s.contains('@'));
//!
//! // Check if a format is registered
//! assert!(registry.has("email"));
//!
//! // Validate a string
//! assert!(registry.validate("email", "test@example.com").unwrap());
//! assert!(!registry.validate("email", "invalid").unwrap());
//! ```

use std::collections::HashMap;

/// Function type for format validation.
pub type FormatValidator = fn(&str) -> bool;

/// Registry for custom string format validators.
#[derive(Clone)]
pub struct FormatRegistry {
    formats: HashMap<String, FormatValidator>,
}

impl FormatRegistry {
    /// Creates a new empty format registry.
    pub fn new() -> Self {
        Self {
            formats: HashMap::new(),
        }
    }

    /// Registers a format validator.
    pub fn register(&mut self, name: impl Into<String>, validator: FormatValidator) {
        self.formats.insert(name.into(), validator);
    }

    /// Returns the validator for a format, if registered.
    pub fn get(&self, name: &str) -> Option<&FormatValidator> {
        self.formats.get(name)
    }

    /// Checks if a format is registered.
    pub fn has(&self, name: &str) -> bool {
        self.formats.contains_key(name)
    }

    /// Removes all registered formats.
    pub fn clear(&mut self) {
        self.formats.clear();
    }

    /// Validates a string against a format.
    ///
    /// Returns `None` if the format is not registered (passes by default).
    /// Returns `Some(bool)` with the validation result if registered.
    pub fn validate(&self, format: &str, value: &str) -> Option<bool> {
        self.formats.get(format).map(|validator| validator(value))
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_validate() {
        let mut registry = FormatRegistry::new();
        registry.register("email", |s| s.contains('@') && s.contains('.'));

        assert!(registry.has("email"));
        assert!(!registry.has("unknown"));

        assert!(registry.validate("email", "test@example.com").unwrap());
        assert!(!registry.validate("email", "invalid").unwrap());
    }

    #[test]
    fn test_unregistered_format() {
        let registry = FormatRegistry::new();
        assert!(registry.validate("email", "test@example.com").is_none());
    }

    #[test]
    fn test_clear() {
        let mut registry = FormatRegistry::new();
        registry.register("email", |_| true);
        assert!(registry.has("email"));

        registry.clear();
        assert!(!registry.has("email"));
    }
}
