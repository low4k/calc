use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl ParseError {
    pub fn new(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at {}: {}", self.offset, self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalError {
    pub message: String,
}

impl EvalError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomainError {
    pub message: String,
}

impl DomainError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    Parse(ParseError),
    Eval(EvalError),
    Domain(DomainError),
    InvalidInput(String),
    NotImplemented(&'static str),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "{error}"),
            Self::Eval(error) => write!(f, "evaluation error: {error}"),
            Self::Domain(error) => write!(f, "domain error: {error}"),
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::NotImplemented(feature) => write!(f, "feature not implemented yet: {feature}"),
        }
    }
}

impl std::error::Error for ParseError {}
impl std::error::Error for EvalError {}
impl std::error::Error for DomainError {}
impl std::error::Error for EngineError {}

impl From<ParseError> for EngineError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<EvalError> for EngineError {
    fn from(value: EvalError) -> Self {
        Self::Eval(value)
    }
}

impl From<DomainError> for EngineError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}
