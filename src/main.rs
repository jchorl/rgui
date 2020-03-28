use std::env;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    let grep_cmd = "rg";
    let args: Vec<String> = env::args().collect();
    let grep_args = &args[1..];

    let output = Command::new(grep_cmd)
        .args(grep_args)
        .output()
        .expect(&format!("failed executing '{}' with args {:?}", grep_cmd, grep_args));
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());

}
