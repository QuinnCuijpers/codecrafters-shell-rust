use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Child, ChildStdin, Command},
};

use thiserror::Error;

use crate::parser::Token;

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("command error: {0}")]
    CommandsError(#[from] crate::commands::error::CommandsError),
    #[error("could not flush stdin buffer due to: {0}")]
    FailedStdoutFlush(#[source] io::Error),
    #[error("attempted to pipe into {0:?}, which is not a command")]
    PipedIntoNonCommand(Option<Token>),
    #[error("Could not spawn command {name:?} due to: {source}")]
    CommandSpawnFailure {
        name: OsString,
        #[source]
        source: io::Error,
    },
    #[error("Waiting on child {0:?} failed due to {1}")]
    CommandWaitFailure(Child, #[source] io::Error),
    #[error("Failed to write {0} into {1:?} due to {2}")]
    WriteStdinFailure(String, ChildStdin, #[source] io::Error),
    #[error("Failed to write {0} into {1:?} due to {2}")]
    WriteFileFailure(String, PathBuf, #[source] io::Error),
    #[error("Child stdin was not piped before command {0:?}")]
    ChildStdinNotPiped(Box<Command>),
    #[error("Failed to take stdout of previous command for piping into next command")]
    FailedToTakeStdout,
    #[error("Attempted to redirect into {0:?}")]
    NoFileForRedirection(Option<Token>),
    #[error("Failed to create dirs required for {0} due to {1}")]
    CouldNotCreateParentDir(PathBuf, #[source] io::Error),
    #[error("Failed to open file {0} due to {1}")]
    FailedToOpenFile(PathBuf, #[source] io::Error),
}
