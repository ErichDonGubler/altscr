use anyhow::{bail, Context};
use clap::Parser;
use dab::{run_in_alt_screen_buf, PauseConfig};
use std::{ffi::OsString, str::FromStr};

#[derive(Debug, Parser)]
#[command(about, author, version)]
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

    run_in_alt_screen_buf(command, args, pause)
        .context("failed to run command in alternate buffer")
}
