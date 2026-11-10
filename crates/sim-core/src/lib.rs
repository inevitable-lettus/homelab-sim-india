//! Deterministic simulation core. See context-docs/Architecture.md.
//!
//! This crate must stay pure: no DOM, wasm, clock, randomness or network.
//! Phase 0 ships only the public shape and a stub `evaluate`.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Bumped whenever the data bundle format changes incompatibly.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub schema_version: u32,
}

impl Bundle {
    pub fn empty() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    #[serde(default)]
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
}

/// Three-valued on purpose: missing data is `Unknown`, never a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Official,
    Community,
    Teaching,
    LineCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub status: Status,
    pub severity: Severity,
    pub involved: Vec<String>,
    pub explanation_id: String,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub findings: Vec<Finding>,
}

/// Phase 0 stub. The real fixed-point evaluation arrives in Phase 4.
pub fn evaluate(world: &World, bundle: &Bundle) -> Report {
    let finding = if bundle.schema_version != SCHEMA_VERSION {
        stub_finding("bundle.schema_mismatch", Status::Unknown, Vec::new())
    } else if world.nodes.is_empty() {
        stub_finding("world.empty", Status::Unknown, Vec::new())
    } else {
        let mut involved: Vec<String> = world.nodes.iter().map(|node| node.id.clone()).collect();
        involved.sort();
        stub_finding("world.has_nodes", Status::Pass, involved)
    };

    Report {
        schema_version: SCHEMA_VERSION,
        findings: vec![finding],
    }
}

fn stub_finding(rule_id: &str, status: Status, involved: Vec<String>) -> Finding {
    Finding {
        rule_id: rule_id.to_owned(),
        status,
        severity: Severity::Info,
        involved,
        explanation_id: rule_id.to_owned(),
        provenance: Provenance::Teaching,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(ids: &[&str]) -> World {
        World {
            nodes: ids
                .iter()
                .map(|id| Node {
                    id: (*id).to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn empty_world_is_unknown_not_pass() {
        let report = evaluate(&World::default(), &Bundle::empty());
        assert_eq!(report.findings[0].status, Status::Unknown);
    }

    #[test]
    fn schema_mismatch_is_unknown_even_with_nodes() {
        let bundle = Bundle {
            schema_version: SCHEMA_VERSION + 1,
        };
        let report = evaluate(&world(&["a"]), &bundle);
        assert_eq!(report.findings[0].rule_id, "bundle.schema_mismatch");
        assert_eq!(report.findings[0].status, Status::Unknown);
    }

    #[test]
    fn identical_input_gives_identical_bytes() {
        let bundle = Bundle::empty();
        let first = serde_json::to_vec(&evaluate(&world(&["b", "a"]), &bundle)).unwrap();
        let second = serde_json::to_vec(&evaluate(&world(&["b", "a"]), &bundle)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn node_order_does_not_change_the_report() {
        let bundle = Bundle::empty();
        assert_eq!(
            evaluate(&world(&["a", "b"]), &bundle),
            evaluate(&world(&["b", "a"]), &bundle)
        );
    }
}
