use std::process::Command;
use snafu::{ensure, ResultExt, Snafu};

mod types;

#[derive(Debug, Snafu)]
pub enum Error {
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

    #[snafu(display("Results not UTF-8"))]
    NonUtf8Results {
        source: std::string::FromUtf8Error,
    },

    #[snafu(display("Failed to parse"))]
    GrepResultsParseError {
        source: serde_json::error::Error,
    },
}

pub fn run_rg(args: Vec<String>) -> Result<String, Error> {
    let grep_cmd = "rg";
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
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    });

    let single_str = String::from_utf8(output.stdout).context(NonUtf8Results {})?;
    Ok(single_str)
}
