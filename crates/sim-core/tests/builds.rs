//! Sandbox builds: every rule in Sandbox.md 7.1 to 7.3 with a pass, a fail
//! and an unknown, the goals they add up to, and determinism.
//!
//! The packs here are teaching packs made for the tests. Rules read their
//! capabilities, so nothing below depends on a product.

use serde_json::{Value, json};
use sim_core::{Bundle, Reason, Report, Severity, Status, World};

fn bundle() -> Bundle {
    let computer = |id: &str, extra: Value| {
        let mut pack = json!({ "id": id, "kind": "archetype", "category": "computer", "label": id });
        pack.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        pack
    };
    let laptop = json!({
        "form": "laptop",
        "compute": { "arch": "x86_64", "ram_gb": 8, "virtualization": true, "nested_virtualization": false,
            "firmware": "uefi", "cooling": "fan", "ac_power_recovery": true,
            "video": { "apis": ["vaapi", "qsv"], "encode": { "h264": true } } },
        "network": { "nic": [{ "speed_mbps": 1000 }], "wifi": "wifi5" },
        "ports": { "usb": [{ "kind": "usb5_a", "count": 2 }] },
        "storage": { "bays": [{ "interface": "sata_2_5", "count": 1 }], "soldered_gb": 0, "boot_from_usb": true },
        "power": { "idle_w": 12, "load_w": 45 },
        "battery": { "wh": 45 }
    });
    let mut small = laptop.clone();
    small["compute"]["ram_gb"] = json!(2);
    let devices = vec![
        computer("laptop", laptop),
        computer("small", small),
        computer(
            "air",
            json!({
                "form": "laptop",
                "compute": { "arch": "aarch64", "ram_gb": 8, "virtualization": true, "nested_virtualization": true,
                    "firmware": "apple_silicon", "cooling": "fanless" },
                "network": { "nic": [], "wifi": "wifi6" },
                "ports": { "usb": [{ "kind": "usb40_c", "count": 2 }] },
                "storage": { "bays": [], "soldered_gb": 256 },
                "power": { "idle_w": 4, "load_w": 30 },
                "battery": { "wh": 50 },
                "os_support": {
                    "macos": { "level": "full", "lid_closed": "full", "charge_limit": true },
                    "asahi": { "level": "partial", "gaps": ["no Thunderbolt"] },
                    "hyper": { "level": "none" }
                }
            }),
        ),
        computer(
            "sbc",
            json!({
                "form": "sbc",
                "compute": { "arch": "aarch64", "ram_gb": 1, "virtualization": false, "firmware": "uboot",
                    "video": { "apis": [] } },
                "network": { "nic": [{ "speed_mbps": 1000 }], "wifi": "none" },
                "ports": { "usb": [{ "kind": "usb2_a", "count": 2 }] },
                "storage": { "bays": [{ "interface": "sd_card", "count": 1 }], "soldered_gb": 0 },
                "power": { "idle_w": 3, "load_w": 7 }
            }),
        ),
        computer("mystery", json!({})),
        json!({ "id": "good-router", "kind": "archetype", "category": "router", "label": "Router you control",
            "network": { "wifi": "wifi6" },
            "router": { "lan": [{}, {}, {}, {}], "admin": "full", "port_forwarding": true, "lan_dns_setting": true,
                "dhcp_can_disable": true, "dhcp_reservation": true } }),
        json!({ "id": "isp-router", "kind": "archetype", "category": "router", "label": "Locked ISP router",
            "network": { "wifi": "wifi5" },
            "router": { "lan": [{}, {}, {}, {}], "admin": "isp_only", "port_forwarding": false,
                "lan_dns_setting": false, "dhcp_can_disable": false, "dhcp_reservation": false } }),
        json!({ "id": "tiny-router", "kind": "archetype", "category": "router", "label": "One-port router",
            "network": { "wifi": "wifi5" }, "router": { "lan": [{}], "admin": "full", "port_forwarding": true } }),
        json!({ "id": "blank-router", "kind": "archetype", "category": "router", "label": "Router we know nothing about" }),
        json!({ "id": "switch5", "kind": "archetype", "category": "switch", "label": "Five-port switch",
            "network": { "nic": [{}, {}, {}, {}, {}] } }),
        json!({ "id": "usb-nic", "kind": "archetype", "category": "usb_nic", "label": "USB Ethernet adapter",
            "adapter": { "host_side": "usb5_a", "provides": { "nic": [{ "speed_mbps": 1000 }] } } }),
        json!({ "id": "hub", "kind": "archetype", "category": "usb_hub", "label": "USB hub",
            "adapter": { "host_side": "usb5_a", "provides": { "usb": [{ "kind": "usb5_a", "count": 4 }] } } }),
        json!({ "id": "enclosure-25", "kind": "archetype", "category": "enclosure", "label": "2.5 inch enclosure",
            "adapter": { "host_side": "usb5_a", "provides": { "bays": [{ "interface": "sata_2_5", "count": 1 }] },
                "uas": true, "needs_power": false } }),
        json!({ "id": "enclosure-35", "kind": "archetype", "category": "enclosure", "label": "3.5 inch enclosure",
            "adapter": { "host_side": "usb5_a", "provides": { "bays": [{ "interface": "sata_3_5", "count": 1 }] },
                "uas": false, "needs_power": true } }),
        json!({ "id": "enclosure-fast", "kind": "archetype", "category": "enclosure", "label": "Fast enclosure",
            "adapter": { "host_side": "usb10_c", "provides": { "bays": [{ "interface": "m2_nvme", "count": 1 }] } } }),
        json!({ "id": "enclosure-blank", "kind": "archetype", "category": "enclosure", "label": "Unknown enclosure" }),
        json!({ "id": "hdd-35", "kind": "archetype", "category": "drive", "label": "3.5 inch hard drive",
            "drive": { "interface": "sata_3_5", "medium": "hdd", "capacity_gb": 4000, "needs_12v": true } }),
        json!({ "id": "ssd-25", "kind": "archetype", "category": "drive", "label": "2.5 inch SSD",
            "drive": { "interface": "sata_2_5", "medium": "ssd", "capacity_gb": 500, "needs_12v": false } }),
        json!({ "id": "sd-32", "kind": "archetype", "category": "drive", "label": "Memory card",
            "drive": { "interface": "sd_card", "medium": "ssd", "capacity_gb": 32, "needs_12v": false } }),
        json!({ "id": "drive-blank", "kind": "archetype", "category": "drive", "label": "Unknown drive" }),
        json!({ "id": "tv", "kind": "archetype", "category": "client", "label": "TV",
            "network": { "wifi": "wifi5" },
            "client": { "plays": { "h264": true, "hevc_10": false }, "apps": ["media"] } }),
        json!({ "id": "tv-noapp", "kind": "archetype", "category": "client", "label": "TV with no app",
            "network": { "wifi": "wifi5" }, "client": { "plays": { "h264": true } } }),
        json!({ "id": "tv-blank", "kind": "archetype", "category": "client", "label": "Unknown TV" }),
    ];
    let os = |id: &str, extra: Value| {
        let mut pack = json!({ "id": id, "label": id, "kind": "os", "teaching": true, "family": "linux" });
        pack.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        pack
    };
    let systems = vec![
        os(
            "linux",
            json!({ "arches": ["x86_64", "aarch64"], "firmware": ["uefi", "bios", "uboot"],
            "requires": { "virtualization": false, "ram_min_mb": 512, "storage_min_gb": 8, "wired_nic": false },
            "base_ram_mb": 300, "provides": { "vms": true, "lxc": true } }),
        ),
        os(
            "hyper",
            json!({ "kind": "hypervisor", "arches": ["x86_64"], "firmware": ["uefi", "bios"],
            "requires": { "virtualization": true, "ram_min_mb": 2048, "storage_min_gb": 32, "wired_nic": true },
            "base_ram_mb": 1024, "provides": { "vms": true, "lxc": true, "vm_emulated_arches": ["aarch64"] } }),
        ),
        os(
            "macos",
            json!({ "family": "macos", "arches": ["aarch64"], "firmware": ["apple_silicon"],
            "requires": { "virtualization": false, "ram_min_mb": 4096, "storage_min_gb": 64, "wired_nic": false },
            "base_ram_mb": 3000, "provides": { "vms": true, "lxc": false, "vm_accelerated_arches": ["aarch64"] } }),
        ),
        os(
            "asahi",
            json!({ "arches": ["aarch64"], "firmware": ["apple_silicon"], "listed_devices_only": true,
            "requires": { "virtualization": false, "ram_min_mb": 1024, "storage_min_gb": 16, "wired_nic": false },
            "base_ram_mb": 400, "provides": { "vms": true, "lxc": true } }),
        ),
        os("blank-os", json!({ "arches": ["x86_64", "aarch64"] })),
    ];
    let runtimes = vec![
        json!({ "id": "docker", "label": "Containers on Linux", "teaching": true, "os_families": ["linux"],
            "via_vm": false, "host_network": "yes", "device_passthrough": ["vaapi", "qsv"] }),
        json!({ "id": "desktop", "label": "Containers in a hidden VM", "teaching": true, "os_families": ["macos"],
            "via_vm": true, "vm_overhead_mb": 2048, "host_network": "limited", "device_passthrough": [] }),
        json!({ "id": "blank-rt", "label": "Unknown runtime", "teaching": true, "os_families": ["linux"] }),
    ];
    let both = json!(["x86_64", "aarch64"]);
    let services = vec![
        json!({ "id": "media", "label": "Media server", "teaching": true, "category": "media",
            "install": [
                { "method": "native", "os_families": ["linux"], "arches": both },
                { "method": "container", "os_families": ["linux"], "arches": both, "runtime_needed": true }
            ],
            "ram_min_mb": 1024, "ram_rec_mb": 2048, "storage_min_gb": 2,
            "ports": [{ "proto": "tcp", "port": 8096, "purpose": "web" }],
            "needs": { "host_network_for": [], "static_lan_ip": false, "transcode_apis": { "linux": ["vaapi", "qsv"] } },
            "provides": ["media_server"], "paid_features": [] }),
        json!({ "id": "dns", "label": "DNS ad-blocker", "teaching": true, "category": "dns",
            "install": [
                { "method": "native", "os_families": ["linux"], "arches": both },
                { "method": "container", "os_families": ["linux"], "arches": both, "runtime_needed": true }
            ],
            "ram_min_mb": 256, "needs": { "host_network_for": ["DHCP server"], "static_lan_ip": true },
            "provides": ["dns_server", "dhcp_server"] }),
        json!({ "id": "dns-only", "label": "DNS without DHCP", "teaching": true, "category": "dns",
            "install": [{ "method": "native", "os_families": ["linux"], "arches": both }],
            "ram_min_mb": 64, "needs": { "static_lan_ip": false }, "provides": ["dns_server"] }),
        json!({ "id": "x86-only", "label": "x86 only", "teaching": true, "category": "dev",
            "install": [{ "method": "native", "os_families": ["linux"], "arches": ["x86_64"] }], "ram_min_mb": 100 }),
        json!({ "id": "blank-svc", "label": "Unknown service", "teaching": true, "category": "dev",
            "install": [{ "method": "native", "os_families": ["linux"] }] }),
        json!({ "id": "paid", "label": "Paid remote access", "teaching": true, "category": "media",
            "install": [{ "method": "native", "os_families": ["linux"], "arches": both }], "ram_min_mb": 100,
            "paid_features": [{ "feature": "remote access", "needed_for": ["reach"],
                "source": "teaching", "retrieved_at": "2026-11-12" }] }),
    ];
    let by_id = |packs: Vec<Value>| -> Value {
        Value::Object(packs.into_iter().map(|p| (p["id"].as_str().unwrap().to_owned(), p)).collect())
    };
    let bundle = json!({
        "schema_version": sim_core::SCHEMA_VERSION,
        "devices": by_id(devices),
        "os": by_id(systems),
        "runtimes": by_id(runtimes),
        "services": by_id(services),
        "media": {
            "h264_1080p": { "label": "H.264 1080p", "codec": "h264", "kbps": 6000, "teaching": true },
            "hevc_2160p_10bit": { "label": "HEVC 4K", "codec": "hevc_10", "kbps": 14000, "teaching": true },
            "av1_1080p": { "label": "AV1 1080p", "codec": "av1", "kbps": "unknown", "teaching": true }
        }
    });
    serde_json::from_value(bundle).expect("the test bundle parses")
}

/// A laptop running Linux with a media server, wired to a router the learner
/// controls, on a line with a public address. A TV at home, a phone away.
fn base() -> Value {
    json!({
        "home": {},
        "nodes": [
            { "kind": "internet", "id": "internet", "label": "Internet" },
            { "kind": "isp", "id": "mobile", "label": "Mobile network", "ipv4": "cgnat",
              "public_pool": "198.51.100.0/24", "cgnat_pool": "100.72.5.0/24", "udp_timeout_s": 30 },
            { "kind": "host", "id": "phone", "label": "Phone away", "role": "client" },
            { "kind": "isp", "id": "isp", "label": "Home ISP", "ipv4": "public",
              "public_pool": "203.0.113.0/24", "udp_timeout_s": 60 },
            { "kind": "router", "id": "router", "label": "Router", "device": "good-router",
              "lan": "192.168.1.0/24", "udp_timeout_s": 120 },
            { "kind": "host", "id": "server", "label": "Server", "device": "laptop", "os": "linux",
              "services": [{ "id": "jellyfin", "label": "Media server", "proto": "tcp", "port": 8096, "app": "media" }] },
            { "kind": "host", "id": "tv", "label": "TV", "role": "client", "device": "tv" }
        ],
        "links": [
            { "id": "phone-mobile", "a": "phone", "b": "mobile", "medium": "cellular", "length_m": 2000,
              "rate_ab_bps": 8000000, "rate_ba_bps": 20000000 },
            { "id": "mobile-internet", "a": "mobile", "b": "internet", "medium": "backbone", "length_m": 840000,
              "rate_ab_bps": 10000000000u64, "rate_ba_bps": 10000000000u64 },
            { "id": "isp-internet", "a": "isp", "b": "internet", "medium": "backbone", "length_m": 120000,
              "rate_ab_bps": 10000000000u64, "rate_ba_bps": 10000000000u64 },
            { "id": "home-isp", "a": "router", "b": "isp", "medium": "fibre", "length_m": 8000,
              "rate_ab_bps": 50000000, "rate_ba_bps": 100000000 },
            { "id": "server-home", "a": "server", "b": "router", "medium": "ethernet", "length_m": 5,
              "rate_ab_bps": 1000000000, "rate_ba_bps": 1000000000 },
            { "id": "tv-home", "a": "tv", "b": "router", "medium": "wifi", "length_m": 5,
              "rate_ab_bps": 100000000, "rate_ba_bps": 100000000 }
        ]
    })
}

fn node<'a>(world: &'a mut Value, id: &str) -> &'a mut Value {
    world["nodes"].as_array_mut().unwrap().iter_mut().find(|n| n["id"] == id).unwrap_or_else(|| panic!("node {id}"))
}

fn push(world: &mut Value, key: &str, value: Value) {
    let world = world.as_object_mut().unwrap();
    world.entry(key).or_insert_with(|| json!([])).as_array_mut().unwrap().push(value);
}

fn evaluate(world: &Value) -> Report {
    let world: World = serde_json::from_value(world.clone()).expect("the test world parses");
    sim_core::evaluate(&world, &bundle())
}

/// What a rule said, as the sandbox words it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Said {
    Pass,
    Fail,
    Warn,
    Unknown,
    UnknownNote,
    Nothing,
}

fn said(report: &Report, rule: Reason) -> Vec<Said> {
    report
        .findings
        .iter()
        .filter(|f| f.rule_id == rule)
        .map(|f| match (f.status, f.severity) {
            (Status::Pass, _) => Said::Pass,
            (Status::Fail, Severity::Error) => Said::Fail,
            (Status::Fail, _) => Said::Warn,
            (Status::Unknown, Severity::Info) => Said::UnknownNote,
            (Status::Unknown, _) => Said::Unknown,
        })
        .collect()
}

fn intent(report: &Report, id: &str) -> Status {
    let build = report.build.as_ref().expect("a build report");
    build.intents.iter().find(|i| i.id == id).unwrap_or_else(|| panic!("intent {id}")).status
}

/// The findings that count against a goal, for a failing assertion to show.
fn against(report: &Report, id: &str) -> Vec<(&'static str, Status, Severity, Vec<String>)> {
    report
        .findings
        .iter()
        .filter(|f| f.intents.iter().any(|i| i == id) && f.status != Status::Pass)
        .map(|f| (f.rule_id.id(), f.status, f.severity, f.involved.clone()))
        .collect()
}

type Change = fn(&mut Value);

fn check(cases: &[(&str, Reason, Said, Change)]) {
    for (name, rule, want, change) in cases {
        let mut world = base();
        change(&mut world);
        let report = evaluate(&world);
        let got = said(&report, *rule);
        match want {
            Said::Nothing => assert!(got.is_empty(), "{name}: `{}` should say nothing, said {got:?}", rule.id()),
            want => assert!(got.contains(want), "{name}: `{}` should say {want:?}, said {got:?}", rule.id()),
        }
    }
}

fn server(world: &mut Value) -> &mut Value {
    node(world, "server")
}

fn add_vm(world: &mut Value, id: &str, on: &str, extra: Value) {
    let mut guest = json!({ "kind": "host", "id": id, "label": id, "runs_on": on, "guest": "vm" });
    guest.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    push(world, "nodes", guest);
}

fn parts(world: &mut Value, list: Value) {
    server(world)["parts"] = list;
}

fn stream(world: &mut Value, client: &str, format: &str, streams: u8) {
    push(
        world,
        "intents",
        json!({ "kind": "stream", "id": "watch", "client": client, "host": "server",
        "service": "jellyfin", "format": format, "streams": streams }),
    );
}

fn forward(world: &mut Value) {
    node(world, "router")["port_forwards"] = json!([{ "id": "f1", "proto": "tcp", "external_port": 8096,
        "to_ip": "192.168.1.20", "internal_port": 8096 }]);
}

fn reach(world: &mut Value) {
    push(
        world,
        "intents",
        json!({ "kind": "reach", "id": "remote", "from": "phone", "host": "server",
        "service": "jellyfin", "via": { "kind": "public_of", "node": "router" } }),
    );
}

fn dns_service(world: &mut Value, app: &str) {
    server(world)["services"] = json!([{ "id": "dns", "label": "DNS", "proto": "udp", "port": 53, "app": app }]);
}

fn home_dns(world: &mut Value) {
    dns_service(world, "dns");
    push(
        world,
        "intents",
        json!({ "kind": "home_dns", "id": "adblock", "host": "server", "service": "dns",
        "router": "router" }),
    );
}

#[test]
fn will_it_run() {
    use Reason::*;
    check(&[
        ("linux boots on an x86 laptop", OsBootsOnDevice, Said::Pass, |_| {}),
        ("an ARM-only system on x86", OsBootsOnDevice, Said::Fail, |w| server(w)["os"] = json!("macos")),
        ("the device says partial", OsBootsOnDevice, Said::Warn, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("asahi");
        }),
        ("the device says no", OsBootsOnDevice, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("hyper");
        }),
        ("architecture unknown", OsBootsOnDevice, Said::Unknown, |w| server(w)["device"] = json!("mystery")),
        ("listed devices only, and this one is silent", OsBootsOnDevice, Said::Unknown, |w| {
            server(w)["os"] = json!("asahi")
        }),
        ("no OS chosen", OsBootsOnDevice, Said::Unknown, |w| {
            server(w).as_object_mut().unwrap().remove("os");
        }),
        ("requirements met", OsRequirementsMet, Said::Pass, |_| {}),
        ("too little memory and no virtualization", OsRequirementsMet, Said::Fail, |w| {
            server(w)["device"] = json!("sbc");
            server(w)["os"] = json!("hyper");
        }),
        ("needs a wired port the machine lacks", OsRequirementsMet, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("hyper");
        }),
        ("machine values unknown", OsRequirementsMet, Said::Unknown, |w| server(w)["device"] = json!("mystery")),
        ("requirements not stated", OsRequirementsMet, Said::UnknownNote, |w| server(w)["os"] = json!("blank-os")),
        ("a VM on a hypervisor", HypervisorNeedsVirtualization, Said::Pass, |w| {
            server(w)["os"] = json!("hyper");
            add_vm(w, "vm", "server", json!({}));
        }),
        ("a VM with no hardware virtualization", HypervisorNeedsVirtualization, Said::Fail, |w| {
            server(w)["device"] = json!("sbc");
            add_vm(w, "vm", "server", json!({}));
        }),
        ("a VM where the system has none", HypervisorNeedsVirtualization, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            add_vm(w, "ct", "server", json!({ "guest": "lxc" }));
        }),
        ("an LXC needs no hardware help", HypervisorNeedsVirtualization, Said::Pass, |w| {
            server(w)["device"] = json!("sbc");
            add_vm(w, "ct", "server", json!({ "guest": "lxc" }));
        }),
        ("virtualization unknown", HypervisorNeedsVirtualization, Said::Unknown, |w| {
            server(w)["device"] = json!("mystery");
            add_vm(w, "vm", "server", json!({}));
        }),
        ("a container is not a VM", HypervisorNeedsVirtualization, Said::Nothing, |w| {
            server(w)["runtime"] = json!("docker");
            add_vm(w, "c", "server", json!({ "guest": "container" }));
        }),
        ("a guest for the same processor", GuestArchEmulated, Said::Pass, |w| {
            server(w)["os"] = json!("hyper");
            add_vm(w, "vm", "server", json!({ "os": "linux" }));
        }),
        ("a guest that has to be emulated", GuestArchEmulated, Said::Warn, |w| {
            server(w)["os"] = json!("hyper");
            add_vm(w, "vm", "server", json!({ "os": "macos" }));
        }),
        ("a guest that cannot run at all", GuestArchEmulated, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            add_vm(w, "vm", "server", json!({ "os": "hyper" }));
        }),
        ("host processor unknown", GuestArchEmulated, Said::Unknown, |w| {
            server(w)["device"] = json!("mystery");
            server(w)["os"] = json!("hyper");
            add_vm(w, "vm", "server", json!({ "os": "linux" }));
        }),
        ("nested, and the processor refuses", NestedVirtualization, Said::Fail, |w| {
            server(w)["os"] = json!("hyper");
            add_vm(w, "vm", "server", json!({ "os": "hyper" }));
            add_vm(w, "inner", "vm", json!({}));
        }),
        ("nested, and the processor allows", NestedVirtualization, Said::Pass, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            add_vm(w, "vm", "server", json!({ "os": "asahi" }));
            add_vm(w, "inner", "vm", json!({}));
        }),
        ("nested, unknown", NestedVirtualization, Said::Unknown, |w| {
            server(w)["device"] = json!("mystery");
            add_vm(w, "vm", "server", json!({}));
            add_vm(w, "inner", "vm", json!({}));
        }),
        ("one level is not nested", NestedVirtualization, Said::Nothing, |w| add_vm(w, "vm", "server", json!({}))),
        ("a native build exists", ServiceAvailable, Said::Pass, |_| {}),
        ("no build for this processor", ServiceAvailable, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("asahi");
            server(w)["services"][0]["app"] = json!("x86-only");
        }),
        ("no build for this system", ServiceAvailable, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
        }),
        ("architectures not stated", ServiceAvailable, Said::Unknown, |w| {
            server(w)["services"][0]["app"] = json!("blank-svc");
        }),
        ("a container with no runtime", ServiceAvailable, Said::Fail, |w| {
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("a container with a runtime", ServiceAvailable, Said::Pass, |w| {
            server(w)["runtime"] = json!("docker");
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("a Linux container on another system, through its VM", ServiceAvailable, Said::Pass, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            server(w)["runtime"] = json!("desktop");
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("runtime fits the system", RuntimeOnOs, Said::Pass, |w| server(w)["runtime"] = json!("docker")),
        ("runtime for another system", RuntimeOnOs, Said::Fail, |w| server(w)["runtime"] = json!("desktop")),
        ("runtime, system not chosen", RuntimeOnOs, Said::Unknown, |w| {
            server(w)["runtime"] = json!("docker");
            server(w).as_object_mut().unwrap().remove("os");
        }),
        ("a container guest with nothing to run it", RuntimeOnOs, Said::Fail, |w| {
            add_vm(w, "c", "server", json!({ "guest": "container" }));
        }),
        ("host networking works", ContainerHostNetwork, Said::Pass, |w| {
            server(w)["runtime"] = json!("docker");
            dns_service(w, "dns");
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("host networking stops at the hidden VM", ContainerHostNetwork, Said::Warn, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            server(w)["runtime"] = json!("desktop");
            dns_service(w, "dns");
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("host networking unknown", ContainerHostNetwork, Said::Unknown, |w| {
            server(w)["runtime"] = json!("blank-rt");
            dns_service(w, "dns");
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("nothing needs host networking", ContainerHostNetwork, Said::Nothing, |w| {
            server(w)["runtime"] = json!("docker");
            server(w)["services"][0]["install"] = json!("container");
        }),
        ("the TV plays it as it is", TranscodeSupported, Said::Pass, |w| stream(w, "tv", "h264_1080p", 1)),
        ("converted with hardware help", TranscodeSupported, Said::Pass, |w| stream(w, "tv", "hevc_2160p_10bit", 1)),
        ("converted on the processor alone", TranscodeSupported, Said::Warn, |w| {
            server(w)["device"] = json!("sbc");
            stream(w, "tv", "hevc_2160p_10bit", 1);
        }),
        ("media engine unknown", TranscodeSupported, Said::Unknown, |w| {
            server(w)["device"] = json!("mystery");
            stream(w, "tv", "hevc_2160p_10bit", 1);
        }),
        ("what the player plays is unknown", TranscodeSupported, Said::Unknown, |w| {
            node(w, "tv")["device"] = json!("tv-blank");
            stream(w, "tv", "h264_1080p", 1);
        }),
        ("a paid feature is noted, never failed", PaidFeatureNeeded, Said::Pass, |w| {
            server(w)["services"][0]["app"] = json!("paid");
            forward(w);
            reach(w);
        }),
    ]);
}

#[test]
fn will_it_fit() {
    use Reason::*;
    check(&[
        ("memory fits", RamBudget, Said::Pass, |_| {}),
        ("memory over at minimums", RamBudget, Said::Fail, |w| server(w)["device"] = json!("sbc")),
        ("memory over at recommended sizes", RamBudget, Said::Warn, |w| server(w)["device"] = json!("small")),
        ("memory size unknown", RamBudget, Said::Unknown, |w| server(w)["device"] = json!("mystery")),
        ("a service with no data", RamBudget, Said::Unknown, |w| {
            server(w)["services"][0]["app"] = json!("blank-svc");
        }),
        ("a hidden VM takes its share", RamBudget, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            server(w)["runtime"] = json!("desktop");
            add_vm(w, "vm", "server", json!({ "alloc": { "ram_mb": 4096 } }));
        }),
        ("guests fit", GuestAllocExceedsHost, Said::Pass, |w| {
            add_vm(w, "vm", "server", json!({ "alloc": { "ram_mb": 4096 } }));
        }),
        ("guests given more than there is", GuestAllocExceedsHost, Said::Fail, |w| {
            add_vm(w, "vm", "server", json!({ "alloc": { "ram_mb": 8000 } }));
        }),
        ("guests, machine unknown", GuestAllocExceedsHost, Said::Unknown, |w| {
            server(w)["device"] = json!("mystery");
            add_vm(w, "vm", "server", json!({ "alloc": { "ram_mb": 1024 } }));
        }),
        ("a drive in the one bay", BayAvailable, Said::Pass, |w| parts(w, json!([{ "id": "d1", "pack": "ssd-25" }]))),
        ("two drives, one bay", BayAvailable, Said::Fail, |w| {
            parts(w, json!([{ "id": "d1", "pack": "ssd-25" }, { "id": "d2", "pack": "ssd-25" }]));
        }),
        ("the wrong kind of bay", BayAvailable, Said::Fail, |w| parts(w, json!([{ "id": "d1", "pack": "hdd-35" }]))),
        ("drive interface unknown", BayAvailable, Said::Unknown, |w| {
            parts(w, json!([{ "id": "d1", "pack": "drive-blank" }]));
        }),
        ("a second drive in an enclosure", BayAvailable, Said::Pass, |w| {
            parts(
                w,
                json!([{ "id": "d1", "pack": "ssd-25" }, { "id": "e1", "pack": "enclosure-25" },
                { "id": "d2", "pack": "ssd-25", "into": "e1" }]),
            );
        }),
        ("no drives, nothing to say", BayAvailable, Said::Nothing, |_| {}),
        ("an adapter in a free port", UsbPortAvailable, Said::Pass, |w| {
            parts(w, json!([{ "id": "n1", "pack": "usb-nic" }]))
        }),
        ("three things, two ports", UsbPortAvailable, Said::Fail, |w| {
            parts(
                w,
                json!([{ "id": "n1", "pack": "usb-nic" }, { "id": "n2", "pack": "usb-nic" },
                { "id": "e1", "pack": "enclosure-25" }]),
            );
        }),
        ("a hub makes room", UsbPortAvailable, Said::Pass, |w| {
            parts(
                w,
                json!([{ "id": "h1", "pack": "hub" }, { "id": "n1", "pack": "usb-nic", "into": "h1" },
                { "id": "n2", "pack": "usb-nic", "into": "h1" }, { "id": "e1", "pack": "enclosure-25", "into": "h1" }]),
            );
        }),
        ("a fast enclosure on a slower port", UsbPortAvailable, Said::Warn, |w| {
            parts(w, json!([{ "id": "e1", "pack": "enclosure-fast" }]));
        }),
        ("ports unknown", UsbPortAvailable, Said::Unknown, |w| {
            server(w)["device"] = json!("mystery");
            parts(w, json!([{ "id": "n1", "pack": "usb-nic" }]));
        }),
        ("a big drive in a powered enclosure", DrivePower, Said::Pass, |w| {
            parts(w, json!([{ "id": "e1", "pack": "enclosure-35" }, { "id": "d1", "pack": "hdd-35", "into": "e1" }]));
        }),
        ("a big drive on USB power alone", DrivePower, Said::Fail, |w| {
            parts(w, json!([{ "id": "e1", "pack": "enclosure-25" }, { "id": "d1", "pack": "hdd-35", "into": "e1" }]));
        }),
        ("drive power unknown", DrivePower, Said::Unknown, |w| {
            parts(
                w,
                json!([{ "id": "e1", "pack": "enclosure-25" }, { "id": "d1", "pack": "drive-blank", "into": "e1" }]),
            );
        }),
        ("enclosure with UAS", EnclosureUas, Said::Pass, |w| {
            parts(w, json!([{ "id": "e1", "pack": "enclosure-25" }, { "id": "d1", "pack": "ssd-25", "into": "e1" }]));
        }),
        ("enclosure without UAS", EnclosureUas, Said::Warn, |w| {
            parts(w, json!([{ "id": "e1", "pack": "enclosure-35" }, { "id": "d1", "pack": "hdd-35", "into": "e1" }]));
        }),
        ("enclosure unknown", EnclosureUas, Said::Unknown, |w| {
            parts(
                w,
                json!([{ "id": "e1", "pack": "enclosure-blank" }, { "id": "d1", "pack": "ssd-25", "into": "e1" }]),
            );
        }),
        ("a cable into a wired port", NoEthernetPort, Said::Pass, |_| {}),
        ("a cable and no port", NoEthernetPort, Said::Fail, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
        }),
        ("a USB adapter gives it a port", NoEthernetPort, Said::Pass, |w| {
            server(w)["device"] = json!("air");
            server(w)["os"] = json!("macos");
            parts(w, json!([{ "id": "n1", "pack": "usb-nic" }]));
        }),
        ("wired ports unknown", NoEthernetPort, Said::Unknown, |w| server(w)["device"] = json!("mystery")),
        ("router has a free port", EthernetPortsAvailable, Said::Pass, |_| {}),
        ("two cables, one port", EthernetPortsAvailable, Said::Fail, |w| {
            node(w, "router")["device"] = json!("tiny-router");
            push(w, "nodes", json!({ "kind": "host", "id": "second", "label": "Second", "device": "laptop" }));
            push(w, "links", json!({ "id": "second-home", "a": "second", "b": "router", "medium": "ethernet" }));
        }),
        ("router ports unknown", EthernetPortsAvailable, Said::Unknown, |w| {
            node(w, "router")["device"] = json!("blank-router");
        }),
        ("a server on Wi-Fi only", WifiOnlyServer, Said::Warn, |w| w["links"][4]["medium"] = json!("wifi")),
        ("a wired server is not warned", WifiOnlyServer, Said::Nothing, |_| {}),
        ("Wi-Fi at both ends", WifiAvailable, Said::Pass, |_| {}),
        ("no radio in the machine", WifiAvailable, Said::Fail, |w| {
            server(w)["device"] = json!("sbc");
            w["links"][4]["medium"] = json!("wifi");
        }),
        ("radio unknown", WifiAvailable, Said::Unknown, |w| node(w, "tv")["device"] = json!("tv-blank")),
        ("room for the data", StorageCapacity, Said::Pass, |w| {
            parts(w, json!([{ "id": "d1", "pack": "ssd-25" }]));
            push(w, "intents", json!({ "kind": "store", "id": "keep", "host": "server", "gb": 400 }));
        }),
        ("not enough room", StorageCapacity, Said::Fail, |w| {
            parts(w, json!([{ "id": "d1", "pack": "ssd-25" }]));
            push(w, "intents", json!({ "kind": "store", "id": "keep", "host": "server", "gb": 4000 }));
        }),
        ("two copies need two drives", StorageCapacity, Said::Fail, |w| {
            parts(w, json!([{ "id": "d1", "pack": "ssd-25" }]));
            push(w, "intents", json!({ "kind": "store", "id": "keep", "host": "server", "gb": 400, "copies": 2 }));
        }),
        ("drive size unknown", StorageCapacity, Said::Unknown, |w| {
            parts(w, json!([{ "id": "d1", "pack": "drive-blank" }]));
            push(w, "intents", json!({ "kind": "store", "id": "keep", "host": "server", "gb": 400 }));
        }),
        ("one copy is a risk to mention", SingleCopy, Said::Pass, |w| {
            push(w, "intents", json!({ "kind": "store", "id": "keep", "host": "server", "gb": 1 }));
        }),
        ("two copies are not", SingleCopy, Said::Nothing, |w| {
            push(w, "intents", json!({ "kind": "store", "id": "keep", "host": "server", "gb": 1, "copies": 2 }));
        }),
        ("nowhere to install the system", OsStorage, Said::Fail, |_| {}),
        ("a drive to install on", OsStorage, Said::Pass, |w| parts(w, json!([{ "id": "d1", "pack": "ssd-25" }]))),
        ("storage unknown", OsStorage, Said::Unknown, |w| server(w)["device"] = json!("mystery")),
    ]);
}

#[test]
fn will_it_connect() {
    use Reason::*;
    check(&[
        ("a router that forwards", RouterCanForward, Said::Pass, forward),
        ("a router the ISP keeps locked", RouterCanForward, Said::Fail, |w| {
            node(w, "router")["device"] = json!("isp-router");
            forward(w);
        }),
        ("a router we know nothing about", RouterCanForward, Said::Unknown, |w| {
            node(w, "router")["device"] = json!("blank-router");
            forward(w);
        }),
        ("no forward, nothing to say", RouterCanForward, Said::Nothing, |_| {}),
        ("the router hands out your DNS", HomeDnsPossible, Said::Pass, home_dns),
        ("the router allows neither way", HomeDnsPossible, Said::Fail, |w| {
            node(w, "router")["device"] = json!("isp-router");
            home_dns(w);
        }),
        ("the router is unknown", HomeDnsPossible, Said::Unknown, |w| {
            node(w, "router")["device"] = json!("blank-router");
            home_dns(w);
        }),
        ("the service is not a DNS server", HomeDnsPossible, Said::Fail, |w| {
            push(
                w,
                "intents",
                json!({ "kind": "home_dns", "id": "adblock", "host": "server", "service": "jellyfin",
                "router": "router" }),
            );
        }),
        ("a reserved address", StaticLanIp, Said::Pass, |w| dns_service(w, "dns")),
        ("no reservation on this router", StaticLanIp, Said::Warn, |w| {
            node(w, "router")["device"] = json!("isp-router");
            dns_service(w, "dns");
        }),
        ("a fixed address set on the machine", StaticLanIp, Said::Pass, |w| {
            node(w, "router")["device"] = json!("isp-router");
            dns_service(w, "dns");
            server(w)["static_ipv4"] = json!("192.168.1.9");
        }),
        ("reservation unknown", StaticLanIp, Said::UnknownNote, |w| {
            node(w, "router")["device"] = json!("blank-router");
            dns_service(w, "dns");
        }),
        ("software that does not care", StaticLanIp, Said::Nothing, |_| {}),
        ("reached through a forward", IntentReach, Said::Pass, |w| {
            forward(w);
            reach(w);
        }),
        ("no forward, stopped at the router", IntentReach, Said::Fail, reach),
        ("behind carrier NAT", IntentReach, Said::Fail, |w| {
            node(w, "isp")["ipv4"] = json!("cgnat");
            forward(w);
            reach(w);
        }),
        ("line type unknown", IntentReach, Said::Unknown, |w| {
            node(w, "isp")["ipv4"] = json!("unknown");
            forward(w);
            reach(w);
        }),
        ("a goal about a service that is not there", IntentReach, Said::Unknown, |w| {
            push(
                w,
                "intents",
                json!({ "kind": "reach", "id": "remote", "from": "phone", "host": "server",
                "service": "nope", "via": { "kind": "public_of", "node": "router" } }),
            );
        }),
        ("the TV has the app", ClientHasApp, Said::Pass, |w| stream(w, "tv", "h264_1080p", 1)),
        ("no app, a browser may do", ClientHasApp, Said::Warn, |w| {
            node(w, "tv")["device"] = json!("tv-noapp");
            stream(w, "tv", "h264_1080p", 1);
        }),
        ("a player we know nothing about", ClientHasApp, Said::Unknown, |w| {
            forward(w);
            stream(w, "phone", "h264_1080p", 1);
        }),
        ("the uplink carries one stream", StreamBandwidth, Said::Pass, |w| {
            forward(w);
            stream(w, "phone", "h264_1080p", 1);
        }),
        ("four 4K streams do not fit", StreamBandwidth, Said::Fail, |w| {
            forward(w);
            stream(w, "phone", "hevc_2160p_10bit", 4);
        }),
        ("bit rate unknown", StreamBandwidth, Said::Unknown, |w| {
            forward(w);
            stream(w, "phone", "av1_1080p", 1);
        }),
        ("a link with no rate", StreamBandwidth, Said::Unknown, |w| {
            w["links"][3].as_object_mut().unwrap().remove("rate_ab_bps");
            forward(w);
            stream(w, "phone", "h264_1080p", 1);
        }),
    ]);
}

#[test]
fn a_goal_takes_the_worst_of_what_is_under_it() {
    // An ARM laptop with no Ethernet port, cabled to the router.
    let mut world = base();
    server(&mut world)["device"] = json!("air");
    server(&mut world)["os"] = json!("macos");
    server(&mut world)["runtime"] = json!("desktop");
    server(&mut world)["services"][0]["install"] = json!("container");
    push(&mut world, "intents", json!({ "kind": "run", "id": "run-it", "host": "server", "service": "jellyfin" }));
    let report = evaluate(&world);
    assert_eq!(intent(&report, "run-it"), Status::Fail, "no Ethernet port for the cable");
    assert!(said(&report, Reason::NoEthernetPort).contains(&Said::Fail));

    // The cheapest fix: a USB Ethernet adapter.
    parts(&mut world, json!([{ "id": "n1", "pack": "usb-nic" }]));
    let report = evaluate(&world);
    assert_eq!(intent(&report, "run-it"), Status::Pass);
    let build = report.build.as_ref().unwrap();
    let ram = &build.machines.iter().find(|m| m.node == "server").unwrap().ram;
    // macOS 3000 + the hidden VM 2048 + the media server 1024, of 8192.
    assert_eq!((ram.used, ram.total), (Some(6072), Some(8192)));
    assert_eq!(ram.terms.len(), 3);

    // Asahi on the same laptop is partial: a warning, and the goal still stands.
    server(&mut world)["os"] = json!("asahi");
    server(&mut world)["runtime"] = json!("docker");
    let report = evaluate(&world);
    assert!(said(&report, Reason::OsBootsOnDevice).contains(&Said::Warn));
    assert_eq!(intent(&report, "run-it"), Status::Pass);

    // A machine we know nothing about is "can't tell", never a pass.
    server(&mut world)["device"] = json!("mystery");
    let report = evaluate(&world);
    assert_eq!(intent(&report, "run-it"), Status::Unknown);
    let unknowns = &report.build.as_ref().unwrap().unknowns;
    assert!(unknowns.iter().any(|u| u.pack == "mystery" && u.field == "compute.ram_gb"), "{unknowns:?}");
}

#[test]
fn whole_house_dns_depends_on_the_router() {
    for (router, want) in
        [("good-router", Status::Pass), ("isp-router", Status::Fail), ("blank-router", Status::Unknown)]
    {
        let mut world = base();
        server(&mut world)["device"] = json!("sbc");
        node(&mut world, "router")["device"] = json!(router);
        home_dns(&mut world);
        // A lighter DNS service, so memory on the small board is not the story.
        server(&mut world)["services"][0]["app"] = json!("dns");
        parts(&mut world, json!([{ "id": "d1", "pack": "sd-32" }]));
        let report = evaluate(&world);
        assert_eq!(intent(&report, "adblock"), want, "{router}: {:?}", against(&report, "adblock"));
    }
}

#[test]
fn a_goal_nothing_spoke_about_is_not_a_pass() {
    let mut world = base();
    push(&mut world, "intents", json!({ "kind": "yearly_cost", "id": "cheap", "max_paise": 0 }));
    let report = evaluate(&world);
    let result = report.build.as_ref().unwrap().intents.iter().find(|i| i.id == "cheap").unwrap();
    assert!(result.status != Status::Pass || !result.findings.is_empty());
}

#[test]
fn a_host_behind_a_switch_is_on_the_routers_network() {
    let mut world = base();
    push(&mut world, "nodes", json!({ "kind": "switch", "id": "switch", "label": "Switch", "device": "switch5" }));
    world["links"][4] = json!({ "id": "server-switch", "a": "server", "b": "switch", "medium": "ethernet",
        "length_m": 2, "rate_ab_bps": 1000000000, "rate_ba_bps": 1000000000 });
    push(
        &mut world,
        "links",
        json!({ "id": "switch-home", "a": "switch", "b": "router", "medium": "ethernet",
        "length_m": 5, "rate_ab_bps": 1000000000, "rate_ba_bps": 1000000000 }),
    );
    parts(&mut world, json!([{ "id": "d1", "pack": "ssd-25" }]));
    forward(&mut world);
    reach(&mut world);
    let report = evaluate(&world);
    let server = report.facts.nodes.iter().find(|n| n.id == "server").unwrap();
    assert_eq!(serde_json::to_value(server.ipv4).unwrap(), json!("192.168.1.20"));
    assert_eq!(intent(&report, "remote"), Status::Pass, "{:?}", against(&report, "remote"));
    let trace = report.attempts.iter().find(|t| t.attempt == "intent:remote").unwrap();
    let through_switch =
        trace.legs.iter().flat_map(|leg| &leg.steps).filter(|s| s.reason == Reason::SwitchForwards).count();
    assert_eq!(through_switch, 2, "once on the way in, once on the way back");
    assert!(said(&report, Reason::EthernetPortsAvailable).iter().all(|s| *s == Said::Pass));
}

#[test]
fn prices_stay_what_the_learner_typed() {
    let mut world = base();
    server(&mut world)["price"] = json!({ "paise": 1500000, "owned": true });
    node(&mut world, "router")["price"] = json!({ "paise": 250000 });
    parts(&mut world, json!([{ "id": "d1", "pack": "ssd-25", "price": { "paise": 400000 } }]));
    let build = evaluate(&world).build.unwrap();
    assert_eq!(build.price_total_paise, Some(650000), "what is owned is not part of what there is to buy");
    assert_eq!(build.parts.len(), 4, "server, drive, router, TV");
    assert_eq!(evaluate(&base()).build.unwrap().price_total_paise, None, "nothing priced, no total");
}

#[test]
fn a_learners_own_device_is_worded_as_theirs() {
    let mut world = base();
    push(
        &mut world,
        "custom_devices",
        json!({ "id": "my-box", "kind": "learner", "category": "computer",
        "label": "My box", "compute": { "arch": "x86_64", "ram_gb": 4 } }),
    );
    server(&mut world)["device"] = json!("my-box");
    let report = evaluate(&world);
    let ram = report.findings.iter().find(|f| f.rule_id == Reason::RamBudget).unwrap();
    assert_eq!(ram.provenance, sim_core::Provenance::Learner);
    assert!(said(&report, Reason::LearnerValueUsed).contains(&Said::Pass));
    // What they did not enter stays unknown.
    assert!(said(&report, Reason::NoEthernetPort).contains(&Said::Unknown));
}

#[test]
fn the_same_build_gives_the_same_bytes() {
    let mut world = base();
    forward(&mut world);
    reach(&mut world);
    stream(&mut world, "tv", "hevc_2160p_10bit", 1);
    parts(&mut world, json!([{ "id": "e1", "pack": "enclosure-35" }, { "id": "d1", "pack": "hdd-35", "into": "e1" }]));
    add_vm(&mut world, "vm", "server", json!({ "os": "linux", "alloc": { "ram_mb": 2048 } }));
    let first = serde_json::to_string(&evaluate(&world)).unwrap();
    for _ in 0..3 {
        assert_eq!(first, serde_json::to_string(&evaluate(&world)).unwrap());
    }
    // A world with no build fields has no build section at all.
    let mut plain = base();
    plain.as_object_mut().unwrap().remove("home");
    server(&mut plain).as_object_mut().unwrap().remove("os");
    assert!(!serde_json::to_string(&evaluate(&plain)).unwrap().contains("\"build\""));
}

#[test]
fn a_broken_build_is_refused_with_a_reason() {
    use sim_core::build::check::{ProblemKind, problems};
    let found = |world: &Value| -> Vec<(ProblemKind, String)> {
        let world: World = serde_json::from_value(world.clone()).unwrap();
        problems(&world, &bundle()).into_iter().map(|p| (p.kind, p.at)).collect()
    };
    assert_eq!(found(&base()), vec![]);

    let mut world = base();
    server(&mut world)["device"] = json!("no-such-machine");
    server(&mut world)["os"] = json!("no-such-os");
    world["links"][4]["b"] = json!("nowhere");
    parts(&mut world, json!([{ "id": "a", "pack": "hub", "into": "b" }, { "id": "b", "pack": "hub", "into": "a" }]));
    push(&mut world, "nodes", json!({ "kind": "internet", "id": "internet", "label": "Again" }));
    push(&mut world, "intents", json!({ "kind": "run", "id": "run-it", "host": "server", "service": "nope" }));
    let found = found(&world);
    for want in [
        (ProblemKind::UnknownPack, "server/no-such-machine"),
        (ProblemKind::UnknownPack, "server/no-such-os"),
        (ProblemKind::UnknownNode, "nowhere"),
        (ProblemKind::Loop, "server/a"),
        (ProblemKind::DuplicateId, "internet"),
        (ProblemKind::UnknownService, "run-it/nope"),
    ] {
        assert!(found.contains(&(want.0, want.1.to_owned())), "{want:?} not in {found:?}");
    }
}
