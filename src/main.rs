use std::env;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    let grep_cmd = "rg";
    let args: Vec<String> = env::args().collect();
    let mut grep_args: Vec<String> = args[1..].iter().cloned().collect();
    grep_args.push("--json".to_string());

    let output = Command::new(grep_cmd)
        .args(grep_args)
        .output()
        .expect(&format!("failed executing '{}'", grep_cmd));
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());

}
