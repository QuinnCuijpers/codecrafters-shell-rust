use crate::input_parsing::Builtin;
use crate::util::find_exec_file;
use anyhow::Result;
use std::{env, path::PathBuf, str::FromStr};

pub(crate) fn invoke_pwd<I, S>(_cmd_list: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let curr = env::current_dir()?;
    Ok(format!("{}", curr.display()))
}

pub(crate) fn invoke_cd<I, S>(cmd_list: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let cmd_list: Vec<_> = cmd_list.into_iter().collect();
    assert!(cmd_list.len() == 1);
    let path = match cmd_list[0].as_ref() {
        "~" => PathBuf::from(&std::env::var_os("HOME").expect("HOME env key not set")),
        _ => PathBuf::from(&cmd_list[0].as_ref()),
    };
    if path.exists() {
        // if cd fails then proceed to next REPL iter
        let _ = env::set_current_dir(path);
        None
    } else {
        Some(format!("cd: {}: No such file or directory", path.display()))
    }
}

pub(crate) fn invoke_echo<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    cmd_list
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn invoke_type<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use std::fmt::Write;
    let mut buf = String::new();
    for (i, cmd) in cmd_list.into_iter().enumerate() {
        let cmd = cmd.as_ref();
        let cmd_str = if i != 0 {
            format!("\n{cmd}")
        } else {
            cmd.to_string()
        };
        let cmd_str = cmd_str.as_str();
        if Builtin::from_str(cmd).is_ok() {
            let _ = write!(buf, "{cmd_str} is a shell builtin");
        }
        // go through every directory and check if a file with the name exist that has exec permissions
        else if let Some(file_path) = find_exec_file(cmd) {
            let _ = write!(buf, "{cmd_str} is {}", file_path.display());
        } else {
            let _ = write!(buf, "{cmd_str}: not found");
        }
    }
    buf
}
