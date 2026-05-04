//! Entrypoint for the clawsh shell.
use clawsh::Shell;

fn main() -> clawsh::Result<()> {
    let mut shell = Shell::setup()?;
    shell.run();
    shell.exit()?;
    Ok(())
}
