use crate::error::EngineError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub start: f64,
    pub end: f64,
}

impl Interval {
    pub fn new(start: f64, end: f64) -> Result<Self, EngineError> {
        if !start.is_finite() || !end.is_finite() {
            return Err(EngineError::InvalidInput(
                "interval bounds must be finite".to_string(),
            ));
        }

        if start >= end {
            return Err(EngineError::InvalidInput(
                "interval start must be less than interval end".to_string(),
            ));
        }

        Ok(Self { start, end })
    }

    pub fn width(self) -> f64 {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
}

impl Diagnostic {
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningFlag {
    Discontinuity,
    UnreliableReference,
    NegativeRadius,
}
