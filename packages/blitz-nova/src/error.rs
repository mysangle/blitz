
pub enum BlitzError {
    RuntimeError {
        message: String,
    },
}

impl BlitzError {
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
        }
    }
}

pub struct ErrorReporter;

impl ErrorReporter {
    pub fn print_error(error: &BlitzError) {
        
    }
}


