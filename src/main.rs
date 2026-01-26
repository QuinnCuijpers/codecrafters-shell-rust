mod handle_command;
mod input_parsing;
mod invoke;
mod readline;
mod trie;
mod util;

use anyhow::Context;
use handle_command::{handle_builtin, handle_external_exec};
use input_parsing::Builtin;
use input_parsing::Token;
use input_parsing::parse_input;
use input_parsing::tokenize_input;
use invoke::{invoke_cd, invoke_echo, invoke_pwd, invoke_type};
use rustyline::CompletionType;
use rustyline::Config;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::io::{self, Write};
use std::str::FromStr;

use crate::input_parsing::BUILTIN_COMMANDS;
use crate::readline::TrieCompleter;
use crate::util::find_exec_file;

fn main() -> anyhow::Result<()> {
    loop {
        let helper = TrieCompleter::with_builtin_commands(&BUILTIN_COMMANDS);
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .completion_show_all_if_ambiguous(true)
            .build();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(helper));
        let readline = rl.readline("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let input = match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                line
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };
        let trimmed_input = input.trim_end();
        let Ok(command_list) = parse_input(trimmed_input) else {
            continue;
        };

        let Some(tokens) = tokenize_input(command_list) else {
            continue;
        };
        // println!("{:?}", tokens);
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
