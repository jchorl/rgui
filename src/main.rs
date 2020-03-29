use serde::Deserialize;
use snafu::{ensure, ErrorCompat, ResultExt};
use std::env;
use std::process::Command;

mod errors;

type Result<T, E = errors::Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum RipGrepLine {
    Match {line_number: i32},
}

fn run_app() -> Result<()> {
    let grep_cmd = "rg";
    let args: Vec<String> = env::args().collect();
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

    let unparsed = String::from_utf8(output.stdout).unwrap();
    for s in unparsed.split("\n") {
        // the last line may be empty
        if s.is_empty() {
            break;
        }

        let mut untyped: serde_json::Value = serde_json::from_str(s).unwrap();

        // get the type
        let type_ = {
            let obj = untyped.as_object_mut().expect("object");
            let type_ = obj.remove("type").expect("`type` field");
            if let serde_json::Value::String(s) = type_ {
                s
            } else {
                panic!("type field not a string");
            }
        };

        // ignore non-match lines
        if type_ != "match" {
            continue;
        }

        println!("{}", s);
        println!("{}", type_);
    }

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
