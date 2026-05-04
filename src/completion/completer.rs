use crate::commands::BUILTIN_COMMANDS;
use crate::completion::error::CompletionError;
use faccess::PathExt;
use rustyline::completion::Completer;
use rustyline::{Helper, Highlighter, Hinter, Validator};

use crate::completion::trie::{TRIE_ASCII_SIZE, TrieNode};

#[derive(Debug, Helper, Highlighter, Validator, Hinter)]
// TODO: add doc tests for `TrieCompleter` and additional explanation of how it works and how to use it
/// Trie-based completer implementing `rustyline::completion::Completer` for autocompletion of built-in commands and external commands on $PATH
pub struct TrieCompleter {
    builtin: TrieNode<TRIE_ASCII_SIZE>,
}

impl TrieCompleter {
    #[must_use]
    /// Create a new `TrieCompleter` with the given built-in command words inserted into the trie for autocompletion
    pub fn with_builtin_commands(builtin_words: &[&str]) -> Self {
        let mut builtin = TrieNode::new();
        for word in builtin_words {
            builtin.insert(word);
        }

        Self { builtin }
    }

    fn get_files_current_dir(prefix: &str) -> Vec<String> {
        let mut candidates = vec![];
        if let Ok(current_dir) = std::env::current_dir() {
            let split = prefix.rsplitn(2, '/').collect::<Vec<_>>();
            let prefix = split[0];
            let search_dir_str = split.get(1).unwrap_or(&"").to_string();
            let search_dir = current_dir.join(&search_dir_str);

            if let Ok(search_dir) = search_dir.read_dir() {
                for entry in search_dir.flatten() {
                    let file_path = entry.path();
                    let file_name = entry.file_name();
                    let name_str = file_name.to_str();
                    let Some(name_str) = name_str else {
                        continue;
                    };
                    if name_str.starts_with(prefix) && file_path.exists() {
                        let candidate = if search_dir_str.is_empty() {
                            if file_path.is_dir() {
                                name_str.to_string() + "/"
                            } else {
                                name_str.to_string()
                            }
                        } else if file_path.is_dir() {
                            search_dir_str.clone() + "/" + name_str + "/"
                        } else {
                            search_dir_str.clone() + "/" + name_str
                        };
                        candidates.push(candidate);
                    }
                }
            }
        }
        candidates
    }

    fn get_path_exec(prefix: &str) -> Result<Vec<String>, CompletionError> {
        let mut res = vec![];

        let Some(env_path) = std::env::var_os("PATH") else {
            return Err(CompletionError::PathNotSet);
        };

        for path in std::env::split_paths(&env_path) {
            if let Ok(exists) = path.try_exists() {
                if !exists {
                    continue;
                }

                let Ok(dir) = path.read_dir() else { continue };
                for entry in dir.flatten() {
                    let file_path = entry.path();
                    let file_name = entry.file_name();
                    let name_str = file_name.to_str();
                    let Some(name_str) = name_str else {
                        continue;
                    };
                    if name_str.starts_with(prefix)
                        && file_path.executable()
                        && !BUILTIN_COMMANDS.contains(&name_str)
                    {
                        res.push(name_str.to_string());
                    }
                }
            }
        }
        Ok(res)
    }
}

impl Completer for TrieCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix = line.split(' ').next_back().unwrap_or("");

        let ignore_builtins = prefix.is_empty() && !line.is_empty() && !line.starts_with("type");
        let mut candidates = vec![];

        if !ignore_builtins {
            candidates = self.builtin.auto_complete(prefix).unwrap_or(vec![]);
        }

        if !line.contains(' ') {
            candidates.append(&mut TrieCompleter::get_path_exec(prefix)?);
        }

        candidates.append(&mut TrieCompleter::get_files_current_dir(prefix));

        let idx = if line.chars().nth(pos - 1) == Some(' ') {
            pos
        } else {
            start_of_last_word(line, pos - 1)
        };

        if candidates.len() == 1 {
            if !candidates[0].ends_with('/') {
                candidates[0].push(' ');
            }
            return Ok((idx, candidates));
        }
        candidates.sort();
        Ok((idx, candidates))
    }

    fn update(
        &self,
        line: &mut rustyline::line_buffer::LineBuffer,
        start: usize,
        elected: &str,
        cl: &mut rustyline::Changeset,
    ) {
        let end = line.pos();
        line.replace(start..end, elected, cl);
    }
}

fn start_of_last_word(s: &str, mut pos: usize) -> usize {
    let bytes = s.as_bytes();

    while pos > 0 && bytes[pos] == b' ' {
        pos -= 1;
    }

    while pos > 0 && bytes[pos] != b' ' {
        pos -= 1;
    }

    if bytes[pos] == b' ' { pos + 1 } else { pos }
}
