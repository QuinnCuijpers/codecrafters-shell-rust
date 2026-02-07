use rustyline::history::FileHistory;

use crate::commands::{
    Builtin,
    builtin::{
        fs::{invoke_cd, invoke_pwd},
        history::invoke_history,
        string::{invoke_echo, invoke_type},
    },
    error::CommandsError,
};

pub(crate) fn invoke_builtin(
    cmd: Builtin,
    args: &[String],
    history: &mut FileHistory,
) -> Result<Option<String>, CommandsError> {
    match cmd {
        Builtin::Echo => Ok(Some(invoke_echo(args))),
        Builtin::Exit => unreachable!(), // unreachable as we check for exit in main beforehand
        Builtin::Tipe => Ok(Some(invoke_type(args))),
        Builtin::Pwd => Ok(Some(invoke_pwd(args)?)),
        Builtin::Cd => invoke_cd(args),
        Builtin::History => Ok(invoke_history(args, history)),
    }
}
