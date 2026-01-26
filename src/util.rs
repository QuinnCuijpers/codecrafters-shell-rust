use faccess::PathExt;
use std::path::PathBuf;

// TODO: consider making this a result to handle distinct errors
pub(crate) fn find_exec_file(cmd: &str) -> Option<PathBuf> {
    let Some(env_path) = std::env::var_os("PATH") else {
        eprintln!("PATH env var not set");
        return None;
    };
    for mut path in std::env::split_paths(&env_path) {
        if let Ok(exists) = path.try_exists() {
            if !exists {
                continue;
            }
            path.push(cmd);
            if path.executable() {
                return Some(path);
            }
        }
    }
    None
}

pub(crate) fn start_of_last_word(s: &str, mut pos: usize) -> usize {
    let bytes = s.as_bytes();

    while pos > 0 && bytes[pos] == b' ' {
        pos -= 1;
    }

    while pos > 0 && bytes[pos] != b' ' {
        pos -= 1;
    }

    if bytes[pos] == b' ' { pos + 1 } else { pos }
}
