use anyhow::{anyhow, bail, Context};
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{ffi::OsString, io::stdout, process::Command, str::FromStr};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, value_parser = PauseOption::from_str)]
    pause: Option<Option<PauseOption>>,
    #[clap(raw(true))]
    command_and_args: Vec<OsString>,
}

#[derive(Clone, Debug)]
enum PauseOption {
    No,
    Silent,
    Print,
}

impl FromStr for PauseOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "no" => Ok(Self::No),
            "print" => Ok(Self::Print),
            "silent" => Ok(Self::Silent),
            unrecognized => bail!("unrecognized pause option {unrecognized:?}"),
        }
    }
}

impl PauseOption {
    fn into_pause_config(self) -> Option<PauseConfig> {
        match self {
            PauseOption::No => None,
            PauseOption::Silent => Some(PauseConfig { silent: true }),
            PauseOption::Print => Some(PauseConfig { silent: false }),
        }
    }
}

#[derive(Debug, Default)]
struct PauseConfig {
    silent: bool,
}

fn main() -> anyhow::Result<()> {
    let Cli {
        pause,
        command_and_args,
    } = Cli::parse();

    let (command, args) = command_and_args
        .split_first()
        .context("no command provided")?;

    let pause = pause
        .unwrap_or(Some(PauseOption::Print))
        .unwrap_or(PauseOption::Print)
        .into_pause_config();

    execute!(stdout(), EnterAlternateScreen)
        .map_err(|e| anyhow!("failed to enter alternate screen: {}", e))?;

    let mut child = Command::new(command)
        .args(args)
        .spawn()
        .context("failed to spawn command")?;

    child
        .wait()
        .context("failed to get command's exit status")?;

    if let Some(PauseConfig { silent }) = pause {
        if !silent {
            println!("dab: Press any key to exit alternate buffer...");
        }
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
