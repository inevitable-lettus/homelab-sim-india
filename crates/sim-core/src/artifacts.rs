//! Files the learner wrote, turned into facts in the world before grading.
//! The parsers arrive with the terminal missions.

use serde::Serialize;

use crate::Status;
use crate::bundle::Mission;
use crate::grade::Artifacts;
use crate::model::{Id, World};
use crate::reason::Reason;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactResult {
    pub id: Id,
    pub status: Status,
    pub reason: Reason,
    /// Line-level problems, in file order.
    pub problems: Vec<Problem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub line: u32,
    pub message: String,
}

pub(crate) fn apply(mission: &Mission, world: &World, artifacts: &Artifacts) -> (World, Vec<ArtifactResult>) {
    let world = world.clone();
    let results = mission
        .artifacts
        .iter()
        .map(|spec| match artifacts.files.get(&spec.path) {
            Some(_) => ArtifactResult {
                id: spec.id.clone(),
                status: Status::Unknown,
                reason: Reason::ArtifactInvalid,
                problems: Vec::new(),
            },
            None => ArtifactResult {
                id: spec.id.clone(),
                status: Status::Fail,
                reason: Reason::ArtifactMissing,
                problems: Vec::new(),
            },
        })
        .collect();
    (world, results)
}
