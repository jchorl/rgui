use snafu::{ErrorCompat, ResultExt, Snafu};
use std::env;

mod rg;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("rg failed"))]
    RgError {
        source: rg::errors::Error,
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn run_app() -> Result<()> {
    let unparsed = rg::run_rg(env::args().collect())
        .context(RgError {})?;

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
