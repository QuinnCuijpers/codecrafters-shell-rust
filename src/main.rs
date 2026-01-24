use anyhow::{Context, Result, anyhow};
use faccess::PathExt;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env::{current_dir, set_current_dir},
    ffi::OsStr,
    fs::File,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Command(String),
    Pipe(String),
    Arg(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Builtin {
    Echo,
    Exit,
    Tipe,
    Pwd,
    Cd,
}

impl FromStr for Builtin {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "exit" => Ok(Builtin::Exit),
            "type" => Ok(Builtin::Tipe),
            "pwd" => Ok(Builtin::Pwd),
            "cd" => Ok(Builtin::Cd),
            _ => Err(anyhow!(format!("unknown builtin {s}"))),
        }
    }
}

fn main() -> anyhow::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf).context("reading stdin")?;
        let input = buf.trim_end();
        let Ok(command_list) = parse_input(input) else {
            continue;
        };

        let Some(tokens) = tokenize_input(command_list) else {
            continue;
        };
        let mut token_iter = tokens.iter().peekable();

        let command = token_iter.next().unwrap();

        let Token::Command(s) = command else {
            // first string should always be a command
            continue;
        };

        let mut args = vec![];
        while let Some(Token::Arg(s)) = token_iter.peek() {
            args.push(s);
            token_iter.next();
        }

        if let Ok(builtin) = Builtin::from_str(s.as_str()) {
            let builtin_out = match builtin {
                Builtin::Echo => Some(invoke_echo(args)),
                Builtin::Exit => break,
                Builtin::Tipe => Some(invoke_type(args)),
                Builtin::Pwd => Some(invoke_pwd(args)?),
                Builtin::Cd => invoke_cd(args),
            };
            handle_builtin(builtin_out, token_iter);
        } else if find_exec_file(s).is_some() {
            handle_external_exec(s, args, token_iter)?;
        } else {
            println!("{s}: command not found")
        }
    }
    anyhow::Ok(())
}

fn handle_builtin<'a, I>(builtin_out: Option<String>, token_iter: I)
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

                if c != ">" && c != "1>" {
                    File::create(&file_path).expect("unable to create file");
                    println!("{builtin_out}");
                    return;
                }

                // when writing to files linux adds a newline character at the end
                builtin_out.push('\n');

                std::fs::write(file_name, builtin_out)
                    .expect("can only fail if parent dir doesn't exist");
            } else {
                eprintln! {"expected file name after redirection"};
            };
        }
        Some(_t) => unreachable!(),
    }
}

fn handle_external_exec<'a, S, I, J>(s: &str, args: J, token_iter: I) -> Result<()>
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
                let file = File::create(file_path)?;
                match c.as_str() {
                    ">" | "1>" => {
                        command.stdout(Stdio::from(file));
                    }
                    "2>" => {
                        command.stderr(Stdio::from(file));
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

fn parse_input(input: &str) -> Result<Vec<String>> {
    let mut command_list: Vec<String> = vec![];
    let mut buf = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                if in_single_quotes || in_double_quotes {
                    buf.push(c);
                } else {
                    if buf.is_empty() {
                        continue;
                    }
                    command_list.push(buf.clone());
                    buf.clear();
                }
            }
            '\\' => {
                if !in_single_quotes
                    && !in_double_quotes
                    && let Some(next_char) = chars.next()
                {
                    buf.push(next_char)
                }
                if in_single_quotes {
                    buf.push(c);
                }
                if in_double_quotes && let Some(&c) = chars.peek() {
                    // unwrap safe as the peek returns Some
                    match c {
                        '\"' => buf.push(chars.next().unwrap()),
                        '\\' => buf.push(chars.next().unwrap()),
                        _ => buf.push('\\'),
                    }
                }
            }
            '\'' => {
                if in_double_quotes {
                    buf.push(c);
                    continue;
                }
                in_single_quotes = !in_single_quotes;
            }
            '\"' => {
                if !in_single_quotes {
                    in_double_quotes = !in_double_quotes;
                } else {
                    buf.push(c);
                }
            }
            _ => buf.push(c),
        }
    }
    if !buf.is_empty() {
        command_list.push(buf.clone());
    }
    Ok(command_list)
}

fn tokenize_input(input: Vec<String>) -> Option<Vec<Token>> {
    if input.is_empty() {
        eprintln!("input string was empty when attempted tokinization");
        return None;
    }
    let mut tokenized = vec![];
    let mut iter = input.into_iter();
    tokenized.push(Token::Command(iter.next().unwrap())); //first String always exists by the above if case

    for s in iter {
        match s.as_str() {
            ">" | "1>" | "2>" => tokenized.push(Token::Pipe(s)),
            _ => tokenized.push(Token::Arg(s)),
        }
    }

    Some(tokenized)
}

fn invoke_echo<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    cmd_list
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn invoke_type<I, S>(cmd_list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use std::fmt::Write;
    let mut buf = String::new();
    for (i, cmd) in cmd_list.into_iter().enumerate() {
        let cmd = cmd.as_ref();
        let cmd_str = if i != 0 {
            format!("\n{cmd}")
        } else {
            cmd.to_string()
        };
        let cmd_str = cmd_str.as_str();
        if Builtin::from_str(cmd).is_ok() {
            let _ = write!(buf, "{cmd_str} is a shell builtin");
        }
        // go through every directory and check if a file with the name exist that has exec permissions
        else if let Some(file_path) = find_exec_file(cmd) {
            let _ = write!(buf, "{cmd_str} is {}", file_path.display());
        } else {
            let _ = write!(buf, "{cmd_str}: not found");
        }
    }
    buf
}

// TODO: consider making this a result to handle distinct errors
fn find_exec_file(cmd: &str) -> Option<PathBuf> {
    let Some(env_path) = std::env::var_os("PATH") else {
        eprintln!("PATH env var not set");
        return None;
    };
    for mut path in std::env::split_paths(&env_path) {
        if let Ok(exists) = path.try_exists() {
            if !exists {
                continue;
            }
            path.push(cmd);
            if path.executable() {
                return Some(path);
            }
        }
    }
    None
}

fn invoke_pwd<I, S>(_cmd_list: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let curr = current_dir()?;
    Ok(format!("{}", curr.display()))
}

fn invoke_cd<I, S>(cmd_list: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let cmd_list: Vec<_> = cmd_list.into_iter().collect();
    assert!(cmd_list.len() == 1);
    let path = match cmd_list[0].as_ref() {
        "~" => PathBuf::from(&std::env::var_os("HOME").expect("HOME env key not set")),
        _ => PathBuf::from(&cmd_list[0].as_ref()),
    };
    if path.exists() {
        // if cd fails then proceed to next REPL iter
        let _ = set_current_dir(path);
        None
    } else {
        Some(format!("cd: {}: No such file or directory", path.display()))
    }
}
