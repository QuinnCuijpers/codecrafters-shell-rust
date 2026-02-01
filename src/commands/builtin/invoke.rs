use std::ffi::OsStr;

use rustyline::history::FileHistory;

use crate::commands::{
    Builtin,
    builtin::{
        fs::{invoke_cd, invoke_pwd},
        history::invoke_history,
        string::{invoke_echo, invoke_type},
    },
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
        Builtin::History => invoke_history(&args_str[..], history),
    }
}
