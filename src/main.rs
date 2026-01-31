mod handle_command;
mod input_parsing;
mod invoke;
mod readline;
mod trie;
mod util;

use anyhow::Context;
use input_parsing::Token;
use input_parsing::parse_input;
use input_parsing::tokenize_input;
use rustyline::CompletionType;
use rustyline::Config;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::read;
use std::io::{self, Write};
use std::path::Path;

use crate::handle_command::handle_command;
use crate::input_parsing::BUILTIN_COMMANDS;
use crate::readline::TrieCompleter;

fn main() -> anyhow::Result<()> {
    let env_file = std::env::var_os("HISTFILE");

    let history_file = if let Some(file_name) = env_file {
        if !Path::new(&file_name).exists() {
            File::create(&file_name)?;
        }
        file_name
    } else {
        File::create("/tmp/history.txt")?;
        "/tmp/history.txt".into()
    };
    
    let helper = TrieCompleter::with_builtin_commands(&BUILTIN_COMMANDS);
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(helper));
    rl.load_history(&history_file)?;

    let old_contents = read(&history_file)?;
    
    loop {
        
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
                println!("Error: {err:?}");
                break;
            }
        };

        let trimmed_input = input.trim_end();
        let command_list = parse_input(trimmed_input);

        let Some(tokens) = tokenize_input(command_list) else {
            continue;
        };
        // eprintln!("{:?}", tokens);
        let mut token_iter = tokens.iter().peekable();

        let command = token_iter.next().unwrap();

        let Token::Command(cmd_str) = command else {
            // first string should always be a command
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

        let history = rl.history_mut();        
        handle_command(cmd_str, args.iter(), &mut token_iter, history)?;

    }

    // can't save history directly as it will prepend #V2
    // if Path::new(&history_file).exists() {
    //     rl.save_history(&history_file)?;
    // } else {
    //     File::create(&history_file)?;
    //     rl.save_history(&history_file)?;
    // }

    let mut file = OpenOptions::new().append(true).create(true).open(&history_file)?;
    let mut new_contents = vec![];
    for entry in rl.history() {
        let mut new_entry = entry.clone();
        new_entry.push('\n');
        new_contents.append(&mut new_entry.as_bytes().to_owned());
    }

    if new_contents.starts_with(&old_contents) {
        new_contents = new_contents[old_contents.len()..].to_vec();
    }

    _ = file.write_all(&new_contents);



    if history_file == "/tmp/history.txt" {
        fs::remove_file("/tmp/history.txt")?;
    }

    anyhow::Ok(())
}
