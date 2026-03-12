#![warn(missing_docs)]
//! `clawsh` is a simple command-line shell implemented in Rust, designed to provide a basic REPL interface for executing commands, managing history, and supporting features like tab completion and command parsing.
mod commands;
mod completion;
mod parser;
mod shell;

/// Type alias for `Result` with the error type set to `ClawshError`
pub type Result<T> = std::result::Result<T, ClawshError>;

pub use crate::commands::BUILTIN_COMMANDS;
pub use crate::completion::TrieCompleter;
use crate::shell::ClawshError;
pub use crate::shell::Shell;
