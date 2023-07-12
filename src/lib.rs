use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::{
    ExitAltScreenBufError, ExitRawModeAfterPauseError, PauseError, RunError, RunInAltScreenBufError,
};
use std::{
    ffi::{OsStr, OsString},
    io::stdout,
    process::{Command, ExitStatus},
};

#[derive(Debug, Default)]
pub struct PauseConfig {
    pub silent: bool,
}

pub fn run_in_alt_screen_buf(
    command: &OsStr,
    args: &[OsString],
    pause: Option<PauseConfig>,
) -> RunInAltScreenBufOutcome {
    match execute!(stdout(), EnterAlternateScreen)
        .map_err(|source| RunInAltScreenBufError::EnterAltScreenBuf { source })
    {
        Ok(()) => (),
        Err(e) => {
            return RunInAltScreenBufOutcome {
                run_res: Err(e),
                exit_raw_mode_after_pause_err: Ok(()),
                exit_alt_screen_err: Ok(()),
            }
        }
    };

    let RunAndPauseOutcome {
        run_res,
        exit_raw_mode_after_pause_err,
    } = run_and_pause(command, args, pause);
    let run_res = run_res.map_err(|source| RunInAltScreenBufError::Run { source });

    let exit_alt_screen_err =
        execute!(stdout(), LeaveAlternateScreen).map_err(|source| ExitAltScreenBufError { source });

    RunInAltScreenBufOutcome {
        run_res,
        exit_raw_mode_after_pause_err,
        exit_alt_screen_err,
    }
}

/// The outcome of calling [`run_in_alt_screen_buf`].
#[must_use]
#[derive(Debug)]
pub struct RunInAltScreenBufOutcome {
    /// Primary result of the operation. If this failed, then something prevented the execution of
    /// the command specified for [`run_in_alt_screen_buf`].
    pub run_res: Result<FinishedRunOutcome, RunInAltScreenBufError>,
    /// An error in exiting the alternate buffer that may have been encountered while trying to
    /// exit raw mode in the pause flow (after full command execution).
    pub exit_raw_mode_after_pause_err: Result<(), ExitRawModeAfterPauseError>,
    /// An error in exiting the alternate buffer that may have been encountered while exiting the
    /// alternate screen buffer (after a full execution and pause flow).
    pub exit_alt_screen_err: Result<(), ExitAltScreenBufError>,
}

/// The outcome of calling [`run_in_alt_screen_buf`].
#[must_use]
#[derive(Debug)]
pub struct FinishedRunOutcome {
    pub exit_status: ExitStatus,
    pub pause_res: Result<(), PauseError>,
}

fn run_and_pause(
    command: &OsStr,
    args: &[OsString],
    pause: Option<PauseConfig>,
) -> RunAndPauseOutcome {
    let mut exit_raw_mode_after_pause_err = Ok(());
    let run_res = Command::new(command)
        .args(args)
        .spawn()
        .map_err(|source| RunError::Spawn { source })
        .and_then(|mut child| child.wait().map_err(|source| RunError::Wait { source }))
        .map(|exit_status| {
            let mut pause_res = Ok(());
            if let Some(PauseConfig { silent }) = pause {
                if !silent {
                    println!("dab: Press any key to exit alternate buffer...");
                }
                pause_res = enable_raw_mode()
                    .map_err(|source| PauseError::EnterRawMode { source })
                    .and_then(|()| {
                        loop {
                            if let Event::Key(_key) = event::read()
                                .map_err(|source| PauseError::GetInputEvent { source })?
                            {
                                break;
                            }
                        }
                        Ok(())
                    })
                    .map(|res| {
                        exit_raw_mode_after_pause_err = disable_raw_mode()
                            .map_err(|source| ExitRawModeAfterPauseError { source });
                        res
                    });
            }
            FinishedRunOutcome {
                exit_status,
                pause_res,
            }
        });

    RunAndPauseOutcome {
        run_res,
        exit_raw_mode_after_pause_err,
    }
}

#[derive(Debug)]
struct RunAndPauseOutcome {
    pub(crate) run_res: Result<FinishedRunOutcome, RunError>,
    pub(crate) exit_raw_mode_after_pause_err: Result<(), ExitRawModeAfterPauseError>,
}

mod error {
    use std::{
        fmt::{self, Display, Formatter},
        io,
    };

    /// An error encountered while running [`crate::run_in_alt_screen_buf`] that may interfere with
    /// the run's correct execution, or a human's ability to evaluate it. Notably, it does _not_
    /// include clean-up errors that may still be undesirable, but should not affect a human's
    /// ability to determine such for themselves. For such error information, see
    /// [`crate::RunInAltScreenBufOutcome`].
    ///
    /// This error representation does not implement [`Display`] because [`Self::Run`] may contain
    /// multiple errors. Consuming code is encouraged to use the [`Self::description`] for standard
    /// error descriptions of each variant in the error flow it uses.
    #[derive(Debug)]
    pub enum RunInAltScreenBufError {
        EnterAltScreenBuf { source: io::Error },
        Run { source: RunError },
    }

    impl Display for RunInAltScreenBufError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let desc = match self {
                Self::EnterAltScreenBuf { .. } => "failed to enter alternate screen",
                Self::Run { .. } => "failed to run command",
            };
            write!(f, "{desc}")
        }
    }

    impl std::error::Error for RunInAltScreenBufError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(match self {
                Self::EnterAltScreenBuf { source } => source,
                Self::Run { source } => source,
            })
        }
    }

    /// An inner error source for [`Error::Run`]. You are strongly encouraged to handle
    /// [`Self::Pause`], which may contain the exit status of the command that was run in
    /// [`crate::run_in_alternate_screen`]
    #[derive(Debug)]
    pub enum RunError {
        Spawn { source: io::Error },
        Wait { source: io::Error },
    }

    impl Display for RunError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let desc = match self {
                Self::Spawn { .. } => "failed to spawn command",
                Self::Wait { .. } => "failed to get command's exit status",
            };
            write!(f, "{desc}")
        }
    }

    impl std::error::Error for RunError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(match self {
                Self::Spawn { source } => source,
                Self::Wait { source } => source,
            })
        }
    }

    /// Data associated with [`crate::RunInAltScreenBufOutcome`] `pause_res` field.
    #[derive(Debug)]
    pub enum PauseError {
        EnterRawMode { source: io::Error },
        GetInputEvent { source: io::Error },
    }

    impl Display for PauseError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let desc = match self {
                Self::EnterRawMode { .. } => "failed to enter raw mode",
                Self::GetInputEvent { .. } => "failed to read next input event",
            };
            write!(f, "{desc} in post-command pause flow")
        }
    }

    impl std::error::Error for PauseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(match self {
                Self::EnterRawMode { source } => source,
                Self::GetInputEvent { source } => source,
            })
        }
    }

    #[derive(Debug)]
    pub struct ExitRawModeAfterPauseError {
        pub(crate) source: io::Error,
    }

    impl Display for ExitRawModeAfterPauseError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "failed to exit raw mode in post-command pause flow")
        }
    }

    impl std::error::Error for ExitRawModeAfterPauseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.source)
        }
    }

    #[derive(Debug)]
    pub struct ExitAltScreenBufError {
        pub(crate) source: io::Error,
    }

    impl Display for ExitAltScreenBufError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "failed to exit alternate screen")
        }
    }

    impl std::error::Error for ExitAltScreenBufError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.source)
        }
    }
}
