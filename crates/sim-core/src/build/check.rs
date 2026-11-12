//! Is this a build the core can read? Import and share links ask this before
//! they load anything, so a broken file is refused with a reason rather than
//! half-loaded.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::bundle::Bundle;
use crate::model::{Intent, Node, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemKind {
    /// The text is not a world at all. `at` carries where the parser stopped.
    Parse,
    DuplicateId,
    UnknownNode,
    UnknownPack,
    UnknownService,
    /// A part plugged into itself, directly or through other parts.
    Loop,
    TooBig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub kind: ProblemKind,
    /// What it is about: an id, or `owner/id`.
    pub at: String,
}

/// More than this and the board is not something a person laid out.
const MAX_NODES: usize = 200;

pub fn problems(world: &World, bundle: &Bundle) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut say = |kind: ProblemKind, at: String| out.push(Problem { kind, at });
    if world.nodes.len() > MAX_NODES || world.links.len() > 4 * MAX_NODES {
        say(ProblemKind::TooBig, String::new());
    }

    let mut ids = BTreeSet::new();
    for node in &world.nodes {
        if !ids.insert(node.id()) {
            say(ProblemKind::DuplicateId, node.id().to_owned());
        }
    }
    let mut seen = BTreeSet::new();
    for id in world.links.iter().map(|l| l.id.as_str()).chain(world.intents.iter().map(Intent::id)) {
        if !seen.insert(id) {
            say(ProblemKind::DuplicateId, id.to_owned());
        }
    }
    let node = |id: &str, say: &mut dyn FnMut(ProblemKind, String)| {
        if !ids.contains(id) {
            say(ProblemKind::UnknownNode, id.to_owned());
        }
    };

    let custom: BTreeSet<&str> = world.custom_devices.iter().map(|d| d.id.as_str()).collect();
    if custom.len() != world.custom_devices.len() {
        say(ProblemKind::DuplicateId, "custom_devices".to_owned());
    }
    let device = |id: &str| custom.contains(id) || bundle.devices.contains_key(id);
    let pack = |known: bool, owner: &str, id: &str, say: &mut dyn FnMut(ProblemKind, String)| {
        if !known {
            say(ProblemKind::UnknownPack, format!("{owner}/{id}"));
        }
    };

    for link in &world.links {
        node(&link.a, &mut say);
        node(&link.b, &mut say);
    }
    for feed in &world.power_feeds {
        node(&feed.from, &mut say);
        node(&feed.to, &mut say);
    }
    for entry in &world.nodes {
        let id = entry.id();
        let named = match entry {
            Node::Host(host) => &host.device,
            Node::Router(router) => &router.device,
            Node::Switch(switch) => &switch.device,
            Node::Power(unit) => &unit.device,
            Node::Isp(_) | Node::Internet(_) => &None,
        };
        if let Some(named) = named {
            pack(device(named), id, named, &mut say);
        }
        let Node::Host(host) = entry else { continue };
        if let Some(parent) = &host.runs_on {
            node(parent, &mut say);
        }
        if let Some(os) = &host.os {
            pack(bundle.os.contains_key(os), id, os, &mut say);
        }
        if let Some(runtime) = &host.runtime {
            pack(bundle.runtimes.contains_key(runtime), id, runtime, &mut say);
        }
        for app in host.services.iter().filter_map(|s| s.app.as_ref()) {
            pack(bundle.services.contains_key(app), id, app, &mut say);
        }
        let mut parts = BTreeSet::new();
        for part in &host.parts {
            if !parts.insert(part.id.as_str()) {
                say(ProblemKind::DuplicateId, format!("{id}/{}", part.id));
            }
            pack(device(&part.pack), id, &part.pack, &mut say);
        }
        for part in &host.parts {
            // Walk up what it is plugged into. More steps than parts is a loop.
            let mut at = part.into.as_deref();
            let mut steps = 0;
            while let Some(into) = at {
                let Some(next) = host.parts.iter().find(|p| p.id == into) else {
                    say(ProblemKind::UnknownNode, format!("{id}/{into}"));
                    break;
                };
                steps += 1;
                if steps > host.parts.len() {
                    say(ProblemKind::Loop, format!("{id}/{}", part.id));
                    break;
                }
                at = next.into.as_deref();
            }
        }
    }
    // A guest that runs on itself, directly or through other guests.
    for entry in &world.nodes {
        let mut at = world.host(entry.id()).and_then(|h| h.runs_on.as_deref());
        let mut steps = 0;
        while let Some(parent) = at {
            steps += 1;
            if steps > world.nodes.len() {
                say(ProblemKind::Loop, entry.id().to_owned());
                break;
            }
            at = world.host(parent).and_then(|h| h.runs_on.as_deref());
        }
    }

    for intent in &world.intents {
        let id = intent.id();
        if let Some((host, service)) = intent.subject() {
            node(host, &mut say);
            if let (Some(service), Some(host)) = (service, world.host(host))
                && !host.services.iter().any(|s| s.id == service)
            {
                say(ProblemKind::UnknownService, format!("{id}/{service}"));
            }
        }
        match intent {
            Intent::Reach { from, .. } => node(from, &mut say),
            Intent::Stream { client, format, .. } => {
                node(client, &mut say);
                pack(bundle.media.contains_key(format), id, format, &mut say);
            }
            Intent::HomeDns { router, .. } => node(router, &mut say),
            Intent::SurviveOutage { outage, keep, .. } => {
                pack(bundle.outages.contains_key(outage), id, outage, &mut say);
                for kept in keep {
                    if !world.intents.iter().any(|i| i.id() == kept) {
                        say(ProblemKind::UnknownNode, format!("{id}/{kept}"));
                    }
                }
            }
            Intent::Run { .. } | Intent::Store { .. } | Intent::YearlyCost { .. } => {}
        }
    }
    if let Some(home) = &world.home {
        if let Some(tariff) = &home.tariff {
            pack(bundle.tariffs.contains_key(tariff), "home", tariff, &mut say);
        }
        if let Some(outage) = &home.outage {
            pack(bundle.outages.contains_key(outage), "home", outage, &mut say);
        }
    }
    out
}
