//! Deterministic simulation core. See context-docs/Architecture.md.
//!
//! This crate must stay pure: no DOM, wasm, clock, randomness or network, no
//! floats in decisions and no hash-ordered iteration. The same input always
//! gives the same bytes out.

#![forbid(unsafe_code)]

pub mod artifacts;
pub mod build;
pub mod bundle;
pub mod facts;
pub mod grade;
pub mod ip;
pub mod model;
pub mod packs;
pub mod physics;
pub mod reason;
mod rules;
pub mod trace;
mod walk;
pub mod wireguard;
pub mod x25519;

use serde::{Deserialize, Serialize};

pub use bundle::{Bundle, Depth, Mission, SCHEMA_VERSION};
pub use grade::{Artifacts, Grade, grade};
pub use model::World;
pub use reason::Reason;
pub use trace::{Trace, trace};

/// Three-valued on purpose: missing data is `Unknown`, never a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Fail,
    Unknown,
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Where the data behind a verdict came from. The UI words verdicts by this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Official,
    Community,
    /// A teaching model. Says nothing about any real product or provider.
    Teaching,
    /// The learner's own line check.
    LineCheck,
    /// Values the learner typed in for a device we have no pack for.
    Learner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tri {
    Yes,
    No,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule_id: Reason,
    pub status: Status,
    pub severity: Severity,
    pub involved: Vec<String>,
    pub explanation_id: Reason,
    pub provenance: Provenance,
    /// Shown in the debrief only, so beginners are not nagged mid-mission.
    pub debrief_only: bool,
    /// The sandbox goals this finding counts towards.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub facts: facts::Facts,
    pub findings: Vec<Finding>,
    /// One trace per attempt in the world, in the world's order. In a sandbox
    /// build, the attempts its goals expand to follow, as `intent:<goal id>`.
    pub attempts: Vec<Trace>,
    /// Sandbox builds only. Missions leave it out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<build::BuildReport>,
}

/// What a dial target points at right now, for showing next to the choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolved {
    pub address: facts::Addr,
    /// Why there is no address, when there is none.
    pub reason: Option<Reason>,
}

pub fn resolve_target(world: &World, target: &model::Target, bundle: &Bundle) -> Resolved {
    let net = facts::Net::build(world, bundle, &facts::Assumptions::new());
    match walk::resolve(&net, target, 0) {
        Ok(ip) => Resolved { address: facts::Addr::Known(ip), reason: None },
        Err(walk::NoAddr::Fail(reason)) => Resolved { address: facts::Addr::Absent, reason: Some(reason) },
        Err(walk::NoAddr::Unknown(reason)) => Resolved { address: facts::Addr::Unknown, reason: Some(reason) },
    }
}

/// Derive addresses to a fixed point, run every rule, and trace every attempt.
pub fn evaluate(world: &World, bundle: &Bundle) -> Report {
    let net = facts::Net::build(world, bundle, &facts::Assumptions::new());
    if bundle.schema_version != SCHEMA_VERSION {
        // Data we cannot read is not a pass and not a fail.
        return Report {
            schema_version: SCHEMA_VERSION,
            facts: facts::facts(&net),
            findings: vec![Finding {
                rule_id: Reason::BundleSchemaMismatch,
                status: Status::Unknown,
                severity: Severity::Error,
                involved: Vec::new(),
                explanation_id: Reason::BundleSchemaMismatch,
                provenance: Provenance::Teaching,
                debrief_only: false,
                intents: Vec::new(),
            }],
            attempts: Vec::new(),
            build: None,
        };
    }
    if !world.is_build() {
        return Report {
            schema_version: SCHEMA_VERSION,
            facts: facts::facts(&net),
            findings: rules::findings(&net, bundle),
            attempts: world.attempts.iter().map(|attempt| trace(world, attempt, bundle)).collect(),
            build: None,
        };
    }
    // A build's goals become attempts first, so the trace decides them.
    let expanded = build::expand(world, bundle);
    let net = facts::Net::build(&expanded, bundle, &facts::Assumptions::new());
    let attempts: Vec<Trace> = expanded.attempts.iter().map(|attempt| trace(&expanded, attempt, bundle)).collect();
    let mut findings = rules::findings(&net, bundle);
    let mut cx = build::Cx::new(&net, bundle, &attempts);
    build::rules(&mut cx);
    let build = cx.finish(&mut findings);
    Report { schema_version: SCHEMA_VERSION, facts: facts::facts(&net), findings, attempts, build: Some(build) }
}
