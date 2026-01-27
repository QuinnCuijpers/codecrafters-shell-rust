use std::{
    ffi::OsStr,
    fs::File,
    io::Write,
    iter::Peekable,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::input_parsing::Token;
use anyhow::Result;

pub(crate) fn handle_external_exec<'a, S, I, J>(
    s: &str,
    args: J,
    token_iter: &mut Peekable<I>,
    prev_command: Option<&mut Child>,
) -> Result<()>
where
    I: Iterator<Item = &'a Token>,
    J: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(s);
    command.args(args);
    match token_iter.next() {
        // no more tokens
        None => {
            if let Some(prev) = prev_command {
                if let Some(stdout) = prev.stdout.take() {
                    command.stdin(stdout);
                }
            }

            let mut child = command.spawn()?;
            child.wait()?;
        }
        Some(Token::Redirect(c)) => {
            let Some(Token::Arg(file_name)) = token_iter.next() else {
                anyhow::bail!("expected file name after redirection");
            };

            let file_path = PathBuf::from(file_name);
            if let Some(parent_dir) = file_path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }

            let mut file_options = File::options();
            file_options.create(true).write(true);

            match c.as_str() {
                ">" | "1>" => {
                    file_options.truncate(true);
                    let file = file_options.open(file_path)?;
                    command.stdout(Stdio::from(file));
                }
                "2>" => {
                    file_options.truncate(true);
                    let file = file_options.open(file_path)?;
                    command.stderr(Stdio::from(file));
                }
                "2>>" => {
                    let file = file_options.append(true).open(file_path)?;
                    command.stderr(Stdio::from(file));
                }
                ">>" | "1>>" => {
                    let file = file_options.append(true).open(file_path)?;
                    command.stdout(Stdio::from(file));
                }
                _ => unreachable!("Unknown redirection operator"),
            }

            let mut child = command.spawn()?;
            child.wait()?;
        }
        Some(Token::Pipe(_)) => {
            
            command.stdout(Stdio::piped());

            if let Some(prev) = prev_command {
                if let Some(stdout) = prev.stdout.take() {
                    command.stdin(stdout);
                }
            }

            let mut child = command.spawn()?;

            let Some(Token::Command(cmd)) = token_iter.next() else {
                anyhow::bail!("Piped into nothing");
            };

            let mut next_args = vec![];
            while let Some(Token::Arg(s)) = token_iter.peek() {
                next_args.push(s);
                token_iter.next();
            }

            // create pipeline recursively
            handle_external_exec(cmd, next_args, token_iter, Some(&mut child))?;

            // wait on this subcommand after recursively creating the pipeline
            child.wait()?;
        }
        Some(t) => unreachable!("found unhandled token: {:?}", t),
    }
    Ok(())
}

pub(crate) fn handle_builtin<'a, I>(builtin_out: Option<String>, token_iter: I)
where
    I: IntoIterator<Item = &'a Token>,
{
    let Some(mut builtin_out) = builtin_out else {
        return;
    };
    let mut iter = token_iter.into_iter();
    match iter.next() {
        None => println!("{builtin_out}"),
        Some(Token::Redirect(c)) => {
            if let Some(Token::Arg(file_name)) = iter.next() {
                let file_path = PathBuf::from(file_name);
                if let Some(parent_dir) = file_path.parent()
                    && std::fs::create_dir_all(parent_dir).is_err()
                {
                    eprintln!("Failed to create dirs required for {}", file_path.display());
                    return;
                };

                let mut file_options = File::options();
                file_options.create(true).write(true);

                if c == "2>" {
                    let _ = file_options
                        .open(file_path)
                        .expect("couldnt open file for redirecting stderr");
                    println!("{builtin_out}");
                    return;
                } else if c == "2>>" {
                    file_options.append(true);
                    let _ = file_options
                        .open(file_path)
                        .expect("couldnt open file for appending stderr");
                    println!("{builtin_out}");
                    return;
                }

                // when writing to files linux adds a newline character at the end
                builtin_out.push('\n');

                if c == ">>" || c == "1>>" {
                    file_options.append(true);
                }

                let mut file = file_options
                    .open(file_path)
                    .expect("couldnt open file for stdout redirection");
                let _ = file.write_all(builtin_out.as_bytes());
            } else {
                eprintln! {"expected file name after redirection"};
            };
        }
        Some(Token::Pipe(_t)) => todo!(),
        Some(_t) => unreachable!(),
    }
}
