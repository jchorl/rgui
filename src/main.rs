use snafu::{ensure, ResultExt, Snafu};
use std::env;
use std::process::Command;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Command '{}' failed", command))]
    CommandError {
        command: String,
        source: std::io::Error,
    },
    #[snafu(display("Command '{} {:?}' failed", command, args))]
    CommandResultError {
        command: String,
        args: Vec<String>,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn run_app() -> Result<()> {
    let grep_cmd = "rg";
    let args: Vec<String> = env::args().collect();
    let mut grep_args_builder: Vec<String> = args[1..].iter().cloned().collect();
    grep_args_builder.push("--json".to_string());
    let grep_args = &*grep_args_builder;

    let output = Command::new(grep_cmd)
        .args(grep_args)
        .output()
        .context(CommandError { command: grep_cmd })?;

    ensure!(output.status.success(), CommandResultError { command: grep_cmd, args: grep_args });

    Ok(())
}

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
