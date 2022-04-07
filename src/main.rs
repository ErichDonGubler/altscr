use anyhow::{anyhow, Context};
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{ffi::OsString, io::stdout, process::Command};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long)]
    pause: bool,
    command: OsString,
    args: Vec<OsString>,
}

fn main() -> anyhow::Result<()> {
    let Cli {
        pause,
        command,
        args,
    } = Cli::parse();

    execute!(stdout(), EnterAlternateScreen)
        .map_err(|e| anyhow!("failed to enter alternate screen: {}", e))?;

    let mut child = Command::new(command)
        .args(&args)
        .spawn()
        .context("failed to spawn command")?;

    child
        .wait()
        .context("failed to get command's exit status")?;

    if pause {
        println!("dab: Press any key to exit alternate buffer...");
        enable_raw_mode().map_err(|e| anyhow!("failed to enter raw mode: {e}"))?;
        loop {
            if let Event::Key(_key) =
                event::read().map_err(|e| anyhow!("failed to read next input event: {e}"))?
            {
                break;
            }
        }
        disable_raw_mode().map_err(|e| anyhow!("failed to enter raw mode: {e}"))?;
    }

    execute!(stdout(), LeaveAlternateScreen)
        .map_err(|e| anyhow!("failed to exit alternate screen: {e}"))?;

    Ok(())
}
