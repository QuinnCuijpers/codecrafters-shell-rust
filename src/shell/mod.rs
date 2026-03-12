use std::{
    ffi::OsString,
    fs::{File, OpenOptions, read},
    io::{self, Write},
    path::Path,
};

use rustyline::{CompletionType, Config, Editor, error::ReadlineError, history::FileHistory};
use thiserror::Error;

use crate::{BUILTIN_COMMANDS, TrieCompleter};

mod builtin_exec;
mod error;
mod exec;
mod handle_command;
mod pipeline;
mod redirect;
mod repl;

pub(crate) use handle_command::handle_command;

// TODO: create a proper error module
#[derive(Debug, Error)]
/// Enum representing the different types of errors that can occur when using the shell, including setup errors and exit errors
/// errors during execution of commands are not represented here and are instead printed to stderr but do not exit the shell
pub enum ClawshError {
    #[error("Error during setup: {0}")]
    /// Errors that can occur during shell setup, such as issues with history file or creating the editor
    SetupError(#[from] ClawshSetupError),
    #[error("Error when exiting: {0}")]
    /// Errors that can occur during shell exit, such as issues with writing history to file
    ExitError(#[from] ClawshExitError),
}

#[derive(Debug, Error)]
/// Enum representing errors that can occur during shell setup, such as issues with history file or creating the editor
pub enum ClawshSetupError {
    #[error("Failed to create file {0:?} due to: {1}")]
    /// Error when creating the history file specified by `HISTFILE` environment variable, including the file name and the underlying I/O error
    FailedToCreateHistFile(OsString, #[source] io::Error),
    #[error("Failed to read from file {0:?} due to {1}")]
    /// Error when reading the history file specified by `HISTFILE` environment variable, including the file name and the underlying I/O error
    FailedToReadHistFile(OsString, #[source] io::Error),
    #[error("Failed to create an editor due to: {0}")]
    /// Error when creating the `rustyline::Editor` for the REPL, including the underlying error from `rustyline`
    FailedToCreateEditor(#[from] ReadlineError),
}

#[derive(Debug, Error)]
/// Enum representing errors that can occur during shell exit, such as issues with writing history to file
pub enum ClawshExitError {
    #[error("Failed to open file {0:?} to write history to, due to: {1}")]
    /// Error when opening the history file specified by `HISTFILE` environment variable for writing during shell exit, including the file name and the underlying I/O error
    CouldNotOpenHistFile(OsString, #[source] io::Error),
}

/// Struct representing the state of the shell,
/// it should be created using `Shell::setup()` which will handle all necessary initialization such as setting up the REPL editor and loading history from file, and should be used to run the main REPL loop with `Shell::run()`
/// finally, `Shell::exit()` should be called before exiting the program to handle any necessary cleanup such as writing history back to file
///
/// # Example:
/// ```
/// # use clawsh::Shell;
///
/// fn main() -> clawsh::Result<()> {
///     let mut shell = Shell::setup()?;
///     shell.run();
///     shell.exit()?;
///     Ok(())
/// }
/// ```
pub struct Shell {
    rl: Editor<TrieCompleter, FileHistory>,
    old_contents: Option<Vec<u8>>,
    history_file: Option<OsString>,
}

impl Shell {
    #[allow(clippy::missing_panics_doc)]
    /// Setup a new `Shell` instance
    ///
    /// # Errors
    /// - `ClawshSetupError::FailedToCreateHistFile` if the history file specified by `HISTFILE` environment variable does not exist and cannot be created
    /// - `ClawshSetupError::FailedToReadHistFile` if the history file specified by `HISTFILE` environment variable cannot be read
    /// - `ClawshSetupError::FailedToCreateEditor` if the `rustyline::Editor` cannot be created for the REPL
    ///
    pub fn setup() -> Result<Self, ClawshSetupError> {
        let history_file = std::env::var_os("HISTFILE");

        if let Some(file_name) = history_file.as_ref()
            && !Path::new(&file_name).exists()
        {
            File::create(file_name)
                .map_err(|e| ClawshSetupError::FailedToCreateHistFile(file_name.clone(), e))?;
        }

        let helper = TrieCompleter::with_builtin_commands(&BUILTIN_COMMANDS);
        #[allow(clippy::expect_used)]
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .history_ignore_dups(false)
            .expect("Rustyline's implementation cannot err")
            .build();

        let mut rl = Editor::with_config(config).map_err(ClawshSetupError::FailedToCreateEditor)?;
        rl.set_helper(Some(helper));

        let mut old_contents = None;
        if let Some(file) = history_file.as_ref() {
            #[allow(clippy::expect_used)]
            rl.load_history(&file)
                .expect("Rustyline implementation cannot Error");
            old_contents = Some(
                read(file).map_err(|e| ClawshSetupError::FailedToReadHistFile(file.clone(), e))?,
            );
        }
        Ok(Self {
            rl,
            old_contents,
            history_file,
        })
    }
    /// Exit the shell writing history back to file specified by `HISTFILE` environment variable if it is set
    ///  if `HISTFILE` is not set, no history will be written and the function will return `Ok(())`
    ///
    /// # Errors
    /// - `ClawshExitError::CouldNotOpenHistFile` if the history file specified by `HISTFILE` environment variable cannot be opened for writing during shell exit
    pub fn exit(self) -> Result<(), ClawshExitError> {
        if let Some(history_file) = self.history_file.as_ref() {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(history_file)
                .map_err(|e| ClawshExitError::CouldNotOpenHistFile(history_file.clone(), e))?;
            let mut new_contents = vec![];
            for entry in self.rl.history() {
                let mut new_entry = entry.clone();
                new_entry.push('\n');
                new_contents.append(&mut new_entry.as_bytes().to_owned());
            }

            let Some(old_contents) = self.old_contents.as_ref() else {
                unreachable!();
            };
            if new_contents.starts_with(old_contents) {
                new_contents = new_contents[old_contents.len()..].to_vec();
            }

            _ = file.write_all(&new_contents);
        }
        Ok(())
    }
}
