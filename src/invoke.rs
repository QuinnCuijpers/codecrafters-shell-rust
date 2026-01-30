use crate::input_parsing::Builtin;
use crate::util::find_exec_file;
use anyhow::Result;
use rustyline::history::{FileHistory, History};
use std::{
    cmp::min,
    env,
    ffi::OsStr,
    fs::{read, write},
    path::{Path, PathBuf},
    str::FromStr,
};

pub(crate) fn invoke_builtin<I, S>(
    cmd: Builtin,
    args: I,
    history: &mut FileHistory,
) -> Option<String>
where
    I: Iterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args_str: Vec<_> = args
        .map(|s| s.as_ref().to_str().unwrap().to_string())
        .collect();
    match cmd {
        Builtin::Echo => Some(invoke_echo(args_str)),
        Builtin::Exit => unreachable!(), // unreachable as we check for exit in main beforehand
        Builtin::Tipe => Some(invoke_type(args_str)),
        Builtin::Pwd => Some(invoke_pwd(args_str).unwrap()),
        Builtin::Cd => invoke_cd(args_str),
        Builtin::History => invoke_history(args_str, history),
    }
}

fn invoke_history(args_str: Vec<String>, history: &mut FileHistory) -> Option<String>
where
{
    let mut args_iter = args_str.iter();
    let length = if let Some(arg) = args_iter.next() {
        match arg.as_str() {
            s if s.parse::<usize>().is_ok() => {
                let n: usize = s.parse().unwrap();
                min(n, history.len())
            }
            "-r" => {
                if let Some(file_name) = args_iter.next() {
                    if history.load(Path::new(file_name)).is_err() {
                        eprintln!("Could not read history from file {file_name}");
                    } else {
                        let mut new_contents = format!("history -r {file_name}\n").as_bytes().to_owned();
                        if let Ok(mut contents) = read(file_name) {
                            new_contents.append(&mut contents);
                            let _ = write("history.txt", new_contents);
                            let _ = history.load(Path::new("history.txt"));
                        };
                        let _ = history;
                    }
                };
                0
            }
            _ => history.len(),
        }
    } else {
        history.len()
    };

    use std::fmt::Write;
    let mut buf = String::new();
    for i in 0..length {
        let entry_idx = history.len() - length + i;
        let entry = history
            .get(entry_idx, rustyline::history::SearchDirection::Reverse)
            .unwrap()?
            .entry;
        let _ = writeln!(buf, "  {} {}", entry_idx + 1, entry);
    }
    if buf.is_empty() { None } else { Some(buf) }
}

pub(crate) fn invoke_pwd<I, S>(_cmd_list: I) -> Result<String>
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

pub(crate) fn invoke_echo<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut s = cmd_list
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(" ");
    s.push('\n');
    s
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
    buf.push('\n');
    buf
}
