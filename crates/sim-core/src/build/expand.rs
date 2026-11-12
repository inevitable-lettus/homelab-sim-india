//! Turns "reach it from there" and "watch it on that" into attempts, so the
//! trace decides them like any other connection. The UI never makes these.

use crate::bundle::Bundle;
use crate::facts::{Assumptions, Net};
use crate::model::{Attempt, Direction, Intent, Node, Target, Transfer, TunnelKind, World};

/// Seconds of media the transfer estimate is sized for.
const STREAM_SAMPLE_S: u64 = 3_600;

pub(crate) fn attempt_id(intent: &str) -> String {
    format!("intent:{intent}")
}

pub(crate) fn expand(world: &World, bundle: &Bundle) -> World {
    let mut out = world.clone();
    let net = Net::build(world, bundle, &Assumptions::new());
    for intent in &world.intents {
        let attempt = match intent {
            Intent::Reach { id, from, host, service, via } => {
                attempt(world, id, from, host, service, via.clone(), None)
            }
            Intent::Stream { id, client, host, service, format, streams, via } => {
                let via = via.clone().unwrap_or_else(|| default_via(world, &net, client, host));
                let transfer = bundle.media.get(format).and_then(|f| f.kbps.copied()).map(|kbps| Transfer {
                    bytes: u64::from(kbps) * 1_000 / 8 * STREAM_SAMPLE_S * u64::from(*streams),
                    direction: Direction::Reply,
                });
                attempt(world, id, client, host, service, via, transfer)
            }
            _ => None,
        };
        out.attempts.extend(attempt);
    }
    out
}

fn attempt(
    world: &World,
    intent: &str,
    from: &str,
    host: &str,
    service: &str,
    via: Target,
    transfer: Option<Transfer>,
) -> Option<Attempt> {
    let service = world.host(host)?.services.iter().find(|s| s.id == service)?;
    world.node(from)?;
    let mut port = service.port;
    if matches!(via, Target::PublicOf { .. } | Target::WanOf { .. } | Target::Name { .. }) {
        // Through a port forward, the port to dial is the outside one.
        let forward = world
            .nodes
            .iter()
            .filter_map(|node| match node {
                Node::Router(router) => Some(&router.port_forwards),
                _ => None,
            })
            .flatten()
            .find(|f| f.enabled && f.proto == service.proto && f.internal_port == service.port);
        port = forward.map_or(port, |f| f.external_port);
    }
    Some(Attempt {
        id: attempt_id(intent),
        label: service.label.clone(),
        from: from.to_owned(),
        to: via,
        proto: service.proto,
        port,
        idle_s: 0,
        packet_bytes: 1_000,
        transfer,
    })
}

/// How a client gets to a server when the learner did not say: the home
/// address on the same network, then a mesh both have joined, then the
/// address the internet sees.
fn default_via(world: &World, net: &Net, client: &str, host: &str) -> Target {
    let node = host.to_owned();
    if let (Some(c), Some(h)) = (net.index(client), net.index(host))
        && net.gw[c].is_some()
        && net.gw[c] == net.gw[h]
    {
        return Target::LanOf { node };
    }
    let meshes = |id: &str| -> Vec<&str> {
        world
            .host(id)
            .into_iter()
            .flat_map(|h| &h.tunnels)
            .filter(|t| t.enabled && t.kind == TunnelKind::Overlay)
            .filter_map(|t| t.network.as_deref())
            .collect()
    };
    let theirs = meshes(host);
    match meshes(client).into_iter().find(|network| theirs.contains(network)) {
        Some(network) => Target::OverlayOf { node, network: network.to_owned() },
        None => Target::PublicOf { node },
    }
}
