mod fs;
mod history;
mod invoke;
mod string;

use std::str::FromStr;
use thiserror::Error;

pub(crate) use invoke::invoke_builtin;

pub const BUILTIN_COMMANDS: [&str; 6] = ["echo", "exit", "type", "pwd", "cd", "history"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enum representing the commmands built into this shell
pub enum Builtin {
    /// Print arguments to stdout
    Echo,
    /// Exit the shell
    Exit,
    /// Display whether a command is builtin or where it is located on $PATH
    Tipe,
    /// Print working directory
    Pwd,
    /// Change directory
    Cd,
    /// Command for interacting with history
    /// # Subcommands
    ///
    /// - `history` — print all history
    /// - `history <n>` — print last `n` entries
    /// - `history -r <file>` — read history
    /// - `history -w <file>` — write history
    /// - `history -a <file>` — append history
    History,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FromStrError {
    #[error("Unknown builtin {0}")]
    UnknownBuiltin(String),
}

impl FromStr for Builtin {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "exit" => Ok(Builtin::Exit),
            "type" => Ok(Builtin::Tipe),
            "pwd" => Ok(Builtin::Pwd),
            "cd" => Ok(Builtin::Cd),
            "history" => Ok(Builtin::History),
            _ => Err(FromStrError::UnknownBuiltin(s.to_string())),
        }
    }
}
