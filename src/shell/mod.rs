use std::{
    ffi::OsString,
    fs::{File, OpenOptions, read},
    io::Write,
    path::Path,
};

use rustyline::{CompletionType, Config, Editor, history::FileHistory};

use crate::{BUILTIN_COMMANDS, TrieCompleter};

mod builtin_exec;
mod error;
mod exec;
mod handle_command;
mod pipeline;
mod redirect;
mod repl;

pub(crate) use handle_command::handle_command;

pub struct Shell {
    rl: Editor<TrieCompleter, FileHistory>,
    old_contents: Option<Vec<u8>>,
    history_file: Option<OsString>,
}

impl Shell {
    pub fn setup() -> anyhow::Result<Self> {
        let history_file = std::env::var_os("HISTFILE");

        if let Some(file_name) = history_file.as_ref()
            && !Path::new(&file_name).exists()
        {
            File::create(file_name)?;
        }

        let helper = TrieCompleter::with_builtin_commands(&BUILTIN_COMMANDS);
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .history_ignore_dups(false)?
            .build();

        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(helper));

        let mut old_contents = None;
        if let Some(file) = history_file.as_ref() {
            rl.load_history(&file)?;
            old_contents = Some(read(file)?);
        }
        anyhow::Ok(Self {
            rl,
            old_contents,
            history_file,
        })
    }

    pub fn exit(&mut self) -> anyhow::Result<()> {
        if let Some(history_file) = self.history_file.as_ref() {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(history_file)?;
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
        anyhow::Ok(())
    }
}
