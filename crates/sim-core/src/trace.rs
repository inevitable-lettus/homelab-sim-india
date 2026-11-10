//! A trace is the ordered story of one connection attempt: every hop, what
//! each node did to the packet, how long it took, and where it ended. The UI
//! only plays this back. It never decides anything itself.

use serde::Serialize;

use crate::Status;
use crate::bundle::Bundle;
use crate::facts::{Assumptions, Mode, Net};
use crate::model::{Attempt, Direction, Id, Node, Proto, TunnelKind, World};
use crate::physics::{TCP_IPV4_HEADER_BYTES, TCP_IPV6_HEADER_BYTES, spare_rate_bps, transfer_ms, wire_bytes};
use crate::reason::Reason;
use crate::walk::{CLIENT_PORT, End, Engine, Flow, NoAddr, Pkt, Sock, resolve};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Delivered,
    Dropped,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    Send,
    Forward,
    Rewrite,
    Encapsulate,
    Decapsulate,
    Drop,
    Deliver,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LegRole {
    /// A tunnel dialling out before the attempt, which opens the way back in.
    Setup,
    Request,
    Reply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketKind {
    Data,
    Handshake,
}

/// Source and destination as seen at one hop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PacketView {
    pub kind: PacketKind,
    pub proto: Proto,
    pub src: String,
    pub dst: String,
    pub bytes: u32,
    pub reply: bool,
    /// The packet carried inside, when this one is a tunnel wrapper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner: Option<InnerView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InnerView {
    pub tunnel: Id,
    pub proto: Proto,
    pub src: String,
    pub dst: String,
}

/// Time spent crossing one link and being handled on arrival. A missing part
/// means the world did not say enough to compute it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HopTiming {
    pub bytes: u32,
    pub mtu: u32,
    pub rate_bps: Option<u64>,
    pub load_permille: u16,
    pub propagation_ns: Option<u64>,
    pub serialization_ns: Option<u64>,
    pub queue_ns: Option<u64>,
    /// Fixed delay of the medium, such as radio scheduling.
    pub extra_ns: u64,
    pub processing_ns: u64,
    pub total_ns: Option<u64>,
    /// Time since the leg started.
    pub arrive_ns: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TableKind {
    MappingCreated,
    MappingHit,
    ForwardHit,
    NoMatch,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableRow {
    pub proto: Proto,
    pub inside: String,
    pub outside: String,
    pub remote: String,
    pub timeout_s: Option<u32>,
    pub idle_s: u32,
}

/// What a NAT table or forward list did with the packet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableEvent {
    pub kind: TableKind,
    pub row: Option<TableRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    pub at: Id,
    pub action: StepAction,
    /// The packet as it leaves this node, or as it arrived if it stops here.
    pub packet: PacketView,
    /// The packet before this node changed it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<PacketView>,
    pub reason: Reason,
    /// The link crossed to get here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via_link: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<HopTiming>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<TableEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Leg {
    pub id: String,
    pub role: LegRole,
    pub from: Id,
    pub steps: Vec<Step>,
    pub duration_ns: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransferEstimate {
    pub bytes: u64,
    pub direction: Direction,
    /// Payload plus headers and tunnel wrappers.
    pub wire_bytes: Option<u64>,
    pub bottleneck_link: Option<Id>,
    pub bottleneck_bps: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub hops: u32,
    pub one_way_ns: Option<u64>,
    pub round_trip_ns: Option<u64>,
    /// The largest packet from the sender that fits end to end.
    pub path_mtu: Option<u32>,
    pub transfer: Option<TransferEstimate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assumed {
    pub isp: Id,
    pub ipv4: Mode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trace {
    pub attempt: Id,
    pub status: Status,
    pub outcome: Outcome,
    /// Where the request ended: the node that took it, or dropped it.
    pub ended_at: Option<Id>,
    pub reason: Option<Reason>,
    /// The address the attempt was aimed at, once resolved.
    pub target: Option<String>,
    /// Line types the result was computed under when the real one is unknown
    /// and the result is the same either way.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assumed: Vec<Assumed>,
    pub legs: Vec<Leg>,
    pub summary: Summary,
}

impl Trace {
    fn empty(attempt: &Attempt, outcome: Outcome, at: Option<&str>, reason: Reason) -> Self {
        Trace {
            attempt: attempt.id.clone(),
            status: status_of(outcome),
            outcome,
            ended_at: at.map(str::to_owned),
            reason: Some(reason),
            target: None,
            assumed: Vec::new(),
            legs: Vec::new(),
            summary: Summary::default(),
        }
    }

    pub fn request(&self) -> Option<&Leg> {
        self.legs.iter().find(|leg| leg.role == LegRole::Request)
    }
}

fn status_of(outcome: Outcome) -> Status {
    match outcome {
        Outcome::Delivered => Status::Pass,
        Outcome::Dropped => Status::Fail,
        Outcome::Unknown => Status::Unknown,
    }
}

/// Trace one attempt. When a line's type is unknown, the attempt is traced
/// under each possibility. If they agree the answer stands. If they disagree
/// the honest answer is `unknown`.
pub fn trace(world: &World, attempt: &Attempt, bundle: &Bundle) -> Trace {
    let base = Net::build(world, bundle, &Assumptions::new());
    let unknown: Vec<&str> = base
        .idx
        .iter()
        .filter(|(_, i)| matches!(base.mode[**i], Some((Mode::Unknown, _))))
        .map(|(id, _)| *id)
        .collect();
    if unknown.is_empty() {
        return run(world, attempt, bundle, &Assumptions::new());
    }
    if unknown.len() > 3 {
        return Trace::empty(attempt, Outcome::Unknown, unknown.first().copied(), Reason::LineTypeUnknown);
    }
    let mut variants = Vec::new();
    for mask in 0..(1u32 << unknown.len()) {
        let assume: Assumptions = unknown
            .iter()
            .enumerate()
            .map(|(bit, id)| ((*id).to_owned(), if mask >> bit & 1 == 1 { Mode::Public } else { Mode::Cgnat }))
            .collect();
        variants.push((run(world, attempt, bundle, &assume), assume));
    }
    let (first, assume) = variants.remove(0);
    let agree = variants.iter().all(|(other, _)| {
        (other.outcome, &other.ended_at, other.reason) == (first.outcome, &first.ended_at, first.reason)
    });
    if !agree {
        return Trace::empty(attempt, Outcome::Unknown, unknown.first().copied(), Reason::LineTypeUnknown);
    }
    Trace { assumed: assume.into_iter().map(|(isp, ipv4)| Assumed { isp, ipv4 }).collect(), ..first }
}

fn run(world: &World, attempt: &Attempt, bundle: &Bundle, assume: &Assumptions) -> Trace {
    let net = Net::build(world, bundle, assume);
    let Some(origin) = net.index(&attempt.from) else {
        return Trace::empty(attempt, Outcome::Unknown, None, Reason::UnknownNode);
    };
    let from = attempt.from.as_str();
    let dst_ip = match resolve(&net, &attempt.to, 0) {
        Ok(ip) => ip,
        Err(NoAddr::Fail(reason)) => {
            return Trace::empty(attempt, Outcome::Dropped, Some(from), reason);
        }
        Err(NoAddr::Unknown(reason)) => {
            return Trace::empty(attempt, Outcome::Unknown, Some(from), reason);
        }
    };
    // A mesh address means nothing to a device that has not joined that mesh.
    let joined = |network: &str| match world.node(from) {
        Some(Node::Host(host)) => host
            .tunnels
            .iter()
            .any(|t| t.enabled && t.kind == TunnelKind::Overlay && t.network.as_deref() == Some(network)),
        _ => false,
    };
    if world.overlays.iter().any(|overlay| overlay.range.contains(dst_ip) && !joined(&overlay.id)) {
        return Trace::empty(attempt, Outcome::Dropped, Some(from), Reason::OverlayNotJoined);
    }
    let mut engine = Engine::new(&net, attempt.idle_s);
    engine.establish();
    let Some(src_ip) = engine.source_for(origin, dst_ip) else {
        let reason = if dst_ip.is_ipv6() { Reason::ClientNoIpv6 } else { Reason::HostNoAddress };
        return Trace::empty(attempt, Outcome::Dropped, Some(from), reason);
    };
    let dst = Sock { ip: dst_ip, port: attempt.port };
    let flow = Flow {
        proto: attempt.proto,
        src: Sock { ip: src_ip, port: CLIENT_PORT },
        dst,
        bytes: attempt.packet_bytes,
        reply: false,
    };
    let (outcome, ended, reason) = match engine.walk(origin, Pkt::plain(flow), LegRole::Request, "request".into(), None)
    {
        End::Delivered { node, seen } => {
            // The answer has to make it back for the connection to work.
            let back = Flow { proto: seen.proto, src: seen.dst, dst: seen.src, bytes: seen.bytes, reply: true };
            match engine.walk(node, Pkt::plain(back), LegRole::Reply, "reply".into(), None) {
                End::Delivered { node: home, .. } if home == origin => (Outcome::Delivered, node, None),
                End::Delivered { node: stray, .. } => (Outcome::Dropped, stray, Some(Reason::NoRoute)),
                End::Dropped { node, reason } => (Outcome::Dropped, node, Some(reason)),
                End::Unknown { node, reason } => (Outcome::Unknown, node, Some(reason)),
            }
        }
        End::Dropped { node, reason } => (Outcome::Dropped, node, Some(reason)),
        End::Unknown { node, reason } => (Outcome::Unknown, node, Some(reason)),
    };
    let legs = engine.into_legs();
    let summary = summarise(&legs, attempt, outcome, dst_ip.is_ipv6());
    Trace {
        attempt: attempt.id.clone(),
        status: status_of(outcome),
        outcome,
        ended_at: Some(net.id(ended).to_owned()),
        reason,
        target: Some(crate::ip::sock(dst.ip, dst.port)),
        assumed: Vec::new(),
        legs,
        summary,
    }
}

fn summarise(legs: &[Leg], attempt: &Attempt, outcome: Outcome, v6: bool) -> Summary {
    let leg = |role| legs.iter().find(|leg: &&Leg| leg.role == role);
    let Some(request) = leg(LegRole::Request) else {
        return Summary::default();
    };
    let reply = leg(LegRole::Reply);
    let delivered = outcome == Outcome::Delivered;
    // Each hop can carry `mtu` bytes, and the wrappers added so far use some.
    let path_mtu = |leg: &Leg| {
        leg.steps
            .iter()
            .filter_map(|step| step.timing.as_ref())
            .map(|t| t.mtu.saturating_sub(t.bytes.saturating_sub(attempt.packet_bytes)))
            .min()
    };
    let transfer = attempt.transfer.as_ref().filter(|_| delivered).map(|transfer| {
        let carrier = match transfer.direction {
            Direction::Request => Some(request),
            Direction::Reply => reply,
        };
        let hops: Vec<_> = carrier
            .map(|leg| leg.steps.iter().filter_map(|s| Some((s.via_link.as_ref()?, s.timing.as_ref()?))).collect())
            .unwrap_or_default();
        let rates: Option<Vec<(u64, &Id)>> =
            hops.iter().map(|(link, t)| Some((spare_rate_bps(t.rate_bps?, t.load_permille), *link))).collect();
        let bottleneck = rates.and_then(|rates| rates.into_iter().min());
        let wrapper = hops.iter().map(|(_, t)| t.bytes.saturating_sub(attempt.packet_bytes)).max().unwrap_or(0);
        let header = if v6 { TCP_IPV6_HEADER_BYTES } else { TCP_IPV4_HEADER_BYTES };
        let wire =
            carrier.and_then(path_mtu).and_then(|inner| wire_bytes(transfer.bytes, inner + wrapper, header + wrapper));
        TransferEstimate {
            bytes: transfer.bytes,
            direction: transfer.direction,
            wire_bytes: wire,
            bottleneck_link: bottleneck.map(|(_, link)| link.clone()),
            bottleneck_bps: bottleneck.map(|(rate, _)| rate),
            duration_ms: match (wire, bottleneck) {
                (Some(wire), Some((rate, _))) => transfer_ms(wire, rate),
                _ => None,
            },
        }
    });
    Summary {
        hops: u32::try_from(request.steps.iter().filter(|s| s.via_link.is_some()).count()).unwrap_or(u32::MAX),
        one_way_ns: request.duration_ns.filter(|_| delivered),
        round_trip_ns: match (request.duration_ns, reply.and_then(|r| r.duration_ns)) {
            (Some(out), Some(back)) if delivered => Some(out + back),
            _ => None,
        },
        path_mtu: path_mtu(request),
        transfer,
    }
}
