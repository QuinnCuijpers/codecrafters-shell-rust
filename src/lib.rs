#![warn(missing_docs)]

mod commands;
mod completion;
pub mod parser;
pub mod shell;

pub type Result<T> = std::result::Result<T, ClawshError>;

pub use crate::commands::BUILTIN_COMMANDS;
pub use crate::completion::TrieCompleter;
use crate::shell::ClawshError;
pub use crate::shell::Shell;
