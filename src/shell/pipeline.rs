use std::{
    io::Write as _,
    iter::Peekable,
    process::{Child, Command, Stdio},
    str::FromStr as _,
};

use rustyline::history::FileHistory;

use crate::{
    commands::Builtin,
    parser::Token,
    shell::{builtin_exec, error::ShellError, exec},
};

pub(crate) fn run_pipeline_builtin<'a, I>(
    builtin_out: String,
    token_iter: &mut Peekable<I>,
    history: &mut FileHistory,
) -> Result<(), ShellError>
where
    I: Iterator<Item = &'a Token>,
{
    let cmd = match token_iter.next() {
        Some(Token::Command(str)) => str,
        Some(t) => return Err(ShellError::PipedIntoNonCommand(Some(t.to_owned())))?,
        None => return Err(ShellError::PipedIntoNonCommand(None))?,
    };

    let mut next_args = vec![];
    while let Some(Token::Arg(s)) = token_iter.peek() {
        next_args.push(s.clone());
        token_iter.next();
    }

    // create pipeline recursively
    if let Ok(cmd) = Builtin::from_str(cmd) {
        builtin_exec::handle_builtin(
            cmd,
            &next_args[..],
            token_iter,
            Some(builtin_out),
            None,
            history,
        )?;
    } else {
        exec::handle_external_exec(
            cmd,
            &next_args,
            token_iter,
            Some(builtin_out),
            None,
            history,
        )?;
    }
    Ok(())
}

pub(crate) fn run_pipeline_external<'a, I>(
    mut command: Command,
    prev_command: Option<&mut Child>,
    prev_command_output: Option<String>,
    token_iter: &mut Peekable<I>,
    history: &mut FileHistory,
) -> Result<(), ShellError>
where
    I: Iterator<Item = &'a Token>,
{
    command.stdout(Stdio::piped());

    if let Some(prev) = prev_command
        && let Some(stdout) = prev.stdout.take()
    {
        command.stdin(stdout);
    }

    if prev_command_output.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .spawn()
        .map_err(|e| ShellError::CommandSpawnFailure {
            name: command.get_program().to_os_string(),
            source: e,
        })?;

    if let Some(prev) = prev_command_output {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| ShellError::ChildStdinNotPiped(Box::new(command)))?;
        match stdin.write_all(prev.as_bytes()) {
            Ok(()) => drop(stdin),
            Err(e) => return Err(ShellError::WriteStdinFailure(prev, stdin, e))?,
        }
    }

    let cmd = match token_iter.next() {
        Some(Token::Command(str)) => str,
        Some(t) => return Err(ShellError::PipedIntoNonCommand(Some(t.to_owned())))?,
        None => return Err(ShellError::PipedIntoNonCommand(None))?,
    };

    let mut next_args = vec![];
    while let Some(Token::Arg(s)) = token_iter.peek() {
        next_args.push(s.clone());
        token_iter.next();
    }

    // create pipeline recursively
    if let Ok(cmd) = Builtin::from_str(cmd) {
        builtin_exec::handle_builtin(cmd, &next_args, token_iter, None, Some(&mut child), history)?;
    } else {
        exec::handle_external_exec(cmd, &next_args, token_iter, None, Some(&mut child), history)?;
    }

    child
        .wait()
        .map_err(|e| ShellError::CommandWaitFailure(child, e))?;
    Ok(())
}
