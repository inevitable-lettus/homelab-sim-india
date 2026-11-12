//! Sandbox builds: a world the learner put together, with what they want it
//! to do. See context-docs/Sandbox.md.
//!
//! The rules here answer four questions, each three-valued: will it run, will
//! it fit, will it connect, will it last. They read capabilities from packs
//! and never a product name. All arithmetic is in whole units.

pub mod check;
mod connect;
mod expand;
mod fit;
pub mod power;
mod run;

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

pub(crate) use expand::expand;

use crate::bundle::Bundle;
use crate::facts::Net;
use crate::model::{Host, Id, Intent, Node, Price, World};
use crate::packs::{Arch, Cap, Category, DevicePack, Origin, OsFamily, OsPack, weaker};
use crate::reason::Reason;
use crate::trace::Trace;
use crate::{Finding, Provenance, Severity, Status};

// ----------------------------------------------------------------- report

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildReport {
    pub intents: Vec<IntentResult>,
    pub machines: Vec<MachineBudget>,
    pub power: power::PowerSummary,
    pub ups: Vec<power::UpsSummary>,
    pub parts: Vec<PartLine>,
    /// What there is to buy, leaving out what is owned. `None` when nothing is priced.
    pub price_total_paise: Option<u64>,
    /// Missing pack fields that turned a verdict into "can't tell".
    pub unknowns: Vec<UnknownItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntentResult {
    pub id: Id,
    pub status: Status,
    /// Indexes into `Report.findings`.
    pub findings: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<Id>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Budget {
    /// `None` when a term is unknown.
    pub used: Option<u64>,
    /// With recommended rather than minimum sizes, where that differs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<u64>,
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Term {
    pub label: String,
    pub value: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MachineBudget {
    pub node: Id,
    /// In MB.
    pub ram: Budget,
    pub bays: Budget,
    pub usb: Budget,
    /// Wired Ethernet ports.
    pub ports: Budget,
    pub avg_w: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PartLine {
    pub node: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount: Option<Id>,
    pub pack: Id,
    pub label: String,
    pub price_paise: Option<u64>,
    pub owned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnknownItem {
    pub pack: Id,
    pub field: String,
    pub used_by: Vec<Reason>,
}

// ---------------------------------------------------------------- context

/// What a rule concluded, before it is given a status and a severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    Pass,
    /// Something worth saying that is not a problem.
    Note,
    Fail,
    /// A problem that does not stop the build from working.
    Warn,
    Unknown,
    /// Not known, and not enough to make a goal "can't tell".
    UnknownNote,
}

/// Records what a rule read: how good the sources were, and what was missing.
#[derive(Debug, Default)]
pub(crate) struct Read {
    provenance: Option<Provenance>,
    missing: Vec<(Id, String)>,
}

impl Read {
    fn note(&mut self, provenance: Provenance) {
        self.provenance = Some(self.provenance.map_or(provenance, |p| weaker(p, provenance)));
    }

    /// Read one sourced value. A missing one is remembered by pack and field.
    pub fn cap<T: Clone>(&mut self, origin: Origin, pack: &str, field: &str, cap: &Cap<T>) -> Option<T> {
        match cap.get() {
            Some(value) => {
                self.note(origin.provenance(cap));
                Some(value.clone())
            }
            None => {
                self.missing(pack, field);
                None
            }
        }
    }

    /// A value read from a pack that is not a `Cap`: a list of architectures, say.
    pub fn plain(&mut self, origin: Origin) {
        self.note(match origin {
            Origin::Teaching => Provenance::Teaching,
            Origin::Real => Provenance::Official,
            Origin::Learner => Provenance::Learner,
        });
    }

    pub fn missing(&mut self, pack: &str, field: &str) {
        self.missing.push((pack.to_owned(), field.to_owned()));
    }
}

pub(crate) fn origin(teaching: bool) -> Origin {
    if teaching { Origin::Teaching } else { Origin::Real }
}

/// Which goals a finding belongs under.
#[derive(Debug, Default)]
pub(crate) struct Subject {
    /// A host: goals about it, or about a guest on it, list this finding.
    pub node: Option<Id>,
    /// With `node`: only goals about this service.
    pub service: Option<Id>,
    /// Goals named outright.
    pub intents: Vec<Id>,
}

impl Subject {
    pub fn node(id: &str) -> Self {
        Self { node: Some(id.to_owned()), ..Self::default() }
    }

    pub fn service(node: &str, service: &str) -> Self {
        Self { node: Some(node.to_owned()), service: Some(service.to_owned()), intents: Vec::new() }
    }

    pub fn intent(id: &str) -> Self {
        Self { intents: vec![id.to_owned()], ..Self::default() }
    }

    pub fn none() -> Self {
        Self::default()
    }
}

struct Draft {
    finding: Finding,
    subject: Subject,
}

pub(crate) struct Cx<'a> {
    pub net: &'a Net<'a>,
    pub world: &'a World,
    pub bundle: &'a Bundle,
    pub traces: &'a [Trace],
    drafts: Vec<Draft>,
    unknowns: BTreeMap<(Id, String), BTreeSet<Reason>>,
    machines: Vec<MachineBudget>,
    power: power::PowerSummary,
    ups: Vec<power::UpsSummary>,
}

impl<'a> Cx<'a> {
    pub fn new(net: &'a Net<'a>, bundle: &'a Bundle, traces: &'a [Trace]) -> Self {
        Self {
            net,
            world: net.world,
            bundle,
            traces,
            drafts: Vec::new(),
            unknowns: BTreeMap::new(),
            machines: Vec::new(),
            power: power::PowerSummary::default(),
            ups: Vec::new(),
        }
    }

    pub fn emit(&mut self, rule: Reason, verdict: Verdict, involved: Vec<Id>, read: Read, subject: Subject) {
        let explanation = match verdict {
            Verdict::Unknown | Verdict::UnknownNote => Reason::DeviceDataMissing,
            _ => rule,
        };
        self.emit_as(rule, verdict, explanation, involved, read, subject);
    }

    pub fn emit_as(
        &mut self,
        rule: Reason,
        verdict: Verdict,
        explanation: Reason,
        involved: Vec<Id>,
        read: Read,
        subject: Subject,
    ) {
        let (status, severity) = match verdict {
            Verdict::Pass | Verdict::Note => (Status::Pass, Severity::Info),
            Verdict::Fail => (Status::Fail, Severity::Error),
            Verdict::Warn => (Status::Fail, Severity::Warning),
            Verdict::Unknown => (Status::Unknown, Severity::Warning),
            Verdict::UnknownNote => (Status::Unknown, Severity::Info),
        };
        if status == Status::Unknown {
            for key in read.missing {
                self.unknowns.entry(key).or_default().insert(rule);
            }
        }
        self.drafts.push(Draft {
            finding: Finding {
                rule_id: rule,
                status,
                severity,
                involved,
                explanation_id: explanation,
                provenance: read.provenance.unwrap_or(Provenance::Teaching),
                debrief_only: false,
                intents: Vec::new(),
            },
            subject,
        });
    }

    /// Mark the finding just emitted as one for the risks list only.
    pub fn risk(&mut self) {
        if let Some(draft) = self.drafts.last_mut() {
            draft.finding.debrief_only = true;
        }
    }

    /// A device pack by id: the learner's own first, then the data.
    pub fn device(&self, id: &str) -> Option<&'a DevicePack> {
        self.world.custom_devices.iter().find(|pack| pack.id == id).or_else(|| self.bundle.devices.get(id))
    }

    pub fn pack_of(&self, device: &Option<Id>) -> Option<&'a DevicePack> {
        device.as_deref().and_then(|id| self.device(id))
    }

    pub fn os(&self, host: &Host) -> Option<&'a OsPack> {
        host.os.as_deref().and_then(|id| self.bundle.os.get(id))
    }

    pub fn parent(&self, host: &Host) -> Option<&'a Host> {
        host.runs_on.as_deref().and_then(|id| self.world.host(id))
    }

    /// The physical machine a host is, or runs on.
    pub fn root(&self, host: &'a Host) -> &'a Host {
        let mut cur = host;
        for _ in 0..8 {
            match self.parent(cur) {
                Some(parent) => cur = parent,
                None => break,
            }
        }
        cur
    }

    /// The host and every machine it runs on, nearest first.
    pub fn chain(&self, host: &'a Host) -> Vec<&'a Host> {
        let mut chain = vec![host];
        while let Some(parent) = chain.last().and_then(|h| self.parent(h)).filter(|_| chain.len() < 8) {
            chain.push(parent);
        }
        chain
    }

    /// Servers in the build, in node order. Clients and scaffolding are left out.
    pub fn servers(&self) -> Vec<&'a Host> {
        self.world
            .nodes
            .iter()
            .filter_map(|node| match node {
                Node::Host(host) if host.role == crate::model::Role::Server => Some(host),
                _ => None,
            })
            .filter(|host| host.device.is_some() || host.os.is_some() || host.runs_on.is_some())
            .collect()
    }

    pub fn guests_of(&self, machine: &Host) -> Vec<&'a Host> {
        self.world
            .nodes
            .iter()
            .filter_map(|node| match node {
                Node::Host(host) if host.runs_on.as_deref() == Some(machine.id.as_str()) => Some(host),
                _ => None,
            })
            .collect()
    }

    /// The OS family and CPU architecture software on this host has to match.
    pub fn platform(&self, host: &'a Host, read: &mut Read) -> (Option<OsFamily>, Option<Arch>) {
        use crate::model::GuestKind;
        let root = self.root(host);
        let arch = match self.pack_of(&root.device) {
            Some(device) => {
                let compute = device.compute.clone().unwrap_or_default();
                read.cap(device.origin(), &device.id, "compute.arch", &compute.arch)
            }
            None => None,
        };
        let family = match (host.runs_on.is_some(), host.guest) {
            // Containers are Linux containers, whatever runs them.
            (true, Some(GuestKind::Container | GuestKind::Lxc)) => Some(OsFamily::Linux),
            _ => self.os(host).map(|os| {
                read.plain(origin(os.teaching));
                os.family
            }),
        };
        (family, arch)
    }

    pub fn trace(&self, intent: &str) -> Option<&'a Trace> {
        let id = expand::attempt_id(intent);
        self.traces.iter().find(|trace| trace.attempt == id)
    }

    fn parts(&self) -> (Vec<PartLine>, Option<u64>) {
        let mut lines = Vec::new();
        let mut line = |node: &str, mount: Option<&str>, pack: &str, fallback: &str, price: Option<Price>| {
            lines.push(PartLine {
                node: node.to_owned(),
                mount: mount.map(str::to_owned),
                pack: pack.to_owned(),
                label: self.device(pack).map_or_else(|| fallback.to_owned(), |p| p.label.clone()),
                price_paise: price.map(|p| p.paise),
                owned: price.is_some_and(|p| p.owned),
            });
        };
        for node in &self.world.nodes {
            match node {
                Node::Host(host) => {
                    if let Some(device) = &host.device {
                        line(&host.id, None, device, &host.label, host.price);
                    }
                    for mount in &host.parts {
                        line(&host.id, Some(&mount.id), &mount.pack, &mount.pack, mount.price);
                    }
                }
                Node::Router(n) => {
                    if let Some(device) = &n.device {
                        line(&n.id, None, device, &n.label, n.price);
                    }
                }
                Node::Switch(n) => {
                    if let Some(device) = &n.device {
                        line(&n.id, None, device, &n.label, n.price);
                    }
                }
                Node::Power(n) => {
                    if let Some(device) = &n.device {
                        line(&n.id, None, device, &n.label, n.price);
                    }
                }
                Node::Isp(_) | Node::Internet(_) => {}
            }
        }
        let priced: Vec<u64> = lines.iter().filter(|l| !l.owned).filter_map(|l| l.price_paise).collect();
        let total = lines.iter().any(|l| l.price_paise.is_some()).then(|| priced.iter().sum());
        (lines, total)
    }

    /// Put each finding under the goals it belongs to, add them to the
    /// report's findings, and work out each goal's status.
    pub fn finish(mut self, findings: &mut Vec<Finding>) -> BuildReport {
        let world = self.world;
        // For each goal: the host it is about and every machine under it.
        let chains: Vec<(&str, Option<&str>, Vec<&str>)> = world
            .intents
            .iter()
            .filter_map(|intent| {
                let (host, service) = intent.subject()?;
                let chain = world.host(host).map(|h| self.chain(h).iter().map(|h| h.id.as_str()).collect());
                Some((intent.id(), service, chain.unwrap_or_default()))
            })
            .collect();
        for draft in std::mem::take(&mut self.drafts) {
            let mut finding = draft.finding;
            let mut tags: BTreeSet<Id> = draft.subject.intents.into_iter().collect();
            if let Some(node) = &draft.subject.node {
                for (intent, service, chain) in &chains {
                    let same_service = match (&draft.subject.service, service) {
                        // A finding about one service on the goal's own host.
                        (Some(mine), Some(theirs)) => mine == theirs && chain.first() == Some(&node.as_str()),
                        (Some(_), None) => false,
                        (None, _) => true,
                    };
                    if same_service && chain.contains(&node.as_str()) {
                        tags.insert((*intent).to_owned());
                    }
                }
            }
            finding.intents = tags.into_iter().collect();
            findings.push(finding);
        }
        crate::rules::sort(findings);

        let intents = world
            .intents
            .iter()
            .map(|intent| {
                let id = intent.id();
                let tagged: Vec<usize> = findings
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.intents.iter().any(|i| i == id))
                    .map(|(i, _)| i)
                    .collect();
                let has = |status: Status, severities: &[Severity]| {
                    tagged.iter().any(|i| findings[*i].status == status && severities.contains(&findings[*i].severity))
                };
                // A rule that said nothing has not passed.
                let status = if tagged.is_empty() || has(Status::Unknown, &[Severity::Error, Severity::Warning]) {
                    Status::Unknown
                } else {
                    Status::Pass
                };
                let status = if has(Status::Fail, &[Severity::Error]) { Status::Fail } else { status };
                IntentResult {
                    id: id.to_owned(),
                    status,
                    findings: tagged,
                    attempt: self.trace(id).map(|trace| trace.attempt.clone()),
                }
            })
            .collect();
        let (parts, price_total_paise) = self.parts();
        BuildReport {
            intents,
            machines: self.machines,
            power: self.power,
            ups: self.ups,
            parts,
            price_total_paise,
            unknowns: self
                .unknowns
                .into_iter()
                .map(|((pack, field), rules)| UnknownItem { pack, field, used_by: rules.into_iter().collect() })
                .collect(),
        }
    }
}

/// Every build rule, in a fixed order.
pub(crate) fn rules(cx: &mut Cx) {
    run::rules(cx);
    let mut machines = fit::rules(cx);
    connect::rules(cx);
    power::rules(cx, &mut machines);
    cx.machines = machines;

    // A goal nothing spoke about is "can't tell", and says so.
    let world = cx.world;
    for intent in &world.intents {
        if let Intent::Run { id, host, service } = intent
            && world.host(host).is_none_or(|h| !h.services.iter().any(|s| s.id == *service))
        {
            cx.emit_as(
                Reason::ServiceAvailable,
                Verdict::Unknown,
                Reason::UnknownNode,
                vec![host.clone()],
                Read::default(),
                Subject::intent(id),
            );
        }
    }
}

pub(crate) fn has_category(pack: &DevicePack, categories: &[Category]) -> bool {
    categories.contains(&pack.category)
}
