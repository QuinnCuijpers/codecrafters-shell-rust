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

#[derive(Debug, Error)]
pub enum ClawshError {
    #[error("Error during setup: {0}")]
    SetupError(#[from] ClawshSetupError),
    #[error("Error when exiting: {0}")]
    ExitError(#[from] ClawshExitError),
}

#[derive(Debug, Error)]
pub enum ClawshSetupError {
    #[error("Failed to create file {0:?} due to: {1}")]
    FailedToCreateHistFile(OsString, #[source] io::Error),
    #[error("Failed to read from file {0:?} due to {1}")]
    FailedToReadHistFile(OsString, #[source] io::Error),
    #[error("Failed to create an editor due to: {0}")]
    FailedToCreateEditor(#[from] ReadlineError),
}

#[derive(Debug, Error)]
pub enum ClawshExitError {
    #[error("Failed to open file {0:?} to write history to, due to: {1}")]
    CouldNotOpenHistFile(OsString, #[source] io::Error),
}

pub struct Shell {
    rl: Editor<TrieCompleter, FileHistory>,
    old_contents: Option<Vec<u8>>,
    history_file: Option<OsString>,
}

impl Shell {
    #[allow(clippy::missing_panics_doc)]
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

    pub fn exit(&mut self) -> Result<(), ClawshExitError> {
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
