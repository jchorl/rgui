use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
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
    #[snafu(display("Failed to parse"))]
    GrepResultsParseError {
        source: serde_json::error::Error,
    }
}
