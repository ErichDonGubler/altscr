use anyhow::{anyhow, Context};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    ffi::{OsStr, OsString},
    io::stdout,
    process::Command,
};

#[derive(Debug, Default)]
pub struct PauseConfig {
    pub silent: bool,
}

pub fn run_in_alt_screen_buf(
    command: &OsStr,
    args: &[OsString],
    pause: Option<PauseConfig>,
) -> anyhow::Result<()> {
    execute!(stdout(), EnterAlternateScreen)
        // We can still bail here if this doesn't work, whew!
        .map_err(|e| anyhow!("failed to enter alternate screen: {}", e))?;

    let run_res = run_and_pause(command, args, pause);

    if let Err(e) = execute!(stdout(), LeaveAlternateScreen) {
        let e = anyhow!("warning: failed to exit alternate screen: {e}");
        eprintln!("{e:#}");
    }

    run_res
}

// TODO: preserve exit status
fn run_and_pause(
    command: &OsStr,
    args: &[OsString],
    pause: Option<PauseConfig>,
) -> anyhow::Result<()> {
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

    Ok(())
}
