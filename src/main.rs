#[allow(unused_imports)]
use std::io::{self, Write};

use anyhow::{Context, Ok};

fn main() -> anyhow::Result<()> {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let mut buf = String::new();
        let _input = io::stdin().read_line(&mut buf).context("reading stdin")?;
        let command = buf.trim_end();
        if command == "exit" {
            break;  
        } 
        let command_list: Vec<_> = command.split(" ").collect();
        match command_list[0] {
            "echo" => invoke_echo(&command_list[1..]),
            _ => println!("{command}: command not found"),
        }
    }
    Ok(())
}

fn invoke_echo(cmd_list: &[&str]) {
    let out = cmd_list.join(" ");
    println!("{out}");
}
