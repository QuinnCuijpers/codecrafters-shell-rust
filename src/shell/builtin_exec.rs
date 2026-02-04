use std::{iter::Peekable, process::Child};

use rustyline::history::FileHistory;

use crate::{
    commands::{Builtin, invoke_builtin},
    parser::{Token, split_words},
    shell::{pipeline, redirect},
};

pub(crate) fn handle_builtin<'a, I>(
    builtin: Builtin,
    args: &[String],
    token_iter: &mut Peekable<I>,
    prev_command_output: Option<String>,
    _prev_command: Option<&mut Child>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
{
    let mut all_args = vec![];
    for s in args {
        all_args.push(s.clone());
    }

    if let Some(out) = prev_command_output {
        let extra_args = split_words(&out);

        all_args.extend(extra_args);
    }

    // if let Some(child) = prev_command
    // && let Some(stdout) = child.stdout.as_mut()
    // {
    //     // builtins do not read stdin
    // }

    let Some(builtin_out) = invoke_builtin(builtin, &all_args, history)? else {
        // early return for cd
        return Ok(());
    };

    match token_iter.next() {
        None => print!("{builtin_out}"),
        Some(Token::Redirect(redirect_symb)) => {
            redirect::redirect_builtin_output(redirect_symb, &builtin_out, token_iter)?;
        }
        Some(Token::Pipe) => {
            pipeline::run_pipeline_builtin(builtin_out, token_iter, history)?;
        }
        Some(_t) => unreachable!(),
    }
    Ok(())
}
