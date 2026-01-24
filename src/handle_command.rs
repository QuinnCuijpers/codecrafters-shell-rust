use std::{
    ffi::OsStr, fs::File, io::Write, path::PathBuf, process::{Command, Stdio}
};

use crate::input_parsing::Token;
use anyhow::Result;

pub(crate) fn handle_external_exec<'a, S, I, J>(s: &str, args: J, token_iter: I) -> Result<()>
where
    I: IntoIterator<Item = &'a Token>,
    J: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut iter = token_iter.into_iter();
    let mut command = Command::new(s);
    command.args(args);
    match iter.next() {
        None => {
            let mut child = command.spawn()?;
            child.wait()?;
        }
        Some(Token::Pipe(c)) => {
            if let Some(Token::Arg(file_name)) = iter.next() {
                let file_path = PathBuf::from(file_name);
                if let Some(parent_dir) = file_path.parent() {
                    std::fs::create_dir_all(parent_dir)?;
                }

                let mut file_options = File::options();
                file_options.create(true).write(true);
                match c.as_str() {
                    ">" | "1>" => {
                        let file = file_options.open(file_path)?;
                        command.stdout(Stdio::from(file));
                    }
                    "2>" => {
                        let file = file_options.open(file_path)?;
                        command.stderr(Stdio::from(file));
                    }
                    ">>" | "1>>" => {
                        let file = file_options.append(true).open(file_path)?;
                        command.stdout(Stdio::from(file));
                    }
                    _ => unreachable!("Unknown pipe operator"),
                }

                let mut child = command.spawn()?;
                child.wait()?;
            } else {
                eprintln! {"expected file name after redirection"};
            };
        }
        Some(_t) => unreachable!(),
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
        Some(Token::Pipe(c)) => {
            if let Some(Token::Arg(file_name)) = iter.next() {
                let file_path = PathBuf::from(file_name);
                if let Some(parent_dir) = file_path.parent()
                    && std::fs::create_dir_all(parent_dir).is_err()
                {
                    eprintln!("Failed to create dirs required for {}", file_path.display());
                    return;
                };

                if c == "2>" {
                    File::create(&file_path).expect("unable to create file");
                    println!("{builtin_out}");
                    return;
                }

                // when writing to files linux adds a newline character at the end
                builtin_out.push('\n');

                let mut file_options = File::options();
                file_options.create(true).write(true);

                if c == ">>" || c == "1>>" {
                    file_options.append(true);
                }

                let mut file = file_options.open(file_path).expect("couldnt open file for stdout redirection");
                let _ = file.write_all(builtin_out.as_bytes());
            } else {
                eprintln! {"expected file name after redirection"};
            };
        }
        Some(_t) => unreachable!(),
    }
}
