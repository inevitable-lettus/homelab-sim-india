//! The data bundle: everything authored in `data/`, `content/` and
//! `missions/`, validated by `tools/data-build` and shipped as one JSON file.
//!
//! These Rust types are the single definition of the schema.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Id, Target, World};
pub use crate::packs::{
    Cap, DevicePack, MediaFormat, OsPack, Outage, PackKind, RuntimePack, ServicePack, Tariff, Template, UnknownMark,
};

/// Bumped whenever the bundle format changes incompatibly.
pub const SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema_version: u32,
    /// Archetypes and real devices share one schema and one map.
    #[serde(default)]
    pub devices: BTreeMap<Id, DevicePack>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub os: BTreeMap<Id, OsPack>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub runtimes: BTreeMap<Id, RuntimePack>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub services: BTreeMap<Id, ServicePack>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tariffs: BTreeMap<Id, Tariff>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outages: BTreeMap<Id, Outage>,
    /// Typical bitrates per media format, from `data/reference/media.yaml`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub media: BTreeMap<Id, MediaFormat>,
    /// Starting builds for the sandbox.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub templates: BTreeMap<Id, Template>,
    #[serde(default)]
    pub isps: BTreeMap<Id, IspProfile>,
    #[serde(default)]
    pub explanations: BTreeMap<Id, Explanation>,
    #[serde(default)]
    pub missions: BTreeMap<Id, Mission>,
    /// Tracks in the order they are taught.
    #[serde(default)]
    pub tracks: Vec<Track>,
}

/// An ordered set of missions, with the ones still to be written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub id: Id,
    pub title: String,
    pub summary: String,
    pub missions: Vec<Id>,
    /// Written up in the curriculum but not playable yet.
    #[serde(default)]
    pub planned: Vec<PlannedMission>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedMission {
    pub title: String,
    pub concept: String,
}

impl Bundle {
    pub fn empty() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            devices: BTreeMap::new(),
            os: BTreeMap::new(),
            runtimes: BTreeMap::new(),
            services: BTreeMap::new(),
            tariffs: BTreeMap::new(),
            outages: BTreeMap::new(),
            media: BTreeMap::new(),
            templates: BTreeMap::new(),
            isps: BTreeMap::new(),
            explanations: BTreeMap::new(),
            missions: BTreeMap::new(),
            tracks: Vec::new(),
        }
    }

    /// Add the sandbox catalog to a loaded bundle. Packs already present stay.
    pub fn merge(&mut self, catalog: Bundle) {
        fn add<T>(into: &mut BTreeMap<Id, T>, from: BTreeMap<Id, T>) {
            for (id, pack) in from {
                into.entry(id).or_insert(pack);
            }
        }
        add(&mut self.devices, catalog.devices);
        add(&mut self.os, catalog.os);
        add(&mut self.runtimes, catalog.runtimes);
        add(&mut self.services, catalog.services);
        add(&mut self.tariffs, catalog.tariffs);
        add(&mut self.outages, catalog.outages);
        add(&mut self.media, catalog.media);
        add(&mut self.templates, catalog.templates);
        add(&mut self.isps, catalog.isps);
        add(&mut self.explanations, catalog.explanations);
    }
}

// ------------------------------------------------------------------- isps

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    /// Invented for teaching. Makes no claim about any real provider.
    Teaching,
    Real,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimValue {
    Yes,
    No,
    Likely,
    Varies,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub value: ClaimValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    /// ISO date the claim was last checked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IspProfile {
    pub id: Id,
    pub label: String,
    pub kind: ProfileKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default)]
    pub claims: BTreeMap<String, Claim>,
}

// ----------------------------------------------------------- explanations

/// Teaching text for one reason, rule or goal. `line` is the one-sentence
/// version for the step log. The longer sections follow Design.md.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Explanation {
    pub title: String,
    pub line: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub what_happened: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why_it_matters: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub how_to_fix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub real_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub go_deeper: Option<String>,
}

// --------------------------------------------------------------- missions

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Depth {
    Guided,
    Challenge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mission {
    pub id: Id,
    pub order: u32,
    pub title: String,
    pub concept: String,
    pub scenario: String,
    pub guided: DepthText,
    pub challenge: DepthText,
    /// Where each node sits on the board, for wide and narrow screens.
    pub layout: Layout,
    /// What the learner is allowed to change.
    #[serde(default)]
    pub controls: Vec<Control>,
    #[serde(default)]
    pub questions: Vec<Question>,
    pub goals: Vec<Goal>,
    /// Files read from the terminal or the editor and graded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ArtifactSpec>,
    pub debrief: Debrief,
    pub world: World,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepthText {
    pub brief: String,
    #[serde(default)]
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub wide: BTreeMap<Id, [i32; 2]>,
    pub narrow: BTreeMap<Id, [i32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Control {
    /// Choose where an attempt is aimed.
    Dial { attempt: Id, options: Vec<DialOption> },
    /// Add, edit and remove port forwards on a router.
    PortForwards { router: Id },
    /// Join or leave an overlay on these hosts.
    OverlayJoin { network: Id, nodes: Vec<Id> },
    /// Answer one of the mission's questions.
    Question { question: Id },
    /// Edit a tunnel's settings by hand.
    Tunnel { node: Id, tunnel: Id },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialOption {
    pub label: String,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: Id,
    pub prompt: String,
    pub options: Vec<AnswerOption>,
    pub check: AnswerCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnswerOption {
    pub value: String,
    pub label: String,
}

/// How the core works out the right answer. The mission never stores it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnswerCheck {
    /// The line check question: is the router's WAN address the same as the
    /// public one? `same` on a public line, `different` behind carrier NAT.
    LineType { isp: Id },
    /// Which node stopped this attempt? The answer is a node id, or `nowhere`.
    BlockedAt { attempt: Id },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Goal {
    pub id: Id,
    pub text: String,
    /// Set when the goal applies at one depth only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<Depth>,
    pub check: GoalCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GoalCheck {
    /// The attempt connects, optionally to a named node and within a round-trip time.
    Delivered {
        attempt: Id,
        #[serde(default)]
        to: Option<Id>,
        #[serde(default)]
        max_rtt_ms: Option<u32>,
    },
    /// The attempt's first packet gets at least as far as this node.
    Reaches { attempt: Id, node: Id },
    /// The learner answered this question correctly.
    Answer { question: Id },
    /// No finding with this rule id has failed.
    RulePasses { rule: Id },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    WireguardConf,
    /// A single WireGuard public key, as printed by `wg pubkey`.
    WireguardPublicKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSpec {
    pub id: Id,
    /// Path inside the guest, for example `/root/wg0.conf`.
    pub path: String,
    pub kind: ArtifactKind,
    pub label: String,
    /// The host and tunnel this file configures.
    pub node: Id,
    pub tunnel: Id,
    /// For a public key: the host whose tunnel should list it as a peer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub register_on: Option<Id>,
    /// Starting text for the file editor at guided depth. Challenge starts empty.
    #[serde(default)]
    pub starter: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Debrief {
    pub did: String,
    /// The explanation that carries this mission's concept.
    pub concept: Id,
    pub real_line: String,
}
