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
    pub fn get_external_candidates(
        prefix: &str,
        get_exec: bool,
    ) -> Result<Option<Vec<String>>, CompletionError> {
        let mut external_trie: TrieNode<TRIE_ASCII_SIZE> = TrieNode::new();

        // add current directory files to trie
        let current_dir_candidates = TrieCompleter::get_files_current_dir(prefix);
        for candidate in current_dir_candidates {
            external_trie.insert(&candidate);
        }

        // add executable files in path to trie
        if get_exec {
            let exec_files_candidates = TrieCompleter::get_path_exec(prefix)?;
            for candidate in exec_files_candidates {
                external_trie.insert(&candidate);
            }
        }

        Ok(external_trie.auto_complete(prefix))
    }

    fn get_files_current_dir(prefix: &str) -> Vec<String> {
        let mut candidates = Vec::new();
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
                            if file_path.is_dir() {
                                candidates.push(name_str.to_string() + "/");
                                continue;
                            }
                            candidates.push(name_str.to_string());
                            continue;
                        } else if file_path.is_dir() {
                            candidates.push(search_dir_str.clone() + "/" + name_str + "/");
                            continue;
                        }
                        candidates.push(search_dir_str.clone() + "/" + name_str);
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

        let mut candidates = self.builtin_trie.auto_complete(prefix).unwrap_or(vec![]);

        // TODO: only look for path execs and builtins if the line doesn't contain space
        // except builtins if the line starts with type I guess
        let mut external_candidates =
            TrieCompleter::get_external_candidates(prefix, !line.contains(' '))
                .map_err(rustyline::error::ReadlineError::from)?
                .unwrap_or(vec![]);

        candidates.append(&mut external_candidates);

        let idx = if line.chars().nth(pos - 1) == Some(' ') {
            pos
        } else {
            start_of_last_word(line, pos - 1)
        };

        if ignore_builtins {
            candidates.retain(|c| !BUILTIN_COMMANDS.contains(&c.as_str()));
        }

        if candidates.len() == 1 {
            if !candidates[0].ends_with('/') {
                candidates[0].push(' ');
            }
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
