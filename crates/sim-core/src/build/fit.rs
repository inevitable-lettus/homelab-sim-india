//! Will it fit: memory, drive bays, USB ports, Ethernet ports, storage.

use super::run::wired_ports;
use super::{Budget, Cx, MachineBudget, Read, Subject, Term, Verdict, has_category, origin};
use crate::model::{GuestKind, Host, Intent, Mount, Node};
use crate::packs::{Bay, BayKind, Category, DevicePack, UsbKind, UsbPort, WifiGen};
use crate::physics::Medium;
use crate::reason::Reason;

pub(super) fn rules(cx: &mut Cx) -> Vec<MachineBudget> {
    let mut machines = Vec::new();
    for host in cx.servers().into_iter().filter(|h| h.runs_on.is_none()) {
        let ram = ram(cx, host);
        let bays = bays(cx, host);
        let usb = usb(cx, host);
        let ports = ethernet(cx, host);
        os_storage(cx, host);
        machines.push(MachineBudget { node: host.id.clone(), ram, bays, usb, ports, avg_w: None });
    }
    switch_ports(cx);
    wifi(cx);
    let world = cx.world;
    for intent in &world.intents {
        if let Intent::Store { id, host, gb, copies } = intent {
            store(cx, id, host, u64::from(*gb), usize::from(*copies).max(1));
        }
    }
    machines
}

/// A sum that remembers whether every term was known.
#[derive(Default)]
struct Sum {
    known: u64,
    complete: bool,
}

impl Sum {
    fn new() -> Self {
        Self { known: 0, complete: true }
    }

    fn add(&mut self, value: Option<u64>) {
        match value {
            Some(value) => self.known += value,
            None => self.complete = false,
        }
    }

    fn total(&self) -> Option<u64> {
        self.complete.then_some(self.known)
    }
}

fn ram(cx: &mut Cx, host: &Host) -> Budget {
    let mut read = Read::default();
    let mut terms = Vec::new();
    let (mut min, mut rec) = (Sum::new(), Sum::new());
    let mut term = |label: &str, at_min: Option<u64>, at_rec: Option<u64>| {
        terms.push(Term { label: label.to_owned(), value: at_min });
        min.add(at_min);
        rec.add(at_rec.or(at_min));
    };
    let service_ram = |read: &mut Read, host: &Host| -> Vec<(String, Option<u64>, Option<u64>)> {
        host.services
            .iter()
            .map(|service| {
                let pack = service.app.as_deref().and_then(|app| cx.bundle.services.get(app));
                let sizes = pack.map(|pack| {
                    let from = origin(pack.teaching);
                    (
                        read.cap(from, &pack.id, "ram_min_mb", &pack.ram_min_mb).map(u64::from),
                        pack.ram_rec_mb.copied().map(u64::from),
                    )
                });
                let (at_min, at_rec) = sizes.unwrap_or((None, None));
                (service.label.clone(), at_min, at_rec)
            })
            .collect()
    };

    match cx.os(host) {
        Some(os) => {
            let base = read.cap(origin(os.teaching), &os.id, "base_ram_mb", &os.base_ram_mb).map(u64::from);
            term(&os.label, base, None);
        }
        None => term("Operating system", None, None),
    }
    for (label, at_min, at_rec) in service_ram(&mut read, host) {
        term(&label, at_min, at_rec);
    }
    if let Some(runtime) = host.runtime.as_deref().and_then(|id| cx.bundle.runtimes.get(id)) {
        let from = origin(runtime.teaching);
        // A runtime that hides a Linux VM takes memory before any container starts.
        match read.cap(from, &runtime.id, "via_vm", &runtime.via_vm) {
            Some(false) => {}
            Some(true) => {
                let overhead = read.cap(from, &runtime.id, "vm_overhead_mb", &runtime.vm_overhead_mb);
                term(&runtime.label, overhead.map(u64::from), None);
            }
            None => term(&runtime.label, None, None),
        }
    }
    let guests = cx.guests_of(host);
    let mut allocated = Sum::new();
    for guest in &guests {
        let given = guest.alloc.and_then(|a| a.ram_mb).map(u64::from);
        allocated.add(given);
        match (given, guest.guest) {
            (Some(mb), _) => term(&guest.label, Some(mb), None),
            // A container with no limit takes what its services take.
            (None, Some(GuestKind::Container)) => {
                for (label, at_min, at_rec) in service_ram(&mut read, guest) {
                    term(&label, at_min, at_rec);
                }
            }
            (None, _) => term(&guest.label, None, None),
        }
    }

    let device = cx.pack_of(&host.device);
    let total = device.and_then(|d| {
        let compute = d.compute.clone().unwrap_or_default();
        read.cap(d.origin(), &d.id, "compute.ram_gb", &compute.ram_gb).map(|gb| u64::from(gb) * 1024)
    });
    let verdict = match total {
        Some(total) if min.known > total => Verdict::Fail,
        Some(total) if min.complete && rec.known > total => Verdict::Warn,
        Some(_) if min.complete => Verdict::Pass,
        _ => Verdict::Unknown,
    };
    let (used, recommended) = (min.total(), rec.total().filter(|r| Some(*r) != min.total()));
    cx.emit(Reason::RamBudget, verdict, vec![host.id.clone()], read, Subject::node(&host.id));

    if guests.iter().any(|g| g.alloc.is_some_and(|a| a.ram_mb.is_some())) {
        let mut read = Read::default();
        let base = cx.os(host).and_then(|os| os.base_ram_mb.copied()).map_or(0, u64::from);
        let device_total = device.and_then(|d| {
            let compute = d.compute.clone().unwrap_or_default();
            read.cap(d.origin(), &d.id, "compute.ram_gb", &compute.ram_gb).map(|gb| u64::from(gb) * 1024)
        });
        let verdict = match device_total {
            Some(total) if allocated.known + base > total => Verdict::Fail,
            Some(_) => Verdict::Pass,
            None => Verdict::Unknown,
        };
        cx.emit(Reason::GuestAllocExceedsHost, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
    }
    Budget { used, recommended, total, terms }
}

fn parts<'a>(cx: &Cx<'a>, host: &'a Host, categories: &[Category]) -> Vec<(&'a Mount, &'a DevicePack)> {
    host.parts
        .iter()
        .filter_map(|mount| cx.device(&mount.pack).map(|pack| (mount, pack)))
        .filter(|(_, pack)| has_category(pack, categories))
        .collect()
}

fn fits(drive: BayKind, bay: BayKind) -> bool {
    drive == bay || (drive == BayKind::Sata25 && bay == BayKind::Sata35)
}

/// Put drives into slots. `None` when something needed is unknown.
fn place(slots: Option<Vec<Bay>>, drives: &[Option<BayKind>]) -> Option<bool> {
    let mut slots = slots?;
    let mut ok = true;
    for drive in drives {
        let drive = (*drive)?;
        // An exact slot first, so a small drive does not take a big bay it does not need.
        let slot = slots
            .iter()
            .position(|s| s.count > 0 && s.interface == drive)
            .or_else(|| slots.iter().position(|s| s.count > 0 && fits(drive, s.interface)));
        match slot {
            Some(i) => slots[i].count -= 1,
            None => ok = false,
        }
    }
    Some(ok)
}

fn bays(cx: &mut Cx, host: &Host) -> Budget {
    let drives = parts(cx, host, &[Category::Drive]);
    let device = cx.pack_of(&host.device);
    let mut read = Read::default();
    let own = device.and_then(|d| {
        let storage = d.storage.clone().unwrap_or_default();
        read.cap(d.origin(), &d.id, "storage.bays", &storage.bays)
    });
    let total = own.as_ref().map(|bays| bays.iter().map(|b| u64::from(b.count)).sum());
    let inside = drives.iter().filter(|(mount, _)| mount.into.is_none()).count() as u64;
    if drives.is_empty() {
        return Budget { used: Some(0), total, ..Budget::default() };
    }
    let interface = |read: &mut Read, pack: &DevicePack| {
        let drive = pack.drive.clone().unwrap_or_default();
        read.cap(pack.origin(), &pack.id, "drive.interface", &drive.interface)
    };
    // The machine's own bays, then each enclosure's.
    let mut homes: Vec<(Option<&str>, Option<Vec<Bay>>)> = vec![(None, own)];
    for (mount, pack) in parts(cx, host, &[Category::Enclosure]) {
        let adapter = pack.adapter.clone().unwrap_or_default();
        let slots = read.cap(pack.origin(), &pack.id, "adapter.provides.bays", &adapter.provides.bays);
        homes.push((Some(&mount.id), slots));
    }
    let mut verdict = Verdict::Pass;
    for (home, slots) in homes {
        let here: Vec<Option<BayKind>> = drives
            .iter()
            .filter(|(mount, _)| mount.into.as_deref() == home)
            .map(|(_, pack)| interface(&mut read, pack))
            .collect();
        if here.is_empty() {
            continue;
        }
        match place(slots, &here) {
            Some(false) => verdict = Verdict::Fail,
            None if verdict == Verdict::Pass => verdict = Verdict::Unknown,
            _ => {}
        }
    }
    // A drive pointed at a mount that is not an enclosure has nowhere to go.
    let enclosures = parts(cx, host, &[Category::Enclosure]);
    if drives.iter().any(|(m, _)| m.into.as_ref().is_some_and(|into| !enclosures.iter().any(|(e, _)| e.id == *into))) {
        verdict = Verdict::Fail;
    }
    cx.emit(Reason::BayAvailable, verdict, vec![host.id.clone()], read, Subject::node(&host.id));

    for (mount, drive) in drives.iter().filter(|(m, _)| m.into.is_some()) {
        let Some((_, enclosure)) = enclosures.iter().find(|(e, _)| Some(&e.id) == mount.into.as_ref()) else {
            continue;
        };
        let mut read = Read::default();
        let caps = drive.drive.clone().unwrap_or_default();
        let adapter = enclosure.adapter.clone().unwrap_or_default();
        let needs_12v = read.cap(drive.origin(), &drive.id, "drive.needs_12v", &caps.needs_12v);
        let verdict = match needs_12v {
            Some(false) => Verdict::Pass,
            // A 3.5 inch drive needs more than a USB port gives.
            Some(true) => {
                match read.cap(enclosure.origin(), &enclosure.id, "adapter.needs_power", &adapter.needs_power) {
                    Some(true) => Verdict::Pass,
                    Some(false) => Verdict::Fail,
                    None => Verdict::Unknown,
                }
            }
            None => Verdict::Unknown,
        };
        cx.emit(Reason::DrivePower, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
    }
    if host.services.iter().any(|s| s.app.is_some()) {
        for (mount, enclosure) in
            enclosures.iter().filter(|(e, _)| drives.iter().any(|(d, _)| d.into.as_ref() == Some(&e.id)))
        {
            let _ = mount;
            let mut read = Read::default();
            let adapter = enclosure.adapter.clone().unwrap_or_default();
            let verdict = match read.cap(enclosure.origin(), &enclosure.id, "adapter.uas", &adapter.uas) {
                Some(true) => Verdict::Pass,
                Some(false) => Verdict::Warn,
                None => Verdict::Unknown,
            };
            cx.emit(Reason::EnclosureUas, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
        }
    }
    Budget { used: Some(inside), total, ..Budget::default() }
}

fn usb_mbps(kind: UsbKind) -> u32 {
    match kind {
        UsbKind::Usb2A => 480,
        UsbKind::Usb5A | UsbKind::Usb5C => 5_000,
        UsbKind::Usb10A | UsbKind::Usb10C => 10_000,
        UsbKind::Usb40C => 40_000,
    }
}

fn usb(cx: &mut Cx, host: &Host) -> Budget {
    let plugs = parts(cx, host, &[Category::Enclosure, Category::UsbNic, Category::UsbHub]);
    let device = cx.pack_of(&host.device);
    let mut read = Read::default();
    let own: Option<Vec<UsbPort>> = device.and_then(|d| {
        let ports = d.ports.clone().unwrap_or_default();
        read.cap(d.origin(), &d.id, "ports.usb", &ports.usb)
    });
    let count =
        |ports: &Option<Vec<UsbPort>>| ports.as_ref().map(|p| p.iter().map(|p| u64::from(p.count)).sum::<u64>());
    let total = count(&own);
    let direct = plugs.iter().filter(|(mount, _)| mount.into.is_none()).count() as u64;
    if plugs.is_empty() {
        return Budget { used: Some(0), total, ..Budget::default() };
    }
    let mut verdict = match total {
        Some(total) if direct > total => Verdict::Fail,
        Some(_) => Verdict::Pass,
        None => Verdict::Unknown,
    };
    for (hub_mount, hub) in parts(cx, host, &[Category::UsbHub]) {
        let on_hub = plugs.iter().filter(|(mount, _)| mount.into.as_ref() == Some(&hub_mount.id)).count() as u64;
        if on_hub == 0 {
            continue;
        }
        let adapter = hub.adapter.clone().unwrap_or_default();
        match count(&read.cap(hub.origin(), &hub.id, "adapter.provides.usb", &adapter.provides.usb)) {
            Some(ports) if on_hub > ports => verdict = Verdict::Fail,
            None if verdict == Verdict::Pass => verdict = Verdict::Unknown,
            _ => {}
        }
    }
    if verdict == Verdict::Pass {
        // A fast adapter on a slow port works, at the port's speed.
        let fastest = own.iter().flatten().map(|p| usb_mbps(p.kind)).max().unwrap_or(0);
        let wants = plugs
            .iter()
            .filter_map(|(_, pack)| pack.adapter.as_ref()?.host_side.copied())
            .map(usb_mbps)
            .max()
            .unwrap_or(0);
        if wants > fastest {
            verdict = Verdict::Warn;
        }
    }
    cx.emit(Reason::UsbPortAvailable, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
    Budget { used: Some(direct), total, ..Budget::default() }
}

fn links_of<'a>(cx: &Cx<'a>, node: &str, medium: Medium) -> Vec<&'a crate::model::Link> {
    cx.world.links.iter().filter(|l| l.medium == medium && (l.a == node || l.b == node)).collect()
}

fn ethernet(cx: &mut Cx, host: &Host) -> Budget {
    let wired = links_of(cx, &host.id, Medium::Ethernet).len() as u64;
    let wifi = links_of(cx, &host.id, Medium::Wifi).len() as u64;
    let mut read = Read::default();
    let total = wired_ports(cx, host, &mut read);
    if wired > 0 {
        let verdict = match total {
            Some(ports) if wired > ports => Verdict::Fail,
            Some(_) => Verdict::Pass,
            None => Verdict::Unknown,
        };
        cx.emit(Reason::NoEthernetPort, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
    } else if wifi > 0 && (host.os.is_some() || !host.services.is_empty()) {
        cx.emit(Reason::WifiOnlyServer, Verdict::Warn, vec![host.id.clone()], Read::default(), Subject::node(&host.id));
        cx.risk();
    }
    Budget { used: Some(wired), total, ..Budget::default() }
}

/// Cables into a router or a switch from the home side, against its ports.
fn switch_ports(cx: &mut Cx) {
    let net = cx.net;
    for &i in net.idx.values() {
        let (id, device, uplinks) = match &cx.world.nodes[i] {
            Node::Router(router) => (&router.id, &router.device, 0),
            // One port of a switch goes up to the router.
            Node::Switch(switch) => (&switch.id, &switch.device, 1),
            _ => continue,
        };
        let below = net.adj[i]
            .iter()
            .filter(|&&(link, nb)| net.up[nb] == Some((link, i)) && cx.world.links[link].medium == Medium::Ethernet)
            .count() as u64;
        let Some(pack) = cx.pack_of(device).filter(|_| below > 0) else {
            continue;
        };
        let mut read = Read::default();
        let ports = match &pack.router {
            Some(router) => read.cap(pack.origin(), &pack.id, "router.lan", &router.lan),
            None => {
                let network = pack.network.clone().unwrap_or_default();
                read.cap(pack.origin(), &pack.id, "network.nic", &network.nic)
            }
        };
        let verdict = match ports {
            Some(ports) if below + uplinks > ports.len() as u64 => Verdict::Fail,
            Some(_) => Verdict::Pass,
            None => Verdict::Unknown,
        };
        cx.emit(Reason::EthernetPortsAvailable, verdict, vec![id.clone()], read, Subject::none());
    }
}

fn wifi(cx: &mut Cx) {
    let world = cx.world;
    for link in world.links.iter().filter(|l| l.medium == Medium::Wifi) {
        let mut read = Read::default();
        let mut verdict = Verdict::Pass;
        let mut subject = Subject::none();
        for end in [&link.a, &link.b] {
            let device = match world.node(end) {
                Some(Node::Host(host)) => {
                    // The away phone and other scaffolding have no pack and nothing to check.
                    if host.device.is_none() {
                        continue;
                    }
                    subject = Subject::node(&host.id);
                    &host.device
                }
                Some(Node::Router(router)) => &router.device,
                Some(Node::Switch(switch)) => &switch.device,
                _ => continue,
            };
            let radio = cx.pack_of(device).and_then(|pack| {
                let network = pack.network.clone().unwrap_or_default();
                read.cap(pack.origin(), &pack.id, "network.wifi", &network.wifi)
            });
            match radio {
                Some(WifiGen::None) => verdict = Verdict::Fail,
                None if verdict == Verdict::Pass => verdict = Verdict::Unknown,
                _ => {}
            }
        }
        cx.emit(Reason::WifiAvailable, verdict, vec![link.a.clone(), link.b.clone()], read, subject);
    }
}

/// Each place data can live on a machine, in GB. `None` is a size we do not know.
fn volumes(cx: &Cx, host: &Host, read: &mut Read) -> Vec<Option<u64>> {
    let mut found = Vec::new();
    match cx.pack_of(&host.device) {
        Some(device) => match &device.storage {
            Some(storage) => match read.cap(device.origin(), &device.id, "storage.soldered_gb", &storage.soldered_gb) {
                Some(0) => {}
                other => found.push(other.map(u64::from)),
            },
            None => {
                read.missing(&device.id, "storage");
                found.push(None);
            }
        },
        None => found.push(None),
    }
    for (_, pack) in parts(cx, host, &[Category::Drive]) {
        let drive = pack.drive.clone().unwrap_or_default();
        found.push(read.cap(pack.origin(), &pack.id, "drive.capacity_gb", &drive.capacity_gb).map(u64::from));
    }
    found
}

fn os_storage(cx: &mut Cx, host: &Host) {
    let Some(os) = cx.os(host) else {
        return;
    };
    let mut read = Read::default();
    let need = read.cap(origin(os.teaching), &os.id, "requires.storage_min_gb", &os.requires.storage_min_gb);
    let volumes = volumes(cx, host, &mut read);
    let known: u64 = volumes.iter().flatten().sum();
    let verdict = match need {
        None => Verdict::UnknownNote,
        Some(need) if known >= u64::from(need) => Verdict::Pass,
        Some(_) if volumes.iter().any(Option::is_none) => Verdict::Unknown,
        Some(_) => Verdict::Fail,
    };
    cx.emit(Reason::OsStorage, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
}

fn store(cx: &mut Cx, intent: &str, host: &str, gb: u64, copies: usize) {
    let Some(host) = cx.world.host(host) else {
        let read = Read::default();
        return cx.emit_as(
            Reason::StorageCapacity,
            Verdict::Unknown,
            Reason::UnknownNode,
            vec![host.to_owned()],
            read,
            Subject::intent(intent),
        );
    };
    let machine = cx.root(host);
    let mut read = Read::default();
    let volumes = volumes(cx, machine, &mut read);
    let unknown = volumes.iter().any(Option::is_none);
    let enough = if copies <= 1 {
        volumes.iter().flatten().sum::<u64>() >= gb
    } else {
        // Each copy on its own drive, or it is not a second copy.
        volumes.iter().flatten().filter(|size| **size >= gb).count() >= copies
    };
    let verdict = match (enough, unknown) {
        (true, _) => Verdict::Pass,
        (false, true) => Verdict::Unknown,
        (false, false) => Verdict::Fail,
    };
    cx.emit(Reason::StorageCapacity, verdict, vec![machine.id.clone()], read, Subject::intent(intent));
    if copies <= 1 {
        cx.emit(Reason::SingleCopy, Verdict::Note, vec![machine.id.clone()], Read::default(), Subject::intent(intent));
        cx.risk();
    }
}
