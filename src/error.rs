//! Error types for schema validation and value operations.

use thiserror::Error;

/// Top-level error type combining all error variants.
#[derive(Debug, Error)]
pub enum Error {
    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Template rendering error (codegen feature).
    #[cfg(feature = "codegen")]
    #[error("Template error: {0}")]
    Template(#[from] handlebars::RenderError),

    /// Schema not found in registry.
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
}

/// Parse errors for value construction.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Type mismatch during parsing.
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        /// Expected type.
        expected: String,
        /// Actual type received.
        got: String,
    },

    /// Missing required field.
    #[error("Missing required field: {field}")]
    MissingField {
        /// Field name.
        field: String,
    },

    /// Invalid length.
    #[error("Invalid length: expected {expected}, got {got}")]
    InvalidLength {
        /// Expected length.
        expected: usize,
        /// Actual length.
        got: usize,
    },

    /// No matching union variant.
    #[error("No matching variant in union")]
    NoMatchingVariant,

    /// Literal value mismatch.
    #[error("Literal value mismatch")]
    LiteralMismatch,

    /// Enum value not in allowed values.
    #[error("Enum value mismatch: got {got}, expected one of {allowed:?}")]
    EnumMismatch {
        /// Allowed values.
        allowed: Vec<String>,
        /// Value received.
        got: String,
    },

    /// Invalid regex pattern.
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Validation errors with detailed context.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Type mismatch.
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        /// Expected type.
        expected: String,
        /// Actual type received.
        actual: String,
    },

    /// Missing required field.
    #[error("Missing required field: {field}")]
    MissingField {
        /// Field name.
        field: String,
    },

    /// Unknown field not in schema.
    #[error("Unknown field: {field}")]
    UnknownField {
        /// Field name.
        field: String,
    },

    /// Array below minimum items.
    #[error("Array min items: expected at least {min}, got {actual}")]
    MinItems {
        /// Minimum items.
        min: usize,
        /// Actual item count.
        actual: usize,
    },

    /// Array above maximum items.
    #[error("Array max items: expected at most {max}, got {actual}")]
    MaxItems {
        /// Maximum items.
        max: usize,
        /// Actual item count.
        actual: usize,
    },

    /// String below minimum length.
    #[error("String min length: expected at least {min}, got {actual}")]
    MinLength {
        /// Minimum length.
        min: usize,
        /// Actual length.
        actual: usize,
    },

    /// String above maximum length.
    #[error("String max length: expected at most {max}, got {actual}")]
    MaxLength {
        /// Maximum length.
        max: usize,
        /// Actual length.
        actual: usize,
    },

    /// Number below minimum.
    #[error("Number below minimum: {value} < {minimum}")]
    BelowMinimum {
        /// Actual value.
        value: f64,
        /// Minimum value.
        minimum: f64,
    },

    /// Number above maximum.
    #[error("Number above maximum: {value} > {maximum}")]
    AboveMaximum {
        /// Actual value.
        value: f64,
        /// Maximum value.
        maximum: f64,
    },

    /// No matching union variant.
    #[error("No matching variant in union")]
    NoMatchingVariant,

    /// Invalid literal value.
    #[error("Invalid literal value")]
    InvalidLiteral,

    /// Value not in enum.
    #[error("Value not in enum: {0}")]
    NotInEnum(String),

    /// String format validation failed.
    #[error("Invalid format '{format}': value '{value}' does not match")]
    InvalidFormat {
        /// Format name.
        format: String,
        /// Value that failed validation.
        value: String,
    },

    /// Duplicate item in array with uniqueItems.
    #[error("Duplicate item in array with uniqueItems constraint")]
    DuplicateItem,

    /// String pattern validation failed.
    #[error("String does not match pattern '{pattern}': '{value}'")]
    PatternMismatch {
        /// Pattern regex.
        pattern: String,
        /// Value that failed validation.
        value: String,
    },

    /// Invalid regex pattern.
    #[error("Invalid regex pattern: '{pattern}'")]
    InvalidPattern {
        /// Invalid pattern.
        pattern: String,
    },

    /// Error at a specific path.
    #[error("At path '{path}': {inner}")]
    AtPath {
        /// JSON path.
        path: String,
        /// Inner error.
        #[source]
        inner: Box<ValidationError>,
    },
}

impl ValidationError {
    /// Wraps error with a path segment.
    pub fn with_path(self, segment: impl Into<String>) -> Self {
        ValidationError::AtPath {
            path: segment.into(),
            inner: Box::new(self),
        }
    }
}

/// Errors from fake value generation.
#[derive(Debug, Error)]
pub enum FakeError {
    /// Cannot generate value for schema type.
    #[error("Cannot generate value for schema: {0}")]
    UnsupportedSchema(String),

    /// Maximum recursion depth exceeded.
    #[error("Maximum recursion depth exceeded")]
    MaxDepthExceeded,

    /// Invalid regex pattern.
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Errors from value creation.
#[derive(Debug, Error)]
pub enum CreateError {
    /// Cannot create value for schema type.
    #[error("Cannot create value for schema: {0}")]
    UnsupportedSchema(String),

    /// Recursive type without default value.
    #[error("Recursive type without default: {0}")]
    RecursiveWithoutDefault(String),
}

/// Errors from value casting.
#[derive(Debug, Error)]
pub enum CastError {
    /// Cannot cast value to schema.
    #[error("Cannot cast value: {0}")]
    CannotCast(String),
}

/// Errors from patch operations.
#[derive(Debug, Error)]
pub enum PatchError {
    /// Invalid patch path.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Type mismatch during patch.
    #[error("Type mismatch at {path}: {message}")]
    TypeMismatch {
        /// Path where mismatch occurred.
        path: String,
        /// Error message.
        message: String,
    },
}

/// Errors from clean operations.
#[derive(Debug, Error)]
pub enum CleanError {
    /// Cannot clean value.
    #[error("Cannot clean value: {0}")]
    CannotClean(String),
}

/// Errors from pointer operations.
#[derive(Debug, Error)]
pub enum PointerError {
    /// Cannot operate on empty pointer.
    #[error("Cannot operate on empty pointer (root)")]
    EmptyPointer,

    /// Path not found in value.
    #[error("Path not found: {0}")]
    NotFound(String),

    /// Invalid pointer path.
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Errors from mutate operations.
#[derive(Debug, Error)]
pub enum MutateError {
    /// Only objects and arrays can be mutated at root.
    #[error("Only objects and arrays can be mutated at root level")]
    NotMutable,

    /// Type mismatch during mutation.
    #[error("Cannot mutate: type mismatch between current and next")]
    TypeMismatch,
}

/// Errors from schema registry operations.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Schema not found in registry.
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    /// Circular reference detected.
    #[error("Circular reference detected: {0}")]
    CircularRef(String),
}
