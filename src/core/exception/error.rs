use std::fmt;

#[derive(Debug, Clone)]
pub enum SiddhiError {
    SendError(String),
    ProcessorError(String),
    Other(String),
}

impl fmt::Display for SiddhiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SiddhiError::SendError(m) => write!(f, "SendError: {}", m),
            SiddhiError::ProcessorError(m) => write!(f, "ProcessorError: {}", m),
            SiddhiError::Other(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for SiddhiError {}
