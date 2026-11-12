//! Will it connect: what the router allows, and the goals that are decided
//! by a trace or by what plays the media.

use super::run::{install_method, runtime_host};
use super::{Cx, Read, Subject, Verdict, origin};
use crate::model::{Host, Intent, Node};
use crate::packs::{AdminAccess, Codec, InstallMethod, IntentKind, OsFamily, Provides};
use crate::reason::Reason;
use crate::trace::Outcome;

pub(super) fn rules(cx: &mut Cx) {
    let world = cx.world;
    for node in &world.nodes {
        if let Node::Router(router) = node
            && router.port_forwards.iter().any(|f| f.enabled)
            && let Some(pack) = cx.pack_of(&router.device)
        {
            let mut read = Read::default();
            let caps = pack.router.clone().unwrap_or_default();
            let can = read.cap(pack.origin(), &pack.id, "router.port_forwarding", &caps.port_forwarding);
            let admin = read.cap(pack.origin(), &pack.id, "router.admin", &caps.admin);
            // Many ISP-supplied routers are the real reason a forward cannot be made.
            let verdict = match (can, admin) {
                (Some(false), _) | (_, Some(AdminAccess::IspOnly)) => Verdict::Fail,
                (Some(true), Some(_)) => Verdict::Pass,
                _ => Verdict::Unknown,
            };
            cx.emit(Reason::RouterCanForward, verdict, vec![router.id.clone()], read, Subject::none());
        }
    }
    for host in cx.servers() {
        static_lan_ip(cx, host);
    }
    for intent in &world.intents {
        match intent {
            Intent::Reach { id, .. } => reach(cx, id),
            Intent::Stream { id, client, host, service, format, streams, .. } => {
                reach(cx, id);
                stream(cx, id, client, host, service, format, u64::from(*streams));
            }
            Intent::HomeDns { id, host, service, router } => home_dns(cx, id, host, service, router),
            _ => {}
        }
        paid_features(cx, intent);
    }
}

fn static_lan_ip(cx: &mut Cx, host: &Host) {
    for service in &host.services {
        let Some(pack) = service.app.as_deref().and_then(|app| cx.bundle.services.get(app)) else {
            continue;
        };
        let mut read = Read::default();
        let wanted = read.cap(origin(pack.teaching), &pack.id, "needs.static_lan_ip", &pack.needs.static_lan_ip);
        if wanted != Some(true) {
            continue;
        }
        let gateway = cx.net.index(&host.id).and_then(|i| cx.net.gw[i]).map(|g| &cx.world.nodes[g]);
        let verdict = match gateway {
            _ if host.static_ipv4.is_some() => Verdict::Pass,
            Some(Node::Router(router)) => match cx.pack_of(&router.device) {
                Some(device) => {
                    let caps = device.router.clone().unwrap_or_default();
                    match read.cap(device.origin(), &device.id, "router.dhcp_reservation", &caps.dhcp_reservation) {
                        Some(true) => Verdict::Pass,
                        Some(false) => Verdict::Warn,
                        None => Verdict::UnknownNote,
                    }
                }
                None => Verdict::UnknownNote,
            },
            _ => continue,
        };
        cx.emit(Reason::StaticLanIp, verdict, vec![host.id.clone()], read, Subject::service(&host.id, &service.id));
    }
}

/// The trace decides whether the connection is made.
fn reach(cx: &mut Cx, intent: &str) {
    let rule = Reason::IntentReach;
    let subject = Subject::intent(intent);
    let read = Read::default();
    let Some(trace) = cx.trace(intent) else {
        // The goal names a node or a service that is not in the build.
        return cx.emit_as(rule, Verdict::Unknown, Reason::UnknownNode, Vec::new(), read, subject);
    };
    let involved: Vec<String> = trace.ended_at.iter().cloned().collect();
    match trace.outcome {
        Outcome::Delivered => cx.emit(rule, Verdict::Pass, involved, read, subject),
        Outcome::Dropped => cx.emit_as(rule, Verdict::Fail, trace.reason.unwrap_or(rule), involved, read, subject),
        Outcome::Unknown => {
            cx.emit_as(
                rule,
                Verdict::Unknown,
                trace.reason.unwrap_or(Reason::LineTypeUnknown),
                involved,
                read,
                subject,
            );
        }
    }
}

fn stream(cx: &mut Cx, intent: &str, client: &str, host: &str, service: &str, format: &str, streams: u64) {
    let world = cx.world;
    let (Some(client), Some(server)) = (world.host(client), world.host(host)) else {
        return;
    };
    let Some(service) = server.services.iter().find(|s| s.id == service) else {
        return;
    };
    let app = service.app.as_deref().and_then(|app| cx.bundle.services.get(app));
    let player = cx.pack_of(&client.device);
    let involved = vec![client.id.clone(), server.id.clone()];

    // Is there an app for it on what plays the media?
    let mut read = Read::default();
    let verdict = match (player.and_then(|p| p.client.as_ref()), &service.app) {
        (Some(caps), Some(app)) => {
            read.plain(player.map_or(crate::packs::Origin::Teaching, |p| p.origin()));
            if caps.apps.contains(app) { Verdict::Pass } else { Verdict::Warn }
        }
        _ => {
            read.missing(client.device.as_deref().unwrap_or(&client.id), "client.apps");
            Verdict::Unknown
        }
    };
    cx.emit(Reason::ClientHasApp, verdict, involved.clone(), read, Subject::intent(intent));

    // Can the client play the format as it is, or must the server convert it?
    let media = cx.bundle.media.get(format);
    let mut read = Read::default();
    let plays = match (media, player.and_then(|p| p.client.as_ref())) {
        (Some(media), Some(caps)) => {
            read.plain(player.map_or(crate::packs::Origin::Teaching, |p| p.origin()));
            caps.plays.get(&media.codec).copied()
        }
        _ => None,
    };
    let verdict = match plays {
        Some(true) => Verdict::Pass,
        None => {
            read.missing(client.device.as_deref().unwrap_or(&client.id), "client.plays");
            Verdict::Unknown
        }
        Some(false) => transcode(cx, server, service, app, &mut read),
    };
    cx.emit(Reason::TranscodeSupported, verdict, involved.clone(), read, Subject::intent(intent));

    // Does the path carry it?
    if let (Some(trace), Some(media)) = (cx.trace(intent), media)
        && trace.outcome == Outcome::Delivered
    {
        let mut read = Read::default();
        let kbps = read.cap(origin(media.teaching), format, "kbps", &media.kbps);
        let spare = trace.summary.transfer.as_ref().and_then(|t| t.bottleneck_bps);
        let verdict = match (kbps, spare) {
            (Some(kbps), Some(spare)) if u64::from(kbps) * 1_000 * streams <= spare => Verdict::Pass,
            (Some(_), Some(_)) => Verdict::Fail,
            _ => Verdict::Unknown,
        };
        cx.emit(Reason::StreamBandwidth, verdict, involved, read, Subject::intent(intent));
    }
}

/// Hardware help for converting video, or the CPU alone. CPU speed is not modelled.
fn transcode<'a>(
    cx: &Cx<'a>,
    server: &'a Host,
    service: &crate::model::Service,
    app: Option<&crate::packs::ServicePack>,
    read: &mut Read,
) -> Verdict {
    let machine = cx.root(server);
    let (Some(device), Some(app)) = (cx.pack_of(&machine.device), app) else {
        return Verdict::Unknown;
    };
    let compute = device.compute.clone().unwrap_or_default();
    let Some(video) = read.cap(device.origin(), &device.id, "compute.video", &compute.video) else {
        return Verdict::Unknown;
    };
    let containerised = install_method(server, service) == InstallMethod::Container;
    let family = if containerised { Some(OsFamily::Linux) } else { cx.platform(server, read).0 };
    let Some(family) = family else {
        return Verdict::Unknown;
    };
    let field = format!("needs.transcode_apis.{family:?}").to_lowercase();
    let Some(wanted) =
        app.needs.transcode_apis.get(&family).and_then(|cap| read.cap(origin(app.teaching), &app.id, &field, cap))
    else {
        read.missing(&app.id, &field);
        return Verdict::Unknown;
    };
    let mut usable: Vec<_> = video.apis.iter().filter(|api| wanted.contains(api)).collect();
    if containerised {
        let runtime = runtime_host(cx, server).runtime.as_deref().and_then(|id| cx.bundle.runtimes.get(id));
        let passed =
            runtime.and_then(|r| read.cap(origin(r.teaching), &r.id, "device_passthrough", &r.device_passthrough));
        match passed {
            Some(passed) => usable.retain(|api| passed.contains(api)),
            None => return Verdict::Unknown,
        }
    }
    if !usable.is_empty() && video.encode.get(&Codec::H264) == Some(&true) { Verdict::Pass } else { Verdict::Warn }
}

fn home_dns(cx: &mut Cx, intent: &str, host: &str, service: &str, router: &str) {
    let rule = Reason::HomeDnsPossible;
    let subject = Subject::intent(intent);
    let involved = vec![router.to_owned(), host.to_owned()];
    let mut read = Read::default();
    let world = cx.world;
    let app = world
        .host(host)
        .and_then(|h| h.services.iter().find(|s| s.id == service))
        .and_then(|s| s.app.as_deref())
        .and_then(|app| cx.bundle.services.get(app));
    let (Some(app), Some(Node::Router(node))) = (app, world.node(router)) else {
        return cx.emit_as(rule, Verdict::Unknown, Reason::UnknownNode, involved, read, subject);
    };
    read.plain(origin(app.teaching));
    let on_lan = cx.net.index(host).and_then(|i| cx.net.gw[i]) == cx.net.index(router);
    if !app.provides.contains(&Provides::DnsServer) || !on_lan {
        return cx.emit(rule, Verdict::Fail, involved, read, subject);
    }
    let Some(pack) = cx.pack_of(&node.device) else {
        read.missing(node.device.as_deref().unwrap_or(router), "router.lan_dns_setting");
        return cx.emit(rule, Verdict::Unknown, involved, read, subject);
    };
    let caps = pack.router.clone().unwrap_or_default();
    let dns = read.cap(pack.origin(), &pack.id, "router.lan_dns_setting", &caps.lan_dns_setting);
    // Second way: switch the router's DHCP off and let the service hand out addresses.
    let own_dhcp = app.provides.contains(&Provides::DhcpServer);
    let off = if dns == Some(true) || !own_dhcp {
        None
    } else {
        read.cap(pack.origin(), &pack.id, "router.dhcp_can_disable", &caps.dhcp_can_disable)
    };
    let verdict = match (dns, off) {
        (Some(true), _) | (_, Some(true)) => Verdict::Pass,
        (Some(false), Some(false)) => Verdict::Fail,
        (Some(false), None) if !own_dhcp => Verdict::Fail,
        _ => Verdict::Unknown,
    };
    cx.emit(rule, verdict, involved, read, subject);
}

fn paid_features(cx: &mut Cx, intent: &Intent) {
    let kind = match intent {
        Intent::Run { .. } => IntentKind::Run,
        Intent::Reach { .. } => IntentKind::Reach,
        Intent::Stream { .. } => IntentKind::Stream,
        Intent::HomeDns { .. } => IntentKind::HomeDns,
        _ => return,
    };
    let Some((host, Some(service))) = intent.subject() else {
        return;
    };
    let app = cx
        .world
        .host(host)
        .and_then(|h| h.services.iter().find(|s| s.id == service))
        .and_then(|s| s.app.as_deref())
        .and_then(|app| cx.bundle.services.get(app));
    let Some(app) = app else {
        return;
    };
    let mut read = Read::default();
    let needed = app.paid_features.get().is_some_and(|list| list.iter().any(|f| f.needed_for.contains(&kind)));
    if needed {
        read.cap(origin(app.teaching), &app.id, "paid_features", &app.paid_features);
        cx.emit(Reason::PaidFeatureNeeded, Verdict::Note, vec![host.to_owned()], read, Subject::intent(intent.id()));
    }
}
