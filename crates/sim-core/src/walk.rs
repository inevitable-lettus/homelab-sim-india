//! The packet walk. One packet is carried hop by hop through the world, and
//! every node it meets decides what happens to it: forward, rewrite, wrap,
//! unwrap, deliver or drop. NAT tables and tunnel sessions persist across the
//! legs of one attempt, which is what lets a reply, or a tunnel that dialled
//! out earlier, get back in.

use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;

use crate::facts::{Addr, Mode, Net};
use crate::ip::{IpNet, routable, sock};
use crate::model::{Bind, Host, Ipv6Mode, Isp, Node, Policy, Proto, Router, Target, TunnelKind};
use crate::physics::{
    DEFAULT_LINK_MTU, WIREGUARD_DEFAULT_MTU, WIREGUARD_HANDSHAKE_BYTES, propagation_ns, queue_ns, serialization_ns,
    tunnel_overhead_bytes,
};
use crate::reason::Reason;
use crate::trace::{
    HopTiming, InnerView, Leg, LegRole, PacketKind, PacketView, Step, StepAction, TableEvent, TableKind, TableRow,
};

const MAX_STEPS: usize = 64;
pub(crate) const OVERLAY_KEEPALIVE_S: u32 = 25;
const OVERLAY_DEFAULT_MTU: u32 = 1_280;
const WIREGUARD_DEFAULT_PORT: u16 = 51_820;
const ROUTER_PORT_FALLBACK: u16 = 61_000;
const CGNAT_PORT_BLOCK: u16 = 62_000;
pub(crate) const CLIENT_PORT: u16 = 51_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Sock {
    pub ip: IpAddr,
    pub port: u16,
}

impl Sock {
    fn text(self) -> String {
        sock(self.ip, self.port)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Flow {
    pub proto: Proto,
    pub src: Sock,
    pub dst: Sock,
    pub bytes: u32,
    pub reply: bool,
}

#[derive(Debug, Clone)]
enum Payload {
    Plain,
    /// `tun` and `peer` say which tunnel sent this, and to which of its peers.
    Handshake {
        tun: usize,
        peer: usize,
    },
    Data {
        tun: usize,
        peer: usize,
        inner: Flow,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct Pkt {
    flow: Flow,
    payload: Payload,
    /// How stale the sender's last packet is when this attempt happens.
    age_s: u32,
}

impl Pkt {
    pub fn plain(flow: Flow) -> Self {
        Pkt { flow, payload: Payload::Plain, age_s: 0 }
    }
}

struct Mapping {
    proto: Proto,
    inside: Sock,
    outside_port: u16,
    remotes: Vec<Sock>,
    age_s: u32,
}

struct Tun {
    host: usize,
    id: String,
    network: Option<String>,
    relay: bool,
    addr: Option<IpAddr>,
    port: u16,
    mtu: u32,
    key: Option<String>,
    peers: Vec<PeerRt>,
}

struct PeerRt {
    key: Option<String>,
    node: Option<usize>,
    endpoint: Option<Sock>,
    allowed: Vec<IpNet>,
    keepalive: Option<u32>,
}

pub(crate) enum End {
    Delivered { node: usize, seen: Flow },
    Dropped { node: usize, reason: Reason },
    Unknown { node: usize, reason: Reason },
}

enum Next {
    Out(usize),
    End(End),
}

enum NatIn {
    Hit(TableEvent),
    Miss,
    Expired(TableEvent),
    TimeoutUnknown,
}

pub(crate) enum NoAddr {
    Fail(Reason),
    Unknown(Reason),
}

fn addr_or(addr: Addr, absent: Reason) -> Result<IpAddr, NoAddr> {
    match addr {
        Addr::Known(ip) => Ok(ip),
        Addr::Unknown => Err(NoAddr::Unknown(Reason::LineTypeUnknown)),
        Addr::Absent => Err(NoAddr::Fail(absent)),
    }
}

/// Turn "the public address of the router" into an actual address.
pub(crate) fn resolve(net: &Net, target: &Target, depth: u8) -> Result<IpAddr, NoAddr> {
    let index = |id: &str| net.index(id).ok_or(NoAddr::Fail(Reason::UnknownNode));
    match target {
        Target::Address { ip } => Ok(*ip),
        Target::Name { name } => {
            let record = net.world.dns.iter().find(|record| record.name.eq_ignore_ascii_case(name));
            match record {
                Some(record) if depth < 4 => resolve(net, &record.target, depth + 1),
                _ => Err(NoAddr::Fail(Reason::DnsNotFound)),
            }
        }
        Target::LanOf { node } => {
            let i = index(node)?;
            match net.gateway4(i) {
                Some(ip) => Ok(ip),
                None => addr_or(net.up4[i], Reason::HostNoAddress),
            }
        }
        Target::WanOf { node } => addr_or(net.up4[index(node)?], Reason::HostNoAddress),
        Target::PublicOf { node } => addr_or(net.public4(index(node)?), Reason::HostNoAddress),
        Target::Ipv6Of { node } => addr_or(net.up6[index(node)?], Reason::Ipv6NotAvailable),
        Target::OverlayOf { node, network } => {
            let i = index(node)?;
            let joined = matches!(&net.world.nodes[i], Node::Host(host) if host.tunnels.iter().any(|t| {
                t.enabled && t.kind == TunnelKind::Overlay && t.network.as_deref() == Some(network.as_str())
            }));
            match net.overlay_address(i, network) {
                Some(ip) if joined => Ok(ip),
                _ => Err(NoAddr::Fail(Reason::OverlayNotJoined)),
            }
        }
        Target::TunnelOf { node, tunnel } => {
            let i = index(node)?;
            let address = match &net.world.nodes[i] {
                Node::Host(host) => host.tunnels.iter().find(|t| t.enabled && t.id == *tunnel).and_then(|t| t.address),
                _ => None,
            };
            address.map(|a| a.addr).ok_or(NoAddr::Fail(Reason::HostNoAddress))
        }
    }
}

/// Steps being collected for the node the packet is at.
struct Cx<'s> {
    steps: &'s mut Vec<Step>,
    at: usize,
    id: String,
    arrival: Option<(usize, String, HopTiming)>,
}

impl Cx<'_> {
    fn arrived_on(&self) -> Option<usize> {
        self.arrival.as_ref().map(|(link, ..)| *link)
    }

    fn emit(
        &mut self,
        action: StepAction,
        packet: PacketView,
        before: Option<PacketView>,
        reason: Reason,
        table: Option<TableEvent>,
    ) {
        let (via_link, timing) = match self.arrival.take() {
            Some((_, id, timing)) => (Some(id), Some(timing)),
            None => (None, None),
        };
        self.steps.push(Step { at: self.id.clone(), action, packet, before, reason, via_link, timing, table });
    }

    fn drop(&mut self, reason: Reason, packet: PacketView, table: Option<TableEvent>) -> Next {
        self.emit(StepAction::Drop, packet, None, reason, table);
        Next::End(End::Dropped { node: self.at, reason })
    }

    fn unknown(&mut self, reason: Reason, packet: PacketView) -> Next {
        self.emit(StepAction::Unknown, packet, None, reason, None);
        Next::End(End::Unknown { node: self.at, reason })
    }
}

pub(crate) struct Engine<'a> {
    net: &'a Net<'a>,
    tuns: Vec<Tun>,
    nat: BTreeMap<usize, Vec<Mapping>>,
    v6_flows: BTreeSet<(usize, Proto, Sock, Sock)>,
    /// Where a tunnel last heard its peer from: (tunnel, peer) to endpoint.
    learned: BTreeMap<(usize, usize), Sock>,
    /// Handshakes already attempted, and whether they got through.
    sessions: BTreeMap<(usize, usize), bool>,
    used: BTreeSet<usize>,
    legs: Vec<(Leg, Option<usize>)>,
    idle_s: u32,
}

impl<'a> Engine<'a> {
    pub fn new(net: &'a Net<'a>, idle_s: u32) -> Self {
        let world = net.world;
        let mut tuns = Vec::new();
        for &i in net.idx.values() {
            let Node::Host(host) = &world.nodes[i] else {
                continue;
            };
            for tunnel in host.tunnels.iter().filter(|t| t.enabled) {
                match tunnel.kind {
                    TunnelKind::Wireguard => tuns.push(Tun {
                        host: i,
                        id: tunnel.id.clone(),
                        network: None,
                        relay: false,
                        addr: tunnel.address.map(|a| a.addr),
                        port: tunnel.listen_port.unwrap_or(WIREGUARD_DEFAULT_PORT),
                        mtu: tunnel.mtu.unwrap_or(WIREGUARD_DEFAULT_MTU),
                        key: tunnel.public_key.clone(),
                        peers: tunnel
                            .peers
                            .iter()
                            .map(|peer| PeerRt {
                                key: peer.public_key.clone(),
                                node: peer.node.as_deref().and_then(|id| net.index(id)),
                                endpoint: peer.endpoint.as_ref().and_then(|e| {
                                    let target = match e.host.parse::<IpAddr>() {
                                        Ok(ip) => Target::Address { ip },
                                        Err(_) => Target::Name { name: e.host.clone() },
                                    };
                                    resolve(net, &target, 0).ok().map(|ip| Sock { ip, port: e.port })
                                }),
                                allowed: peer.allowed_ips.clone(),
                                keepalive: peer.keepalive_s,
                            })
                            .collect(),
                    }),
                    TunnelKind::Overlay => {
                        let Some(overlay) = world.overlays.iter().find(|o| Some(&o.id) == tunnel.network.as_ref())
                        else {
                            continue;
                        };
                        let relay = net.index(&overlay.relay);
                        let endpoint = relay.and_then(|r| net.up4[r].known()).map(|ip| Sock { ip, port: overlay.port });
                        tuns.push(Tun {
                            host: i,
                            id: tunnel.id.clone(),
                            network: Some(overlay.id.clone()),
                            relay: false,
                            addr: net.overlay_address(i, &overlay.id),
                            port: overlay.port,
                            mtu: tunnel.mtu.unwrap_or(OVERLAY_DEFAULT_MTU),
                            key: None,
                            peers: vec![PeerRt {
                                key: None,
                                node: relay,
                                endpoint,
                                allowed: vec![overlay.range],
                                keepalive: Some(OVERLAY_KEEPALIVE_S),
                            }],
                        });
                    }
                }
            }
        }
        // Each overlay's relay knows its members, but only learns where they
        // are when they dial in.
        let mut overlays: Vec<_> = world.overlays.iter().collect();
        overlays.sort_by(|a, b| a.id.cmp(&b.id));
        for overlay in overlays {
            let Some(relay) = net.index(&overlay.relay) else {
                continue;
            };
            let peers = tuns
                .iter()
                .filter(|t| t.network.as_ref() == Some(&overlay.id))
                .map(|t| PeerRt {
                    key: None,
                    node: Some(t.host),
                    endpoint: None,
                    allowed: t
                        .addr
                        .map(|addr| IpNet { addr, prefix: if addr.is_ipv4() { 32 } else { 128 } })
                        .into_iter()
                        .collect(),
                    keepalive: None,
                })
                .collect();
            tuns.push(Tun {
                host: relay,
                id: format!("{}-relay", overlay.id),
                network: Some(overlay.id.clone()),
                relay: true,
                addr: None,
                port: overlay.port,
                mtu: OVERLAY_DEFAULT_MTU,
                key: None,
                peers,
            });
        }
        Engine {
            net,
            tuns,
            nat: BTreeMap::new(),
            v6_flows: BTreeSet::new(),
            learned: BTreeMap::new(),
            sessions: BTreeMap::new(),
            used: BTreeSet::new(),
            legs: Vec::new(),
            idle_s,
        }
    }

    /// A tunnel that knows where its peer is has dialled it. With a keepalive
    /// it keeps dialling, and that is what holds the NAT mappings open.
    /// Without one it last spoke when the tunnel was last used, `idle_s` ago.
    pub fn establish(&mut self) {
        for t in 0..self.tuns.len() {
            for p in 0..self.tuns[t].peers.len() {
                let peer = &self.tuns[t].peers[p];
                if peer.endpoint.is_none() {
                    continue;
                }
                let age = match peer.keepalive {
                    Some(keepalive) => keepalive.min(self.idle_s),
                    None => self.idle_s,
                };
                self.ensure_session(t, p, age);
            }
        }
    }

    /// Setup legs of tunnels the attempt used, then its own legs.
    pub fn into_legs(self) -> Vec<Leg> {
        let used = self.used;
        let tuns = self.tuns;
        self.legs
            .into_iter()
            .filter(|(_, setup)| match setup {
                None => true,
                Some(t) => {
                    // A member's setup matters when its overlay carried data.
                    used.contains(t)
                        || tuns[*t].network.is_some() && used.iter().any(|u| tuns[*u].network == tuns[*t].network)
                }
            })
            .map(|(leg, _)| leg)
            .collect()
    }

    pub fn source_for(&self, node: usize, dst: IpAddr) -> Option<IpAddr> {
        if let Some((t, _)) = self.tunnel_route(node, dst)
            && let Some(addr) = self.tuns[t].addr
        {
            return Some(addr);
        }
        self.own_ip(node, dst.is_ipv6())
    }

    fn own_ip(&self, node: usize, v6: bool) -> Option<IpAddr> {
        if v6 { self.net.up6[node].known() } else { self.net.up4[node].known() }
    }

    fn owns(&self, node: usize, ip: IpAddr) -> bool {
        self.net.up4[node].known() == Some(ip)
            || self.net.up6[node].known() == Some(ip)
            || self.tuns.iter().any(|t| t.host == node && t.addr == Some(ip))
    }

    /// The tunnel and peer that should carry a packet for `dst`, if any.
    fn tunnel_route(&self, node: usize, dst: IpAddr) -> Option<(usize, usize)> {
        let mut best: Option<(u8, usize, usize)> = None;
        for (t, tun) in self.tuns.iter().enumerate() {
            if tun.host != node || tun.relay || tun.addr == Some(dst) {
                continue;
            }
            for (p, peer) in tun.peers.iter().enumerate() {
                for net in peer.allowed.iter().filter(|net| net.contains(dst)) {
                    if best.is_none_or(|(prefix, ..)| net.prefix > prefix) {
                        best = Some((net.prefix, t, p));
                    }
                }
            }
        }
        best.map(|(_, t, p)| (t, p))
    }

    fn ensure_session(&mut self, t: usize, p: usize, age_s: u32) {
        if self.sessions.contains_key(&(t, p)) {
            return;
        }
        self.sessions.insert((t, p), false);
        let tun = &self.tuns[t];
        let Some(endpoint) = tun.peers[p].endpoint else {
            return;
        };
        let Some(src_ip) = self.own_ip(tun.host, endpoint.ip.is_ipv6()) else {
            return;
        };
        let host = tun.host;
        let flow = Flow {
            proto: Proto::Udp,
            src: Sock { ip: src_ip, port: tun.port },
            dst: endpoint,
            bytes: WIREGUARD_HANDSHAKE_BYTES,
            reply: false,
        };
        let id = format!("setup-{}-{}", self.net.id(host), tun.id);
        let pkt = Pkt { flow, payload: Payload::Handshake { tun: t, peer: p }, age_s };
        let ok = matches!(self.walk(host, pkt, LegRole::Setup, id, Some(t)), End::Delivered { .. });
        if !ok && self.tuns[t].network.is_none() {
            // A hand-made tunnel that could not come up is worth showing.
            self.used.insert(t);
        }
        self.sessions.insert((t, p), ok);
    }

    fn view(&self, pkt: &Pkt) -> PacketView {
        let (kind, inner) = match &pkt.payload {
            Payload::Plain => (PacketKind::Data, None),
            Payload::Handshake { .. } => (PacketKind::Handshake, None),
            Payload::Data { tun, inner, .. } => (
                PacketKind::Data,
                Some(InnerView {
                    tunnel: self.tuns[*tun].id.clone(),
                    proto: inner.proto,
                    src: inner.src.text(),
                    dst: inner.dst.text(),
                }),
            ),
        };
        PacketView {
            kind,
            proto: pkt.flow.proto,
            src: pkt.flow.src.text(),
            dst: pkt.flow.dst.text(),
            bytes: pkt.flow.bytes,
            reply: pkt.flow.reply || matches!(&pkt.payload, Payload::Data { inner, .. } if inner.reply),
            inner,
        }
    }

    pub fn walk(&mut self, origin: usize, mut pkt: Pkt, role: LegRole, id: String, setup: Option<usize>) -> End {
        let net = self.net;
        let mut steps = Vec::new();
        let mut cur = origin;
        let mut arrival = None;
        let mut first = true;
        let mut duration = Some(0u64);
        let end = loop {
            let mut cx = Cx { steps: &mut steps, at: cur, id: net.id(cur).to_owned(), arrival: arrival.take() };
            if cx.steps.len() >= MAX_STEPS {
                let view = self.view(&pkt);
                if let Next::End(end) = cx.drop(Reason::RoutingLoop, view, None) {
                    break end;
                }
            }
            let next = match &net.world.nodes[cur] {
                Node::Host(host) => self.at_host(&mut cx, cur, host, &mut pkt, first),
                Node::Router(router) => self.at_router(&mut cx, cur, router, &mut pkt),
                Node::Isp(isp) => self.at_isp(&mut cx, cur, isp, &mut pkt),
                Node::Internet(_) => self.at_internet(&mut cx, cur, &mut pkt),
            };
            first = false;
            let link = match next {
                Next::End(end) => break end,
                Next::Out(link) => link,
            };
            let wire = &net.world.links[link];
            let mtu = wire.mtu.unwrap_or(DEFAULT_LINK_MTU);
            if pkt.flow.bytes > mtu {
                let view = self.view(&pkt);
                if let Next::End(end) = cx.drop(Reason::PacketTooBig, view, None) {
                    break end;
                }
            }
            let forward = wire.a == net.id(cur);
            let other = if forward { net.index(&wire.b) } else { net.index(&wire.a) };
            let Some(other) = other else {
                break End::Dropped { node: cur, reason: Reason::NotConnected };
            };
            let rate = if forward { wire.rate_ab_bps } else { wire.rate_ba_bps };
            let propagation_ns = wire.length_m.map(|m| propagation_ns(wire.medium, m));
            let serialization_ns = rate.and_then(|rate| serialization_ns(pkt.flow.bytes, rate));
            let queue = serialization_ns.map(|s| queue_ns(s, wire.load_permille));
            let processing_ns = match &net.world.nodes[other] {
                Node::Host(_) => 20_000,
                Node::Router(_) => 100_000,
                Node::Isp(_) => 150_000,
                Node::Internet(_) => 50_000,
            };
            let total_ns = match (propagation_ns, serialization_ns, queue) {
                (Some(p), Some(s), Some(q)) => Some(p + s + q + wire.extra_ns + processing_ns),
                _ => None,
            };
            duration = match (duration, total_ns) {
                (Some(d), Some(t)) => Some(d + t),
                _ => None,
            };
            let timing = HopTiming {
                bytes: pkt.flow.bytes,
                mtu,
                rate_bps: rate,
                load_permille: wire.load_permille,
                propagation_ns,
                serialization_ns,
                queue_ns: queue,
                extra_ns: wire.extra_ns,
                processing_ns,
                total_ns,
                arrive_ns: duration,
            };
            arrival = Some((link, wire.id.clone(), timing));
            cur = other;
        };
        let leg = Leg { id, role, from: net.id(origin).to_owned(), steps, duration_ns: duration };
        self.legs.push((leg, setup));
        end
    }

    // ------------------------------------------------------------ hosts

    fn at_host(&mut self, cx: &mut Cx, cur: usize, host: &Host, pkt: &mut Pkt, first: bool) -> Next {
        let net = self.net;
        if first {
            let Some((uplink, _)) = net.up[cur] else {
                return cx.drop(Reason::NotConnected, self.view(pkt), None);
            };
            if matches!(pkt.payload, Payload::Plain)
                && let Some((t, p)) = self.tunnel_route(cur, pkt.flow.dst.ip)
            {
                return self.encapsulate(cx, cur, t, p, pkt, true);
            }
            let reason = match pkt.payload {
                Payload::Handshake { .. } => Reason::TunnelDialsOut,
                _ => Reason::Sent,
            };
            cx.emit(StepAction::Send, self.view(pkt), None, reason, None);
            return Next::Out(uplink);
        }
        if !self.owns(cur, pkt.flow.dst.ip) {
            return cx.drop(Reason::NoHostAtAddress, self.view(pkt), None);
        }
        if !matches!(pkt.payload, Payload::Plain) {
            let listening = self.tuns.iter().position(|t| t.host == cur && t.port == pkt.flow.dst.port);
            return match listening {
                Some(t) if pkt.flow.proto == Proto::Udp => self.tunnel_receive(cx, cur, host, t, pkt),
                _ => cx.drop(Reason::NoService, self.view(pkt), None),
            };
        }
        self.deliver(cx, cur, host, pkt)
    }

    fn deliver(&mut self, cx: &mut Cx, cur: usize, host: &Host, pkt: &Pkt) -> Next {
        let flow = &pkt.flow;
        let verdict = if flow.reply {
            Ok(Reason::ReplyDelivered)
        } else {
            let blocked = host.firewall.as_ref().is_some_and(|firewall| {
                let rule = firewall.rules.iter().find(|r| r.proto == flow.proto && r.port == flow.dst.port);
                rule.map_or(firewall.inbound, |r| r.action) == Policy::Deny
            });
            let service = host.services.iter().find(|s| s.proto == flow.proto && s.port == flow.dst.port);
            match service {
                _ if blocked => Err(Reason::HostFirewall),
                None => Err(Reason::NoService),
                Some(service) if service.bind == Bind::Loopback => Err(Reason::ServiceLoopbackOnly),
                Some(_) => Ok(Reason::Delivered),
            }
        };
        match verdict {
            Ok(reason) => {
                cx.emit(StepAction::Deliver, self.view(pkt), None, reason, None);
                Next::End(End::Delivered { node: cur, seen: flow.clone() })
            }
            Err(reason) => cx.drop(reason, self.view(pkt), None),
        }
    }

    /// Wrap a packet for a tunnel peer and send the wrapper on its way.
    fn encapsulate(&mut self, cx: &mut Cx, cur: usize, t: usize, p: usize, pkt: &mut Pkt, originating: bool) -> Next {
        let before = self.view(pkt);
        if pkt.flow.bytes > self.tuns[t].mtu {
            return cx.drop(Reason::TunnelMtuExceeded, before, None);
        }
        // WireGuard holds data until its handshake completes.
        if self.tuns[t].peers[p].endpoint.is_some() {
            self.ensure_session(t, p, 0);
            if self.sessions.get(&(t, p)) == Some(&false) {
                self.used.insert(t);
                return cx.drop(Reason::TunnelHandshakeFailed, before, None);
            }
        }
        let tun = &self.tuns[t];
        let endpoint = tun.peers[p].endpoint.or_else(|| self.learned.get(&(t, p)).copied());
        let Some(endpoint) = endpoint else {
            return cx.drop(Reason::TunnelNoEndpoint, before, None);
        };
        let (Some(src_ip), Some((uplink, _))) = (self.own_ip(cur, endpoint.ip.is_ipv6()), self.net.up[cur]) else {
            return cx.drop(Reason::HostNoAddress, before, None);
        };
        let mut inner = pkt.flow.clone();
        if originating && let Some(addr) = tun.addr {
            inner.src.ip = addr;
        }
        let outer = Flow {
            proto: Proto::Udp,
            src: Sock { ip: src_ip, port: tun.port },
            dst: endpoint,
            bytes: inner.bytes + tunnel_overhead_bytes(endpoint.ip.is_ipv6()),
            reply: false,
        };
        let reason = if originating { Reason::TunnelEncapsulate } else { Reason::HubForwards };
        *pkt = Pkt { flow: outer, payload: Payload::Data { tun: t, peer: p, inner }, age_s: 0 };
        self.used.insert(t);
        cx.emit(StepAction::Encapsulate, self.view(pkt), Some(before), reason, None);
        Next::Out(uplink)
    }

    fn tunnel_receive(&mut self, cx: &mut Cx, cur: usize, host: &Host, t: usize, pkt: &mut Pkt) -> Next {
        let before = self.view(pkt);
        let (sender, via) = match &pkt.payload {
            Payload::Handshake { tun, peer } | Payload::Data { tun, peer, .. } => (*tun, *peer),
            Payload::Plain => return cx.drop(Reason::NoService, before, None),
        };
        let tun = &self.tuns[t];
        let member = tun.network.is_some() && !tun.relay;
        let peer = if member {
            // An overlay member only ever hears from its relay.
            tun.peers.iter().position(|peer| peer.endpoint == Some(pkt.flow.src))
        } else {
            let from = &self.tuns[sender];
            tun.peers.iter().position(|peer| match &peer.key {
                Some(key) => from.key.as_ref() == Some(key),
                None => peer.node == Some(from.host),
            })
        };
        let Some(p) = peer else {
            return cx.drop(Reason::TunnelUnknownPeer, before, None);
        };
        // The sender locked the packet to the key it has for us. If that is
        // not our key, we cannot open it.
        if tun.network.is_none()
            && let (Some(ours), Some(theirs)) =
                (&tun.key, self.tuns[sender].peers.get(via).and_then(|v| v.key.as_ref()))
            && ours != theirs
        {
            return cx.drop(Reason::TunnelWrongPeerKey, before, None);
        }
        // Roaming: remember the address this peer was last heard from.
        self.learned.insert((t, p), pkt.flow.src);
        self.used.insert(t);
        let inner = match &pkt.payload {
            Payload::Data { inner, .. } => inner.clone(),
            _ => {
                cx.emit(StepAction::Deliver, before, None, Reason::TunnelSessionUp, None);
                return Next::End(End::Delivered { node: cur, seen: pkt.flow.clone() });
            }
        };
        let tun = &self.tuns[t];
        if tun.relay {
            // The relay cannot read the packet. It passes it to the member
            // that owns the inner destination, if that member has dialled in.
            let target = tun.peers.iter().position(|peer| peer.allowed.iter().any(|n| n.contains(inner.dst.ip)));
            let endpoint = target.and_then(|p2| self.learned.get(&(t, p2)).copied());
            let (Some(endpoint), Some(src_ip), Some((uplink, _))) =
                (endpoint, self.net.up4[cur].known(), self.net.up[cur])
            else {
                return cx.drop(Reason::RelayPeerOffline, before, None);
            };
            pkt.flow.src = Sock { ip: src_ip, port: tun.port };
            pkt.flow.dst = endpoint;
            cx.emit(StepAction::Forward, self.view(pkt), Some(before), Reason::RelayForwards, None);
            return Next::Out(uplink);
        }
        // Cryptokey routing: a peer may only send from addresses it was given.
        if !member && !tun.peers[p].allowed.iter().any(|n| n.contains(inner.src.ip)) {
            return cx.drop(Reason::TunnelAllowedIps, before, None);
        }
        let age_s = pkt.age_s;
        *pkt = Pkt { flow: inner, payload: Payload::Plain, age_s };
        if self.owns(cur, pkt.flow.dst.ip) {
            cx.emit(StepAction::Decapsulate, self.view(pkt), Some(before), Reason::TunnelDecapsulate, None);
            return self.deliver(cx, cur, host, pkt);
        }
        if !host.forwarding {
            return cx.drop(Reason::HostNotForwarding, self.view(pkt), None);
        }
        match self.tunnel_route(cur, pkt.flow.dst.ip) {
            Some((t2, p2)) => self.encapsulate(cx, cur, t2, p2, pkt, false),
            None => cx.drop(Reason::TunnelAllowedIps, self.view(pkt), None),
        }
    }

    // ---------------------------------------------------------- routers

    /// The link to the directly attached customer that holds `ip`.
    fn customer_with(&self, cur: usize, ip: IpAddr) -> Option<usize> {
        let net = self.net;
        net.adj[cur].iter().find_map(|&(link, nb)| {
            let mine = net.up[nb].map(|(_, parent)| parent) == Some(cur);
            let holds = net.up4[nb].known() == Some(ip)
                || net.up6[nb].known() == Some(ip)
                || net.lan6[nb].is_some_and(|lan| lan.contains(ip));
            (mine && holds).then_some(link)
        })
    }

    fn came_from_uplink(&self, cx: &Cx, cur: usize) -> bool {
        let uplink = self.net.up[cur].map(|(link, _)| link);
        cx.arrived_on().is_some() && cx.arrived_on() == uplink
    }

    fn at_router(&mut self, cx: &mut Cx, cur: usize, router: &Router, pkt: &mut Pkt) -> Next {
        let net = self.net;
        let from_up = self.came_from_uplink(cx, cur);
        let dst = pkt.flow.dst.ip;
        let before = self.view(pkt);
        if dst.is_ipv6() {
            return self.router_v6(cx, cur, router, pkt, from_up);
        }
        let wan = net.up4[cur].known();
        if !from_up && Some(dst) != wan {
            if Some(dst) == net.gateway4(cur) {
                return cx.drop(Reason::NoService, before, None);
            }
            if router.lan.contains(dst) {
                return match self.customer_with(cur, dst) {
                    Some(link) => {
                        cx.emit(StepAction::Forward, before, None, Reason::Forwarded, None);
                        Next::Out(link)
                    }
                    None => cx.drop(Reason::NoHostAtAddress, before, None),
                };
            }
            let (Some((uplink, _)), Some(wan)) = (net.up[cur], wan) else {
                return cx.drop(Reason::NotConnected, before, None);
            };
            let (event, _) = self.nat_out(cur, pkt, wan, None, router.udp_timeout_s);
            cx.emit(StepAction::Rewrite, self.view(pkt), Some(before), Reason::NatSourceRewrite, Some(event));
            return Next::Out(uplink);
        }
        if Some(dst) != wan {
            return cx.drop(Reason::NoRoute, before, None);
        }
        match self.nat_in(cur, pkt, dst, router.udp_timeout_s) {
            NatIn::Hit(event) => {
                cx.emit(StepAction::Rewrite, self.view(pkt), Some(before), Reason::NatMappingHit, Some(event));
            }
            NatIn::Expired(event) => return cx.drop(Reason::MappingExpired, before, Some(event)),
            NatIn::TimeoutUnknown => return cx.unknown(Reason::NatTimeoutUnknown, before),
            NatIn::Miss => {
                let flow = &mut pkt.flow;
                let forward = router
                    .port_forwards
                    .iter()
                    .find(|f| f.enabled && f.proto == flow.proto && f.external_port == flow.dst.port);
                let Some(forward) = forward else {
                    let miss = TableEvent { kind: TableKind::NoMatch, row: None };
                    return cx.drop(Reason::RouterNoForward, before, Some(miss));
                };
                let inside = Sock { ip: IpAddr::V4(forward.to_ip), port: forward.internal_port };
                let row = TableRow {
                    proto: flow.proto,
                    inside: inside.text(),
                    outside: flow.dst.text(),
                    remote: flow.src.text(),
                    timeout_s: None,
                    idle_s: 0,
                };
                // Remember the connection so the reply is translated back.
                self.nat.entry(cur).or_default().push(Mapping {
                    proto: flow.proto,
                    inside,
                    outside_port: flow.dst.port,
                    remotes: vec![flow.src],
                    age_s: 0,
                });
                flow.dst = inside;
                let hit = TableEvent { kind: TableKind::ForwardHit, row: Some(row) };
                cx.emit(StepAction::Rewrite, self.view(pkt), Some(before), Reason::PortForwardHit, Some(hit));
                return match self.customer_with(cur, inside.ip) {
                    Some(link) => Next::Out(link),
                    None => cx.drop(Reason::ForwardTargetMissing, self.view(pkt), None),
                };
            }
        }
        match self.customer_with(cur, pkt.flow.dst.ip) {
            Some(link) => Next::Out(link),
            None => cx.drop(Reason::NoHostAtAddress, self.view(pkt), None),
        }
    }

    fn router_v6(&mut self, cx: &mut Cx, cur: usize, router: &Router, pkt: &mut Pkt, from_up: bool) -> Next {
        let net = self.net;
        let flow = pkt.flow.clone();
        let view = self.view(pkt);
        let local = net.lan6[cur].is_some_and(|lan| lan.contains(flow.dst.ip));
        if local {
            if from_up {
                let known = self.v6_flows.contains(&(cur, flow.proto, flow.dst, flow.src));
                let pinhole = router.ipv6_pinholes.iter().any(|hole| {
                    let to = net.index(&hole.to).and_then(|i| net.up6[i].known());
                    to == Some(flow.dst.ip) && hole.proto == flow.proto && hole.port == flow.dst.port
                });
                if !(known || pinhole || router.ipv6_inbound == Policy::Allow) {
                    return cx.drop(Reason::Ipv6Firewall, view, None);
                }
            }
            return match self.customer_with(cur, flow.dst.ip) {
                Some(link) => {
                    cx.emit(StepAction::Forward, view, None, Reason::Ipv6NoNat, None);
                    Next::Out(link)
                }
                None => cx.drop(Reason::NoHostAtAddress, view, None),
            };
        }
        match net.up[cur] {
            Some((uplink, _)) if !from_up && net.lan6[cur].is_some() => {
                self.v6_flows.insert((cur, flow.proto, flow.src, flow.dst));
                cx.emit(StepAction::Forward, view, None, Reason::Ipv6NoNat, None);
                Next::Out(uplink)
            }
            _ => cx.drop(Reason::NoRoute, view, None),
        }
    }

    // ------------------------------------------------- ISPs and internet

    fn at_isp(&mut self, cx: &mut Cx, cur: usize, isp: &Isp, pkt: &mut Pkt) -> Next {
        let net = self.net;
        let from_up = self.came_from_uplink(cx, cur);
        let dst = pkt.flow.dst.ip;
        let before = self.view(pkt);
        let uplink = net.up[cur].map(|(link, _)| link);
        if dst.is_ipv6() && isp.ipv6 != Ipv6Mode::Yes {
            return cx.drop(Reason::Ipv6NotAvailable, before, None);
        }
        if let Some(link) = self.customer_with(cur, dst) {
            cx.emit(StepAction::Forward, before, None, Reason::Forwarded, None);
            return Next::Out(link);
        }
        let mode = net.mode[cur].map_or(Mode::Unknown, |(mode, _)| mode);
        let nat = mode == Mode::Cgnat && dst.is_ipv4();
        if mode == Mode::Unknown && dst.is_ipv4() {
            return cx.unknown(Reason::LineTypeUnknown, before);
        }
        if from_up {
            let shared = net.shared4[cur].known();
            if !nat || Some(dst) != shared {
                return cx.drop(Reason::NoRoute, before, None);
            }
            return match self.nat_in(cur, pkt, dst, isp.udp_timeout_s) {
                NatIn::Hit(event) => {
                    cx.emit(StepAction::Rewrite, self.view(pkt), Some(before), Reason::CgnatMappingHit, Some(event));
                    match self.customer_with(cur, pkt.flow.dst.ip) {
                        Some(link) => Next::Out(link),
                        None => cx.drop(Reason::NoHostAtAddress, self.view(pkt), None),
                    }
                }
                NatIn::Expired(event) => cx.drop(Reason::MappingExpired, before, Some(event)),
                NatIn::TimeoutUnknown => cx.unknown(Reason::NatTimeoutUnknown, before),
                NatIn::Miss => {
                    let miss = TableEvent { kind: TableKind::NoMatch, row: None };
                    cx.drop(Reason::CgnatNoMapping, before, Some(miss))
                }
            };
        }
        if !routable(dst) {
            return cx.drop(Reason::PrivateNotRoutable, before, None);
        }
        let Some(uplink) = uplink else {
            return cx.drop(Reason::NotConnected, before, None);
        };
        if !nat {
            cx.emit(StepAction::Forward, before, None, Reason::Forwarded, None);
            return Next::Out(uplink);
        }
        let Some(shared) = net.shared4[cur].known() else {
            return cx.drop(Reason::HostNoAddress, before, None);
        };
        let (event, created) = self.nat_out(cur, pkt, shared, Some(CGNAT_PORT_BLOCK), isp.udp_timeout_s);
        let reason = if created { Reason::CgnatMappingCreated } else { Reason::CgnatSourceRewrite };
        cx.emit(StepAction::Rewrite, self.view(pkt), Some(before), reason, Some(event));
        Next::Out(uplink)
    }

    fn at_internet(&mut self, cx: &mut Cx, cur: usize, pkt: &mut Pkt) -> Next {
        let net = self.net;
        let dst = pkt.flow.dst.ip;
        let view = self.view(pkt);
        let owner = net.adj[cur].iter().find_map(|&(link, nb)| {
            let owns = match &net.world.nodes[nb] {
                Node::Host(_) => net.up4[nb].known() == Some(dst) || net.up6[nb].known() == Some(dst),
                Node::Isp(isp) => isp.public_pool.contains(dst) || isp.ipv6_prefix.is_some_and(|p| p.contains(dst)),
                Node::Router(_) | Node::Internet(_) => false,
            };
            owns.then_some(link)
        });
        match owner {
            Some(link) => {
                cx.emit(StepAction::Forward, view, None, Reason::Forwarded, None);
                Next::Out(link)
            }
            None if !routable(dst) => cx.drop(Reason::PrivateNotRoutable, view, None),
            None => cx.drop(Reason::NoRoute, view, None),
        }
    }

    // --------------------------------------------------------------- NAT

    fn row(mapping: &Mapping, public: IpAddr, remote: Sock, timeout_s: Option<u32>) -> TableRow {
        TableRow {
            proto: mapping.proto,
            inside: mapping.inside.text(),
            outside: sock(public, mapping.outside_port),
            remote: remote.text(),
            timeout_s,
            idle_s: mapping.age_s,
        }
    }

    /// Outbound through a NAT: reuse the sender's mapping or create one, then
    /// rewrite the source. `block` is the port range a carrier NAT allocates from.
    fn nat_out(
        &mut self,
        node: usize,
        pkt: &mut Pkt,
        public: IpAddr,
        block: Option<u16>,
        timeout_s: Option<u32>,
    ) -> (TableEvent, bool) {
        let table = self.nat.entry(node).or_default();
        let flow = &mut pkt.flow;
        let existing = table.iter().position(|m| m.proto == flow.proto && m.inside == flow.src);
        let created = existing.is_none();
        let i = existing.unwrap_or_else(|| {
            let taken = |port: u16| table.iter().any(|m| m.proto == flow.proto && m.outside_port == port);
            let mut port = block.unwrap_or(flow.src.port);
            if block.is_none() && taken(port) {
                port = ROUTER_PORT_FALLBACK;
            }
            while taken(port) {
                port = port.wrapping_add(1);
            }
            table.push(Mapping {
                proto: flow.proto,
                inside: flow.src,
                outside_port: port,
                remotes: Vec::new(),
                age_s: pkt.age_s,
            });
            table.len() - 1
        });
        let mapping = &mut table[i];
        if !mapping.remotes.contains(&flow.dst) {
            mapping.remotes.push(flow.dst);
        }
        mapping.age_s = mapping.age_s.min(pkt.age_s);
        let row = Self::row(mapping, public, flow.dst, timeout_s);
        flow.src = Sock { ip: public, port: mapping.outside_port };
        let kind = if created { TableKind::MappingCreated } else { TableKind::MappingHit };
        (TableEvent { kind, row: Some(row) }, created)
    }

    /// Inbound through a NAT: only a packet that matches a mapping, from the
    /// remote that mapping was opened towards, gets in.
    fn nat_in(&mut self, node: usize, pkt: &mut Pkt, public: IpAddr, timeout_s: Option<u32>) -> NatIn {
        let flow = &mut pkt.flow;
        let mapping = self.nat.get(&node).and_then(|table| {
            table
                .iter()
                .find(|m| m.proto == flow.proto && m.outside_port == flow.dst.port && m.remotes.contains(&flow.src))
        });
        let Some(mapping) = mapping else {
            return NatIn::Miss;
        };
        let row = Self::row(mapping, public, flow.src, timeout_s);
        if flow.proto == Proto::Udp && mapping.age_s > 0 {
            match timeout_s {
                None => return NatIn::TimeoutUnknown,
                Some(timeout) if mapping.age_s >= timeout => {
                    return NatIn::Expired(TableEvent { kind: TableKind::Expired, row: Some(row) });
                }
                Some(_) => {}
            }
        }
        flow.dst = mapping.inside;
        NatIn::Hit(TableEvent { kind: TableKind::MappingHit, row: Some(row) })
    }
}
