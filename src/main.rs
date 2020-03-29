use snafu::{ensure, ErrorCompat, ResultExt, Snafu};
use std::env;
use std::process::Command;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Command '{}' failed", command))]
    CommandError {
        command: String,
        source: std::io::Error,
    },
    #[snafu(display("Command '{} {}' failed:\n\
                    stdout:\n\
                    {}\n\
                    stderr:\n\
                    {}", command, args.join(" "), stdout, stderr))]
    CommandResultError {
        command: String,
        args: Vec<String>,
        stdout: String,
        stderr: String,
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

    ensure!(output.status.success(), CommandResultError {
        command: grep_cmd,
        args: grep_args,
        stdout: String::from_utf8(output.stdout).expect("stdout not utf8"),
        stderr: String::from_utf8(output.stderr).expect("stdout not utf8"),
    });

    Ok(())
}

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                println!("{}", backtrace);
            }
            1
        }
    });
}
