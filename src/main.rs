use anyhow::{bail, Context};
use clap::Parser;
use dab::{run_in_alt_screen_buf, FinishedRunOutcome, PauseConfig, RunInAltScreenBufOutcome};
use std::{ffi::OsString, process::ExitCode, str::FromStr};

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

fn main() -> ExitCode {
    let Cli {
        pause,
        command_and_args,
    } = Cli::parse();

    let (command, args) = match command_and_args
        .split_first()
        .context("no command provided")
    {
        Ok(tup) => tup,
        Err(e) => {
            eprintln!("{e:#}");
            return ExitCode::from(255);
        }
    };

    let pause = pause
        .unwrap_or(Some(PauseOption::Print))
        .unwrap_or(PauseOption::Print)
        .into_pause_config();

    let RunInAltScreenBufOutcome {
        run_res,
        exit_raw_mode_after_pause_err,
        exit_alt_screen_err,
    } = run_in_alt_screen_buf(command, args, pause);

    let exit_code = match run_res {
        Ok(finished) => {
            let FinishedRunOutcome {
                exit_status,
                pause_res,
            } = finished;
            let exit_code = exit_status
                .code()
                .map(|code| {
                    let code = u8::try_from(code).unwrap_or_else(|_e| {
                        eprintln!(
                            "dab: warning: child exit code ({code}) is larger than maximum value \
                            of single byte, truncating"
                        );
                        code as u8
                    });
                    ExitCode::from(code)
                })
                .unwrap_or_else(|| {
                    eprintln!("dab: child process was terminated by signal");
                    ExitCode::from(253)
                });
            match pause_res {
                Ok(()) => (),
                Err(e) => {
                    let e = anyhow::Error::new(e);
                    eprintln!("dab: warning: succeeded in waiting for child process, but {e:#}");
                }
            }
            exit_code
        }
        Err(e) => {
            let e = anyhow::Error::new(e);
            eprintln!("dab: error: {e:#}");
            return ExitCode::from(254);
        }
    };

    match exit_raw_mode_after_pause_err {
        Ok(()) => (),
        Err(e) => {
            let e = anyhow::Error::new(e);
            eprintln!("dab: warning: {e:#}");
            return ExitCode::from(2);
        }
    }
    match exit_alt_screen_err {
        Ok(()) => (),
        Err(e) => {
            let e = anyhow::Error::new(e);
            eprintln!("dab: warning: {e:#}");
            return ExitCode::from(2);
        }
    }

    exit_code
}
