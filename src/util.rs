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
