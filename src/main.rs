use anyhow::{anyhow, Context};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{ffi::OsString, io::stdout, process::Command};

#[derive(Debug, Parser)]
struct Cli {
    command: OsString,
    args: Vec<OsString>,
}

fn main() -> anyhow::Result<()> {
    let Cli { command, args } = Cli::parse();

    execute!(stdout(), EnterAlternateScreen)
        .map_err(|e| anyhow!("failed to enter alternate screen: {}", e))?;

    let mut child = Command::new(command)
        .args(&args)
        .spawn()
        .context("failed to spawn command")?;

    child
        .wait()
        .context("failed to get command's exit status")?;

    execute!(stdout(), LeaveAlternateScreen)
        .map_err(|e| anyhow!("failed to exit alternate screen: {e}"))?;

    Ok(())
}
