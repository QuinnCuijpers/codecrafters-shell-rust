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
    shell::{builtin_exec, exec},
};

pub(crate) fn run_pipeline_builtin<'a, I>(
    builtin_out: String,
    token_iter: &mut Peekable<I>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
{
    let Some(Token::Command(cmd)) = token_iter.next() else {
        anyhow::bail!("Piped into nothing");
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
    anyhow::Ok(())
}

pub(crate) fn run_pipeline_external<'a, I>(
    mut command: Command,
    prev_command: Option<&mut Child>,
    prev_command_output: Option<String>,
    token_iter: &mut Peekable<I>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
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

    let mut child = command.spawn()?;

    if let Some(prev) = prev_command_output {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("child stdin was not piped"))?;
        stdin.write_all(prev.as_bytes())?;
        drop(stdin);
    }

    let Some(Token::Command(cmd)) = token_iter.next() else {
        anyhow::bail!("Piped into nothing");
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

    child.wait()?;
    anyhow::Ok(())
}
