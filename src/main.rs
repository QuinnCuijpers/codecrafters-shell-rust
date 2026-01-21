#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

use anyhow::{Context, anyhow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Builtin {
    Echo,
    Exit, 
    Tipe,
}

impl FromStr for Builtin {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "exit" => Ok(Builtin::Exit),
            "type" => Ok(Builtin::Tipe),
            _ => Err(anyhow!(format!("unknown builtin {s}"))),
        }
    }
}

fn main() -> anyhow::Result<()> {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf).context("reading stdin")?;
        let input = buf.trim_end();
        let command_list: Vec<_> = input.split(" ").collect();
        if let Ok(command) = Builtin::from_str(command_list[0]) {
            match command {
                Builtin::Echo => invoke_echo(&command_list[1..]),
                Builtin::Exit => break,
                Builtin::Tipe => invoke_type(&command_list[1..]),
            }
        } else {
            println!("{input}: command not found")
        }
    }
    anyhow::Ok(())
}

fn invoke_echo(cmd_list: &[&str]) {
    let out = cmd_list.join(" ");
    println!("{out}");
}

fn invoke_type(cmd_list: &[&str]) {
    for cmd in cmd_list {
        if let Ok(_) = Builtin::from_str(cmd) {
            println!("{cmd} is a shell builtin");
        } else {
            println!("{cmd}: not found");
        }
    }
}
