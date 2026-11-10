//! Thin wasm-bindgen wrapper around sim-core: JSON in, JSON out. No logic here.
//!
//! The data bundle is parsed once and kept, because it is large and never
//! changes while the page is open. Everything else is a pure call.

use std::cell::RefCell;

use sim_core::model::Target;
use sim_core::{Artifacts, Bundle, Depth, SCHEMA_VERSION, World};
use wasm_bindgen::prelude::*;

thread_local! {
    static BUNDLE: RefCell<Option<Bundle>> = const { RefCell::new(None) };
}

fn with_bundle<T>(run: impl FnOnce(&Bundle) -> Result<T, JsError>) -> Result<T, JsError> {
    BUNDLE.with(|slot| match slot.borrow().as_ref() {
        Some(bundle) => run(bundle),
        None => Err(JsError::new("the data bundle has not been loaded")),
    })
}

/// Load the data bundle. Call once before anything else.
#[wasm_bindgen]
pub fn set_bundle(bundle_json: &str) -> Result<(), JsError> {
    let bundle: Bundle = serde_json::from_str(bundle_json)?;
    if bundle.schema_version != SCHEMA_VERSION {
        return Err(JsError::new(&format!(
            "data bundle is schema {} and this core reads schema {SCHEMA_VERSION}",
            bundle.schema_version
        )));
    }
    BUNDLE.with(|slot| *slot.borrow_mut() = Some(bundle));
    Ok(())
}

#[wasm_bindgen]
pub fn evaluate_json(world_json: &str) -> Result<String, JsError> {
    let world: World = serde_json::from_str(world_json)?;
    with_bundle(|bundle| Ok(serde_json::to_string(&sim_core::evaluate(&world, bundle))?))
}

/// Grade a mission. The result carries the full report, with every trace.
#[wasm_bindgen]
pub fn grade_json(
    mission_id: &str,
    depth_json: &str,
    world_json: &str,
    artifacts_json: &str,
) -> Result<String, JsError> {
    let world: World = serde_json::from_str(world_json)?;
    let depth: Depth = serde_json::from_str(depth_json)?;
    let artifacts: Artifacts = serde_json::from_str(artifacts_json)?;
    with_bundle(|bundle| {
        let mission = bundle.missions.get(mission_id).ok_or_else(|| JsError::new("no such mission"))?;
        Ok(serde_json::to_string(&sim_core::grade(mission, depth, &world, &artifacts, bundle))?)
    })
}

/// The address a dial target points at in this world.
#[wasm_bindgen]
pub fn resolve_json(world_json: &str, target_json: &str) -> Result<String, JsError> {
    let world: World = serde_json::from_str(world_json)?;
    let target: Target = serde_json::from_str(target_json)?;
    with_bundle(|bundle| Ok(serde_json::to_string(&sim_core::resolve_target(&world, &target, bundle))?))
}
