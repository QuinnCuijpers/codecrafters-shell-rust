mod handle_command;
mod input_parsing;
mod invoke;
mod util;

use anyhow::Context;
use handle_command::{handle_builtin, handle_external_exec};
use input_parsing::Builtin;
use input_parsing::Token;
use input_parsing::parse_input;
use input_parsing::tokenize_input;
use invoke::{invoke_cd, invoke_echo, invoke_pwd, invoke_type};
#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

use crate::util::find_exec_file;

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
