use std::{ffi::OsStr, iter::Peekable, str::FromStr as _};

use rustyline::history::FileHistory;

use crate::{
    commands::{Builtin, find_exec_file},
    parser::Token,
    shell::{builtin_exec::handle_builtin, exec::handle_external_exec},
};

pub fn handle_command<'a, I, J, S>(
    cmd_str: &str,
    args: J,
    token_iter: &mut Peekable<I>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
    J: Iterator<Item = S>,
    S: AsRef<OsStr>,
{
    if let Ok(builtin) = Builtin::from_str(cmd_str) {
        handle_builtin(builtin, args, token_iter, None, None, history)?;
    } else if find_exec_file(cmd_str).is_some() {
        handle_external_exec(cmd_str, args, token_iter, None, None, history)?;
    } else {
        println!("{cmd_str}: command not found");
    }
    Ok(())
}
