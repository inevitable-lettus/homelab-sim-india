//! Files the learner wrote, turned into facts in the world before grading.
//!
//! A config file becomes the settings of the tunnel it describes. A public key
//! file becomes the key the other end has on record. Nothing here decides
//! whether the tunnel works: the trace does that, from the world these files
//! produce.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::Status;
use crate::bundle::{ArtifactKind, ArtifactSpec, Mission};
use crate::grade::Artifacts;
use crate::model::{Id, Node, Peer, Tunnel, World};
use crate::reason::Reason;
use crate::{wireguard, x25519};

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

fn result(spec: &ArtifactSpec, status: Status, reason: Reason, problems: Vec<Problem>) -> ArtifactResult {
    ArtifactResult { id: spec.id.clone(), status, reason, problems }
}

fn tunnel_mut<'w>(world: &'w mut World, node: &str, tunnel: &str) -> Option<&'w mut Tunnel> {
    world.nodes.iter_mut().find_map(|n| match n {
        Node::Host(host) if host.id == node => host.tunnels.iter_mut().find(|t| t.id == tunnel),
        _ => None,
    })
}

/// The entry for `peer_node` on any tunnel of `on`.
fn peer_mut<'w>(world: &'w mut World, on: &str, peer_node: &str) -> Option<&'w mut Peer> {
    world.nodes.iter_mut().find_map(|n| match n {
        Node::Host(host) if host.id == on => {
            host.tunnels.iter_mut().flat_map(|t| t.peers.iter_mut()).find(|p| p.node.as_deref() == Some(peer_node))
        }
        _ => None,
    })
}

pub(crate) fn apply(mission: &Mission, world: &World, artifacts: &Artifacts) -> (World, Vec<ArtifactResult>) {
    let mut world = world.clone();
    let mut results: BTreeMap<usize, ArtifactResult> = BTreeMap::new();
    // The public key each config file works out to, by host and tunnel.
    let mut derived: BTreeMap<(&str, &str), Option<String>> = BTreeMap::new();

    // Configs first: a public key file is checked against its config.
    for (i, spec) in mission.artifacts.iter().enumerate() {
        if spec.kind != ArtifactKind::WireguardConf {
            continue;
        }
        let slot = derived.entry((spec.node.as_str(), spec.tunnel.as_str())).or_insert(None);
        let Some(tunnel) = tunnel_mut(&mut world, &spec.node, &spec.tunnel) else {
            results.insert(i, result(spec, Status::Unknown, Reason::UnknownNode, Vec::new()));
            continue;
        };
        // Until a good file says otherwise, the tunnel is down.
        tunnel.enabled = false;
        let Some(text) = artifacts.files.get(&spec.path) else {
            results.insert(i, result(spec, Status::Fail, Reason::ArtifactMissing, Vec::new()));
            continue;
        };
        match wireguard::parse(text) {
            Err(problems) => {
                results.insert(i, result(spec, Status::Fail, Reason::ArtifactInvalid, problems));
            }
            Ok(conf) => {
                tunnel.enabled = true;
                tunnel.address = conf.addresses.first().copied();
                tunnel.listen_port = conf.listen_port;
                tunnel.mtu = conf.mtu;
                tunnel.public_key = Some(conf.public_key.clone());
                tunnel.peers = conf
                    .peers
                    .into_iter()
                    .map(|peer| Peer {
                        public_key: Some(peer.public_key),
                        node: None,
                        endpoint: peer.endpoint,
                        allowed_ips: peer.allowed_ips,
                        keepalive_s: peer.keepalive_s,
                    })
                    .collect();
                *slot = Some(conf.public_key);
                results.insert(i, result(spec, Status::Pass, Reason::ArtifactOk, Vec::new()));
            }
        }
    }

    for (i, spec) in mission.artifacts.iter().enumerate() {
        if spec.kind != ArtifactKind::WireguardPublicKey {
            continue;
        }
        let text = artifacts.files.get(&spec.path).map(|text| text.trim());
        let key = text.filter(|text| x25519::decode_key(text).is_some());
        // What the other end has on record. With no usable file that is
        // nothing, which must never match: an empty string is not a key.
        if let Some(on) = &spec.register_on {
            match peer_mut(&mut world, on, &spec.node) {
                Some(peer) => peer.public_key = Some(key.unwrap_or_default().to_owned()),
                None => {
                    results.insert(i, result(spec, Status::Unknown, Reason::UnknownNode, Vec::new()));
                    continue;
                }
            }
        }
        let outcome = match (text, key) {
            (None, _) => result(spec, Status::Fail, Reason::ArtifactMissing, Vec::new()),
            (Some(_), None) => {
                let message = "This is not a WireGuard public key. A key is 44 characters and ends in =".to_owned();
                result(spec, Status::Fail, Reason::ArtifactInvalid, vec![Problem { line: 1, message }])
            }
            (Some(_), Some(key)) => match derived.get(&(spec.node.as_str(), spec.tunnel.as_str())) {
                Some(Some(public)) if public == key => result(spec, Status::Pass, Reason::ArtifactOk, Vec::new()),
                Some(Some(_)) => result(spec, Status::Fail, Reason::ArtifactKeyMismatch, Vec::new()),
                // No readable config to compare against: we cannot say.
                Some(None) => result(spec, Status::Unknown, Reason::ArtifactKeyUnchecked, Vec::new()),
                // The mission grades the key on its own.
                None => result(spec, Status::Pass, Reason::ArtifactOk, Vec::new()),
            },
        };
        results.insert(i, outcome);
    }

    (world, results.into_values().collect())
}
