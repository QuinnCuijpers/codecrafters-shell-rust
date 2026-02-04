use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompletionError {
    #[error("PATH env var not set")]
    PathNotSet,
}

impl From<CompletionError> for rustyline::error::ReadlineError {
    fn from(value: CompletionError) -> Self {
        rustyline::error::ReadlineError::Io(io::Error::other(value.to_string()))
    }
}
