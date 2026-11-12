//! Rules turn derived facts into findings. Every rule is three-valued: when
//! the data is not there, the answer is `Unknown`, never a pass.
//!
//! Rules read capabilities and settings. They never read a product name.

use std::collections::BTreeMap;

use crate::bundle::{Bundle, PackKind};
use crate::facts::{Mode, Net};
use crate::model::{Bind, Node, TunnelKind};
use crate::physics::{DEFAULT_LINK_MTU, WIREGUARD_DEFAULT_MTU, tunnel_overhead_bytes};
use crate::reason::Reason;
use crate::walk::OVERLAY_KEEPALIVE_S;
use crate::{Finding, Provenance, Severity, Status, Tri};

fn finding(rule: Reason, status: Status, severity: Severity, explanation: Reason, involved: Vec<String>) -> Finding {
    Finding {
        rule_id: rule,
        status,
        severity,
        involved,
        explanation_id: explanation,
        provenance: Provenance::Teaching,
        debrief_only: false,
    }
}

pub(crate) fn findings(net: &Net, bundle: &Bundle) -> Vec<Finding> {
    let world = net.world;
    let mut out = Vec::new();
    if world.nodes.is_empty() {
        return vec![finding(Reason::WorldEmpty, Status::Unknown, Severity::Info, Reason::WorldEmpty, Vec::new())];
    }
    if !net.settled {
        let rule = Reason::EvaluationDidNotSettle;
        out.push(finding(rule, Status::Fail, Severity::Error, rule, Vec::new()));
    }

    for &i in net.idx.values() {
        let id = net.id(i).to_owned();
        let chain = net.upstream_chain(i);
        let isps: Vec<usize> = chain.iter().map(|(_, n)| *n).filter(|n| net.mode[*n].is_some()).collect();
        let line_provenance = isps.first().and_then(|n| net.mode[*n]).map_or(Provenance::Teaching, |(_, p)| p);
        let mut involved = vec![id.clone()];
        involved.extend(isps.iter().map(|n| net.id(*n).to_owned()));
        let behind = net.behind_carrier_nat(i);

        match &world.nodes[i] {
            Node::Router(router) => {
                if net.dist[i].is_some() {
                    let rule = Reason::LineHasPublicIpv4;
                    let mut f = match behind {
                        Tri::No => finding(rule, Status::Pass, Severity::Info, rule, involved.clone()),
                        Tri::Yes => finding(
                            rule,
                            Status::Fail,
                            Severity::Warning,
                            Reason::WanAddressIsCarrierNat,
                            involved.clone(),
                        ),
                        Tri::Unknown => {
                            finding(rule, Status::Unknown, Severity::Warning, Reason::LineTypeUnknown, involved.clone())
                        }
                    };
                    f.provenance = line_provenance;
                    out.push(f);
                }
                for forward in router.port_forwards.iter().filter(|f| f.enabled) {
                    let rule = Reason::PortForwardUsable;
                    let target = net.customers[i]
                        .iter()
                        .copied()
                        .find(|c| net.up4[*c].known() == Some(std::net::IpAddr::V4(forward.to_ip)));
                    let listening = target.is_some_and(|t| match &world.nodes[t] {
                        Node::Host(host) => host.services.iter().any(|s| {
                            s.proto == forward.proto && s.port == forward.internal_port && s.bind == Bind::All
                        }),
                        _ => false,
                    });
                    let mut with_target = involved.clone();
                    with_target.extend(target.map(|t| net.id(t).to_owned()));
                    let mut f = match (behind, target, listening) {
                        (Tri::Yes, ..) => {
                            finding(rule, Status::Fail, Severity::Error, Reason::ForwardBehindCgnat, with_target)
                        }
                        (Tri::Unknown, ..) => {
                            finding(rule, Status::Unknown, Severity::Warning, Reason::LineTypeUnknown, with_target)
                        }
                        (_, None, _) => {
                            finding(rule, Status::Fail, Severity::Warning, Reason::ForwardTargetMissing, with_target)
                        }
                        (_, _, false) => {
                            finding(rule, Status::Fail, Severity::Warning, Reason::ForwardNoService, with_target)
                        }
                        (Tri::No, Some(_), true) => {
                            // Working, which also means anyone can reach it.
                            let mut note = finding(
                                Reason::ServiceExposed,
                                Status::Pass,
                                Severity::Info,
                                Reason::ServiceExposed,
                                with_target.clone(),
                            );
                            note.debrief_only = true;
                            out.push(note);
                            finding(rule, Status::Pass, Severity::Info, rule, with_target)
                        }
                    };
                    f.provenance = line_provenance;
                    out.push(f);
                }
            }
            Node::Host(host) => {
                let mut ports: BTreeMap<_, u32> = BTreeMap::new();
                for service in &host.services {
                    *ports.entry((service.proto, service.port)).or_default() += 1;
                }
                if ports.values().any(|count| *count > 1) {
                    let rule = Reason::ServicePortConflict;
                    out.push(finding(rule, Status::Fail, Severity::Error, rule, vec![id.clone()]));
                }

                if let Some(parent) = &host.runs_on {
                    let rule = Reason::HostVirtualization;
                    let pack = world
                        .node(parent)
                        .and_then(|node| match node {
                            Node::Host(machine) => machine.device.as_ref(),
                            _ => None,
                        })
                        .and_then(|device| bundle.devices.get(device));
                    let capable = pack.and_then(|pack| pack.compute.as_ref().and_then(|c| c.virtualization.copied()));
                    let both = vec![id.clone(), parent.clone()];
                    let mut f = match capable {
                        Some(true) => finding(rule, Status::Pass, Severity::Info, rule, both),
                        Some(false) => finding(rule, Status::Fail, Severity::Error, rule, both),
                        None => finding(rule, Status::Unknown, Severity::Warning, Reason::DeviceDataMissing, both),
                    };
                    if pack.is_some_and(|pack| pack.kind == PackKind::Device) {
                        f.provenance = Provenance::Official;
                    }
                    out.push(f);
                }

                for tunnel in host.tunnels.iter().filter(|t| t.enabled) {
                    let dials: Vec<_> = tunnel.peers.iter().filter(|p| p.endpoint.is_some()).collect();
                    // A mesh client always dials its relay and keeps that alive itself.
                    let keepalives: Vec<Option<u32>> = match tunnel.kind {
                        TunnelKind::Wireguard => dials.iter().map(|p| p.keepalive_s).collect(),
                        TunnelKind::Overlay => vec![Some(OVERLAY_KEEPALIVE_S)],
                    };
                    if keepalives.is_empty() {
                        continue;
                    }
                    // A tunnel behind NAT is only reachable while its mappings live.
                    if net.nat_layers(i) > 0 {
                        let timeouts: Vec<Option<u32>> = chain
                            .iter()
                            .filter_map(|(_, n)| match (&world.nodes[*n], net.mode[*n]) {
                                (Node::Router(router), _) => Some(router.udp_timeout_s),
                                (Node::Isp(isp), Some((Mode::Cgnat, _))) => Some(isp.udp_timeout_s),
                                _ => None,
                            })
                            .collect();
                        let shortest = timeouts.iter().flatten().min().copied();
                        let rule = Reason::TunnelKeepalive;
                        for keepalive in &keepalives {
                            let f = match (*keepalive, shortest) {
                                (None, _) => (Status::Fail, Severity::Warning, Reason::TunnelKeepaliveMissing),
                                (Some(k), Some(t)) if k >= t => {
                                    (Status::Fail, Severity::Warning, Reason::TunnelKeepaliveTooSlow)
                                }
                                _ if timeouts.iter().any(Option::is_none) => {
                                    (Status::Unknown, Severity::Warning, Reason::NatTimeoutUnknown)
                                }
                                _ => (Status::Pass, Severity::Info, rule),
                            };
                            out.push(finding(rule, f.0, f.1, f.2, involved.clone()));
                        }
                    }
                    let path_mtu = chain
                        .iter()
                        .map(|(link, _)| world.links[*link].mtu.unwrap_or(DEFAULT_LINK_MTU))
                        .min()
                        .unwrap_or(DEFAULT_LINK_MTU);
                    let v6_outer = dials.iter().any(|p| p.endpoint.as_ref().is_some_and(|e| e.host.contains(':')));
                    let needed = tunnel.mtu.unwrap_or(WIREGUARD_DEFAULT_MTU) + tunnel_overhead_bytes(v6_outer);
                    let rule = Reason::TunnelMtuFits;
                    out.push(if needed > path_mtu {
                        finding(rule, Status::Fail, Severity::Warning, Reason::TunnelMtuTooBig, vec![id.clone()])
                    } else {
                        finding(rule, Status::Pass, Severity::Info, rule, vec![id.clone()])
                    });
                }
            }
            Node::Isp(_) | Node::Internet(_) => {}
        }
    }

    // Errors first, then warnings, then information. Stable for equal input.
    out.sort_by(|a, b| {
        (a.severity, a.status, a.rule_id, &a.involved, a.explanation_id).cmp(&(
            b.severity,
            b.status,
            b.rule_id,
            &b.involved,
            b.explanation_id,
        ))
    });
    out
}
