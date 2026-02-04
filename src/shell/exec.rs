use std::{
    io::Write,
    iter::Peekable,
    process::{Child, Command, Stdio},
};

use rustyline::history::FileHistory;

use crate::{
    parser::Token,
    shell::{pipeline, redirect},
};

pub(crate) fn handle_external_exec<'a, I>(
    cmd_str: &str,
    args: &[String],
    token_iter: &mut Peekable<I>,
    prev_command_output: Option<String>,
    prev_command: Option<&mut Child>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
{
    let mut command = Command::new(cmd_str);

    command.args(args);

    match token_iter.next() {
        // no more tokens
        None => {
            if prev_command_output.is_some() {
                command.stdin(Stdio::piped());
            } else if prev_command.is_some() {
                match prev_command.and_then(|p| p.stdout.take()) {
                    Some(stdout) => {
                        command.stdin(stdout);
                    }
                    None => anyhow::bail!("expected previous command to have piped stdout"),
                }
            }

            let mut child = command.spawn()?;

            #[allow(clippy::expect_used)]
            if let Some(prev) = prev_command_output {
                let mut stdin = child
                    .stdin
                    .take()
                    .expect("stdin is set by the previous if-else");
                stdin.write_all(prev.as_bytes())?;
            }

            child.wait()?;
        }
        Some(Token::Redirect(redirect_symb)) => {
            redirect::redirect_external(&mut command, redirect_symb, token_iter)?;
        }
        Some(Token::Pipe) => {
            pipeline::run_pipeline_external(
                command,
                prev_command,
                prev_command_output,
                token_iter,
                history,
            )?;
        }
        Some(t) => unreachable!("found unhandled token: {:?}", t),
    }
    Ok(())
}
