//! Builds the static data bundle from the YAML and Markdown in the repo.
//! Usage: data-build [output.json] [repo-root]

use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let output = env::args().nth(1).map_or_else(|| PathBuf::from("app/public/data/bundle.json"), PathBuf::from);
    let root = env::args().nth(2).map_or_else(|| PathBuf::from("."), PathBuf::from);
    match data_build::build(&root).and_then(|bundle| data_build::write(&bundle, &output)) {
        Ok(summary) => {
            println!("wrote {} ({summary})", output.display());
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            eprintln!("data build failed with {} problem(s)", errors.len());
            ExitCode::FAILURE
        }
    }
}
