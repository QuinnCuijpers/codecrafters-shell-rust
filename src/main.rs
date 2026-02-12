use clawsh::shell::{ClawshError, Shell};

fn main() -> Result<(), ClawshError> {
    let mut shell = Shell::setup()?;
    shell.run();
    shell.exit()?;
    Ok(())
}
