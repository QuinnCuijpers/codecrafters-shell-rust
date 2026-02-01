mod fs;
mod history;
mod invoke;
mod string;

use anyhow::Result;
use std::str::FromStr;

pub(crate) use invoke::invoke_builtin;

pub const BUILTIN_COMMANDS: [&str; 6] = ["echo", "exit", "type", "pwd", "cd", "history"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Echo,
    Exit,
    Tipe,
    Pwd,
    Cd,
    History,
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
            "history" => Ok(Builtin::History),
            _ => Err(anyhow::anyhow!(format!("unknown builtin {s}"))),
        }
    }
}
