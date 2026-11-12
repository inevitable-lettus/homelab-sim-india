//! Will it last: running cost, power cuts, and what long hours do to a laptop.
//! See context-docs/Sandbox.md sections 7.4 and 8.

use serde::Serialize;

use std::collections::BTreeMap;

use super::{Cx, MachineBudget, Read, Subject, Verdict, origin};
use crate::bundle::ProfileKind;
use crate::model::{Host, Id, Intent, Node, Role};
use crate::packs::{Cooling, IspSide, Support, UpsCaps, Waveform};
use crate::reason::Reason;

/// A number the result depends on that we chose, not measured. Shown next to
/// the figures it affects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assumption {
    pub reason: Reason,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RateUsed {
    /// The tariff the slabs came from, or `None` for the learner's own rate.
    pub tariff: Option<Id>,
    /// Paise per kWh on the first unit the lab adds.
    pub from_paise_per_kwh: u32,
    /// Paise per kWh on the last unit the lab adds.
    pub to_paise_per_kwh: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PowerSummary {
    pub lab_avg_w: Option<u32>,
    pub lab_kwh_year: Option<u32>,
    pub yearly_paise: Option<u64>,
    pub rate_used: Option<RateUsed>,
    pub assumptions: Vec<Assumption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpsSummary {
    pub node: Id,
    pub fed: Vec<Id>,
    pub fed_avg_w: Option<u32>,
    pub runtime_min: Option<u32>,
    pub overloaded: Option<bool>,
}

/// Share of the time a machine is busy when the learner has not said.
const DEFAULT_BUSY_PERMILLE: u32 = 100;
/// What the household uses each month before the lab is added, when not given.
const DEFAULT_BASE_KWH_MONTH: u32 = 150;
/// Share of a UPS battery's rated energy that reaches the load: lead-acid
/// depth of discharge and inverter loss. A teaching figure.
const UPS_USABLE_PERMILLE: u32 = 600;
/// Share of a laptop battery's rated energy that is usable. A teaching figure.
const LAPTOP_USABLE_PERMILLE: u32 = 800;
/// Hours in an average month.
const HOURS_PER_MONTH: u64 = 730;
/// Above this share of busy time, a fanless machine may slow down.
const FANLESS_BUSY_PERMILLE: u32 = 300;

/// Average and full-load watts of one node. `None` where the data is missing.
#[derive(Debug, Clone, Copy, Default)]
struct Watts {
    avg: Option<u32>,
    load: Option<u32>,
}

struct State {
    assumptions: BTreeMap<Reason, u64>,
}

impl State {
    fn assume(&mut self, reason: Reason, value: u64) {
        self.assumptions.insert(reason, value);
    }
}

fn busy(host: &Host, state: &mut State) -> u32 {
    match host.busy_permille {
        Some(busy) => u32::from(busy).min(1000),
        None => {
            state.assume(Reason::AssumedBusyShare, u64::from(DEFAULT_BUSY_PERMILLE));
            DEFAULT_BUSY_PERMILLE
        }
    }
}

/// Section 8.2. A guest draws what the machine under it draws.
fn watts(cx: &Cx, id: &str, read: &mut Read, state: &mut State) -> Watts {
    let (device, host) = match cx.world.node(id) {
        Some(Node::Host(host)) => {
            let root = cx.root(host);
            (&root.device, Some(root))
        }
        Some(Node::Router(n)) => (&n.device, None),
        Some(Node::Switch(n)) => (&n.device, None),
        _ => return Watts::default(),
    };
    let Some(pack) = cx.pack_of(device) else {
        read.missing(id, "device");
        return Watts::default();
    };
    let idle = read.cap(pack.origin(), &pack.id, "power.idle_w", &pack.power.idle_w);
    let Some(host) = host else {
        // A router or a switch draws about the same all day.
        return Watts { avg: idle, load: pack.power.load_w.copied().or(idle) };
    };
    let load = read.cap(pack.origin(), &pack.id, "power.load_w", &pack.power.load_w);
    let mut drives = 0;
    for part in &host.parts {
        if let Some(drive) = cx.device(&part.pack).filter(|p| p.drive.is_some()) {
            state.assume(Reason::AssumedDriveLoadIgnored, 0);
            drives += drive.power.idle_w.copied().unwrap_or(0);
        }
    }
    let share = busy(host, state);
    let avg = idle.zip(load).map(|(idle, load)| idle + load.saturating_sub(idle) * share / 1000 + drives);
    Watts { avg, load: load.map(|load| load + drives) }
}

/// The machines and network boxes that draw power for the lab, in node order.
fn powered(cx: &Cx) -> Vec<Id> {
    let servers: Vec<&str> = cx.servers().iter().filter(|h| h.runs_on.is_none()).map(|h| h.id.as_str()).collect();
    cx.world
        .nodes
        .iter()
        .filter(|node| match node {
            Node::Host(host) => servers.contains(&host.id.as_str()),
            Node::Switch(_) => true,
            _ => false,
        })
        .map(|node| node.id().to_owned())
        .collect()
}

/// Section 8.3: what the lab's energy costs in a year, in paise. The lab sits
/// on top of what the household already uses, so it is billed in the slabs
/// that start where the household's own use ends.
fn yearly(cx: &Cx, lab_wh_month: u64, read: &mut Read, state: &mut State) -> Option<(u64, RateUsed)> {
    let home = cx.world.home.as_ref()?;
    if let Some(rate) = home.rate_paise_per_kwh {
        read.note_learner();
        let paise = (12 * lab_wh_month * u64::from(rate) + 500) / 1000;
        return Some((paise, RateUsed { tariff: None, from_paise_per_kwh: rate, to_paise_per_kwh: rate }));
    }
    let id = home.tariff.as_ref()?;
    let Some(tariff) = cx.bundle.tariffs.get(id) else {
        read.missing(id, "tariff");
        return None;
    };
    read.plain(origin(tariff.teaching));
    let duty = read.cap(origin(tariff.teaching), id, "duty_permille", &tariff.duty_permille)?;
    let base = match home.base_kwh_month {
        Some(base) => base,
        None => {
            state.assume(Reason::AssumedBaseUse, u64::from(DEFAULT_BASE_KWH_MONTH));
            DEFAULT_BASE_KWH_MONTH
        }
    };
    let free_wh = match &tariff.free_units {
        Some(free) if home.free_units_apply => u64::from(free.upto_kwh) * 1000,
        _ => 0,
    };
    let (from, to) = (u64::from(base) * 1000, u64::from(base) * 1000 + lab_wh_month);
    let (mut sum, mut lo, mut first, mut last) = (0u64, 0u64, None, 0);
    for slab in &tariff.slabs {
        let hi = slab.upto_kwh.map_or(u64::MAX, |kwh| u64::from(kwh) * 1000);
        // The part of the lab's energy that falls in this slab.
        let (a, b) = (from.max(lo), to.min(hi));
        if b > a {
            first.get_or_insert(slab.paise_per_kwh);
            last = slab.paise_per_kwh;
            // Units under a free-units scheme cost nothing.
            let paid = b - a.max(free_wh).min(b);
            sum += paid * u64::from(slab.paise_per_kwh);
        }
        lo = hi;
    }
    if to > lo {
        // The slabs stop short of the household's use: the tariff is incomplete.
        read.missing(id, "slabs");
        return None;
    }
    let paise = (12 * sum * (1000 + u64::from(duty)) + 500_000) / 1_000_000;
    let rate = RateUsed { tariff: Some(id.clone()), from_paise_per_kwh: first.unwrap_or(last), to_paise_per_kwh: last };
    Some((paise, rate))
}

/// Minutes a UPS holds this load. A maker's chart when there is one, else
/// the battery's energy.
fn ups_minutes(ups: &UpsCaps, load_w: u32, state: &mut State) -> Option<u32> {
    if load_w == 0 {
        return None;
    }
    let mut chart = ups.runtime.get().cloned().unwrap_or_default();
    chart.sort_by_key(|point| point.load_w);
    if let (Some(first), Some(last)) = (chart.first(), chart.last()) {
        if load_w <= first.load_w {
            return Some(first.minutes);
        }
        if load_w >= last.load_w {
            return Some(last.minutes);
        }
        let pair = chart.windows(2).find(|pair| load_w <= pair[1].load_w)?;
        let (l0, m0, l1, m1) = (
            i64::from(pair[0].load_w),
            i64::from(pair[0].minutes),
            i64::from(pair[1].load_w),
            i64::from(pair[1].minutes),
        );
        let minutes = m0 + (m1 - m0) * (i64::from(load_w) - l0) / (l1 - l0).max(1);
        return u32::try_from(minutes).ok();
    }
    let wh = ups.battery_wh.copied()?;
    state.assume(Reason::AssumedUpsUsable, u64::from(UPS_USABLE_PERMILLE));
    Some((u64::from(wh) * u64::from(UPS_USABLE_PERMILLE) * 60 / 1000 / u64::from(load_w)) as u32)
}

/// How long each node keeps running with the mains off. `Some(None)` means
/// it has something behind it and we cannot say for how long.
struct Backup {
    /// Node id to minutes on a UPS.
    ups: BTreeMap<Id, Option<u32>>,
    /// Machine id to minutes on its own battery.
    battery: BTreeMap<Id, Option<u32>>,
}

impl Backup {
    /// Does this node stay up for `minutes`? `None` when we cannot tell.
    fn lasts(&self, id: &str, minutes: u32) -> Option<bool> {
        let held = [self.ups.get(id), self.battery.get(id)];
        if held.iter().flatten().any(|m| m.is_some_and(|m| m >= minutes)) {
            return Some(true);
        }
        if held.iter().flatten().any(|m| m.is_none()) {
            return None;
        }
        Some(false)
    }
}

/// The home-side boxes a goal depends on, in the order a request meets them.
fn path(cx: &Cx, intent: &Intent) -> (Vec<Id>, Vec<Id>) {
    let (mut home, mut isps) = (Vec::new(), Vec::new());
    let mut add = |id: &str, home: &mut Vec<Id>| match cx.world.node(id) {
        Some(Node::Host(host)) if host.role == Role::Server => {
            let root = cx.root(host).id.clone();
            if !home.contains(&root) {
                home.push(root);
            }
        }
        Some(Node::Router(_) | Node::Switch(_)) if !home.iter().any(|h| h == id) => home.push(id.to_owned()),
        Some(Node::Isp(_)) if !isps.iter().any(|i| i == id) => isps.push(id.to_owned()),
        _ => {}
    };
    if let Some(trace) = cx.trace(intent.id()) {
        for step in trace.legs.iter().flat_map(|leg| &leg.steps) {
            add(&step.at, &mut home);
        }
    }
    if let Intent::HomeDns { router, .. } = intent {
        add(router, &mut home);
    }
    if let Some((host, _)) = intent.subject() {
        add(host, &mut home);
    }
    (home, isps)
}

fn outage(cx: &mut Cx, id: &str, scenario: &str, keep: &[Id], backup: &Backup, state: &mut State) {
    let world = cx.world;
    let subject = || Subject::intent(id);
    let Some(cut) = cx.bundle.outages.get(scenario) else {
        let mut read = Read::default();
        read.missing(scenario, "outage");
        cx.emit(Reason::OutageCovered, Verdict::Unknown, Vec::new(), read, subject());
        return;
    };
    let minutes = cut.duration_min;
    for kept in keep {
        let Some(intent) = world.intents.iter().find(|i| i.id() == kept) else { continue };
        let (home, isps) = path(cx, intent);
        let mut read = Read::default();
        read.plain(origin(cut.kind == ProfileKind::Teaching));
        let states: Vec<(&Id, Option<bool>)> = home.iter().map(|n| (n, backup.lasts(n, minutes))).collect();
        let dark = states.iter().find(|(_, lasts)| *lasts == Some(false)).map(|(n, _)| (*n).clone());
        let unsure = states.iter().find(|(_, lasts)| lasts.is_none()).map(|(n, _)| (*n).clone());
        let (verdict, involved) = match (dark, unsure, cut.isp_side) {
            (Some(node), _, _) => (Verdict::Fail, vec![node]),
            (None, Some(node), _) => {
                read.missing(&node, "power");
                (Verdict::Unknown, vec![node])
            }
            (None, None, IspSide::Down) if !isps.is_empty() => (Verdict::Fail, isps.clone()),
            (None, None, IspSide::Unknown) if !isps.is_empty() => {
                read.missing(scenario, "isp_side");
                (Verdict::Unknown, isps.clone())
            }
            (None, None, _) => {
                if !isps.is_empty() {
                    state.assume(Reason::AssumedIspStaysUp, 1);
                }
                (Verdict::Pass, home.clone())
            }
        };
        cx.emit(Reason::OutageCovered, verdict, involved, read, subject());

        // Whatever went dark has to come back by itself when the power does.
        for (node, lasts) in &states {
            let Some(host) = world.host(node) else { continue };
            if *lasts == Some(true) {
                if backup.battery.get(*node).is_some_and(|m| m.is_some_and(|m| m >= minutes)) {
                    cx.emit(Reason::LaptopAsUps, Verdict::Note, vec![(*node).clone()], Read::default(), subject());
                }
                continue;
            }
            let mut read = Read::default();
            let able = cx.pack_of(&host.device).and_then(|pack| {
                let compute = pack.compute.clone().unwrap_or_default();
                read.cap(pack.origin(), &pack.id, "compute.ac_power_recovery", &compute.ac_power_recovery)
            });
            let verdict = match able {
                Some(true) if host.settings.ac_power_recovery => Verdict::Pass,
                Some(_) => Verdict::Fail,
                None => Verdict::Unknown,
            };
            cx.emit(Reason::RecoversAfterOutage, verdict, vec![(*node).clone()], read, subject());
        }
        if let Some((host, Some(service))) = intent.subject()
            && let Some(found) = world.host(host)
            && found.services.iter().any(|s| s.id == service && s.autostart == Some(false))
            && backup.lasts(&cx.root(found).id, minutes) != Some(true)
        {
            cx.emit(Reason::ServiceAutostart, Verdict::Fail, vec![host.to_owned()], Read::default(), subject());
        }
    }
}

/// What long hours do to a laptop used as a server.
fn laptop(cx: &mut Cx, host: &Host, state: &mut State) {
    let Some(pack) = cx.pack_of(&host.device) else { return };
    let subject = || Subject::node(&host.id);
    let involved = || vec![host.id.clone()];
    let support = host.os.as_ref().and_then(|os| pack.os_support.get(os)).and_then(|s| s.get());
    if host.settings.lid_closed {
        let mut read = Read::default();
        let level = support.and_then(|s| read.cap(pack.origin(), &pack.id, "os_support.lid_closed", &s.lid_closed));
        if support.is_none() {
            read.missing(&pack.id, "os_support.lid_closed");
        }
        let verdict = match level {
            Some(Support::Full) => Verdict::Pass,
            Some(Support::Partial) => Verdict::Warn,
            Some(Support::None) => Verdict::Fail,
            None => Verdict::Unknown,
        };
        cx.emit(Reason::LidClosedRunning, verdict, involved(), read, subject());
    }
    if let Some(battery) = &pack.battery
        && host.settings.always_on
    {
        let mut read = Read::default();
        read.plain(pack.origin());
        let by_os = support.and_then(|s| s.charge_limit.copied());
        let by_firmware = battery.firmware_charge_limit.copied();
        let possible = match (by_os, by_firmware) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (Some(false), Some(false)) => Some(false),
            _ => None,
        };
        let verdict = match (host.settings.charge_limit, possible) {
            (true, Some(true)) => Verdict::Pass,
            (true, None) => {
                read.missing(&pack.id, "battery.firmware_charge_limit");
                Verdict::UnknownNote
            }
            _ => Verdict::Warn,
        };
        cx.emit(Reason::BatteryAlwaysCharging, verdict, involved(), read, subject());
    }
    let share = busy(host, state);
    if share > FANLESS_BUSY_PERMILLE && pack.compute.as_ref().and_then(|c| c.cooling.copied()) == Some(Cooling::Fanless)
    {
        let mut read = Read::default();
        read.plain(pack.origin());
        cx.emit(Reason::FanlessSustainedLoad, Verdict::Note, involved(), read, subject());
    }
}

pub(super) fn rules(cx: &mut Cx, machines: &mut [MachineBudget]) {
    let world = cx.world;
    let mut state = State { assumptions: BTreeMap::new() };

    // What everything draws.
    let mut lab_read = Read::default();
    let lab: Vec<(Id, Watts)> =
        powered(cx).into_iter().map(|id| (id.clone(), watts(cx, &id, &mut lab_read, &mut state))).collect();
    for machine in machines.iter_mut() {
        machine.avg_w = lab.iter().find(|(id, _)| *id == machine.node).and_then(|(_, w)| w.avg);
    }
    let lab_avg_w = lab.iter().map(|(_, w)| w.avg).sum::<Option<u32>>().filter(|_| !lab.is_empty());
    let lab_wh_month = lab_avg_w.map(|w| u64::from(w) * HOURS_PER_MONTH);
    let mut cost_read = Read::default();
    let cost = lab_wh_month.and_then(|wh| yearly(cx, wh, &mut cost_read, &mut state));
    let yearly_paise = cost.as_ref().map(|(paise, _)| *paise);

    for intent in &world.intents {
        if let Intent::YearlyCost { id, max_paise } = intent {
            let mut read = Read::default();
            read.merge(&lab_read);
            read.merge(&cost_read);
            let verdict = match yearly_paise {
                Some(paise) if paise <= *max_paise => Verdict::Pass,
                Some(_) => Verdict::Fail,
                None => Verdict::Unknown,
            };
            let involved = lab.iter().map(|(id, _)| id.clone()).collect();
            cx.emit(Reason::YearlyCost, verdict, involved, read, Subject::intent(id));
        }
    }

    // What holds things up when the mains go.
    let survive: Vec<Id> =
        world.intents.iter().filter(|i| matches!(i, Intent::SurviveOutage { .. })).map(|i| i.id().to_owned()).collect();
    let mut backup = Backup { ups: BTreeMap::new(), battery: BTreeMap::new() };
    let mut summaries = Vec::new();
    for node in &world.nodes {
        let Node::Power(unit) = node else { continue };
        let fed: Vec<Id> = world.power_feeds.iter().filter(|f| f.from == unit.id).map(|f| f.to.clone()).collect();
        let mut read = Read::default();
        let drawn: Vec<Watts> = fed.iter().map(|id| watts(cx, id, &mut read, &mut state)).collect();
        let fed_avg_w = drawn.iter().map(|w| w.avg).sum::<Option<u32>>();
        let fed_load_w = drawn.iter().map(|w| w.load).sum::<Option<u32>>();
        let pack = cx.pack_of(&unit.device);
        // A pack that says nothing about itself is a UPS we know nothing about.
        let ups = pack.map(|p| p.ups.clone().unwrap_or_default());
        let (mut runtime_min, mut overloaded) = (None, None);
        if let (Some(pack), Some(ups)) = (pack, &ups) {
            let max_w = read.cap(pack.origin(), &pack.id, "ups.max_w", &ups.max_w);
            overloaded = fed_load_w.zip(max_w).map(|(load, max)| load > max);
            runtime_min = fed_avg_w.and_then(|w| ups_minutes(ups, w, &mut state));
            if runtime_min.is_none() && fed_avg_w.is_some_and(|w| w > 0) {
                read.missing(&pack.id, "ups.battery_wh");
            }
            let subject = || Subject { intents: survive.clone(), ..Subject::default() };
            if !fed.is_empty() {
                let verdict = match overloaded {
                    Some(true) => Verdict::Fail,
                    Some(false) => Verdict::Pass,
                    None => Verdict::Unknown,
                };
                let mut over = Read::default();
                over.merge(&read);
                cx.emit(Reason::UpsOverloaded, verdict, vec![unit.id.clone()], over, subject());
            }
            let feeds_computer = fed.iter().any(|id| world.host(id).is_some());
            if feeds_computer {
                let mut wave = Read::default();
                let verdict = match wave.cap(pack.origin(), &pack.id, "ups.waveform", &ups.waveform) {
                    Some(Waveform::Square) => Verdict::Warn,
                    Some(_) => Verdict::Pass,
                    None => Verdict::UnknownNote,
                };
                cx.emit(Reason::InverterWaveform, verdict, vec![unit.id.clone()], wave, subject());
            }
        } else {
            read.missing(&unit.id, "device");
        }
        // An overloaded UPS holds nothing up.
        let held = if overloaded == Some(true) { Some(0) } else { runtime_min };
        for id in &fed {
            let slot = backup.ups.entry(id.clone()).or_insert(held);
            *slot = match (*slot, held) {
                (Some(a), Some(b)) => Some(a.max(b)),
                _ => None,
            };
        }
        summaries.push(UpsSummary { node: unit.id.clone(), fed, fed_avg_w, runtime_min, overloaded });
    }
    for machine in machines.iter_mut() {
        let Some(host) = world.host(&machine.node) else { continue };
        let Some(wh) = cx.pack_of(&host.device).and_then(|p| p.battery.as_ref()).map(|b| b.wh.copied()) else {
            continue;
        };
        state.assume(Reason::AssumedLaptopUsable, u64::from(LAPTOP_USABLE_PERMILLE));
        let minutes = wh
            .zip(machine.avg_w.filter(|w| *w > 0))
            .map(|(wh, avg)| (u64::from(wh) * u64::from(LAPTOP_USABLE_PERMILLE) * 60 / 1000 / u64::from(avg)) as u32);
        machine.battery_min = minutes;
        backup.battery.insert(machine.node.clone(), minutes);
    }

    for intent in &world.intents {
        if let Intent::SurviveOutage { id, outage: scenario, keep } = intent {
            outage(cx, id, scenario, keep, &backup, &mut state);
        }
    }
    for host in cx.servers() {
        if host.runs_on.is_none() {
            laptop(cx, host, &mut state);
        }
    }

    cx.ups = summaries;
    cx.power = PowerSummary {
        lab_avg_w,
        lab_kwh_year: lab_wh_month.map(|wh| (wh * 12 / 1000) as u32),
        yearly_paise,
        rate_used: cost.map(|(_, rate)| rate),
        assumptions: state.assumptions.into_iter().map(|(reason, value)| Assumption { reason, value }).collect(),
    };
}
