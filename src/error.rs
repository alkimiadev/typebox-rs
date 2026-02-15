use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "codegen")]
    #[error("Template error: {0}")]
    Template(#[from] handlebars::RenderError),

    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error("No matching variant in union")]
    NoMatchingVariant,

    #[error("Literal value mismatch")]
    LiteralMismatch,

    #[error("Enum value mismatch: got {got}, expected one of {allowed:?}")]
    EnumMismatch { allowed: Vec<String>, got: String },

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Unknown field: {field}")]
    UnknownField { field: String },

    #[error("Array min items: expected at least {min}, got {actual}")]
    MinItems { min: usize, actual: usize },

    #[error("Array max items: expected at most {max}, got {actual}")]
    MaxItems { max: usize, actual: usize },

    #[error("String min length: expected at least {min}, got {actual}")]
    MinLength { min: usize, actual: usize },

    #[error("String max length: expected at most {max}, got {actual}")]
    MaxLength { max: usize, actual: usize },

    #[error("Number below minimum: {value} < {minimum}")]
    BelowMinimum { value: f64, minimum: f64 },

    #[error("Number above maximum: {value} > {maximum}")]
    AboveMaximum { value: f64, maximum: f64 },

    #[error("No matching variant in union")]
    NoMatchingVariant,

    #[error("Invalid literal value")]
    InvalidLiteral,

    #[error("Value not in enum: {0}")]
    NotInEnum(String),

    #[error("Invalid format '{format}': value '{value}' does not match")]
    InvalidFormat { format: String, value: String },

    #[error("Duplicate item in array with uniqueItems constraint")]
    DuplicateItem,

    #[error("At path '{path}': {inner}")]
    AtPath {
        path: String,
        #[source]
        inner: Box<ValidationError>,
    },
}

impl ValidationError {
    pub fn with_path(self, segment: impl Into<String>) -> Self {
        ValidationError::AtPath {
            path: segment.into(),
            inner: Box::new(self),
        }
    }
}

#[derive(Debug, Error)]
pub enum FakeError {
    #[error("Cannot generate value for schema: {0}")]
    UnsupportedSchema(String),

    #[error("Maximum recursion depth exceeded")]
    MaxDepthExceeded,

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("Cannot create value for schema: {0}")]
    UnsupportedSchema(String),

    #[error("Recursive type without default: {0}")]
    RecursiveWithoutDefault(String),
}

#[derive(Debug, Error)]
pub enum CastError {
    #[error("Cannot cast value: {0}")]
    CannotCast(String),
}

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Type mismatch at {path}: {message}")]
    TypeMismatch { path: String, message: String },
}

#[derive(Debug, Error)]
pub enum CleanError {
    #[error("Cannot clean value: {0}")]
    CannotClean(String),
}

#[derive(Debug, Error)]
pub enum PointerError {
    #[error("Cannot operate on empty pointer (root)")]
    EmptyPointer,

    #[error("Path not found: {0}")]
    NotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

#[derive(Debug, Error)]
pub enum MutateError {
    #[error("Only objects and arrays can be mutated at root level")]
    NotMutable,

    #[error("Cannot mutate: type mismatch between current and next")]
    TypeMismatch,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Circular reference detected: {0}")]
    CircularRef(String),
}
