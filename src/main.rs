use anyhow::{Context, Result, anyhow};
use faccess::PathExt;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env::{current_dir, set_current_dir},
    path::PathBuf,
    process::Command,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Builtin {
    Echo,
    Exit,
    Tipe,
    Pwd,
    Cd,
}

impl FromStr for Builtin {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "exit" => Ok(Builtin::Exit),
            "type" => Ok(Builtin::Tipe),
            "pwd" => Ok(Builtin::Pwd),
            "cd" => Ok(Builtin::Cd),
            _ => Err(anyhow!(format!("unknown builtin {s}"))),
        }
    }
}

fn main() -> anyhow::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf).context("reading stdin")?;
        let input = buf.trim_end();
        let Ok(command_list) = parse_input(input) else {
            continue;
        };

        if let Ok(command) = Builtin::from_str(&command_list[0]) {
            match command {
                Builtin::Echo => invoke_echo(&command_list[1..]),
                Builtin::Exit => break,
                Builtin::Tipe => invoke_type(&command_list[1..]),
                Builtin::Pwd => invoke_pwd(&command_list[1..]),
                Builtin::Cd => invoke_cd(&command_list[1..]),
            }
        } else {
            let Some(env_path) = std::env::var_os("PATH") else {
                panic!("PATH env var not set");
            };
            let exec = &command_list[0];
            if find_exec_file(exec, env_path).is_some() {
                let mut output = Command::new(exec).args(&command_list[1..]).spawn()?;
                output.wait()?;
                continue;
            }
            println!("{input}: command not found")
        }
    }
    anyhow::Ok(())
}

fn parse_input(input: &str) -> Result<Vec<String>> {
    let mut command_list = vec![];
    let mut buf = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                if in_single_quotes || in_double_quotes {
                    buf.push(c);
                } else {
                    if buf.is_empty() {
                        continue;
                    }
                    command_list.push(buf.clone());
                    buf.clear();
                }
            }
            '\\' => if !in_single_quotes && !in_double_quotes {
                if let Some(next_char) = chars.next() {
                    buf.push(next_char)
                }
            }
            '\'' => {
                if in_double_quotes {
                    buf.push(c);
                    continue;
                }
                if in_single_quotes {
                    in_single_quotes = false;
                } else {
                    in_single_quotes = true;
                }
            }
            '\"' => {
                if in_double_quotes {
                    in_double_quotes = false;
                } else {
                    in_double_quotes = true;
                }
            }
            _ => buf.push(c),
        }
    }
    if !buf.is_empty() {
        command_list.push(buf.clone());
    }
    // all input that can no longer be split on space is still added to the command list
    Ok(command_list)
}

fn invoke_echo(cmd_list: &[String]) {
    let out = cmd_list.join(" ");
    println!("{out}");
}

fn invoke_type(cmd_list: &[String]) {
    for cmd in cmd_list {
        if Builtin::from_str(cmd).is_ok() {
            println!("{cmd} is a shell builtin");
            return;
        }
        // go through every directory and check if a file with the name exist that has exec permissions
        let Some(env_path) = std::env::var_os("PATH") else {
            panic!("PATH env var not set");
        };
        if let Some(file_path) = find_exec_file(cmd, env_path) {
            println!("{cmd} is {}", file_path.display());
        } else {
            println!("{cmd}: not found");
        }
    }
}

fn find_exec_file(cmd: &str, env_path: std::ffi::OsString) -> Option<PathBuf> {
    for path in std::env::split_paths(&env_path) {
        if let Ok(exists) = path.try_exists() {
            if !exists {
                continue;
            }
            for dir in path.read_dir().expect("dir should exist").flatten() {
                let file_name = dir.file_name();
                let file_path = dir.path();
                if file_name == cmd && file_path.executable() {
                    return Some(file_path);
                }
            }
        }
    }
    None
}

fn invoke_pwd(_cmd_list: &[String]) {
    if let Ok(curr) = current_dir() {
        println!("{}", curr.display());
    }
}

fn invoke_cd(cmd_list: &[String]) {
    assert!(cmd_list.len() == 1);
    let path = match cmd_list[0].as_str() {
        "~" => PathBuf::from(&std::env::var_os("HOME").expect("HOME env key not set")),
        _ => PathBuf::from(&cmd_list[0]),
    };
    if path.exists() {
        // if cd fails then proceed to next REPL iter
        let _ = set_current_dir(path);
    } else {
        println!("cd: {}: No such file or directory", path.display());
    }
}
