# Build your own (sandbox)

Spec for the free-build sandbox: a learner puts together a homelab from parts sold or commonly found in India, says what they want it to do, and finds out whether it will work **before spending money or time**. Written 2026-11-11 to be built from without further decisions. Where a choice was taken without the owner, it is marked **(default)** and is cheap to change.

Read [Handoff.md](Handoff.md) first, then [Architecture.md](Architecture.md). The data this needs is listed in [Data-Backlog.md](Data-Backlog.md).

## Contents

1. [Promise and limits](#1-promise-and-limits)
2. [How it fits what exists](#2-how-it-fits-what-exists)
3. [Learner flow](#3-learner-flow)
4. [Data packs: schema v4](#4-data-packs-schema-v4)
5. [World additions](#5-world-additions)
6. [Intents: what the learner wants](#6-intents-what-the-learner-wants)
7. [Rules](#7-rules)
8. [Budget and power arithmetic](#8-budget-and-power-arithmetic)
9. [Report shape](#9-report-shape)
10. [UI](#10-ui)
11. [Saving, sharing, privacy](#11-saving-sharing-privacy)
12. [Templates](#12-templates)
13. [Data build and bundle split](#13-data-build-and-bundle-split)
14. [Build phases S1 to S5](#14-build-phases-s1-to-s5)
15. [Model limits stated to the learner](#15-model-limits-stated-to-the-learner)

## 1. Promise and limits

**Promise:** "Plan your homelab with parts you can get in India. See what works, what doesn't, and what we can't tell yet, before you buy."

The sandbox answers four questions about a build, each three-valued:

| Question | Examples |
|---|---|
| **Will it run?** | Can this OS boot on this machine? Is there a build of Immich for this CPU? Does Proxmox need hardware virtualization here? |
| **Will it fit?** | Is there enough RAM for these services and VMs? Enough drive bays, USB ports, Ethernet ports, storage? |
| **Will it connect?** | Can I watch from my phone on mobile data, on my line? Can the whole house use Pi-hole with this ISP router? Reuses the remote-access core as is. |
| **Will it last?** | What does it cost a year in electricity in my state? Does it survive a 2-hour power cut? Does it come back by itself afterwards? Is a laptop safe plugged in 24/7? |

Limits, stated in the UI and never hidden:

- **No CPU performance model.** There is no open, sourced, comparable CPU benchmark data. The sandbox does not say "fast enough" for CPU-bound work. It says what it can (hardware transcoding support, RAM, arch) and states CPU speed as not modelled. See [15](#15-model-limits-stated-to-the-learner).
- **No shipped prices.** Rupee prices stay deferred (decision log). The learner may type their own prices, which stay on their device. See [11](#11-saving-sharing-privacy).
- **No purchase links.** Neutrality rules in [Design.md](Design.md) apply. No affiliate or seller links, ever, in verdicts or the parts list.
- **Unknown is a first-class answer.** The sandbox will often say "we don't have data for this" for unconventional devices. That is honest, and each unknown links to how to add the data.

## 2. How it fits what exists

Everything the hard rules require still holds. The sandbox is a second way to make a `World`, plus new data packs and new rules.

| Existing piece | Use in the sandbox |
|---|---|
| One world graph, `evaluate(world, bundle) -> Report` | Unchanged signature. The sandbox sends a world with intents; the report gains a `build` section ([9](#9-report-shape)). |
| Three-valued findings, provenance | Unchanged, plus `Provenance::Learner` for values the learner typed in. |
| `Cap<T>` sourced values | Extended with `basis` ([4.1](#41-cap-gains-a-basis)). |
| `DevicePack` | Gains `category` and capability sections. Archetypes and missions migrate ([4](#4-data-packs-schema-v4)). |
| Remote-access walk, traces, NAT, CGNAT, tunnels | Used as is. "Reach from my phone" intents become `Attempt`s, so the trace player replays them. |
| `HostVirtualization` rule | Generalised into the fit rules. Mission behaviour must not change. |
| Mission board, inspector, findings, trace player (Phase 5) | Reused. The sandbox adds a parts palette, free placement, tap-to-connect and a report panel. |
| `data/services/`, `data/power/` (empty) | Filled by the data backlog. |

Boundaries restated for this feature:

1. **The UI never decides validity.** It does not filter the palette by "compatible" either. It may filter by category, text and India availability, which are labels, not verdicts. Every "works / won't / can't tell" comes from the core.
2. **Rules read capabilities, never product names.** A rule may ask `compute.arch`, `os.provides.containers` or `router.custom_dns`. It may never ask whether a device id is `macbook-air-m1-2020`.
3. **Adding a device, part, OS, runtime, service, tariff or template is one YAML file and no code change.** CI proves it for every category.
4. **All arithmetic is integer, in the core.** Watts, kWh, paise, MB, minutes. The UI formats; it does not compute totals.

## 3. Learner flow

1. **Start.** From `#/build`: pick a template ("Old laptop media server", "MacBook Air M1 as a server"…), reopen a saved build, or start empty. An empty build still holds the fixed "away" scaffolding: a phone on a mobile network and the internet ([12](#12-templates)).
2. **Add machines.** Tap a machine in the parts palette. It lands on the board, connected to nothing. The learner connects it: Ethernet to the router, or Wi-Fi.
3. **Set it up.** In the inspector: choose an OS (or hypervisor), add parts (drives, USB Ethernet adapter, enclosure), add guests (VM, LXC, container), add services (Jellyfin, Pi-hole…), set machine settings (lid closed, charge limit, power-loss recovery).
4. **Say what it should do.** In the Goals panel: "Stream films to the TV at home", "Watch from my phone on mobile data", "Ad-blocking for the whole house", "Keep 2 TB of photos", "Survive a 2-hour power cut", "Cost less than ₹X a year to run".
5. **Set the home.** ISP profile (or line check), home router (ISP-supplied or retail), state electricity tariff, optional monthly units from the bill.
6. **Read the report.** One line per goal: works in this model, won't work (and why, at which part), or can't tell (and what data is missing). Then budget, power and cost, the parts list, and "what we don't know".
7. **Try fixes.** Every failing finding has the usual explanation with "How to fix it, cheapest first". Swapping a part or adding a USB Ethernet adapter re-evaluates at once.
8. **Keep it.** Saved on the device. Export as a file, or copy a share link that carries the build in the URL fragment.

There is no "win". Missions teach a concept; the sandbox answers "will my plan work".

## 4. Data packs: schema v4

`SCHEMA_VERSION` goes from 3 to 4. The three archetypes, the missions that reference them, `app/src/lib/types.ts` and the data-build tests migrate in the same change.

### 4.1 Cap gains a basis

```rust
pub enum Cap<T> {
    Sourced { value: T, source: String, retrieved_at: String,
              #[serde(default)] basis: Basis,          // official by default
              #[serde(default)] note: Option<String> },
    Unknown(UnknownMark),
    Value(T),                                            // archetypes and learner devices only
}

pub enum Basis {
    #[default] Official,   // maker's spec page, datasheet, manual, PSREF, official docs
    Review,                // an independent measurement published by a named reviewer
    Community,             // forum, wiki, issue tracker, user report
}
```

Finding provenance comes from the weakest basis among the values a rule used: `Official` → `Provenance::Official`, `Review` or `Community` → `Provenance::Community`, an archetype → `Teaching`, a learner device → `Learner`. Community-report counts ("Reported to work, N reports") arrive with community reports later; until then the wording is "Reported to work" with the source on tap.

### 4.2 PackKind and Category

```rust
pub enum PackKind { Archetype, Device, Learner }   // Learner: only from a build's custom_devices, never from files

pub enum Category {
    Computer,        // laptop, desktop, mini PC, SBC, phone, NAS: anything that runs an OS
    Router,          // home router, including ISP-supplied combined ONT routers
    Switch,
    AccessPoint,
    Ups,             // AC UPS
    Inverter,        // home inverter with external battery
    DcUps,           // 12 V / 5 V mini UPS for routers and ONTs
    Drive,           // HDD, SSD, NVMe
    Enclosure,       // USB to SATA or NVMe
    UsbNic,          // USB Ethernet adapter
    UsbHub,          // hub or dock
    Client,          // TV, streaming stick, phone used as a client: what plays the media
}
```

### 4.3 DevicePack v4

```rust
pub struct DevicePack {
    pub id: Id,
    pub kind: PackKind,
    pub category: Category,
    pub label: String,
    #[serde(default)] pub aka: Vec<String>,             // search aliases, model numbers
    #[serde(default)] pub released: Cap<u16>,           // year
    #[serde(default)] pub form: Cap<Form>,              // laptop | desktop | mini_pc | sbc | phone | tablet | nas | rack | other
    #[serde(default)] pub india: Cap<Availability>,     // see 4.5
    #[serde(default)] pub compute: Option<Compute>,
    #[serde(default)] pub network: Option<NetCaps>,
    #[serde(default)] pub ports: Option<Ports>,
    #[serde(default)] pub storage: Option<StorageCaps>,
    #[serde(default)] pub power: Power,
    #[serde(default)] pub battery: Option<BatteryCaps>,
    #[serde(default)] pub router: Option<RouterCaps>,
    #[serde(default)] pub ups: Option<UpsCaps>,
    #[serde(default)] pub drive: Option<DriveCaps>,
    #[serde(default)] pub adapter: Option<AdapterCaps>,   // enclosure, usb_nic, usb_hub
    #[serde(default)] pub client: Option<ClientCaps>,
    #[serde(default)] pub os_support: BTreeMap<Id, Cap<OsSupport>>,   // keyed by OS pack id
    #[serde(default)] pub notes: Vec<String>,           // shown in the inspector, never read by rules
}
```

`tools/data-build` enforces which sections each category may have. A section that is not allowed is an error; a section that is allowed but absent means every value in it is unknown.

| Category | Allowed sections |
|---|---|
| computer | compute, network, ports, storage, power, battery, os_support |
| router | network, ports, power, router |
| switch, access_point | network, power |
| ups, inverter, dc_ups | power, ups |
| drive | power, drive |
| enclosure, usb_nic, usb_hub | power, adapter, os_support |
| client | network, client |

Section types. Every leaf is a `Cap<_>` unless noted. Lists that can be legitimately empty are `Cap<Vec<_>>`, so "none" (`value: []`) and "unknown" stay different. **This fixes a v3 ambiguity: `nic: []` used to mean both.**

```rust
pub struct Compute {
    arch: Cap<Arch>,                    // x86_64 | aarch64 | armv7 | riscv64
    cores: Cap<u32>, threads: Cap<u32>,
    ram_gb: Cap<u32>,                   // as sold / as configured in this pack
    ram_max_gb: Cap<u32>,
    ram_soldered: Cap<bool>,
    virtualization: Cap<bool>,          // hardware virtualization a hypervisor can use
    nested_virtualization: Cap<bool>,
    firmware: Cap<Firmware>,            // uefi | bios | uefi_and_bios | apple_silicon | uboot | android_bootloader
    video: Cap<VideoCaps>,              // hardware media engine
    cooling: Cap<Cooling>,              // fanless | fan
    ac_power_recovery: Cap<bool>,       // firmware can power on when mains returns
    wake_on_lan: Cap<bool>,
}

pub struct VideoCaps {
    apis: Vec<AccelApi>,                // vaapi | qsv | nvenc | amf | videotoolbox | v4l2m2m | rkmpp
    decode: BTreeMap<Codec, bool>,      // h264 | hevc_8 | hevc_10 | av1 | vp9
    encode: BTreeMap<Codec, bool>,
}

pub struct NetCaps {
    nic: Cap<Vec<Nic>>,                 // wired ports; value: [] means none
    wifi: Cap<WifiGen>,                 // none | wifi4 | wifi5 | wifi6 | wifi6e | wifi7
}
pub struct Nic { speed_mbps: Cap<u32> }

pub struct Ports { usb: Cap<Vec<UsbPort>> }
pub struct UsbPort { kind: UsbKind, count: u32 }   // usb2_a | usb5_a | usb10_a | usb5_c | usb10_c | usb40_c (USB4/Thunderbolt)

pub struct StorageCaps {
    bays: Cap<Vec<Bay>>,                // internal slots a learner can fill
    soldered_gb: Cap<u32>,              // built-in storage that cannot be changed (M1 Air, eMMC SBCs)
    boot_from_usb: Cap<bool>,
}
pub struct Bay { interface: BayKind, count: u32 }  // sata_2_5 | sata_3_5 | m2_nvme | m2_sata | msata | sd_card

pub struct Power {
    idle_w: Cap<u32>, load_w: Cap<u32>,
    input: Cap<PowerInput>,             // ac_230 | dc { volts_mv, max_ma } | usb_c_pd { watts } | usb_5v { max_ma }
}

pub struct BatteryCaps {
    wh: Cap<u32>,
    firmware_charge_limit: Cap<bool>,   // a limit set in firmware, independent of OS
}

pub struct RouterCaps {
    lan: Cap<Vec<Nic>>,
    wan: Cap<WanKind>,                  // gpon_fibre | epon_fibre | ethernet | dsl | cellular
    admin: Cap<AdminAccess>,            // full | limited | isp_only
    port_forwarding: Cap<bool>,
    upnp: Cap<bool>,
    dmz: Cap<bool>,
    bridge_mode: Cap<bool>,
    lan_dns_setting: Cap<bool>,         // can the DHCP server hand out a custom DNS server
    dhcp_can_disable: Cap<bool>,
    dhcp_reservation: Cap<bool>,
    ipv6: Cap<bool>,
    ipv6_firewall_configurable: Cap<bool>,
    udp_timeout_s: Cap<u32>,
    supplied_by: Option<Id>,            // ISP profile id, for ISP-supplied units. A label, never read by rules
}

pub struct UpsCaps {
    topology: Cap<UpsTopology>,         // offline | line_interactive | online | inverter | dc
    va: Cap<u32>, max_w: Cap<u32>,
    battery_wh: Cap<u32>,               // for inverters: of the battery pack in the pack, else unknown
    battery_external: Cap<bool>,        // inverters: battery is a separate purchase
    outlets: Cap<u32>,
    dc_outputs: Cap<Vec<DcOutput>>,     // { volts_mv, max_ma, connector }
    transfer_ms: Cap<u32>,
    waveform: Cap<Waveform>,            // sine | stepped_sine | square
    runtime: Cap<Vec<RuntimePoint>>,    // maker's runtime chart: { load_w, minutes }
}

pub struct DriveCaps {
    interface: Cap<BayKind>,
    medium: Cap<DriveMedium>,           // hdd | ssd
    capacity_gb: Cap<u32>,
    needs_12v: Cap<bool>,               // 3.5" drives cannot run from USB bus power
    rated_24x7: Cap<bool>,
}

pub struct AdapterCaps {
    host_side: Cap<UsbKind>,            // what it plugs into
    provides: AdapterProvides,          // nic: Vec<Nic> | bays: Vec<Bay> | usb: Vec<UsbPort>
    uas: Cap<bool>,                     // enclosures
    needs_power: Cap<bool>,             // external power brick required
}

pub struct ClientCaps {
    plays: BTreeMap<Codec, bool>,       // what it decodes without help
    apps: Vec<Id>,                      // service ids with a client app on it (jellyfin, plex)
    ipv6: Cap<bool>,
}

pub struct OsSupport {
    level: Support,                     // full | partial | none
    lid_closed: Cap<Support>,           // keeps running with the lid shut
    charge_limit: Cap<bool>,            // the OS can cap battery charge
    gaps: Vec<String>,                  // short phrases: "no Thunderbolt", "Wi-Fi only after setup"
}
```

`os_support` is how unconventional pairings are stated. Example: a laptop's entry for `fedora-asahi-remix` is `partial` with its gaps, sourced from the Asahi feature-support pages (basis `official`, since it is the project's own documentation). The rule reads `level` and `gaps`, not the laptop's name.

### 4.4 OS, runtime and service packs

**`data/os/<id>.yaml`**: operating systems, hypervisors and NAS systems.

```rust
pub struct OsPack {
    id: Id, label: String, kind: OsKind,          // os | hypervisor | nas_os | appliance
    family: OsFamily,                             // linux | macos | windows | android | bsd
    arches: Vec<Arch>, firmware: Vec<Firmware>,   // what it can boot on, when the device says nothing
    listed_devices_only: bool,                    // true: a device must name this OS in os_support (Asahi, postmarketOS)
    requires: OsRequires { virtualization: Cap<bool>, ram_min_mb: Cap<u32>, storage_min_gb: Cap<u32>, wired_nic: Cap<bool> },
    base_ram_mb: Cap<u32>,                        // idle footprint used in the RAM budget
    provides: OsProvides {
        vms: Cap<bool>, lxc: Cap<bool>,
        vm_accelerated_arches: Vec<Arch>,         // guests of other arches are emulated
        vm_emulated_arches: Vec<Arch>,
        native_runtimes: Vec<Id>,                 // runtime pack ids available without a VM
    },
    sources: Vec<Source>,
}
```

**`data/runtimes/<id>.yaml`**: container runtimes, because on macOS and Windows they decide a lot.

```rust
pub struct RuntimePack {
    id: Id, label: String,
    os_families: Vec<OsFamily>,
    via_vm: Cap<bool>,                            // runs containers inside a hidden Linux VM
    vm_overhead_mb: Cap<u32>,                     // RAM that VM takes before any container
    host_network: Cap<bool>,                      // `network_mode: host` reaches the real LAN
    device_passthrough: Cap<Vec<AccelApi>>,       // GPU APIs a container can use
    cost: Cap<Cost>,                              // free | free_personal | paid
}
```

**`data/services/<id>.yaml`**: software placed on a host.

```rust
pub struct ServicePack {
    id: Id, label: String, category: ServiceCategory,   // media | photos | files | dns | automation | passwords | sync | backup | monitoring | proxy | vpn | tunnel | games | downloads | dev
    install: Vec<Install>,                    // { method: native | container | lxc | addon | package, os_families, arches: Cap<Vec<Arch>>, runtime_needed: bool }
    ram_min_mb: Cap<u32>, ram_rec_mb: Cap<u32>,
    storage_min_gb: Cap<u32>,
    ports: Vec<DefaultPort>,                  // { proto, port, purpose }: become Service entries
    needs: ServiceNeeds {
        host_network_for: Vec<String>,        // features that need host networking ("device discovery", "DHCP server")
        static_lan_ip: bool,
        always_on: bool,
        transcode_apis: BTreeMap<OsFamily, Vec<AccelApi>>,
    },
    provides: Vec<Provides>,                  // dns_server | dhcp_server | media_server | photo_backup | file_share | vpn_endpoint | tunnel_connector | reverse_proxy | mesh_vpn
    paid_features: Vec<PaidFeature>,          // { feature, needed_for: Vec<IntentKind>, source, retrieved_at }
    licence: Cap<String>,
    sources: Vec<Source>,
}
```

### 4.5 India availability

```rust
pub enum Availability {
    SoldNew,          // sold new in India by the maker or an authorised channel
    UsedCommon,       // not sold new, widely found used or refurbished
    ImportOnly,
    Discontinued,     // not sold new anywhere, rarely found
}
```

`SoldNew` needs an official India page (maker store, authorised distributor). `UsedCommon` is basis `community` at best, and the source must be a durable page (a refurbisher's category page, a community thread), not a single marketplace listing. The palette can filter on this. It is a label, never part of a verdict.

### 4.6 Power data

**`data/power/tariffs/<id>.yaml`**, one per DISCOM and category (residential first):

```rust
pub struct Tariff {
    id: Id, state: String, discom: String, category: String,   // "LT domestic"
    effective_from: String,
    slabs: Vec<Slab>,                          // { upto_kwh: Option<u32> (None = rest), paise_per_kwh: u32 }
    fixed_paise_per_month: Cap<u32>,           // shown, not added to the homelab's marginal cost
    duty_permille: Cap<u32>,                   // electricity duty as a share of energy charges
    free_units: Option<FreeUnits>,             // { upto_kwh, condition: String } for state schemes; the learner says whether it applies
    source: String, retrieved_at: String,
}
```

**`data/power/outages/<id>.yaml`**: outage scenarios. At launch these are `kind: teaching` presets ("Evening cut, 2 hours", "Overnight, 6 hours", "Ten short blips a day"), with fields `duration_min`, `per_day`, `isp_side: up | down | unknown` (default `up`, stated as an assumption). `kind: real` scenarios need an official source such as a DISCOM reliability report.

### 4.7 Reference tables

**`data/reference/media.yaml`**: typical bitrates per format (`h264_1080p`, `hevc_2160p_10bit`…), sourced from media-server documentation. Used by the streaming intent. Values are labelled typical, not exact.

## 5. World additions

All new fields default to empty, so mission worlds are unchanged and their reports stay byte-identical (a test asserts this).

```rust
pub struct World {
    // existing fields …
    #[serde(default)] pub intents: Vec<Intent>,
    #[serde(default)] pub power_feeds: Vec<PowerFeed>,     // { from: power node id, to: node id }
    #[serde(default)] pub home: Option<HomePlan>,
    #[serde(default)] pub custom_devices: Vec<DevicePack>, // kind: learner only; ids prefixed "my-"
}

pub struct HomePlan {
    tariff: Option<Id>,
    rate_paise_per_kwh: Option<u32>,      // learner override; beats the tariff
    base_kwh_month: Option<u32>,          // from their bill; default 150 (default), stated as an assumption
    free_units_apply: bool,
    outage: Option<Id>,
}

pub enum Node {
    Host(Host), Router(Router), Isp(Isp), Internet(Internet),
    Switch(Switch),                       // { id, label, device }: transparent; hosts behind it share the router's LAN
    Power(PowerUnit),                     // { id, label, device }: a UPS, inverter or DC mini UPS
}

pub struct Host {
    // existing fields …
    #[serde(default)] pub os: Option<Id>,
    #[serde(default)] pub runtime: Option<Id>,            // container runtime pack
    #[serde(default)] pub guest: Option<GuestKind>,       // vm | lxc | container, when runs_on is set
    #[serde(default)] pub alloc: Option<Alloc>,           // guests: { ram_mb, disk_gb, threads }
    #[serde(default)] pub parts: Vec<Mount>,              // { id, pack, into: Option<Id> } into = another mount (drive into enclosure)
    #[serde(default)] pub settings: MachineSettings,
    #[serde(default)] pub role: Role,                     // server (default) | client
    #[serde(default)] pub busy_permille: Option<u16>,     // share of time at load; default 100 (default)
    #[serde(default)] pub price: Option<Price>,           // { paise, owned: bool } learner-entered
}

pub struct MachineSettings {
    lid_closed: bool,
    charge_limit: bool,
    ac_power_recovery: bool,              // the learner turned it on in firmware
    always_on: bool,                      // default true for servers
}

pub struct Service {
    // existing fields …
    #[serde(default)] pub app: Option<Id>,                // service pack id
    #[serde(default)] pub install: Option<InstallMethod>,
    #[serde(default)] pub autostart: Option<bool>,        // restart policy or boot unit; missing is unknown
}
```

`Price` also goes on `Mount`, `Router`, `Switch` and `PowerUnit`. `Router.device` already exists and now points at a `router` pack.

**Switch and Wi-Fi in the walk.** `facts.rs` and `walk.rs` treat a `Switch` as part of the LAN segment it is cabled to: a host linked to a switch that is linked to a router is that router's customer, with an extra forwarding step in traces (`switch-forwards`). A `Link` with `medium: wifi` between a host and a router is allowed only when the router or an access point on the segment has Wi-Fi. Port counting (router LAN ports, switch ports) happens in the fit rules, not the walk.

## 6. Intents: what the learner wants

An intent is a goal the learner states. It lives in the world, so "what I want" is data, like `Attempt`.

```rust
pub enum Intent {
    /// "I want this running." Placement is already in the world; this asks the core to judge it.
    Run { id, host: Id, service: Id },
    /// "Reach it from there." Expands to one Attempt.
    Reach { id, from: Id, host: Id, service: Id, via: Target },
    /// "Watch films on that client." Direct play or transcode, plus bandwidth.
    Stream { id, client: Id, host: Id, service: Id, format: Id, streams: u8 },
    /// "Ad-blocking (or my DNS) for every device at home."
    HomeDns { id, host: Id, service: Id, router: Id },
    /// "Keep this much data, optionally with a second copy."
    Store { id, host: Id, gb: u32, copies: u8 },
    /// "Still works during this outage, and comes back after it."
    SurviveOutage { id, outage: Id, keep: Vec<Id> },   // keep: other intent ids that must still hold
    /// "Running cost under this much a year."
    YearlyCost { id, max_paise: u64 },
}
```

**Expansion is in the core** (`build/expand.rs`), deterministic, before facts are built. `Reach` and remote `Stream` add an `Attempt` with id `intent:<intent id>`, proto and port from the service, and `transfer` from the media format for `Stream`. The UI never makes these attempts.

**Intent status** is computed from the findings tagged with that intent id:

- **Fail** if any tagged finding is `Fail` with severity `Error`.
- else **Unknown** if any tagged finding is `Unknown` with severity `Error` or `Warning`.
- else **Pass**, with any warnings listed under it.
- An intent with no tagged findings is **Unknown** with reason `intent-not-checked`. A rule that said nothing has not passed.

Findings gain `#[serde(default)] intents: Vec<Id>` so one finding (say, no RAM) can sit under several intents.

## 7. Rules

New rules live in `crates/sim-core/src/rules/`, split into `network.rs` (today's `rules.rs`), `fit.rs`, `budget.rs`, `power.rs`, `intents.rs`. Every rule is three-valued; missing data gives `Unknown` with reason `device-data-missing` or a more specific one. Each new reason needs a `content/steps.yaml` entry; the ones marked ★ also get a long explanation in `content/explanations/`.

### 7.1 Will it run

| Reason id | Checks | Fail / Warning / Unknown |
|---|---|---|
| `os-boots-on-device` ★ | Device `os_support[os]` if present; else OS `arches` and `firmware` against device, and `listed_devices_only` | Fail on `none` or no match; Warning on `partial` (gaps listed); Unknown if arch or firmware unknown, or `listed_devices_only` and device silent |
| `os-requirements-met` | OS `requires`: virtualization, RAM, storage, wired NIC | Fail when a known value falls short |
| `hypervisor-needs-virtualization` | Replaces `host-virtualization`: guest's parent OS provides VMs/LXC and device has virtualization | Same three outcomes as today. Mission findings keep the old id via an alias test |
| `guest-arch-emulated` ★ | VM guest OS arch not in host OS `vm_accelerated_arches` | Warning if in `vm_emulated_arches` ("runs, very slowly"); Fail if in neither |
| `nested-virtualization` | A hypervisor inside a VM | Fail if device `nested_virtualization` false |
| `service-available` ★ | Some `install` entry matches host OS family, arch and the chosen `install` method; `runtime_needed` needs a runtime | Fail on no match; Unknown when arches unknown |
| `runtime-on-os` | Runtime `os_families` includes host OS family | Fail |
| `container-host-network` ★ | Service `host_network_for` features, runtime `host_network` | Warning naming the features lost (e.g. device discovery); Unknown if runtime value unknown |
| `transcode-supported` ★ | Stream needs transcode (client can't play format): device `video.encode[codec]`, intersection of device `video.apis`, service `transcode_apis[os family]`, runtime `device_passthrough` when containerised | Pass with hardware; Warning "software transcoding, CPU speed not modelled"; Unknown on missing |
| `service-autostart` | Service `autostart` | Warning when false; Unknown when missing, only reported for `SurviveOutage` intents |
| `paid-feature-needed` | Service `paid_features` whose `needed_for` matches an intent kind | Info, dated, never Fail |

### 7.2 Will it fit

| Reason id | Checks | Outcome |
|---|---|---|
| `ram-budget` ★ | [8.1](#81-ram) | Fail over total at minimums; Warning over total at recommended; Unknown on missing |
| `guest-alloc-exceeds-host` | Sum of guest `alloc.ram_mb` + host OS base vs host RAM | Fail |
| `bay-available` | Mounts with a bay interface vs device `bays` (or enclosure `provides.bays`) | Fail when over, or wrong interface |
| `usb-port-available` | Mounts that plug into USB vs device `ports.usb` plus hubs | Fail when over; Warning when a 10 Gbit/s device sits on a slower port |
| `drive-power` | `needs_12v` drive in an enclosure without `needs_power` | Fail |
| `enclosure-uas` | Enclosure `uas` false or unknown with a 24/7 storage service | Warning or Unknown |
| `no-ethernet-port` ★ | An Ethernet link to a computer whose `nic` is `[]` and no `usb_nic` mount | Fail. The fix text names the adapter category, never a product |
| `ethernet-ports-available` | Links per router/switch vs `lan` length | Fail when over; fix: add a switch |
| `wifi-only-server` ★ | A server joined by `medium: wifi` | Warning, with reliability and latency teaching |
| `wifi-available` | Wi-Fi link to a device or router whose `wifi` is `none` | Fail |
| `storage-capacity` | `Store` intent gb vs drives on that host (copies counted on separate drives) | Fail; Unknown on missing capacity |
| `single-copy` | `Store` with `copies: 1` | Info, `debrief_only` style: shown in the report's risks section only |
| `os-storage` | OS `storage_min_gb` vs boot drive or `soldered_gb` | Fail |

### 7.3 Will it connect

Existing network rules and traces, unchanged, plus:

| Reason id | Checks | Outcome |
|---|---|---|
| `router-can-forward` ★ | Router has enabled port forwards and pack `port_forwarding` false, or `admin: isp_only` | Fail; Unknown if unknown. Many ISP-supplied routers in India are the real reason a forward cannot be made, before CGNAT even matters |
| `home-dns-possible` ★ | `HomeDns`: router `lan_dns_setting`, or `dhcp_can_disable` and the service provides `dhcp_server` | Fail if neither; Unknown if unknown. Fix text: per-device DNS, or a second router |
| `static-lan-ip` | Service `static_lan_ip` and the router has no `dhcp_reservation` and host has no static address | Warning |
| `stream-bandwidth` | Remote `Stream`: format bitrate × streams vs the trace path's spare rate (existing physics) | Fail if short; Unknown if any link rate unknown |
| `client-has-app` | `Stream` client `apps` includes the service | Warning (browser fallback); Unknown on missing |

### 7.4 Will it last

| Reason id | Checks | Outcome |
|---|---|---|
| `yearly-cost` | `YearlyCost` vs [8.3](#83-electricity-cost) | Fail over; Unknown if any machine's watts are unknown |
| `outage-covered` ★ | [8.4](#84-outages-and-ups) per kept intent: every node on its path is fed for the whole outage | Fail, naming the first node that goes dark (often the ONT/router); Unknown on missing |
| `ups-overloaded` | Fed load at load watts vs `max_w` | Fail |
| `recovers-after-outage` ★ | A node that powers off during the outage comes back: `ac_power_recovery` capability and setting, or it never lost power; services `autostart` | Fail / Unknown |
| `laptop-as-ups` | A laptop with a battery outlasts the outage on its own battery | Pass with the minutes, Info. Teaching point, not a nag |
| `battery-always-charging` ★ | Laptop, `always_on`, no charge limit available or set | Warning: battery wear and swelling risk, with how to cap charge on this OS if `charge_limit` is true |
| `lid-closed-running` | `settings.lid_closed` and `os_support[os].lid_closed` | Fail on `none`, Warning on `partial`, Unknown on missing |
| `fanless-sustained-load` | `cooling: fanless` and `busy_permille` above 300 | Info: may slow under long load; not modelled |
| `inverter-waveform` | `waveform: square` feeding a computer | Warning |

### 7.5 Reasons summary

New ids to add to `reason.rs` and `content/steps.yaml` (★ have long explanations):

`os-boots-on-device`★, `os-requirements-met`, `hypervisor-needs-virtualization`, `guest-arch-emulated`★, `nested-virtualization`, `service-available`★, `runtime-on-os`, `container-host-network`★, `transcode-supported`★, `service-autostart`, `paid-feature-needed`, `ram-budget`★, `guest-alloc-exceeds-host`, `bay-available`, `usb-port-available`, `drive-power`, `enclosure-uas`, `no-ethernet-port`★, `ethernet-ports-available`, `wifi-only-server`★, `wifi-available`, `storage-capacity`, `single-copy`, `os-storage`, `router-can-forward`★, `home-dns-possible`★, `static-lan-ip`, `stream-bandwidth`, `client-has-app`, `yearly-cost`, `outage-covered`★, `ups-overloaded`, `recovers-after-outage`★, `laptop-as-ups`, `battery-always-charging`★, `lid-closed-running`, `fanless-sustained-load`, `inverter-waveform`, `switch-forwards`, `intent-not-checked`, `learner-value-used`.

## 8. Budget and power arithmetic

Integer only, in `crates/sim-core/src/build/budget.rs` and `physics.rs`. Every assumption constant is named, documented with its source or marked as a teaching assumption, and shown in the UI next to the number it affects.

### 8.1 RAM

For each machine `m` (not a guest):

```
used_min(m) = os.base_ram_mb
            + Σ services on m:             service.ram_min_mb
            + Σ runtimes on m with via_vm: runtime.vm_overhead_mb
            + Σ guests on m:               guest.alloc.ram_mb  (or, for containers without alloc, their services' ram_min_mb)
used_rec(m) = same with ram_rec_mb (falls back to ram_min_mb when rec is unknown and min is known)
total(m)    = compute.ram_gb × 1024
```

Fail if `used_min > total`; Warning if `used_rec > total`; Unknown if any term is unknown. The report shows the bar with each term, so the learner sees what eats the RAM.

### 8.2 Average watts

```
avg_w(n) = idle_w + (load_w − idle_w) × busy_permille / 1000     (busy default 100, i.e. 10%, labelled an assumption)
         + Σ mounted drives: their idle_w (drive load ignored at launch)
```

Laptops with `lid_closed` add nothing; screens-on is not modelled.

### 8.3 Electricity cost

```
lab_kwh_month   = Σ avg_w × 730 / 1000                       (730 h = one average month)
marginal        = the slabs covering [base_kwh_month, base_kwh_month + lab_kwh_month],
                  minus free units if free_units_apply, plus duty
yearly_paise    = 12 × Σ over covered slab parts (kWh in part × paise_per_kwh) × (1000 + duty_permille) / 1000
```

`rate_paise_per_kwh`, when set, replaces the slab calculation. The point of using slabs: a homelab is added **on top of** the household's existing use, so it is billed at the household's top slab, not the cheapest one. That is the India-specific lesson, and the explanation says so.

### 8.4 Outages and UPS

```
fed_load_w(u)  = Σ avg_w of nodes fed by u  (and Σ load_w for the overload check)
runtime_min(u) = interpolate(u.runtime chart, fed_load_w)        if a chart exists (integer linear interpolation)
               = battery_wh × UPS_USABLE_PERMILLE / 1000 × 60 / fed_load_w  otherwise
laptop_min(h)  = battery.wh × LAPTOP_USABLE_PERMILLE / 1000 × 60 / avg_w(h)
```

`UPS_USABLE_PERMILLE` and `LAPTOP_USABLE_PERMILLE` are teaching assumptions (lead-acid depth of discharge and inverter loss; lithium reserve), stated as such. A node survives an outage of `d` minutes if it is fed by something whose runtime ≥ `d`, or is a laptop with `laptop_min ≥ d`. An intent survives if every node on its trace path on the home side survives, and the scenario's `isp_side` is `up`. With `unknown` the answer is Unknown.

## 9. Report shape

`Report` gains one optional section. Missions leave it out.

```rust
pub struct BuildReport {
    pub intents: Vec<IntentResult>,        // { id, status, findings: Vec<usize> (indexes into Report.findings), attempt: Option<Id> }
    pub machines: Vec<MachineBudget>,      // { node, ram: Budget, bays: Budget, usb: Budget, ports: Budget, avg_w: Option<u32> }
    pub power: PowerSummary,               // { lab_avg_w, lab_kwh_year, yearly_paise, rate_used, assumptions: Vec<Reason> }
    pub ups: Vec<UpsSummary>,              // { node, fed: Vec<Id>, fed_avg_w, runtime_min, overloaded }
    pub parts: Vec<PartLine>,              // { node, mount: Option<Id>, pack, label, price_paise, owned }
    pub price_total_paise: Option<u64>,    // to buy, excluding owned; None if nothing priced
    pub unknowns: Vec<UnknownItem>,        // { pack: Id, field: String, used_by: Vec<Reason> }: drives "help add this data"
}

pub struct Budget { used: Option<u64>, total: Option<u64>, terms: Vec<(String, Option<u64>)> }
```

`unknowns` is what makes the data loop work: each entry is one missing field of one pack that changed a verdict to Unknown. The UI turns it into "help add this" links.

## 10. UI

Routes: `#/build` (list, templates, start empty), `#/build/<local id>`, and `#/build/import/<payload>` ([11](#11-saving-sharing-privacy)). Linked from Home as a second entry next to the track, and from each mission's debrief ("Try it with your own parts").

### 10.1 Layout

Desktop (≥ 1024px): parts palette left, board centre, report right; the inspector opens over the report when a node is selected. 768–1023px: palette collapses to a drawer. Under 768px: one panel at a time with a bottom switcher **Parts · Board · Goals · Report**, as the mission screen does. 360px works with no horizontal scroll.

### 10.2 Parts palette

- Search box (label and `aka`), category chips (Computers, Network, Power, Storage, Adapters, Clients), filters: **Teaching models / Real devices**, **Sold new in India / Common used / All**.
- Each row: label, one-line key facts from capabilities (arch, RAM, wired ports, Wi-Fi), a badge for teaching model vs real device, and the availability label. **No compatibility badge**: the palette never pre-judges.
- Tapping a computer, router, switch or power unit adds it to the board. Tapping a drive, adapter or hub adds it to the selected machine (or asks which machine). Keyboard: Enter adds, arrows move.
- **"My device isn't listed"** opens a short form (category, arch, RAM, wired Ethernet yes/no/don't know, Wi-Fi, battery, rough idle watts if known). It creates a `custom_devices` entry with `kind: learner`. Every field defaults to unknown. Verdicts using it say "Based on what you entered". A link offers to contribute it as a real pack (opens the GitHub issue form; see [11](#11-saving-sharing-privacy)).

### 10.3 Board

The Phase 5 board, with free placement:

- New nodes land at the next free slot of a deterministic grid in the current layout (wide or narrow). The learner can drag to move. Positions are UI state, stored in the build, never sent to the core.
- **Tap a port, then tap a port** to connect. A cable picker asks Ethernet or Wi-Fi when both ends allow it. Drag-to-connect is an extra.
- Power feeds draw as a separate dashed line style and can be hidden ("Show power").
- Fixed scaffolding (internet, away phone, mobile network) is locked: it can be moved, not deleted.
- Node cards show the status of the worst intent touching them (icon + word, never colour alone) and a RAM bar for machines.
- Phone layout follows the Phase 1 finding: at most three nodes across, stacked rows.

### 10.4 Inspector

Tabs per node kind:

| Node | Tabs |
|---|---|
| Machine | **Device** (pack, swap, notes, sources) · **System** (OS, runtime) · **Parts** (mounts, with slot picker) · **Guests** (VM / LXC / container with allocations) · **Services** (from service packs; ports filled from defaults, editable) · **Settings** (lid closed, charge limit, power-loss recovery, always on, busy share) · **Price** |
| Router | Device (ISP-supplied or retail pack) · Port forwards (existing control) · DNS and DHCP · Price |
| Power unit | Device · Feeds (checklist of nodes) · Price |
| ISP | Profile or line check (existing) |

### 10.5 Goals panel

Plain-language cards that create intents: "Watch at home on the TV", "Watch from my phone on mobile data", "Back up phone photos", "Ad-blocking for the whole house", "Keep files (how many GB, how many copies)", "Survive a power cut", "Keep yearly electricity under ₹…". Each card asks only for the missing parameters, choosing sensible defaults from the board (the only media server, the only phone). A goal whose parameters cannot be filled yet shows what to add ("Add a media server first").

### 10.6 Report panel

Top to bottom:

1. **Will it work?** One row per intent: status icon + word, the goal text, the one-line reason. Tap opens the findings under it; reach and stream intents have **Show the path**, which plays the trace with the existing player.
2. **What to fix first.** The top three failing findings across intents, ranked as Design.md ranks findings, each with its explanation.
3. **Fit.** Per machine: RAM bar with terms, bays, USB, Ethernet ports.
4. **Power and cost.** Average watts per machine, kWh a year, ₹ a year with the rate used and its source ("BESCOM LT domestic, tariff order 2026, retrieved …"), the slab explanation, and every assumption with its value. UPS runtime vs the chosen outage.
5. **Parts list.** Every part, owned or to buy, with the learner's prices and the total. Heading: "Prices you entered. We don't ship prices."
6. **Risks.** Debrief-style findings: single copy of data, battery always charging, service exposed to the internet, Wi-Fi-only server.
7. **What we don't know.** From `unknowns`: "We don't know whether this router lets you change DNS. That's why 'Ad-blocking for the whole house' says can't tell." Each with **Help add this**.

Verdict wording follows Design.md's table, plus: Learner provenance → "Based on what you entered". The summary words are **Works in this model**, **Won't work**, **Can't tell yet**. Never "Compatible", never "Will work" without the provenance qualifier.

### 10.7 Motion

No new animation kinds. Trace playback reuses [Animations.md](Animations.md). Adding a node uses `dur.fast` fade and scale from 0.96; status changes after an edit cross-fade the icon only. Reduced motion: none of these move.

## 11. Saving, sharing, privacy

- **Storage:** `localStorage` key `hls.builds.v1`, an array of `{ id, name, created, updated, schema_version, world, layout }`. Every access in try/catch; the page works without it (build held in memory, with a notice).
- **Export / import:** JSON file download and file picker. Import validates through the core (`validate_world_json`, a new WASM export that returns schema errors) before loading.
- **Share link:** "Copy link" deflates the build (`CompressionStream('deflate-raw')`) and base64url-encodes it into `#/build/import/<payload>`. The fragment is never sent to any server. Before copying, a checkbox **Include my prices** (default off) strips `price` fields. Over 8 KB of payload, the link option is replaced by export-to-file.
- **No IP addresses** are ever asked for, as with the line check. Builds contain addresses only from the sim's own address plan.
- **Help add this data** opens a GitHub issue form URL with only the pack id and field name as query parameters, never the build, prices or anything about the learner. It opens in a new tab and the learner submits it themselves.
- **No analytics** on builds. Nothing about a build leaves the browser unless the learner exports or shares it.

## 12. Templates

`content/sandbox/templates/<id>.yaml` (CC BY-SA 4.0), validated by the data build:

```yaml
id: old-laptop-media-server
title: Old laptop as a media server
summary: A laptop you already own, Jellyfin, and watching from your phone on mobile data.
world: { … }        # nodes, links, intents, home plan
layout: { wide: { … }, narrow: { … } }
```

Every template, and every empty build, contains the **away scaffolding**: `internet`, `mobile-network` (an ISP node with a teaching mobile profile until real mobile profiles are sourced) and `away-phone` (a `role: client` host). Ids are fixed so intents in templates can name them.

Launch templates, each using archetypes until the matching real packs exist (the template file is updated when they land; no code change):

| Id | Shows |
|---|---|
| `old-laptop-media-server` | Jellyfin, transcoding unknowns, CGNAT remote access, laptop battery as UPS, battery always charging |
| `macbook-air-as-server` | No Ethernet port → USB adapter; macOS runtime runs containers in a VM (RAM overhead, host-network limits); Asahi Linux `partial`; lid-closed running; no x86 VMs |
| `pi-hole-whole-house` | SBC, Pi-hole, ISP router that may not allow DNS changes, `home-dns-possible` |
| `mini-pc-proxmox` | Hypervisor, LXC and VM allocations, RAM budget |
| `old-phone-tiny-server` | Android with Termux: what runs, what doesn't, always-charging battery |
| `power-cut-proof` | UPS feeding the server but not the ONT/router: remote access dies first; DC mini UPS fix; yearly cost by state |

## 13. Data build and bundle split

- `tools/data-build` reads the new directories: `data/devices/` (all categories), `data/os/`, `data/runtimes/`, `data/services/`, `data/power/tariffs/`, `data/power/outages/`, `data/reference/`, `content/sandbox/templates/`.
- Checks: id = file name; category section allow-list; every `kind: device` value sourced or `unknown`; `basis` allowed values; every id referenced (OS in `os_support`, runtime in `native_runtimes`, service in `apps`, tariff, outage, packs in templates) exists; templates evaluate without `evaluation-did-not-settle`; tariff slabs ascending with exactly one open-ended last slab; no URL query strings carrying tracking parameters (`utm_`, `tag=`, `affid`) in sources.
- **Bundle split.** The mission bundle (`bundle.json`) keeps only the packs missions reference. Everything else goes to `catalog.json`, fetched only on `#/build`. The WASM gets a new `add_catalog_json` that merges it into the loaded bundle. Budgets: mission bundle as today; catalog under 250 KB gzipped at 500 packs (measure and record in Handoff when S5 lands).
- **One-file tests**, one per category plus OS, runtime, service, tariff, outage and template: add one YAML file in a temp copy of the repo, rebuild, and show a rule's answer changes with no code change. Extends `tools/data-build/tests/missions.rs`.
- JSON Schema generation (planned for Phase 3) covers every new pack type, so contributors get editor validation. Data agents need it, so S1 builds it.

## 14. Build phases S1 to S5

Each ends with `npm run check` green. Owner decision of 2026-11-11: build after Phase 5 lands (S4 reuses its board). S1 to S3 touch only the core and data build and may start as soon as the uncommitted Phase 5 work is committed, to avoid conflicts in `bundle.rs` and `types.ts`.

### S1: Schema v4 and data build

Cap `basis`, `PackKind::Learner`, `Category`, all section types, OS/runtime/service/tariff/outage/reference/template pack types, `Cap<Vec<_>>` for nic lists, migration of the three archetypes and missions, JSON Schema generation, category allow-lists and the checks in [13](#13-data-build-and-bundle-split), bundle/catalog split, TypeScript mirror.

**Done =** mission reports byte-identical to before (fixture test), every one-file test passes, invalid packs fail with file and field, JSON Schemas generated into `schemas/`.

### S2: World additions, intents, fit rules

World fields from [5](#5-world-additions), `Switch` in facts and walk, intent expansion, `fit.rs`, `budget.rs` RAM/bays/USB/ports, connect rules from 7.3, `BuildReport` minus power, `validate_world_json` and `add_catalog_json` WASM exports.

**Done =** fixture builds in `crates/sim-core/tests/builds/` cover every rule in 7.1–7.3 with pass, fail and unknown each; `macbook-air-as-server` and `pi-hole-whole-house` give the expected intent statuses; determinism tests extended to builds.

### S3: Power

Tariff slabs, duty, free units, average watts, UPS runtime (chart and fallback), laptop battery, outage survival, recovery, the 7.4 rules, `PowerSummary` and `UpsSummary`.

**Done =** hand-worked examples for two tariffs (one with free units) match to the paisa; a UPS that feeds the server but not the router fails `outage-covered` at the router; every assumption constant appears in `assumptions`.

### S4: Sandbox UI

Routes, palette, free board, tap-to-connect, inspector tabs, goals panel, report panel, custom device form, save/export/import/share, entry points from Home and debriefs. Uses `impeccable` and `design-taste-frontend` skills, as Design.md asks.

**Done =** a build can be made from empty to a full report at 360px and desktop, by keyboard and in reduced motion; Playwright covers: template to report, a failing intent fixed by adding a part, unknown shown with its "help add this" link, share link round trip with prices stripped, storage unavailable.

### S5: Templates and first real data

The six templates, steps.yaml text for every new reason, the ★ long explanations, and the P0 items of [Data-Backlog.md](Data-Backlog.md) (enough real packs that the templates can swap at least one archetype each for a real device).

**Done =** every template evaluates with no `intent-not-checked`; catalog size recorded; every real pack sourced; Handoff updated.

## 15. Model limits stated to the learner

Shown on the report's "About these results" disclosure, and in the relevant explanation:

- CPU speed is not modelled. Transcoding without hardware support, game servers and AI workloads say "CPU speed not modelled".
- Average power uses an assumed busy share unless the learner sets one. Real use varies.
- Slab tariffs are applied as published on the date shown. Fixed charges and subsidies other than listed free-unit schemes are left out.
- UPS runtime uses the maker's chart when we have it, else an estimate from battery size. Old batteries hold less.
- Wi-Fi is modelled as working at its nominal rate. Walls, distance and neighbours are not modelled.
- Verdicts about real devices are only as good as their sources, which are shown with dates.
