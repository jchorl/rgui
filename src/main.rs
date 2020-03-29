use snafu::{ErrorCompat, ResultExt, Snafu};
use std::env;

mod bat;
mod rg;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("rg failed: {}", source))]
    RgError { source: rg::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn run_app() -> Result<()> {
    let results = rg::run_rg(env::args().skip(1).collect::<Vec<_>>()).context(RgError {})?;

    let result = &results[0];
    bat::display_file(&result.file);
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
