//! Will it run: can this OS boot here, can this guest exist, is there a
//! build of this service for this machine.

use super::{Cx, Read, Subject, Verdict, origin};
use crate::model::{GuestKind, Host, Service};
use crate::packs::{Firmware, HostNetwork, InstallMethod, OsFamily, PackKind, Support};
use crate::reason::Reason;

pub(super) fn rules(cx: &mut Cx) {
    for host in cx.servers() {
        if host.runs_on.is_none() {
            os_boots(cx, host);
            os_requirements(cx, host);
            if cx.pack_of(&host.device).is_some_and(|pack| pack.kind == PackKind::Learner) {
                let mut read = Read::default();
                read.plain(crate::packs::Origin::Learner);
                cx.emit(Reason::LearnerValueUsed, Verdict::Note, vec![host.id.clone()], read, Subject::node(&host.id));
            }
        } else {
            guest(cx, host);
        }
        runtime(cx, host);
        for service in &host.services {
            service_available(cx, host, service);
        }
    }
}

fn boots(device: Firmware, os: &[Firmware]) -> bool {
    match device {
        Firmware::UefiAndBios => {
            os.iter().any(|f| matches!(f, Firmware::Uefi | Firmware::Bios | Firmware::UefiAndBios))
        }
        other => {
            os.contains(&other)
                || (matches!(other, Firmware::Uefi | Firmware::Bios) && os.contains(&Firmware::UefiAndBios))
        }
    }
}

fn os_boots(cx: &mut Cx, host: &Host) {
    let rule = Reason::OsBootsOnDevice;
    let mut read = Read::default();
    let involved = vec![host.id.clone()];
    let subject = Subject::node(&host.id);
    let (Some(os_id), Some(device)) = (&host.os, cx.pack_of(&host.device)) else {
        // Nothing chosen yet, or a device we have no pack for.
        if let Some(device) = host.device.as_deref().filter(|_| host.os.is_some()) {
            read.missing(device, "pack");
        }
        return cx.emit(rule, Verdict::Unknown, involved, read, subject);
    };
    let Some(os) = cx.bundle.os.get(os_id) else {
        read.missing(os_id, "pack");
        return cx.emit(rule, Verdict::Unknown, involved, read, subject);
    };
    read.plain(origin(os.teaching));
    let field = format!("os_support.{os_id}");
    let verdict = if let Some(entry) = device.os_support.get(os_id) {
        // The device speaks for itself about this OS.
        match read.cap(device.origin(), &device.id, &field, entry) {
            Some(support) => match support.level {
                Support::Full => Verdict::Pass,
                Support::Partial => Verdict::Warn,
                Support::None => Verdict::Fail,
            },
            None => Verdict::Unknown,
        }
    } else if os.listed_devices_only {
        read.missing(&device.id, &field);
        Verdict::Unknown
    } else {
        let compute = device.compute.clone().unwrap_or_default();
        let arch = read.cap(device.origin(), &device.id, "compute.arch", &compute.arch);
        let arch_ok = arch.map(|arch| os.arches.contains(&arch));
        let firmware_ok = if os.firmware.is_empty() {
            Some(true)
        } else {
            read.cap(device.origin(), &device.id, "compute.firmware", &compute.firmware)
                .map(|firmware| boots(firmware, &os.firmware))
        };
        match (arch_ok, firmware_ok) {
            (Some(false), _) | (_, Some(false)) => Verdict::Fail,
            (Some(true), Some(true)) => Verdict::Pass,
            _ => Verdict::Unknown,
        }
    };
    cx.emit(rule, verdict, involved, read, subject);
}

/// How one requirement compares with the machine.
enum Need {
    Met,
    Short,
    MachineUnknown,
    NotStated,
}

/// Wired Ethernet ports on a machine: built in, plus USB adapters. `None`
/// when the built-in count is unknown and no adapter settles it.
pub(super) fn wired_ports(cx: &Cx, host: &Host, read: &mut Read) -> Option<u64> {
    let adapters: u64 = host
        .parts
        .iter()
        .filter_map(|mount| cx.device(&mount.pack))
        .filter_map(|pack| pack.adapter.as_ref()?.provides.nic.get().map(|nics| nics.len() as u64))
        .sum();
    let device = cx.pack_of(&host.device)?;
    let net = device.network.clone().unwrap_or_default();
    match read.cap(device.origin(), &device.id, "network.nic", &net.nic) {
        Some(nics) => Some(nics.len() as u64 + adapters),
        None if adapters > 0 => Some(adapters),
        None => None,
    }
}

fn os_requirements(cx: &mut Cx, host: &Host) {
    let (Some(os), Some(device)) = (cx.os(host), cx.pack_of(&host.device)) else {
        return;
    };
    let mut read = Read::default();
    let compute = device.compute.clone().unwrap_or_default();
    let (from_os, from_device) = (origin(os.teaching), device.origin());
    let need_0 = match read.cap(from_os, &os.id, "requires.virtualization", &os.requires.virtualization) {
        None => Need::NotStated,
        Some(false) => Need::Met,
        Some(true) => match read.cap(from_device, &device.id, "compute.virtualization", &compute.virtualization) {
            Some(true) => Need::Met,
            Some(false) => Need::Short,
            None => Need::MachineUnknown,
        },
    };
    let need_1 = match read.cap(from_os, &os.id, "requires.ram_min_mb", &os.requires.ram_min_mb) {
        None => Need::NotStated,
        Some(min) => match read.cap(from_device, &device.id, "compute.ram_gb", &compute.ram_gb) {
            Some(gb) if u64::from(gb) * 1024 >= u64::from(min) => Need::Met,
            Some(_) => Need::Short,
            None => Need::MachineUnknown,
        },
    };
    let need_2 = match read.cap(from_os, &os.id, "requires.wired_nic", &os.requires.wired_nic) {
        None => Need::NotStated,
        Some(false) => Need::Met,
        Some(true) => match wired_ports(cx, host, &mut read) {
            Some(0) => Need::Short,
            Some(_) => Need::Met,
            None => Need::MachineUnknown,
        },
    };
    let needs = [need_0, need_1, need_2];
    let any = |want: fn(&Need) -> bool| needs.iter().any(want);
    let verdict = if any(|n| matches!(n, Need::Short)) {
        Verdict::Fail
    } else if any(|n| matches!(n, Need::MachineUnknown)) {
        Verdict::Unknown
    } else if any(|n| matches!(n, Need::NotStated)) {
        Verdict::UnknownNote
    } else {
        Verdict::Pass
    };
    cx.emit(Reason::OsRequirementsMet, verdict, vec![host.id.clone()], read, Subject::node(&host.id));
}

fn guest(cx: &mut Cx, host: &Host) {
    let Some(parent) = cx.parent(host) else {
        return;
    };
    let root = cx.root(host);
    let kind = host.guest.unwrap_or(GuestKind::Vm);
    let involved = vec![host.id.clone(), parent.id.clone()];
    let device = cx.pack_of(&root.device);
    let compute = device.and_then(|d| d.compute.clone()).unwrap_or_default();

    if kind != GuestKind::Container {
        let mut read = Read::default();
        let provided = cx.os(parent).and_then(|os| {
            let (field, cap) = match kind {
                GuestKind::Lxc => ("provides.lxc", &os.provides.lxc),
                _ => ("provides.vms", &os.provides.vms),
            };
            read.cap(origin(os.teaching), &os.id, field, cap)
        });
        // Containers and LXC share the host's kernel. Only a VM needs the CPU's help.
        let hardware = match (kind, device) {
            (GuestKind::Vm, Some(d)) => read.cap(d.origin(), &d.id, "compute.virtualization", &compute.virtualization),
            (GuestKind::Vm, None) => None,
            _ => Some(true),
        };
        let verdict = match (provided, hardware) {
            (Some(false), _) | (_, Some(false)) => Verdict::Fail,
            (Some(true), Some(true)) => Verdict::Pass,
            _ => Verdict::Unknown,
        };
        cx.emit(Reason::HypervisorNeedsVirtualization, verdict, involved.clone(), read, Subject::node(&host.id));
    }

    if kind == GuestKind::Vm
        && let (Some(guest_os), Some(parent_os)) = (cx.os(host), cx.os(parent))
    {
        let mut read = Read::default();
        read.plain(origin(guest_os.teaching));
        read.plain(origin(parent_os.teaching));
        let native = device.and_then(|d| read.cap(d.origin(), &d.id, "compute.arch", &compute.arch));
        let accelerated = match (parent_os.provides.vm_accelerated_arches.is_empty(), native) {
            (false, _) => Some(parent_os.provides.vm_accelerated_arches.clone()),
            (true, Some(arch)) => Some(vec![arch]),
            (true, None) => None,
        };
        let verdict = match accelerated {
            None => Verdict::Unknown,
            Some(fast) if guest_os.arches.iter().any(|a| fast.contains(a)) => Verdict::Pass,
            // It runs, very slowly: every instruction is translated.
            Some(_) if guest_os.arches.iter().any(|a| parent_os.provides.vm_emulated_arches.contains(a)) => {
                Verdict::Warn
            }
            Some(_) => Verdict::Fail,
        };
        cx.emit(Reason::GuestArchEmulated, verdict, involved.clone(), read, Subject::node(&host.id));
    }

    if kind == GuestKind::Vm && parent.runs_on.is_some() && parent.guest.unwrap_or(GuestKind::Vm) == GuestKind::Vm {
        let mut read = Read::default();
        let nested = device
            .and_then(|d| read.cap(d.origin(), &d.id, "compute.nested_virtualization", &compute.nested_virtualization));
        let verdict = match nested {
            Some(true) => Verdict::Pass,
            Some(false) => Verdict::Fail,
            None => Verdict::Unknown,
        };
        cx.emit(Reason::NestedVirtualization, verdict, involved, read, Subject::node(&host.id));
    }
}

/// Where a host's containers run: on itself, or for a container guest, on
/// the machine it is a guest of.
pub(super) fn runtime_host<'a>(cx: &Cx<'a>, host: &'a Host) -> &'a Host {
    match (host.guest, cx.parent(host)) {
        (Some(GuestKind::Container), Some(parent)) => parent,
        _ => host,
    }
}

fn runtime(cx: &mut Cx, host: &Host) {
    let rule = Reason::RuntimeOnOs;
    let involved = vec![host.id.clone()];
    let mut read = Read::default();
    if host.guest == Some(GuestKind::Container) && host.runs_on.is_some() && runtime_host(cx, host).runtime.is_none() {
        // A container with nothing to run it.
        return cx.emit(rule, Verdict::Fail, involved, read, Subject::node(&host.id));
    }
    let Some(runtime_id) = &host.runtime else {
        return;
    };
    let Some(runtime) = cx.bundle.runtimes.get(runtime_id) else {
        read.missing(runtime_id, "pack");
        return cx.emit(rule, Verdict::Unknown, involved, read, Subject::node(&host.id));
    };
    read.plain(origin(runtime.teaching));
    let (family, _) = cx.platform(host, &mut read);
    let verdict = match family {
        Some(family) if runtime.os_families.contains(&family) => Verdict::Pass,
        Some(_) => Verdict::Fail,
        None => Verdict::Unknown,
    };
    cx.emit(rule, verdict, involved, read, Subject::node(&host.id));
}

pub(super) fn install_method(host: &Host, service: &Service) -> InstallMethod {
    let in_container = host.guest == Some(GuestKind::Container) && host.runs_on.is_some();
    service.install.unwrap_or(if in_container { InstallMethod::Container } else { InstallMethod::Native })
}

fn service_available(cx: &mut Cx, host: &Host, service: &Service) {
    let Some(app) = &service.app else {
        return;
    };
    let rule = Reason::ServiceAvailable;
    let involved = vec![host.id.clone()];
    let subject = || Subject::service(&host.id, &service.id);
    let mut read = Read::default();
    let Some(pack) = cx.bundle.services.get(app) else {
        read.missing(app, "pack");
        return cx.emit(rule, Verdict::Unknown, involved, read, subject());
    };
    let from = origin(pack.teaching);
    read.plain(from);
    let method = install_method(host, service);
    let runner = runtime_host(cx, host);
    let (family, arch) = cx.platform(host, &mut read);
    // A container image is a Linux image, whatever the machine runs.
    let family = if method == InstallMethod::Container { Some(OsFamily::Linux) } else { family };

    let ways: Vec<_> = pack.install.iter().enumerate().filter(|(_, way)| way.method == method).collect();
    let verdict = match family {
        _ if ways.is_empty() => Verdict::Fail,
        None => Verdict::Unknown,
        Some(family) => {
            let fitting: Vec<_> = ways.into_iter().filter(|(_, way)| way.os_families.contains(&family)).collect();
            let mut unknown = false;
            let mut found = None;
            for (i, way) in &fitting {
                match (read.cap(from, &pack.id, &format!("install[{i}].arches"), &way.arches), arch) {
                    (Some(arches), Some(arch)) if arches.contains(&arch) => found = Some(*way),
                    (Some(_), Some(_)) => {}
                    _ => unknown = true,
                }
            }
            match found {
                Some(way) if way.runtime_needed && runner.runtime.is_none() => Verdict::Fail,
                Some(_) => Verdict::Pass,
                None if unknown => Verdict::Unknown,
                None => Verdict::Fail,
            }
        }
    };
    cx.emit(rule, verdict, involved.clone(), read, subject());

    // Some features need the container to sit on the real LAN.
    if method != InstallMethod::Container {
        return;
    }
    let mut read = Read::default();
    let features = read.cap(from, &pack.id, "needs.host_network_for", &pack.needs.host_network_for);
    let runtime = runner.runtime.as_deref().and_then(|id| cx.bundle.runtimes.get(id));
    let verdict = match (features, runtime) {
        (None, _) => Verdict::UnknownNote,
        (Some(list), _) if list.is_empty() => return,
        (Some(_), None) => return,
        (Some(_), Some(runtime)) => {
            match read.cap(origin(runtime.teaching), &runtime.id, "host_network", &runtime.host_network) {
                Some(value) if value.get() == HostNetwork::Yes => Verdict::Pass,
                Some(_) => Verdict::Warn,
                None => Verdict::Unknown,
            }
        }
    };
    cx.emit(Reason::ContainerHostNetwork, verdict, involved, read, subject());
}
