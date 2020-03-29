use std::process::Command;
use snafu::{ensure, ResultExt};

pub mod errors;
mod types;

pub fn run_rg(args: Vec<String>) -> Result<String, errors::Error> {
    let grep_cmd = "rg";
    let mut grep_args_builder: Vec<String> = args[1..].iter().cloned().collect();
    grep_args_builder.push("--json".to_string());
    let grep_args = &*grep_args_builder;

    let output = Command::new(grep_cmd)
        .args(grep_args)
        .output()
        .context(errors::CommandError { command: grep_cmd })?;

    ensure!(output.status.success(), errors::CommandResultError {
        command: grep_cmd,
        args: grep_args,
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    });

    let single_str = String::from_utf8(output.stdout).context(errors::NonUtf8Results {})?;
    Ok(single_str)
}
