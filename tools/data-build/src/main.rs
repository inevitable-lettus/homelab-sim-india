//! Emits the static data bundle. Phase 0 writes an empty bundle; Phase 3 reads
//! and validates the YAML packs in data/.

use std::{env, error::Error, fs, path::PathBuf};

use sim_core::Bundle;

fn main() -> Result<(), Box<dyn Error>> {
    let output = env::args().nth(1).map_or_else(
        || PathBuf::from("app/public/data/bundle.json"),
        PathBuf::from,
    );

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut json = serde_json::to_string_pretty(&Bundle::empty())?;
    json.push('\n');
    fs::write(&output, json)?;
    println!("wrote {}", output.display());
    Ok(())
}
