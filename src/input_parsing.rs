use anyhow::Result;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Command(String),
    Redirect(String),
    Arg(String),
    Pipe(String), //TODO: may not require to hold data as only one character creates this
}

pub(crate) const BUILTIN_COMMANDS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
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
            _ => Err(anyhow::anyhow!(format!("unknown builtin {s}"))),
        }
    }
}

pub(crate) fn parse_input(input: &str) -> Result<Vec<String>> {
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

pub(crate) fn tokenize_input(input: Vec<String>) -> Option<Vec<Token>> {
    if input.is_empty() {
        eprintln!("input string was empty when attempted tokinization");
        return None;
    }
    let mut tokenized = vec![];
    let mut iter = input.into_iter();
    tokenized.push(Token::Command(iter.next().unwrap())); //first String always exists by the above if case

    let mut new_command = false;
    for s in iter {
        match s.as_str() {
            ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => tokenized.push(Token::Redirect(s)),
            "|" => {
                new_command = true;
                tokenized.push(Token::Pipe(s));
            }
            _ => {
                if new_command {
                    tokenized.push(Token::Command(s));
                    new_command = !new_command;
                } else {
                    tokenized.push(Token::Arg(s));
                }
            }
        }
    }

    Some(tokenized)
}
