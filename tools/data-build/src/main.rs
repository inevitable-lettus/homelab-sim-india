//! Builds the static data from the YAML and Markdown in the repo.
//! Usage: data-build [bundle.json] [repo-root]
//!        data-build schemas [directory]
//!
//! The sandbox catalog is written next to the bundle as `catalog.json`.

use std::{env, path::PathBuf, process::ExitCode};

fn run() -> Result<Vec<String>, data_build::Errors> {
    let first = env::args().nth(1);
    if first.as_deref() == Some("schemas") {
        let dir = env::args().nth(2).map_or_else(|| PathBuf::from("schemas"), PathBuf::from);
        return Ok(vec![format!("wrote {} ({})", dir.display(), data_build::write_schemas(&dir)?)]);
    }
    let output = first.map_or_else(|| PathBuf::from("app/public/data/bundle.json"), PathBuf::from);
    let root = env::args().nth(2).map_or_else(|| PathBuf::from("."), PathBuf::from);
    let (missions, catalog) = data_build::split(data_build::build(&root)?);
    let catalog_path = output.with_file_name("catalog.json");
    Ok(vec![
        format!("wrote {} ({})", output.display(), data_build::write(&missions, &output)?),
        format!("wrote {} ({})", catalog_path.display(), data_build::write(&catalog, &catalog_path)?),
    ])
}

fn main() -> ExitCode {
    match run() {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
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
