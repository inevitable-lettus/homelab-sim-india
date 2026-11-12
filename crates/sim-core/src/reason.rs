//! Every reason the core can give for a step, a drop, a finding or a goal.
//! Each id is also the id of an explanation in `content/`, and the data build
//! fails if one is missing its text.

use serde::{Serialize, Serializer};

macro_rules! reasons {
    ($($variant:ident => $id:literal,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Reason {
            $($variant,)*
        }

        impl Reason {
            pub const ALL: &'static [Reason] = &[$(Reason::$variant,)*];

            pub const fn id(self) -> &'static str {
                match self {
                    $(Reason::$variant => $id,)*
                }
            }
        }
    };
}

reasons! {
    // Ordinary steps.
    Sent => "packet-sent",
    Forwarded => "packet-forwarded",
    Delivered => "delivered",
    ReplyDelivered => "reply-delivered",
    // NAT.
    NatSourceRewrite => "nat-source-rewrite",
    NatMappingHit => "nat-mapping-hit",
    CgnatMappingCreated => "cgnat-mapping-created",
    CgnatMappingHit => "cgnat-mapping-hit",
    CgnatSourceRewrite => "cgnat-source-rewrite",
    CgnatNoMapping => "cgnat-no-inbound-mapping",
    MappingExpired => "nat-mapping-expired",
    NatTimeoutUnknown => "nat-timeout-unknown",
    PortForwardHit => "port-forward-hit",
    RouterNoForward => "router-no-port-forward",
    ForwardTargetMissing => "forward-target-missing",
    // Routing and delivery.
    PrivateNotRoutable => "private-address-not-routable",
    NoRoute => "no-route-to-address",
    NoHostAtAddress => "no-host-at-address",
    NoService => "no-service-on-port",
    ServiceLoopbackOnly => "service-loopback-only",
    HostFirewall => "host-firewall-deny",
    NotConnected => "not-connected",
    HostNoAddress => "host-no-address",
    RoutingLoop => "routing-loop",
    PacketTooBig => "packet-too-big-for-link",
    DnsNotFound => "dns-name-not-found",
    // IPv6.
    ClientNoIpv6 => "client-no-ipv6",
    Ipv6NotAvailable => "ipv6-not-available",
    Ipv6NoNat => "ipv6-no-nat",
    Ipv6Firewall => "router-ipv6-firewall",
    // Tunnels and overlays.
    TunnelDialsOut => "tunnel-dials-out",
    TunnelSessionUp => "tunnel-session-up",
    TunnelEncapsulate => "tunnel-encapsulate",
    TunnelDecapsulate => "tunnel-decapsulate",
    RelayForwards => "relay-forwards",
    RelayPeerOffline => "relay-peer-offline",
    HubForwards => "hub-forwards",
    HostNotForwarding => "host-not-forwarding",
    TunnelNoEndpoint => "tunnel-peer-no-endpoint",
    TunnelHandshakeFailed => "tunnel-handshake-failed",
    TunnelUnknownPeer => "tunnel-unknown-peer",
    TunnelWrongPeerKey => "tunnel-wrong-peer-key",
    TunnelAllowedIps => "tunnel-allowed-ips",
    TunnelMtuExceeded => "tunnel-mtu-exceeded",
    OverlayNotJoined => "overlay-peer-not-joined",
    // Unknowns.
    LineTypeUnknown => "line-type-unknown",
    UnknownNode => "unknown-node",
    // Rules.
    LineHasPublicIpv4 => "line-has-public-ipv4",
    WanAddressIsCarrierNat => "wan-address-is-carrier-nat",
    PortForwardUsable => "port-forward-usable",
    ForwardBehindCgnat => "forward-behind-cgnat",
    ForwardNoService => "forward-no-service",
    ServiceExposed => "service-exposed-to-internet",
    ServicePortConflict => "service-port-conflict",
    HostVirtualization => "host-virtualization",
    DeviceDataMissing => "device-data-missing",
    TunnelKeepalive => "tunnel-keepalive",
    TunnelKeepaliveMissing => "tunnel-keepalive-missing",
    TunnelKeepaliveTooSlow => "tunnel-keepalive-too-slow",
    TunnelMtuFits => "tunnel-mtu-fits",
    TunnelMtuTooBig => "tunnel-mtu-too-big",
    EvaluationDidNotSettle => "evaluation-did-not-settle",
    WorldEmpty => "world-empty",
    BundleSchemaMismatch => "bundle-schema-mismatch",
    // Goals and answers.
    GoalMet => "goal-met",
    GoalNotMet => "goal-not-met",
    GoalTooSlow => "goal-too-slow",
    GoalWrongNode => "goal-wrong-node",
    GoalUnanswered => "goal-unanswered",
    GoalWrongAnswer => "goal-wrong-answer",
    GoalUnknown => "goal-unknown",
    // Graded files.
    ArtifactOk => "artifact-ok",
    ArtifactMissing => "artifact-missing",
    ArtifactInvalid => "artifact-invalid",
    ArtifactKeyMismatch => "artifact-key-mismatch",
    ArtifactKeyUnchecked => "artifact-key-unchecked",
    // Sandbox builds: will it run.
    OsBootsOnDevice => "os-boots-on-device",
    OsRequirementsMet => "os-requirements-met",
    HypervisorNeedsVirtualization => "hypervisor-needs-virtualization",
    GuestArchEmulated => "guest-arch-emulated",
    NestedVirtualization => "nested-virtualization",
    ServiceAvailable => "service-available",
    RuntimeOnOs => "runtime-on-os",
    ContainerHostNetwork => "container-host-network",
    TranscodeSupported => "transcode-supported",
    ServiceAutostart => "service-autostart",
    PaidFeatureNeeded => "paid-feature-needed",
    // Will it fit.
    RamBudget => "ram-budget",
    GuestAllocExceedsHost => "guest-alloc-exceeds-host",
    BayAvailable => "bay-available",
    UsbPortAvailable => "usb-port-available",
    DrivePower => "drive-power",
    EnclosureUas => "enclosure-uas",
    NoEthernetPort => "no-ethernet-port",
    EthernetPortsAvailable => "ethernet-ports-available",
    WifiOnlyServer => "wifi-only-server",
    WifiAvailable => "wifi-available",
    StorageCapacity => "storage-capacity",
    SingleCopy => "single-copy",
    OsStorage => "os-storage",
    // Will it connect.
    RouterCanForward => "router-can-forward",
    HomeDnsPossible => "home-dns-possible",
    StaticLanIp => "static-lan-ip",
    StreamBandwidth => "stream-bandwidth",
    ClientHasApp => "client-has-app",
    IntentReach => "intent-reach",
    // Will it last.
    YearlyCost => "yearly-cost",
    OutageCovered => "outage-covered",
    UpsOverloaded => "ups-overloaded",
    RecoversAfterOutage => "recovers-after-outage",
    LaptopAsUps => "laptop-as-ups",
    BatteryAlwaysCharging => "battery-always-charging",
    LidClosedRunning => "lid-closed-running",
    FanlessSustainedLoad => "fanless-sustained-load",
    InverterWaveform => "inverter-waveform",
    // Build bookkeeping.
    SwitchForwards => "switch-forwards",
    IntentNotChecked => "intent-not-checked",
    LearnerValueUsed => "learner-value-used",
    // Assumptions shown next to the numbers they affect.
    AssumedBusyShare => "assumed-busy-share",
    AssumedBaseUse => "assumed-base-use",
    AssumedUpsUsable => "assumed-ups-usable",
    AssumedLaptopUsable => "assumed-laptop-usable",
    AssumedIspStaysUp => "assumed-isp-stays-up",
    AssumedDriveLoadIgnored => "assumed-drive-load-ignored",
}

impl Serialize for Reason {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn ids_are_unique_and_kebab_case() {
        let ids: BTreeSet<&str> = Reason::ALL.iter().map(|r| r.id()).collect();
        assert_eq!(ids.len(), Reason::ALL.len());
        for id in ids {
            assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        }
    }
}
