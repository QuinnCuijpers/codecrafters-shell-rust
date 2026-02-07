use clawsh::shell::Shell;

fn main() -> anyhow::Result<()> {
    let mut shell = Shell::setup()?;
    shell.run();
    // TODO: handle errors for exiting
    shell.exit()
}
