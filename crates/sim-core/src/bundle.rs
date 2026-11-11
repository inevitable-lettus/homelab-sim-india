//! The data bundle: everything authored in `data/`, `content/` and
//! `missions/`, validated by `tools/data-build` and shipped as one JSON file.
//!
//! These Rust types are the single definition of the schema.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Id, Target, World};

/// Bumped whenever the bundle format changes incompatibly.
pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema_version: u32,
    /// Archetypes and real devices share one schema and one map.
    #[serde(default)]
    pub devices: BTreeMap<Id, DevicePack>,
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
            isps: BTreeMap::new(),
            explanations: BTreeMap::new(),
            missions: BTreeMap::new(),
            tracks: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------- devices

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownMark {
    Unknown,
}

/// One capability value. It is a plain value in an archetype, a value with
/// its source in a real device, or the word `unknown`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cap<T> {
    Sourced { value: T, source: String, retrieved_at: String },
    Unknown(UnknownMark),
    Value(T),
}

impl<T> Default for Cap<T> {
    fn default() -> Self {
        Cap::Unknown(UnknownMark::Unknown)
    }
}

impl<T> Cap<T> {
    pub fn get(&self) -> Option<&T> {
        match self {
            Cap::Sourced { value, .. } | Cap::Value(value) => Some(value),
            Cap::Unknown(_) => None,
        }
    }

    /// A bare value with no source. Fine in an archetype, not in a real device.
    pub fn is_unsourced_value(&self) -> bool {
        matches!(self, Cap::Value(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackKind {
    /// A generic teaching machine. Not a product.
    Archetype,
    /// A real product. Every value needs a source.
    Device,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevicePack {
    pub id: Id,
    pub kind: PackKind,
    pub label: String,
    #[serde(default)]
    pub capabilities: Capabilities,
    #[serde(default)]
    pub power: Power,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default)]
    pub arch: Cap<String>,
    #[serde(default)]
    pub ram_gb: Cap<u32>,
    #[serde(default)]
    pub nic: Vec<Nic>,
    #[serde(default)]
    pub virtualization: Cap<bool>,
    #[serde(default)]
    pub igpu_transcode: Cap<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nic {
    #[serde(default)]
    pub speed_mbps: Cap<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Power {
    #[serde(default)]
    pub idle_w: Cap<u32>,
    #[serde(default)]
    pub load_w: Cap<u32>,
}

// ------------------------------------------------------------------- isps

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Starting text for the file editor.
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
