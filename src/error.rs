/// Type of the builder error that occurred when building an object
#[derive(Debug, Clone)]
pub enum BuilderErrorType {
    MissingValue(&'static str),
    IncorrectValue(&'static str)
}

/// BuilderError struct that contains the type of error that occurred
/// when using a builder
#[derive(Debug, Clone)]
pub struct BuilderError {
    error_type: BuilderErrorType,
}

impl BuilderError {
    /// Create a new BuilderError struct
    pub fn new(error_type: BuilderErrorType) -> Self {
        BuilderError { error_type }
    }
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.error_type {
            BuilderErrorType::MissingValue(val) => write!(f, "missing field {}", val),
            BuilderErrorType::IncorrectValue(val) => write!(f, "incorrect value for {}", val)
        }
    }
}

impl std::error::Error for BuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
