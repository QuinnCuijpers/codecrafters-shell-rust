use std::io::{self, Write};

use anyhow::Context;
use rustyline::error::ReadlineError;

use crate::{
    parser::{Token, split_words, tokenize_input},
    shell::{Shell, handle_command},
};

impl Shell {
    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let readline = self.rl.readline("$ ");
            io::stdout().flush().context("flushing stdout")?;

            let input = match readline {
                Ok(line) => {
                    self.rl.add_history_entry(line.as_str())?;
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
                    println!("Error: {err:?}");
                    break;
                }
            };

            let trimmed_input = input.trim_end();
            let command_list = split_words(trimmed_input);

            let Some(tokens) = tokenize_input(command_list) else {
                continue;
            };
            // eprintln!("{:?}", tokens);
            let mut token_iter = tokens.iter().peekable();

            let Some(Token::Command(cmd_str)) = token_iter.next() else {
                // TODO: consider adding error for this
                continue;
            };

            if cmd_str == "exit" {
                break;
            }

            let mut args = vec![];
            while let Some(Token::Arg(s)) = token_iter.peek() {
                args.push(s);
                token_iter.next();
            }

            // for s in rl.history() {
            //     println!("{s}");
            // }

            let history = self.rl.history_mut();
            handle_command(cmd_str, args.iter(), &mut token_iter, history)?;
        }
        anyhow::Ok(())
    }
}
