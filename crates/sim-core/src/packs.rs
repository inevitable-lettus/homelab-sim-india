//! Schema v4 packs: devices of every category, operating systems, container
//! runtimes, services, tariffs, outage scenarios, reference tables and
//! sandbox templates. See context-docs/Sandbox.md section 4.
//!
//! Every leaf a rule may read is a `Cap`, so "none", "unknown" and "we have a
//! source for this" stay three different things.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::Provenance;
use crate::bundle::{Layout, ProfileKind};
use crate::model::{Id, Proto, World};

/// A plain enum of lower-case words, as written in the YAML.
macro_rules! words {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($(#[$vmeta])* $variant),* }
    };
}

/// A struct of pack fields: unknown keys are an error, missing ones default.
macro_rules! section {
    ($(#[$meta:meta])* $name:ident { $($(#[$fmeta:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(deny_unknown_fields)]
        pub struct $name { $($(#[$fmeta])* #[serde(default)] pub $field: $ty),* }

        impl Audit for $name {
            fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
                $(self.$field.audit(&join(path, stringify!($field)), out);)*
            }
        }
    };
}

// ------------------------------------------------------------------- Cap

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum UnknownMark {
    Unknown,
}

words! {
    /// How good the source behind a value is.
    #[derive(Default)]
    Basis {
        /// The maker's spec page, datasheet, manual or official documentation.
        #[default]
        Official,
        /// An independent measurement published by a named reviewer.
        Review,
        /// A forum, wiki, issue tracker or user report.
        Community,
    }
}

impl Basis {
    fn is_official(&self) -> bool {
        *self == Basis::Official
    }
}

/// One capability value. It is a plain value in a teaching pack, a value with
/// its source in a real one, or the word `unknown`.
///
/// Read by hand rather than as a derived untagged enum: that keeps the error
/// messages about the value itself, and keeps the WebAssembly small.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Cap<T> {
    Sourced {
        value: T,
        source: String,
        retrieved_at: String,
        #[serde(default, skip_serializing_if = "Basis::is_official")]
        basis: Basis,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    Unknown(UnknownMark),
    Value(T),
}

struct CapVisitor<T>(PhantomData<T>);

/// Hands a map back to `T` after its first key has been looked at.
struct Replay<A> {
    first: Option<String>,
    map: A,
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for Replay<A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, A::Error> {
        match self.first.take() {
            Some(key) => seed.deserialize(key.into_deserializer()).map(Some),
            None => self.map.next_key_seed(seed),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, A::Error> {
        self.map.next_value_seed(seed)
    }
}

const SOURCED_KEYS: &[&str] = &["value", "source", "retrieved_at", "basis", "note"];

impl<'de, T: Deserialize<'de>> Visitor<'de> for CapVisitor<T> {
    type Value = Cap<T>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a value, the word `unknown`, or `{ value, source, retrieved_at }`")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Cap<T>, E> {
        T::deserialize(v.into_deserializer()).map(Cap::Value)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Cap<T>, E> {
        T::deserialize(v.into_deserializer()).map(Cap::Value)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Cap<T>, E> {
        T::deserialize(v.into_deserializer()).map(Cap::Value)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Cap<T>, E> {
        if v == "unknown" {
            return Ok(Cap::Unknown(UnknownMark::Unknown));
        }
        T::deserialize(v.into_deserializer()).map(Cap::Value)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Cap<T>, A::Error> {
        T::deserialize(SeqAccessDeserializer::new(seq)).map(Cap::Value)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Cap<T>, A::Error> {
        let mut key = map.next_key::<String>()?;
        if !key.as_deref().is_some_and(|key| SOURCED_KEYS.contains(&key)) {
            // Not a sourced value: the map is the value itself.
            return T::deserialize(MapAccessDeserializer::new(Replay { first: key, map })).map(Cap::Value);
        }
        let (mut value, mut source, mut retrieved_at, mut basis, mut note) = (None, None, None, None, None);
        while let Some(name) = key {
            match name.as_str() {
                "value" => value = Some(map.next_value::<T>()?),
                "source" => source = Some(map.next_value::<String>()?),
                "retrieved_at" => retrieved_at = Some(map.next_value::<String>()?),
                "basis" => basis = Some(map.next_value::<Basis>()?),
                "note" => note = map.next_value::<Option<String>>()?,
                other => return Err(de::Error::unknown_field(other, SOURCED_KEYS)),
            }
            key = map.next_key::<String>()?;
        }
        Ok(Cap::Sourced {
            value: value.ok_or_else(|| de::Error::missing_field("value"))?,
            source: source.ok_or_else(|| de::Error::missing_field("source"))?,
            retrieved_at: retrieved_at.ok_or_else(|| de::Error::missing_field("retrieved_at"))?,
            basis: basis.unwrap_or_default(),
            note,
        })
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Cap<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(CapVisitor(PhantomData))
    }
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

    /// A bare value with no source. Fine in a teaching pack, not in a real one.
    pub fn is_unsourced_value(&self) -> bool {
        matches!(self, Cap::Value(_))
    }

    pub fn basis(&self) -> Option<Basis> {
        match self {
            Cap::Sourced { basis, .. } => Some(*basis),
            _ => None,
        }
    }
}

impl<T: Copy> Cap<T> {
    pub fn copied(&self) -> Option<T> {
        self.get().copied()
    }
}

/// Where a pack's values come from, for wording a verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Teaching,
    Real,
    Learner,
}

/// The weaker of two provenances: the one that promises less.
pub fn weaker(a: Provenance, b: Provenance) -> Provenance {
    fn rank(p: Provenance) -> u8 {
        match p {
            Provenance::Official => 0,
            Provenance::Community | Provenance::LineCheck => 1,
            Provenance::Teaching => 2,
            Provenance::Learner => 3,
        }
    }
    if rank(b) > rank(a) { b } else { a }
}

impl Origin {
    /// The provenance of one value read from a pack of this origin.
    pub fn provenance<T>(self, cap: &Cap<T>) -> Provenance {
        match (self, cap.basis()) {
            (Origin::Learner, _) => Provenance::Learner,
            (Origin::Teaching, _) => Provenance::Teaching,
            (Origin::Real, Some(Basis::Review | Basis::Community)) => Provenance::Community,
            (Origin::Real, _) => Provenance::Official,
        }
    }
}

// ----------------------------------------------------------------- audit

/// One `Cap` found in a pack, for the data build's sourcing check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leaf {
    pub path: String,
    pub state: LeafState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafState {
    Sourced(Basis),
    Unknown,
    /// A value with no source of its own and none above it.
    Bare,
}

fn join(path: &str, field: &str) -> String {
    if path.is_empty() { field.to_owned() } else { format!("{path}.{field}") }
}

/// Lists every `Cap` under a value, with its path.
pub trait Audit {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>);
}

impl<T: Audit> Audit for Cap<T> {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
        let state = match self {
            Cap::Sourced { basis, .. } => LeafState::Sourced(*basis),
            Cap::Unknown(_) => LeafState::Unknown,
            Cap::Value(_) => LeafState::Bare,
        };
        out.push(Leaf { path: path.to_owned(), state });
        if let Some(value) = self.get() {
            let mut inner = Vec::new();
            value.audit(path, &mut inner);
            // A value inside a sourced value is covered by that source.
            if let LeafState::Sourced(basis) = state {
                for leaf in &mut inner {
                    if leaf.state == LeafState::Bare {
                        leaf.state = LeafState::Sourced(basis);
                    }
                }
            }
            out.append(&mut inner);
        }
    }
}

impl<T: Audit> Audit for Option<T> {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
        if let Some(value) = self {
            value.audit(path, out);
        }
    }
}

impl<T: Audit> Audit for Vec<T> {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
        for (i, value) in self.iter().enumerate() {
            value.audit(&format!("{path}[{i}]"), out);
        }
    }
}

impl<K: std::fmt::Debug, T: Audit> Audit for BTreeMap<K, T> {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
        for (key, value) in self {
            let key = format!("{key:?}").trim_matches('"').to_lowercase();
            value.audit(&join(path, &key), out);
        }
    }
}

macro_rules! no_caps {
    ($($ty:ty),* $(,)?) => { $(impl Audit for $ty { fn audit(&self, _: &str, _: &mut Vec<Leaf>) {} })* };
}

no_caps!(bool, u8, u16, u32, u64, String, Proto);

// ----------------------------------------------------------------- words

words! { PackKind {
    /// A generic teaching machine. Not a product.
    Archetype,
    /// A real product. Every value needs a source.
    Device,
    /// Entered by the learner in a build. Never read from a file.
    Learner,
} }

words! { Category {
    Computer, Router, Switch, AccessPoint, Ups, Inverter, DcUps, Drive, Enclosure, UsbNic, UsbHub, Client,
} }

words! { Form { Laptop, Desktop, MiniPc, Sbc, Phone, Tablet, Nas, Rack, Other } }
words! { Availability { SoldNew, UsedCommon, ImportOnly, Discontinued } }
words! { Arch { X86_64, Aarch64, Armv7, Armv6, Riscv64, Ppc64le, S390x } }
words! { Firmware { Uefi, Bios, UefiAndBios, AppleSilicon, Uboot, AndroidBootloader } }
words! { Cooling { Fanless, Fan } }
words! { AccelApi { Vaapi, Qsv, Nvenc, Amf, Videotoolbox, V4l2m2m, Rkmpp } }
words! { Codec {
    H264,
    #[serde(rename = "hevc_8")]
    Hevc8,
    #[serde(rename = "hevc_10")]
    Hevc10,
    Av1,
    Vp9,
} }
words! { WifiGen { None, Wifi4, Wifi5, Wifi6, Wifi6e, Wifi7 } }
words! { UsbKind { Usb2A, Usb5A, Usb10A, Usb5C, Usb10C, Usb40C } }
words! { BayKind {
    #[serde(rename = "sata_2_5")]
    Sata25,
    #[serde(rename = "sata_3_5")]
    Sata35,
    M2Nvme,
    M2Sata,
    Msata,
    SdCard,
} }
words! { WanKind { GponFibre, EponFibre, Ethernet, Dsl, Cellular } }
words! { AdminAccess { Full, Limited, IspOnly } }
words! { UpsTopology { Offline, LineInteractive, Online, Inverter, Dc } }
words! { Waveform { Sine, SteppedSine, Square } }
words! { DriveMedium { Hdd, Ssd } }
words! { Support { Full, Partial, None } }
words! { OsKind { Os, Hypervisor, NasOs, Appliance } }
words! { OsFamily { Linux, Macos, Windows, Android, Bsd } }
words! { Cost { Free, FreePersonal, Paid } }
words! { HostNetwork {
    /// `network_mode: host` reaches the real LAN.
    Yes,
    /// It reaches the hidden VM or localhost only, not the LAN.
    Limited,
    No,
} }
words! { ServiceCategory {
    Media, Photos, Files, Dns, Automation, Passwords, Sync, Backup, Monitoring, Proxy, Vpn, Tunnel, Games,
    Downloads, Dev,
} }
words! { InstallMethod { Native, Container, Lxc, Addon, Package } }
words! { Provides {
    DnsServer, DhcpServer, MediaServer, PhotoBackup, FileShare, VpnEndpoint, TunnelConnector, ReverseProxy, MeshVpn,
} }
words! { IntentKind { Run, Reach, Stream, HomeDns, Store, SurviveOutage, YearlyCost } }
words! {
    #[derive(Default)]
    PortDirection {
        #[default]
        Inbound,
        Outbound,
    }
}
words! {
    #[derive(Default)]
    IspSide {
        #[default]
        Up,
        Down,
        Unknown,
    }
}

/// `host_network` was a yes or no before it could say "limited".
impl From<bool> for HostNetwork {
    fn from(yes: bool) -> Self {
        if yes { HostNetwork::Yes } else { HostNetwork::No }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum HostNetworkValue {
    Word(HostNetwork),
    Flag(bool),
}

impl HostNetworkValue {
    pub fn get(self) -> HostNetwork {
        match self {
            HostNetworkValue::Word(word) => word,
            HostNetworkValue::Flag(yes) => yes.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PowerInput {
    #[serde(rename = "ac_230")]
    Ac230,
    Dc {
        volts_mv: u32,
        max_ma: u32,
    },
    UsbCPd {
        watts: u32,
    },
    #[serde(rename = "usb_5v")]
    Usb5v {
        max_ma: u32,
    },
}

no_caps!(
    Basis,
    Form,
    Availability,
    Arch,
    Firmware,
    Cooling,
    AccelApi,
    Codec,
    WifiGen,
    UsbKind,
    BayKind,
    WanKind,
    AdminAccess,
    UpsTopology,
    Waveform,
    DriveMedium,
    Support,
    OsKind,
    OsFamily,
    Cost,
    HostNetworkValue,
    ServiceCategory,
    InstallMethod,
    Provides,
    IntentKind,
    PortDirection,
    IspSide,
    PowerInput,
    PackKind,
    Category,
    ProfileKind,
);

// --------------------------------------------------------------- devices

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct DevicePack {
    pub id: Id,
    pub kind: PackKind,
    pub category: Category,
    pub label: String,
    /// Search aliases and model numbers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aka: Vec<String>,
    /// Year of release.
    #[serde(default)]
    pub released: Cap<u16>,
    #[serde(default)]
    pub form: Cap<Form>,
    /// A label for the parts list. Never part of a verdict.
    #[serde(default)]
    pub india: Cap<Availability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute: Option<Compute>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetCaps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Ports>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageCaps>,
    #[serde(default)]
    pub power: Power,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery: Option<BatteryCaps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub router: Option<RouterCaps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ups: Option<UpsCaps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drive: Option<DriveCaps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<AdapterCaps>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<ClientCaps>,
    /// Keyed by OS pack id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub os_support: BTreeMap<Id, Cap<OsSupport>>,
    /// Shown in the inspector. No rule reads them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl DevicePack {
    pub fn origin(&self) -> Origin {
        match self.kind {
            PackKind::Archetype => Origin::Teaching,
            PackKind::Device => Origin::Real,
            PackKind::Learner => Origin::Learner,
        }
    }

    /// The sections present, by name, for the category allow-list.
    pub fn sections(&self) -> Vec<&'static str> {
        let mut found = Vec::new();
        let mut note = |present: bool, name: &'static str| {
            if present {
                found.push(name);
            }
        };
        note(self.compute.is_some(), "compute");
        note(self.network.is_some(), "network");
        note(self.ports.is_some(), "ports");
        note(self.storage.is_some(), "storage");
        note(self.battery.is_some(), "battery");
        note(self.router.is_some(), "router");
        note(self.ups.is_some(), "ups");
        note(self.drive.is_some(), "drive");
        note(self.adapter.is_some(), "adapter");
        note(self.client.is_some(), "client");
        note(!self.os_support.is_empty(), "os_support");
        found
    }
}

impl Category {
    /// Sections a pack of this category may carry, besides `power`.
    pub fn allowed_sections(self) -> &'static [&'static str] {
        match self {
            Category::Computer => &["compute", "network", "ports", "storage", "battery", "os_support"],
            Category::Router => &["network", "ports", "router"],
            Category::Switch | Category::AccessPoint => &["network"],
            Category::Ups | Category::Inverter | Category::DcUps => &["ups"],
            Category::Drive => &["drive"],
            Category::Enclosure | Category::UsbNic | Category::UsbHub => &["adapter", "os_support"],
            Category::Client => &["network", "client"],
        }
    }
}

impl Audit for DevicePack {
    fn audit(&self, _: &str, out: &mut Vec<Leaf>) {
        self.released.audit("released", out);
        self.form.audit("form", out);
        self.india.audit("india", out);
        self.compute.audit("compute", out);
        self.network.audit("network", out);
        self.ports.audit("ports", out);
        self.storage.audit("storage", out);
        self.power.audit("power", out);
        self.battery.audit("battery", out);
        self.router.audit("router", out);
        self.ups.audit("ups", out);
        self.drive.audit("drive", out);
        self.adapter.audit("adapter", out);
        self.client.audit("client", out);
        self.os_support.audit("os_support", out);
    }
}

section! { Compute {
    arch: Cap<Arch>,
    cores: Cap<u32>,
    threads: Cap<u32>,
    /// As sold, or as configured in this pack.
    ram_gb: Cap<u32>,
    ram_max_gb: Cap<u32>,
    ram_soldered: Cap<bool>,
    /// Hardware virtualization a hypervisor can use.
    virtualization: Cap<bool>,
    nested_virtualization: Cap<bool>,
    firmware: Cap<Firmware>,
    /// The hardware media engine.
    video: Cap<VideoCaps>,
    cooling: Cap<Cooling>,
    /// Firmware can power the machine on when mains returns.
    ac_power_recovery: Cap<bool>,
    wake_on_lan: Cap<bool>,
} }

section! { VideoCaps {
    apis: Vec<AccelApi>,
    decode: BTreeMap<Codec, bool>,
    encode: BTreeMap<Codec, bool>,
} }

section! { NetCaps {
    /// Wired ports. `value: []` means none, which is not the same as unknown.
    nic: Cap<Vec<Nic>>,
    wifi: Cap<WifiGen>,
} }

section! { Nic { speed_mbps: Cap<u32> } }

section! { Ports { usb: Cap<Vec<UsbPort>> } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct UsbPort {
    pub kind: UsbKind,
    pub count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Bay {
    pub interface: BayKind,
    pub count: u32,
}

section! { StorageCaps {
    /// Internal slots a learner can fill.
    bays: Cap<Vec<Bay>>,
    /// Built-in storage that cannot be changed.
    soldered_gb: Cap<u32>,
    boot_from_usb: Cap<bool>,
} }

section! { Power {
    idle_w: Cap<u32>,
    load_w: Cap<u32>,
    input: Cap<PowerInput>,
} }

section! { BatteryCaps {
    wh: Cap<u32>,
    /// A charge limit set in firmware, independent of the OS.
    firmware_charge_limit: Cap<bool>,
} }

section! { RouterCaps {
    lan: Cap<Vec<Nic>>,
    wan: Cap<WanKind>,
    admin: Cap<AdminAccess>,
    port_forwarding: Cap<bool>,
    upnp: Cap<bool>,
    dmz: Cap<bool>,
    bridge_mode: Cap<bool>,
    /// The DHCP server can hand out a custom DNS server.
    lan_dns_setting: Cap<bool>,
    dhcp_can_disable: Cap<bool>,
    dhcp_reservation: Cap<bool>,
    ipv6: Cap<bool>,
    ipv6_firewall_configurable: Cap<bool>,
    udp_timeout_s: Cap<u32>,
    /// ISP profile id, for ISP-supplied units. A label, never read by rules.
    supplied_by: Option<Id>,
} }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct DcOutput {
    pub volts_mv: u32,
    pub max_ma: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connector: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RuntimePoint {
    pub load_w: u32,
    pub minutes: u32,
}

section! { UpsCaps {
    topology: Cap<UpsTopology>,
    va: Cap<u32>,
    max_w: Cap<u32>,
    battery_wh: Cap<u32>,
    /// Inverters: the battery is a separate purchase.
    battery_external: Cap<bool>,
    outlets: Cap<u32>,
    dc_outputs: Cap<Vec<DcOutput>>,
    transfer_ms: Cap<u32>,
    waveform: Cap<Waveform>,
    /// The maker's runtime chart.
    runtime: Cap<Vec<RuntimePoint>>,
} }

section! { DriveCaps {
    interface: Cap<BayKind>,
    medium: Cap<DriveMedium>,
    capacity_gb: Cap<u32>,
    /// 3.5 inch drives cannot run from USB bus power.
    needs_12v: Cap<bool>,
    rated_24x7: Cap<bool>,
} }

section! { AdapterProvides {
    nic: Cap<Vec<Nic>>,
    bays: Cap<Vec<Bay>>,
    usb: Cap<Vec<UsbPort>>,
} }

section! { AdapterCaps {
    /// What it plugs into.
    host_side: Cap<UsbKind>,
    provides: AdapterProvides,
    uas: Cap<bool>,
    /// An external power brick is required.
    needs_power: Cap<bool>,
} }

section! { ClientCaps {
    /// What it decodes without help.
    plays: BTreeMap<Codec, bool>,
    /// Service ids with a client app on it.
    apps: Vec<Id>,
    ipv6: Cap<bool>,
} }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct OsSupport {
    pub level: Support,
    /// Keeps running with the lid shut.
    #[serde(default)]
    pub lid_closed: Cap<Support>,
    /// The OS can cap battery charge.
    #[serde(default)]
    pub charge_limit: Cap<bool>,
    /// Short phrases: "no Thunderbolt".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
}

impl Audit for OsSupport {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
        self.lid_closed.audit(&join(path, "lid_closed"), out);
        self.charge_limit.audit(&join(path, "charge_limit"), out);
    }
}

no_caps!(UsbPort, Bay, DcOutput, RuntimePoint);

// ---------------------------------------------- OS, runtimes and services

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub url: String,
    pub retrieved_at: String,
    #[serde(default, skip_serializing_if = "Basis::is_official")]
    pub basis: Basis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

section! { OsRequires {
    virtualization: Cap<bool>,
    ram_min_mb: Cap<u32>,
    storage_min_gb: Cap<u32>,
    wired_nic: Cap<bool>,
} }

section! { OsProvides {
    vms: Cap<bool>,
    lxc: Cap<bool>,
    /// Guests of other architectures are emulated, or do not run.
    vm_accelerated_arches: Vec<Arch>,
    vm_emulated_arches: Vec<Arch>,
    /// Runtime pack ids available without a VM.
    native_runtimes: Vec<Id>,
} }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct OsPack {
    pub id: Id,
    pub label: String,
    pub kind: OsKind,
    /// A generic teaching system. Says nothing about any real product.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub teaching: bool,
    pub family: OsFamily,
    /// What it can boot on, when the device says nothing.
    #[serde(default)]
    pub arches: Vec<Arch>,
    #[serde(default)]
    pub firmware: Vec<Firmware>,
    /// A device must name this OS in `os_support`.
    #[serde(default)]
    pub listed_devices_only: bool,
    #[serde(default)]
    pub requires: OsRequires,
    /// Idle footprint used in the RAM budget.
    #[serde(default)]
    pub base_ram_mb: Cap<u32>,
    #[serde(default)]
    pub provides: OsProvides,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Audit for OsPack {
    fn audit(&self, _: &str, out: &mut Vec<Leaf>) {
        self.requires.audit("requires", out);
        self.base_ram_mb.audit("base_ram_mb", out);
        self.provides.audit("provides", out);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RuntimePack {
    pub id: Id,
    pub label: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub teaching: bool,
    pub os_families: Vec<OsFamily>,
    /// Runs containers inside a hidden Linux VM.
    #[serde(default)]
    pub via_vm: Cap<bool>,
    /// RAM that VM takes before any container.
    #[serde(default)]
    pub vm_overhead_mb: Cap<u32>,
    #[serde(default)]
    pub host_network: Cap<HostNetworkValue>,
    /// GPU APIs a container can use.
    #[serde(default)]
    pub device_passthrough: Cap<Vec<AccelApi>>,
    #[serde(default)]
    pub cost: Cap<Cost>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Audit for RuntimePack {
    fn audit(&self, _: &str, out: &mut Vec<Leaf>) {
        self.via_vm.audit("via_vm", out);
        self.vm_overhead_mb.audit("vm_overhead_mb", out);
        self.host_network.audit("host_network", out);
        self.device_passthrough.audit("device_passthrough", out);
        self.cost.audit("cost", out);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Install {
    pub method: InstallMethod,
    pub os_families: Vec<OsFamily>,
    #[serde(default)]
    pub arches: Cap<Vec<Arch>>,
    #[serde(default)]
    pub runtime_needed: bool,
    /// Shown to the learner. No rule reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_min_version: Option<String>,
}

impl Audit for Install {
    fn audit(&self, path: &str, out: &mut Vec<Leaf>) {
        self.arches.audit(&join(path, "arches"), out);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct DefaultPort {
    pub proto: Proto,
    pub port: u16,
    pub purpose: String,
    /// Only needed for an optional feature.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    #[serde(default)]
    pub direction: PortDirection,
}

section! { ServiceNeeds {
    /// Features that need host networking: "device discovery", "DHCP server".
    host_network_for: Cap<Vec<String>>,
    static_lan_ip: Cap<bool>,
    always_on: Cap<bool>,
    transcode_apis: BTreeMap<OsFamily, Cap<Vec<AccelApi>>>,
} }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct PaidFeature {
    pub feature: String,
    #[serde(default)]
    pub needed_for: Vec<IntentKind>,
    pub source: String,
    pub retrieved_at: String,
}

no_caps!(DefaultPort, PaidFeature, Source);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ServicePack {
    pub id: Id,
    pub label: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub teaching: bool,
    pub category: ServiceCategory,
    #[serde(default)]
    pub install: Vec<Install>,
    #[serde(default)]
    pub ram_min_mb: Cap<u32>,
    #[serde(default)]
    pub ram_rec_mb: Cap<u32>,
    #[serde(default)]
    pub storage_min_gb: Cap<u32>,
    /// Become `Service` entries on the host.
    #[serde(default)]
    pub ports: Vec<DefaultPort>,
    #[serde(default)]
    pub needs: ServiceNeeds,
    #[serde(default)]
    pub provides: Vec<Provides>,
    #[serde(default)]
    pub paid_features: Cap<Vec<PaidFeature>>,
    #[serde(default)]
    pub licence: Cap<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Audit for ServicePack {
    fn audit(&self, _: &str, out: &mut Vec<Leaf>) {
        self.install.audit("install", out);
        self.ram_min_mb.audit("ram_min_mb", out);
        self.ram_rec_mb.audit("ram_rec_mb", out);
        self.storage_min_gb.audit("storage_min_gb", out);
        self.needs.audit("needs", out);
        self.paid_features.audit("paid_features", out);
        self.licence.audit("licence", out);
    }
}

// ----------------------------------------------------------------- power

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Slab {
    /// Missing on the last slab, which covers the rest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upto_kwh: Option<u32>,
    pub paise_per_kwh: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FreeUnits {
    pub upto_kwh: u32,
    /// The learner says whether it applies.
    pub condition: String,
}

/// One DISCOM's tariff for one category of customer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Tariff {
    pub id: Id,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub teaching: bool,
    pub state: String,
    pub discom: String,
    pub category: String,
    pub effective_from: String,
    pub slabs: Vec<Slab>,
    /// Shown, not added to the homelab's marginal cost.
    #[serde(default)]
    pub fixed_paise_per_month: Cap<u32>,
    /// A fixed charge per kW of sanctioned load. Shown, not added.
    #[serde(default)]
    pub fixed_paise_per_kw_month: Cap<u32>,
    /// Electricity duty as a share of energy charges.
    #[serde(default)]
    pub duty_permille: Cap<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub free_units: Option<FreeUnits>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub retrieved_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Audit for Tariff {
    fn audit(&self, _: &str, out: &mut Vec<Leaf>) {
        self.fixed_paise_per_month.audit("fixed_paise_per_month", out);
        self.fixed_paise_per_kw_month.audit("fixed_paise_per_kw_month", out);
        self.duty_permille.audit("duty_permille", out);
    }
}

/// A power cut to plan for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Outage {
    pub id: Id,
    pub label: String,
    pub kind: ProfileKind,
    pub duration_min: u32,
    #[serde(default = "one")]
    pub per_day: u32,
    /// Whether the provider's side stays up. `up` is an assumption, and says so.
    #[serde(default)]
    pub isp_side: IspSide,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved_at: Option<String>,
}

fn one() -> u32 {
    1
}

/// A typical bitrate for one media format. Typical, not exact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct MediaFormat {
    pub label: String,
    pub codec: Codec,
    #[serde(default)]
    pub kbps: Cap<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub teaching: bool,
}

// ------------------------------------------------------------- templates

/// A starting build for the sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub id: Id,
    pub title: String,
    pub summary: String,
    pub world: World,
    pub layout: Layout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_list_is_not_unknown() {
        let none: Cap<Vec<Nic>> = serde_json::from_str("[]").unwrap();
        let unknown: Cap<Vec<Nic>> = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(none.get().map(Vec::len), Some(0));
        assert_eq!(unknown.get(), None);
    }

    #[test]
    fn a_sourced_value_keeps_its_basis() {
        let text =
            r#"{"value": 8, "source": "https://example.org", "retrieved_at": "2026-11-12", "basis": "community"}"#;
        let cap: Cap<u32> = serde_json::from_str(text).unwrap();
        assert_eq!(cap.copied(), Some(8));
        assert_eq!(cap.basis(), Some(Basis::Community));
        assert_eq!(Origin::Real.provenance(&cap), Provenance::Community);
        assert_eq!(Origin::Teaching.provenance(&cap), Provenance::Teaching);
        let plain: Cap<u32> = serde_json::from_str(r#"{"value": 8, "source": "s", "retrieved_at": "d"}"#).unwrap();
        assert_eq!(Origin::Real.provenance(&plain), Provenance::Official);
    }

    #[test]
    fn the_audit_finds_bare_values_and_lets_a_source_cover_what_is_inside_it() {
        let text = r#"{
            "nic": {"value": [{"speed_mbps": 1000}], "source": "s", "retrieved_at": "d"},
            "wifi": "wifi5"
        }"#;
        let net: NetCaps = serde_json::from_str(text).unwrap();
        let mut leaves = Vec::new();
        net.audit("network", &mut leaves);
        let state = |path: &str| leaves.iter().find(|leaf| leaf.path == path).map(|leaf| leaf.state);
        assert_eq!(state("network.nic"), Some(LeafState::Sourced(Basis::Official)));
        assert_eq!(state("network.nic[0].speed_mbps"), Some(LeafState::Sourced(Basis::Official)));
        assert_eq!(state("network.wifi"), Some(LeafState::Bare));
    }

    #[test]
    fn words_are_written_as_in_the_spec() {
        let json = |value: &dyn erased::Json| value.json();
        assert_eq!(json(&Arch::X86_64), "\"x86_64\"");
        assert_eq!(json(&Codec::Hevc10), "\"hevc_10\"");
        assert_eq!(json(&UsbKind::Usb40C), "\"usb40_c\"");
        assert_eq!(json(&BayKind::Sata25), "\"sata_2_5\"");
        assert_eq!(json(&BayKind::M2Nvme), "\"m2_nvme\"");
        assert_eq!(json(&PowerInput::Ac230), "{\"kind\":\"ac_230\"}");
        assert_eq!(json(&PowerInput::UsbCPd { watts: 30 }), "{\"kind\":\"usb_c_pd\",\"watts\":30}");
        assert_eq!(json(&Firmware::UefiAndBios), "\"uefi_and_bios\"");
    }

    mod erased {
        pub trait Json {
            fn json(&self) -> String;
        }
        impl<T: serde::Serialize> Json for T {
            fn json(&self) -> String {
                serde_json::to_string(self).unwrap()
            }
        }
    }
}
