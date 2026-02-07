use std::io::{self, Write};

use rustyline::error::ReadlineError;

use crate::{
    parser::{Token, split_words, tokenize_input},
    shell::{Shell, error::ShellError, handle_command},
};

impl Shell {
    #[allow(clippy::missing_panics_doc)]
    pub fn run(&mut self) {
        loop {
            let readline = self.rl.readline("$ ");
            match io::stdout().flush() {
                Ok(()) => {}
                Err(e) => {
                    let err = ShellError::FailedStdoutFlush(e);
                    eprintln!("{err}");
                }
            }

            let input = match readline {
                Ok(line) => {
                    #[allow(clippy::expect_used)]
                    self.rl.add_history_entry(line.as_str())
                    .expect("`add_history_entry` cannot error for filehistory due to how the trait function is implemented by rusytline");
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
                args.push(s.clone());
                token_iter.next();
            }

            // for s in rl.history() {
            //     println!("{s}");
            // }

            let history = self.rl.history_mut();
            match handle_command(cmd_str, &args, &mut token_iter, history) {
                Ok(()) => {}
                Err(e) => eprintln!("{e}"),
            }
        }
    }
}
