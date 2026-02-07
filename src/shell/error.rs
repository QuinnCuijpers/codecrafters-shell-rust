use std::{
    ffi::OsString,
    io,
    process::{Child, ChildStdin, Command, CommandArgs},
};

use thiserror::Error;

use crate::parser::Token;

#[derive(Debug, Error)]
pub enum ShellError {
    #[error("could not flush stdin buffer due to: {0}")]
    FailedStdoutFlush(#[source] io::Error),
    #[error("attempted to pipe into {0:?}, which is not a command")]
    PipedIntoNonCommand(Token),
    #[error("No command to pipe into was found after pipe symbol")]
    PipedIntoNothing,
    #[error("Could not spawn command {name:?} due to: {source}")]
    CommandSpawnFailure {
        name: OsString,
        #[source]
        source: io::Error,
    },
    #[error("Waiting on child {0:?} failed due to {1}")]
    CommandWaitFailure(Child, #[source] io::Error),
    #[error("Failed to write {0} into {1:?} due to {2}")]
    WriteFailure(String, ChildStdin, #[source] io::Error),
    #[error("child stdin was not piped before command {0:?}")]
    ChildStdinNotPiped(Command),
}
