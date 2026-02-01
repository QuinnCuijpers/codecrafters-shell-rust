use std::{env, path::PathBuf};

pub(crate) fn invoke_pwd<I, S>(_cmd_list: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let curr = env::current_dir()?;
    Ok(format!("{}\n", curr.display()))
}

pub(crate) fn invoke_cd<I, S>(cmd_list: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let cmd_list: Vec<_> = cmd_list.into_iter().collect();
    if cmd_list.is_empty() {
        return None;
    }

    let path = match cmd_list[0].as_ref() {
        "~" => PathBuf::from(&std::env::var_os("HOME").expect("HOME env key not set")),
        _ => PathBuf::from(&cmd_list[0].as_ref()),
    };
    if path.exists() {
        // if cd fails then proceed to next REPL iter
        let _ = env::set_current_dir(path);
        None
    } else {
        Some(format!(
            "cd: {}: No such file or directory\n",
            path.display()
        ))
    }
}
