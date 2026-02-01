use std::{fs::File, io::Write, iter::Peekable, path::PathBuf, process::{Command, Stdio}};

use crate::parser::Token;

pub(crate) fn redirect_builtin_output<'a, I>(
    redirect_symb: &str,
    builtin_out: &str,
    token_iter: &mut Peekable<I>,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
{
    let Some(Token::Arg(file_name)) = token_iter.next() else {
        anyhow::bail! {"expected file name after redirection"};
    };
    let file_path = PathBuf::from(file_name);
    if let Some(parent_dir) = file_path.parent()
        && std::fs::create_dir_all(parent_dir).is_err()
    {
        anyhow::bail!("Failed to create dirs required for {}", file_path.display());
    }

    let mut file_options = File::options();
    file_options.create(true).write(true);

    match redirect_symb {
        "2>" => {
            let _ = file_options
                .open(file_path)
                .expect("couldnt open file for redirecting stderr");
            print!("{builtin_out}");
        }
        "2>>" => {
            file_options.append(true);
            let _ = file_options
                .open(file_path)
                .expect("couldnt open file for appending stderr");
            print!("{builtin_out}");
        }
        ">>" | "1>>" => {
            // when writing to files linux adds a newline character at the end
            file_options.append(true);
            let mut file = file_options
                .open(file_path)
                .expect("couldnt open file for stdout appending");
            let _ = file.write_all(builtin_out.as_bytes());
        }
        ">" | "1>" => {
            // when writing to files linux adds a newline character at the end
            let mut file = file_options
                .open(file_path)
                .expect("couldnt open file for stdout redirection");
            let _ = file.write_all(builtin_out.as_bytes());
        }
        _ => unreachable!(),
    }
    anyhow::Ok(())
}

pub(crate) fn redirect_external<'a, I>(command: &mut Command, redirect_symb: &str, token_iter: &mut Peekable<I>) -> anyhow::Result<()> 
where
    I: Iterator<Item = &'a Token>,
{
    let Some(Token::Arg(file_name)) = token_iter.next() else {
                anyhow::bail!("expected file name after redirection");
            };

            let file_path = PathBuf::from(file_name);
            if let Some(parent_dir) = file_path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }

            let mut file_options = File::options();
            file_options.create(true).write(true);

            match redirect_symb {
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
            anyhow::Ok(())
}