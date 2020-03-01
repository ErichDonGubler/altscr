use {
    anyhow::{anyhow, Error as AnyhowError},
    atty::{is as is_a_tty, Stream},
    std::{
        env::args,
        io::{stdout, Write},
        process::{exit, Command},
    },
};
fn main() {
    let found_tty = is_a_tty(Stream::Stdin) && is_a_tty(Stream::Stderr);
    eprintln!("found_tty: {:#?}", found_tty);
    if found_tty {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        write!(stdout, "\x1b[?1049h").unwrap();
        stdout.flush().unwrap();
    }
    let error_code = run();
    if found_tty {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        write!(stdout, "\x1b[?1049l").unwrap();
        stdout.flush().unwrap();
    }
    exit(error_code.unwrap_or_else(|e| {
        eprintln!("{}", e);
        1
    }));
}

fn run() -> Result<i32, AnyhowError> {
    let mut args = args().skip(1);
    Command::new(
        args.next()
            .ok_or_else(|| anyhow!("fatal: no command provided to execute"))?,
    )
    .args(args)
    .spawn()
    .map_err(|e| anyhow!("fatal: failed to spawn child process: {}", e))?
    .wait()
    .map_err(|e| anyhow!("fatal: failed waiting for child process: {}", e))
    .and_then(|status| {
        status
            .code()
            .ok_or_else(|| anyhow!("warning: failed to get error code from child process"))
    })
}
