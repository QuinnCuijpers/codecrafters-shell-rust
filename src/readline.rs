use crate::trie::TRIE_ASCII_SIZE;
use crate::util::start_of_last_word;
use crate::{input_parsing::BUILTIN_COMMANDS, trie::TrieNode};
use faccess::PathExt;
use rustyline::{Helper, Highlighter, Hinter, Validator, completion::Completer};

#[derive(Debug, Helper, Highlighter, Validator, Hinter)]
pub struct TrieCompleter {
    builtin_trie: TrieNode<TRIE_ASCII_SIZE>,
}

impl TrieCompleter {
    pub(crate) fn with_builtin_commands(builtin_words: &[&str]) -> Self {
        let mut builtin_trie = TrieNode::new();
        for word in builtin_words {
            builtin_trie.insert(word);
        }

        Self { builtin_trie }
    }

    pub(crate) fn get_external_candidates(&self, prefix: &str) -> Option<Vec<String>> {
        let Some(env_path) = std::env::var_os("PATH") else {
            eprintln!("PATH env var not set");
            return None;
        };

        let mut external_trie: TrieNode<TRIE_ASCII_SIZE> = TrieNode::new();
        for path in std::env::split_paths(&env_path) {
            if let Ok(exists) = path.try_exists() {
                if !exists {
                    continue;
                }
                for entry in path.read_dir().unwrap().flatten() {
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
        external_trie.auto_complete(prefix)
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
        let mut candidates = self.builtin_trie.auto_complete(line).unwrap_or(vec![]);

        let mut external_candidates = self.get_external_candidates(line).unwrap_or(vec![]);

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
