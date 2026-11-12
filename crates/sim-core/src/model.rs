//! The world a learner edits. The UI sends this to the core as JSON, and
//! missions store their starting world in the same shape.
//!
//! The world is one graph: nodes joined by links. Layers such as "network" and
//! "services" are a curriculum idea only and are not modelled here.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

use crate::ip::IpNet;
use crate::physics::Medium;

pub type Id = String;

fn yes() -> bool {
    true
}

fn default_packet_bytes() -> u32 {
    1_000
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlays: Vec<Overlay>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dns: Vec<DnsRecord>,
    /// Connections the learner wants to make. Each one is traced.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<Attempt>,
    /// The learner's answers to a mission's questions. The core checks them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub answers: Vec<Answer>,
    /// Results of the learner's own line check, for lines the data cannot settle.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_checks: Vec<LineCheck>,
}

/// A world is described by its own types. In a pack schema it is an object.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for World {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "World".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "object" })
    }
}

impl World {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id() == id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|node| node.id() == id)
    }

    pub fn host_mut(&mut self, id: &str) -> Option<&mut Host> {
        match self.node_mut(id) {
            Some(Node::Host(host)) => Some(host),
            _ => None,
        }
    }

    pub fn router_mut(&mut self, id: &str) -> Option<&mut Router> {
        match self.node_mut(id) {
            Some(Node::Router(router)) => Some(router),
            _ => None,
        }
    }

    pub fn isp_mut(&mut self, id: &str) -> Option<&mut Isp> {
        match self.node_mut(id) {
            Some(Node::Isp(isp)) => Some(isp),
            _ => None,
        }
    }

    pub fn attempt_mut(&mut self, id: &str) -> Option<&mut Attempt> {
        self.attempts.iter_mut().find(|attempt| attempt.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Node {
    Host(Host),
    Router(Router),
    Isp(Isp),
    Internet(Internet),
}

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Node::Host(n) => &n.id,
            Node::Router(n) => &n.id,
            Node::Isp(n) => &n.id,
            Node::Internet(n) => &n.id,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Node::Host(n) => &n.label,
            Node::Router(n) => &n.label,
            Node::Isp(n) => &n.label,
            Node::Internet(n) => &n.label,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Node::Host(_) => "host",
            Node::Router(_) => "router",
            Node::Isp(_) => "isp",
            Node::Internet(_) => "internet",
        }
    }
}

/// Anything with an address that starts or ends a connection: a server, a
/// phone, a rented VPS, a virtual machine.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub id: Id,
    pub label: String,
    /// A pack id from `data/archetypes` or `data/devices`. Rules read its
    /// capabilities, never this id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<Id>,
    /// Picture hint for the UI. No rule reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Fixed address. Needed for hosts attached straight to the internet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub static_ipv4: Option<Ipv4Addr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub static_ipv6: Option<Ipv6Addr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<Service>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tunnels: Vec<Tunnel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firewall: Option<HostFirewall>,
    /// The machine this host is a guest of, for VMs and containers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs_on: Option<Id>,
    /// Whether the host passes packets between tunnel peers (`ip_forward`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub forwarding: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Proto {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bind {
    /// Listens on every address (`0.0.0.0`).
    #[default]
    All,
    /// Listens on `127.0.0.1` only, so nothing else can reach it.
    Loopback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub id: Id,
    pub label: String,
    pub proto: Proto,
    pub port: u16,
    #[serde(default)]
    pub bind: Bind,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    #[default]
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostFirewall {
    #[serde(default)]
    pub inbound: Policy,
    /// Exceptions to `inbound`, first match wins.
    #[serde(default)]
    pub rules: Vec<FirewallRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirewallRule {
    pub proto: Proto,
    pub port: u16,
    pub action: Policy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelKind {
    #[default]
    Wireguard,
    /// A managed mesh VPN in the style of Tailscale. The core fills in
    /// addresses and peers from the `Overlay` it names.
    Overlay,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tunnel {
    pub id: Id,
    #[serde(default)]
    pub kind: TunnelKind,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// For `kind: overlay`, the overlay this host has joined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<Id>,
    /// Address inside the tunnel, with its prefix: `10.8.0.2/24`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<IpNet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Teaching shortcut: name the peer by node when keys are not part of the lesson.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<Endpoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_ips: Vec<IpNet>,
    /// `PersistentKeepalive`, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keepalive_s: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    /// An IP address or a DNS name.
    pub host: String,
    pub port: u16,
}

/// A home router: one LAN, NAT towards its uplink, optional port forwards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Router {
    pub id: Id,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<Id>,
    /// The LAN, for example `192.168.1.0/24`. The router takes the first address.
    pub lan: IpNet,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub port_forwards: Vec<PortForward>,
    /// How long an idle UDP mapping lives. Missing means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub udp_timeout_s: Option<u32>,
    /// Unasked-for inbound IPv6. Home routers deny it by default.
    #[serde(default = "deny")]
    pub ipv6_inbound: Policy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ipv6_pinholes: Vec<Pinhole>,
}

fn deny() -> Policy {
    Policy::Deny
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortForward {
    pub id: Id,
    #[serde(default = "yes")]
    pub enabled: bool,
    pub proto: Proto,
    pub external_port: u16,
    pub to_ip: Ipv4Addr,
    pub internal_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pinhole {
    pub to: Id,
    pub proto: Proto,
    pub port: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineMode {
    /// The customer gets a public IPv4 address.
    Public,
    /// The customer sits behind the provider's carrier-grade NAT.
    Cgnat,
    /// Not known. Results that depend on it come back as `unknown`.
    #[default]
    Unknown,
    /// Take it from the ISP profile's `ipv4_cgnat` claim.
    Profile,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ipv6Mode {
    Yes,
    #[default]
    No,
    Unknown,
}

/// The access provider between a home, or a phone, and the internet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Isp {
    pub id: Id,
    pub label: String,
    /// An id from `data/isps`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Id>,
    #[serde(default)]
    pub ipv4: LineMode,
    #[serde(default)]
    pub ipv6: Ipv6Mode,
    /// Public addresses this provider owns.
    pub public_pool: IpNet,
    /// Addresses handed to customers behind carrier NAT.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cgnat_pool: Option<IpNet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipv6_prefix: Option<IpNet>,
    /// How long an idle UDP mapping lives in the carrier NAT. Missing means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub udp_timeout_s: Option<u32>,
}

/// The public internet, as one node that routes by who owns an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Internet {
    pub id: Id,
    pub label: String,
}

/// A cable, a radio link or a long-distance route between two nodes.
/// Missing numbers are unknown: timing that depends on them is not reported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub id: Id,
    pub a: Id,
    pub b: Id,
    #[serde(default)]
    pub medium: Medium,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_m: Option<u64>,
    /// Bit rate from `a` to `b`, and back. Home lines are rarely symmetric.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_ab_bps: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_ba_bps: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    /// How busy the link already is, in thousandths.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub load_permille: u16,
    /// Fixed delay the medium adds on top of distance and bit rate, such as
    /// radio scheduling on a mobile network.
    #[serde(default, skip_serializing_if = "is_zero_ns")]
    pub extra_ns: u64,
}

fn is_zero(n: &u16) -> bool {
    *n == 0
}

fn is_zero_ns(n: &u64) -> bool {
    *n == 0
}

/// A managed mesh VPN. Members reach each other through its relay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    pub id: Id,
    pub label: String,
    /// A host with a public address that relays between members.
    pub relay: Id,
    pub range: IpNet,
    #[serde(default = "overlay_port")]
    pub port: u16,
}

fn overlay_port() -> u16 {
    41_641
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnsRecord {
    pub name: String,
    pub target: Target,
}

/// Where a connection is aimed. Most variants name an address by what it is,
/// so a mission does not hard-code numbers the core derives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Address {
        ip: IpAddr,
    },
    Name {
        name: String,
    },
    /// The node's own address on its local network.
    LanOf {
        node: Id,
    },
    /// A router's address on its uplink side.
    WanOf {
        node: Id,
    },
    /// The address the internet sees for traffic from this node.
    PublicOf {
        node: Id,
    },
    Ipv6Of {
        node: Id,
    },
    /// The node's address inside an overlay it has joined.
    OverlayOf {
        node: Id,
        network: Id,
    },
    /// The node's address inside one of its tunnels.
    TunnelOf {
        node: Id,
        tunnel: Id,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Data flows from the side that opened the connection (an upload).
    #[default]
    Request,
    /// Data flows back to it (a download or a stream).
    Reply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transfer {
    pub bytes: u64,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub id: Id,
    pub label: String,
    pub from: Id,
    pub to: Target,
    pub proto: Proto,
    pub port: u16,
    /// Seconds of silence before this attempt. Idle NAT mappings can expire.
    #[serde(default)]
    pub idle_s: u32,
    #[serde(default = "default_packet_bytes")]
    pub packet_bytes: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer: Option<Transfer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    pub question: Id,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineCheckResult {
    Public,
    Cgnat,
    /// "Can't find it" is a valid answer.
    Unknown,
}

/// The outcome of a learner's own line check. Never an address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineCheck {
    pub isp: Id,
    pub result: LineCheckResult,
}
