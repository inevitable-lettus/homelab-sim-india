//! Thin wasm-bindgen wrapper around sim-core: JSON in, JSON out. No logic here.

use sim_core::{Bundle, World};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate_json(world_json: &str, bundle_json: &str) -> Result<String, JsError> {
    let world: World = serde_json::from_str(world_json)?;
    let bundle: Bundle = serde_json::from_str(bundle_json)?;
    Ok(serde_json::to_string(&sim_core::evaluate(&world, &bundle))?)
}
