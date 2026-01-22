use anyhow::{Context, anyhow};
use faccess::PathExt;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{path::PathBuf, process::Command, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Builtin {
    Echo,
    Exit,
    Tipe,
}

impl FromStr for Builtin {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "exit" => Ok(Builtin::Exit),
            "type" => Ok(Builtin::Tipe),
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
        let command_list: Vec<_> = input.split(" ").collect();
        if let Ok(command) = Builtin::from_str(command_list[0]) {
            match command {
                Builtin::Echo => invoke_echo(&command_list[1..]),
                Builtin::Exit => break,
                Builtin::Tipe => invoke_type(&command_list[1..]),
            }
        } else {
            // TODO: add support for executing files
            let Some(env_path) = std::env::var_os("PATH") else {
                panic!("PATH env var not set");
            };
            let exec = command_list[0];
            if let Some(_) = find_exec_file(exec, env_path) {
                let mut output = Command::new(exec).args(&command_list[1..]).spawn()?;
                output.wait()?;
                continue;
            }
            println!("{input}: command not found")
        }
    }
    anyhow::Ok(())
}

fn invoke_echo(cmd_list: &[&str]) {
    let out = cmd_list.join(" ");
    println!("{out}");
}

fn invoke_type(cmd_list: &[&str]) {
    for &cmd in cmd_list {
        if let Ok(_) = Builtin::from_str(cmd) {
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
            for dir in path.read_dir().expect("dir should exist") {
                if let Ok(dir) = dir {
                    let file_name = dir.file_name();
                    let file_path = dir.path();
                    if file_name == cmd && file_path.executable() {
                        return Some(file_path);
                    }
                }
            }
        }
    }
    None
}
