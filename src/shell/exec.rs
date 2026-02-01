use std::{
    ffi::OsStr,
    io::Write,
    iter::Peekable,
    process::{Child, Command, Stdio},
};

use rustyline::history::FileHistory;

use crate::{parser::Token, shell::{pipeline, redirect}};

pub(crate) fn handle_external_exec<'a, S, I, J>(
    cmd_str: &str,
    args: J,
    token_iter: &mut Peekable<I>,
    prev_command_output: Option<String>,
    prev_command: Option<&mut Child>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
    J: Iterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd_str);

    command.args(args);

    match token_iter.next() {
        // no more tokens
        None => {
            if prev_command_output.is_some() {
                command.stdin(Stdio::piped());
            } else if let Some(prev) = prev_command
                && prev.stdout.is_some()
            {
                command.stdin(prev.stdout.take().unwrap());
            }

            let mut child = command.spawn()?;

            if let Some(prev) = prev_command_output {
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(prev.as_bytes())?;
            }

            child.wait()?;
        }
        Some(Token::Redirect(redirect_symb)) => {
            redirect::redirect_external(&mut command, redirect_symb, token_iter)?;
        }
        Some(Token::Pipe) => {
            pipeline::run_pipeline_external(command, prev_command, prev_command_output, token_iter, history)?;
        }
        Some(t) => unreachable!("found unhandled token: {:?}", t),
    }
    Ok(())
}
