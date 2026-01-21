#[allow(unused_imports)]
use std::io::{self, Write};

use anyhow::{Context};

fn main() -> anyhow::Result<()> {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let mut buf = String::new();
        let _input = io::stdin().read_line(&mut buf).context("reading stdin")?;
        let command = buf.trim_end();
        println!("{command}: command not found");
    }
}
