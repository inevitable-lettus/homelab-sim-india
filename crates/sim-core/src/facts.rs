//! Facts derived from the world: who is upstream of whom, and which address
//! each node ends up with. Addresses are handed down from the internet side
//! and iterated to a fixed point, so flipping a line from public to carrier
//! NAT changes the router's WAN address without anyone editing it.

use std::collections::{BTreeMap, VecDeque};
use std::net::IpAddr;

use serde::{Serialize, Serializer};

use crate::bundle::{Bundle, ClaimValue, ProfileKind};
use crate::ip::IpNet;
use crate::model::{Ipv6Mode, Isp, LineCheckResult, LineMode, Node, TunnelKind, World};
use crate::{Provenance, Tri};

/// An address that may be missing or not knowable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Addr {
    #[default]
    Absent,
    Unknown,
    Known(IpAddr),
}

impl Addr {
    pub fn known(self) -> Option<IpAddr> {
        match self {
            Addr::Known(ip) => Some(ip),
            _ => None,
        }
    }
}

impl Serialize for Addr {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Addr::Absent => serializer.serialize_none(),
            Addr::Unknown => serializer.serialize_str("unknown"),
            Addr::Known(ip) => serializer.collect_str(ip),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Public,
    Cgnat,
    Unknown,
}

/// Line types to assume for ISPs whose real type is unknown.
pub(crate) type Assumptions = BTreeMap<String, Mode>;

const DEFAULT_CGNAT_POOL: &str = "100.64.0.0/10";
const ISP_FIRST_CUSTOMER: u32 = 50;
const CGNAT_FIRST_CUSTOMER: u32 = 34;
const LAN_FIRST_DHCP: u32 = 20;

pub(crate) struct Net<'a> {
    pub world: &'a World,
    pub idx: BTreeMap<&'a str, usize>,
    /// Per node: (link index, neighbour index), in link-id order.
    pub adj: Vec<Vec<(usize, usize)>>,
    /// The neighbour one step closer to the internet.
    pub up: Vec<Option<(usize, usize)>>,
    pub dist: Vec<Option<u32>>,
    /// Nodes whose upstream is this node, in id order.
    pub customers: Vec<Vec<usize>>,
    /// The node's address on its upstream side. A router's WAN address.
    pub up4: Vec<Addr>,
    pub up6: Vec<Addr>,
    /// A router's delegated IPv6 LAN.
    pub lan6: Vec<Option<IpNet>>,
    /// An ISP's shared public address when it runs carrier NAT.
    pub shared4: Vec<Addr>,
    pub mode: Vec<Option<(Mode, Provenance)>>,
    pub settled: bool,
}

impl<'a> Net<'a> {
    pub fn build(world: &'a World, bundle: &Bundle, assume: &Assumptions) -> Self {
        let n = world.nodes.len();
        let mut idx = BTreeMap::new();
        for (i, node) in world.nodes.iter().enumerate() {
            idx.entry(node.id()).or_insert(i);
        }

        let mut link_order: Vec<usize> = (0..world.links.len()).collect();
        link_order.sort_by(|a, b| world.links[*a].id.cmp(&world.links[*b].id).then(a.cmp(b)));
        let mut adj = vec![Vec::new(); n];
        for l in link_order {
            let link = &world.links[l];
            if let (Some(&a), Some(&b)) = (idx.get(link.a.as_str()), idx.get(link.b.as_str()))
                && a != b
            {
                adj[a].push((l, b));
                adj[b].push((l, a));
            }
        }

        // Distance from the internet, breadth first, in a stable order.
        let mut dist: Vec<Option<u32>> = vec![None; n];
        let mut queue = VecDeque::new();
        for &i in idx.values() {
            if matches!(world.nodes[i], Node::Internet(_)) {
                dist[i] = Some(0);
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            let next = dist[i].unwrap_or(0) + 1;
            for &(_, nb) in &adj[i] {
                if dist[nb].is_none() {
                    dist[nb] = Some(next);
                    queue.push_back(nb);
                }
            }
        }

        let mut up = vec![None; n];
        let mut customers = vec![Vec::new(); n];
        for &i in idx.values() {
            let Some(d) = dist[i].filter(|d| *d > 0) else {
                continue;
            };
            up[i] = adj[i].iter().copied().find(|&(_, nb)| dist[nb] == Some(d - 1));
            if let Some((_, parent)) = up[i] {
                customers[parent].push(i);
            }
        }

        let mode = world
            .nodes
            .iter()
            .map(|node| match node {
                Node::Isp(isp) => Some(line_mode(isp, world, bundle, assume)),
                _ => None,
            })
            .collect();

        let mut net = Net {
            world,
            idx,
            adj,
            up,
            dist,
            customers,
            up4: vec![Addr::Absent; n],
            up6: vec![Addr::Absent; n],
            lan6: vec![None; n],
            shared4: vec![Addr::Absent; n],
            mode,
            settled: false,
        };
        net.assign_addresses();
        net
    }

    /// Hand addresses down from upstream until nothing changes. The bound is
    /// generous: each pass settles at least one more level of the tree.
    fn assign_addresses(&mut self) {
        let order: Vec<usize> = self.idx.values().copied().collect();
        for _ in 0..order.len() + 2 {
            let mut changed = false;
            for &i in &order {
                let (up4, up6, lan6, shared4) = self.derive(i);
                if (up4, up6, lan6, shared4) != (self.up4[i], self.up6[i], self.lan6[i], self.shared4[i]) {
                    self.up4[i] = up4;
                    self.up6[i] = up6;
                    self.lan6[i] = lan6;
                    self.shared4[i] = shared4;
                    changed = true;
                }
            }
            if !changed {
                self.settled = true;
                return;
            }
        }
    }

    fn derive(&self, i: usize) -> (Addr, Addr, Option<IpNet>, Addr) {
        let node = &self.world.nodes[i];
        let shared4 = match (node, self.mode[i]) {
            (Node::Isp(isp), Some((Mode::Cgnat, _))) => Addr::Known(isp.public_pool.host(ISP_FIRST_CUSTOMER)),
            _ => Addr::Absent,
        };
        let (static4, static6) = match node {
            Node::Host(host) => (host.static_ipv4.map(IpAddr::V4), host.static_ipv6.map(IpAddr::V6)),
            _ => (None, None),
        };
        if matches!(node, Node::Isp(_) | Node::Internet(_)) {
            return (Addr::Absent, Addr::Absent, None, shared4);
        }

        let mut up4 = Addr::Absent;
        let mut up6 = Addr::Absent;
        let mut lan6 = None;
        if let Some((_, parent)) = self.up[i] {
            let position = self.customers[parent].iter().position(|c| *c == i).unwrap_or(0);
            let position = u32::try_from(position).unwrap_or(u32::MAX);
            match &self.world.nodes[parent] {
                Node::Isp(isp) => {
                    up4 = match self.mode[parent].map(|(mode, _)| mode) {
                        Some(Mode::Public) => Addr::Known(isp.public_pool.host(ISP_FIRST_CUSTOMER + position)),
                        Some(Mode::Cgnat) => {
                            let pool = isp.cgnat_pool.unwrap_or_else(|| DEFAULT_CGNAT_POOL.parse().expect("valid"));
                            Addr::Known(pool.host(CGNAT_FIRST_CUSTOMER + position))
                        }
                        _ => Addr::Unknown,
                    };
                    match (isp.ipv6, isp.ipv6_prefix.and_then(|p| p.subnet64(position + 1))) {
                        (Ipv6Mode::Yes, Some(subnet)) => {
                            if matches!(node, Node::Router(_)) {
                                up6 = Addr::Known(subnet.host(1));
                                lan6 = Some(subnet);
                            } else {
                                up6 = Addr::Known(subnet.host(2));
                            }
                        }
                        (Ipv6Mode::Unknown, _) => up6 = Addr::Unknown,
                        _ => {}
                    }
                }
                Node::Router(router) => {
                    // Hosts with a fixed address do not take a DHCP slot.
                    let slot = self.customers[parent]
                        .iter()
                        .filter(|c| !matches!(&self.world.nodes[**c], Node::Host(h) if h.static_ipv4.is_some()))
                        .position(|c| *c == i)
                        .unwrap_or(0);
                    up4 = Addr::Known(router.lan.host(LAN_FIRST_DHCP + u32::try_from(slot).unwrap_or(0)));
                    up6 = match (self.lan6[parent], self.up6[parent]) {
                        (Some(lan), _) => Addr::Known(lan.host(0x20 + position)),
                        (None, Addr::Unknown) => Addr::Unknown,
                        _ => Addr::Absent,
                    };
                }
                Node::Host(_) | Node::Internet(_) => {}
            }
        }
        if let Some(ip) = static4 {
            up4 = Addr::Known(ip);
        }
        if let Some(ip) = static6 {
            up6 = Addr::Known(ip);
        }
        (up4, up6, lan6, shared4)
    }

    pub fn id(&self, i: usize) -> &'a str {
        self.world.nodes[i].id()
    }

    pub fn index(&self, id: &str) -> Option<usize> {
        self.idx.get(id).copied()
    }

    /// The router's own LAN address, the gateway its hosts use.
    pub fn gateway4(&self, i: usize) -> Option<IpAddr> {
        match &self.world.nodes[i] {
            Node::Router(router) => Some(router.lan.host(1)),
            _ => None,
        }
    }

    /// The IPv4 address the internet sees for traffic leaving this node.
    pub fn public4(&self, i: usize) -> Addr {
        if self.dist[i].is_none() {
            return Addr::Absent;
        }
        let mut addr = self.up4[i];
        let mut cur = i;
        while let Some((_, parent)) = self.up[cur] {
            match &self.world.nodes[parent] {
                Node::Router(_) => addr = self.up4[parent],
                Node::Isp(_) => match self.mode[parent].map(|(mode, _)| mode) {
                    Some(Mode::Cgnat) => addr = self.shared4[parent],
                    Some(Mode::Public) => {}
                    _ => addr = Addr::Unknown,
                },
                Node::Host(_) | Node::Internet(_) => {}
            }
            cur = parent;
        }
        addr
    }

    /// The nodes between this one and the internet, nearest first.
    pub fn upstream_chain(&self, i: usize) -> Vec<(usize, usize)> {
        let mut chain = Vec::new();
        let mut cur = i;
        while let Some((link, parent)) = self.up[cur] {
            chain.push((link, parent));
            cur = parent;
        }
        chain
    }

    pub fn behind_carrier_nat(&self, i: usize) -> Tri {
        let mut result = Tri::No;
        for (_, parent) in self.upstream_chain(i) {
            match self.mode[parent].map(|(mode, _)| mode) {
                Some(Mode::Cgnat) => return Tri::Yes,
                Some(Mode::Unknown) => result = Tri::Unknown,
                _ => {}
            }
        }
        result
    }

    /// How many address translators sit between this node and the internet.
    pub fn nat_layers(&self, i: usize) -> u32 {
        let mut layers = 0;
        for (_, parent) in self.upstream_chain(i) {
            match (&self.world.nodes[parent], self.mode[parent].map(|(mode, _)| mode)) {
                (Node::Router(_), _) | (Node::Isp(_), Some(Mode::Cgnat)) => layers += 1,
                _ => {}
            }
        }
        layers
    }

    /// Hosts that have a tunnel entry for this overlay, joined or not, in id
    /// order. Positions are stable when a host joins or leaves.
    pub fn overlay_seats(&self, network: &str) -> Vec<usize> {
        self.idx
            .values()
            .copied()
            .filter(|&i| match &self.world.nodes[i] {
                Node::Host(host) => {
                    host.tunnels.iter().any(|t| t.kind == TunnelKind::Overlay && t.network.as_deref() == Some(network))
                }
                _ => false,
            })
            .collect()
    }

    pub fn overlay_address(&self, node: usize, network: &str) -> Option<IpAddr> {
        let overlay = self.world.overlays.iter().find(|o| o.id == network)?;
        let seat = self.overlay_seats(network).iter().position(|i| *i == node)?;
        Some(overlay.range.host(10 + u32::try_from(seat).ok()?))
    }
}

fn line_mode(isp: &Isp, world: &World, bundle: &Bundle, assume: &Assumptions) -> (Mode, Provenance) {
    // "Can't find it" is an answer too. It does not fall back to our data.
    let checked = world.line_checks.iter().find(|check| check.isp == isp.id).map(|check| {
        let mode = match check.result {
            LineCheckResult::Public => Mode::Public,
            LineCheckResult::Cgnat => Mode::Cgnat,
            LineCheckResult::Unknown => Mode::Unknown,
        };
        (mode, Provenance::LineCheck)
    });
    let settled = match isp.ipv4 {
        LineMode::Public => (Mode::Public, Provenance::Teaching),
        LineMode::Cgnat => (Mode::Cgnat, Provenance::Teaching),
        // The learner's own answer beats our data.
        LineMode::Unknown => checked.unwrap_or((Mode::Unknown, Provenance::Teaching)),
        LineMode::Profile => checked.unwrap_or_else(|| {
            let profile = isp.profile.as_ref().and_then(|id| bundle.isps.get(id));
            let provenance = match profile.map(|p| p.kind) {
                Some(ProfileKind::Real) => Provenance::Community,
                _ => Provenance::Teaching,
            };
            // "Likely" and "varies" are not facts about this learner's line.
            let mode = match profile.and_then(|p| p.claims.get("ipv4_cgnat")).map(|claim| claim.value) {
                Some(ClaimValue::Yes) => Mode::Cgnat,
                Some(ClaimValue::No) => Mode::Public,
                _ => Mode::Unknown,
            };
            (mode, provenance)
        }),
    };
    match (settled.0, assume.get(&isp.id)) {
        (Mode::Unknown, Some(&assumed)) => (assumed, settled.1),
        _ => settled,
    }
}

// ------------------------------------------------------------ report facts

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Facts {
    pub nodes: Vec<NodeFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeFacts {
    pub id: String,
    pub kind: &'static str,
    /// Address on the node's own network. For a router, its LAN address.
    pub ipv4: Addr,
    /// A router's address on its uplink.
    pub wan_ipv4: Addr,
    /// What the internet sees for traffic from this node.
    pub public_ipv4: Addr,
    pub ipv6: Addr,
    pub behind_carrier_nat: Tri,
    pub nat_layers: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<LineFacts>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tunnels: Vec<TunnelFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LineFacts {
    pub ipv4: Mode,
    pub provenance: Provenance,
    pub shared_ipv4: Addr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TunnelFacts {
    pub id: String,
    pub enabled: bool,
    pub address: Option<String>,
}

pub(crate) fn facts(net: &Net) -> Facts {
    let nodes = net
        .idx
        .values()
        .map(|&i| {
            let node = &net.world.nodes[i];
            let router = matches!(node, Node::Router(_));
            let tunnels = match node {
                Node::Host(host) => host
                    .tunnels
                    .iter()
                    .map(|tunnel| TunnelFacts {
                        id: tunnel.id.clone(),
                        enabled: tunnel.enabled,
                        address: match (tunnel.kind, &tunnel.network) {
                            (TunnelKind::Overlay, Some(network)) => {
                                net.overlay_address(i, network).map(|ip| ip.to_string())
                            }
                            _ => tunnel.address.map(|a| a.addr.to_string()),
                        },
                    })
                    .collect(),
                _ => Vec::new(),
            };
            NodeFacts {
                id: node.id().to_owned(),
                kind: node.kind(),
                ipv4: if router { net.gateway4(i).map_or(Addr::Absent, Addr::Known) } else { net.up4[i] },
                wan_ipv4: if router { net.up4[i] } else { Addr::Absent },
                public_ipv4: net.public4(i),
                ipv6: net.up6[i],
                behind_carrier_nat: net.behind_carrier_nat(i),
                nat_layers: net.nat_layers(i),
                line: net.mode[i].map(|(mode, provenance)| LineFacts {
                    ipv4: mode,
                    provenance,
                    shared_ipv4: net.shared4[i],
                }),
                tunnels,
            }
        })
        .collect();
    Facts { nodes }
}
