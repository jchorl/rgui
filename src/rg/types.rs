use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum RipGrepLine {
    Match {line_number: i32},
}
