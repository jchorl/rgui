use snafu::{ensure, ResultExt, Snafu};
use std::convert::TryFrom;
use std::process::Command;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Command '{}' failed: {}", command, source))]
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

    #[snafu(display("Results not UTF-8: {}", source))]
    NonUtf8Results { source: std::string::FromUtf8Error },

    #[snafu(display("Failed to parse: {}", source))]
    ResultsParseError { source: serde_json::error::Error },
}

#[derive(Debug, Clone)]
pub struct Match {
    pub file: String,
    pub line_number: u16,
}

pub fn run_rg(grep_args: Vec<String>) -> Result<Vec<Match>, Error> {
    let unparsed = run_cmd(grep_args)?;
    if unparsed == "" {
        return Ok(Vec::new());
    };
    let parsed = parse_results(unparsed)?;
    Ok(parsed)
}

fn run_cmd(mut grep_args: Vec<String>) -> Result<String, Error> {
    let grep_cmd = "rg";
    grep_args.push("--json".to_string());

    let output = Command::new(grep_cmd)
        .args(&*grep_args)
        .output()
        .context(CommandError { command: grep_cmd })?;

    // if the command is not found, looks like 127 gets returned.
    // so if 1 gets returned, assume no matches.
    if output.status.code().unwrap() == 1 {
        return Ok(String::from(""));
    };

    ensure!(
        output.status.success(),
        CommandResultError {
            command: grep_cmd,
            args: grep_args,
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        }
    );

    let single_str = String::from_utf8(output.stdout).context(NonUtf8Results {})?;
    Ok(single_str)
}

fn parse_results(unparsed: String) -> Result<Vec<Match>, Error> {
    let mut matches = Vec::new();

    for s in unparsed.split("\n") {
        // the last line may be empty
        if s.is_empty() {
            break;
        }

        let parsed: serde_json::Value = serde_json::from_str(s).context(ResultsParseError {})?;

        // ignore non-matches
        if parsed["type"] != "match" {
            continue;
        }

        matches.push(Match {
            file: String::from(parsed["data"]["path"]["text"].as_str().unwrap()),
            line_number: u16::try_from(parsed["data"]["line_number"].as_u64().unwrap()).unwrap(),
        })
    }

    Ok(matches)
}
