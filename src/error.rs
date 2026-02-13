use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Code generation error: {0}")]
    Codegen(String),

    #[error("Template error: {0}")]
    Template(#[from] handlebars::RenderError),

    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
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

    #[error("No matching variant in union")]
    NoMatchingVariant,

    #[error("Invalid literal value")]
    InvalidLiteral,

    #[error("Value not in enum: {0}")]
    NotInEnum(String),

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
