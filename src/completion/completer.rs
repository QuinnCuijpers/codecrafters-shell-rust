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
    builtin_trie: TrieNode<TRIE_ASCII_SIZE>,
}

impl TrieCompleter {
    #[must_use]
    /// Create a new `TrieCompleter` with the given built-in command words inserted into the trie for autocompletion
    pub fn with_builtin_commands(builtin_words: &[&str]) -> Self {
        let mut builtin_trie = TrieNode::new();
        for word in builtin_words {
            builtin_trie.insert(word);
        }

        Self { builtin_trie }
    }
    /// Add external commands on $PATH to the trie for autocompletion, returning a vector of candidates matching the given prefix
    ///
    /// # Errors
    /// - `CompletionError::PathNotSet` if the `PATH` environment variable is not set
    pub fn get_external_candidates(prefix: &str) -> Result<Option<Vec<String>>, CompletionError> {
        let Some(env_path) = std::env::var_os("PATH") else {
            return Err(CompletionError::PathNotSet)?;
        };
        let mut external_trie: TrieNode<TRIE_ASCII_SIZE> = TrieNode::new();

        // add current directory files to trie
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
                        if search_dir_str.is_empty() {
                            external_trie.insert(name_str);
                            continue;
                        }
                        let name_str = search_dir_str.clone() + "/" + name_str;
                        external_trie.insert(&name_str);
                    }
                }
            }
        }

        // add executable files in path to trie
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
                        external_trie.insert(name_str);
                    }
                }
            }
        }
        Ok(external_trie.auto_complete(prefix))
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
        let mut candidates = self.builtin_trie.auto_complete(prefix).unwrap_or(vec![]);

        let mut external_candidates = TrieCompleter::get_external_candidates(prefix)
            .map_err(rustyline::error::ReadlineError::from)?
            .unwrap_or(vec![]);

        candidates.append(&mut external_candidates);

        let idx = start_of_last_word(line, pos - 1);

        if candidates.len() == 1 {
            candidates[0].push(' ');
            return Ok((idx, candidates));
        }
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
