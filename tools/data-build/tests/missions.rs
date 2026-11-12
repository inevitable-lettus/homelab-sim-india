//! Every win and failure path of Missions 1 to 5, run against the real mission
//! files and the real data build. If a mission file changes in a way that
//! breaks its own lesson, this fails.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use sim_core::model::{Answer, Host, Ipv6Mode, LineCheck, LineCheckResult, LineMode, Node, PortForward, Proto, Target};
use sim_core::trace::{LegRole, Outcome, StepAction, TableKind, Trace};
use sim_core::{Artifacts, Bundle, Depth, Grade, Mission, Reason, Status, World};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bundle() -> &'static Bundle {
    static BUNDLE: OnceLock<Bundle> = OnceLock::new();
    BUNDLE.get_or_init(|| data_build::build(&root()).expect("the repo's data builds"))
}

fn mission(id: &str) -> &'static Mission {
    &bundle().missions[id]
}

fn grade(mission: &Mission, world: &World) -> Grade {
    sim_core::grade(mission, Depth::Guided, world, &Artifacts::default(), bundle())
}

fn attempt<'a>(grade: &'a Grade, id: &str) -> &'a Trace {
    grade.report.attempts.iter().find(|t| t.attempt == id).expect("attempt exists")
}

fn dial(world: &mut World, target: Target) {
    world.attempt_mut("open-jellyfin").expect("attempt").to = target;
}

fn lan() -> Target {
    Target::LanOf { node: "server".into() }
}
fn wan() -> Target {
    Target::WanOf { node: "home-router".into() }
}
fn public() -> Target {
    Target::PublicOf { node: "home-router".into() }
}
fn mesh() -> Target {
    Target::OverlayOf { node: "server".into(), network: "mesh".into() }
}

fn forward(external: u16, to: &str, internal: u16) -> PortForward {
    PortForward {
        id: "learner".into(),
        enabled: true,
        proto: Proto::Tcp,
        external_port: external,
        to_ip: to.parse().expect("ip"),
        internal_port: internal,
    }
}

fn answer(world: &mut World, question: &str, value: &str) {
    world.answers.retain(|a| a.question != question);
    world.answers.push(Answer { question: question.into(), value: value.into() });
}

fn goal(grade: &Grade, id: &str) -> (Status, Reason) {
    let goal = grade.goals.iter().find(|g| g.id == id).expect("goal exists");
    (goal.status, goal.reason)
}

fn dropped(trace: &Trace, at: &str, reason: Reason) {
    assert_eq!(
        (trace.outcome, trace.ended_at.as_deref(), trace.reason),
        (Outcome::Dropped, Some(at), Some(reason)),
        "expected a drop at {at}"
    );
    assert_eq!(trace.status, Status::Fail);
}

fn finding(grade: &Grade, rule: Reason) -> (Status, Reason) {
    let f = grade.report.findings.iter().find(|f| f.rule_id == rule).expect("finding exists");
    (f.status, f.explanation_id)
}

// ---------------------------------------------------------------- Mission 1

#[test]
fn m1_home_address_dies_on_the_mobile_network() {
    let m = mission("which-address");
    let g = grade(m, &m.world);
    dropped(attempt(&g, "open-jellyfin"), "mobile", Reason::PrivateNotRoutable);
    assert_eq!(goal(&g, "reach-router"), (Status::Fail, Reason::GoalNotMet));
    assert_eq!(goal(&g, "line-type"), (Status::Fail, Reason::GoalUnanswered));
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m1_wan_and_public_address_both_reach_the_router() {
    let m = mission("which-address");
    for target in [wan(), public()] {
        let mut w = m.world.clone();
        dial(&mut w, target);
        let g = grade(m, &w);
        // No forward yet, so the router is where it stops. That is the goal here.
        dropped(attempt(&g, "open-jellyfin"), "home-router", Reason::RouterNoForward);
        assert_eq!(goal(&g, "reach-router"), (Status::Pass, Reason::GoalMet));
    }
}

#[test]
fn m1_what_is_my_ip_shows_the_router_rewriting_the_source() {
    let m = mission("which-address");
    let g = grade(m, &m.world);
    let t = attempt(&g, "check-ip");
    assert_eq!(t.outcome, Outcome::Delivered);
    let request = t.request().expect("request leg");
    let rewrite = request.steps.iter().find(|s| s.at == "home-router").expect("router step");
    assert_eq!(rewrite.action, StepAction::Rewrite);
    assert_eq!(rewrite.before.as_ref().expect("before").src, "192.168.1.20:51000");
    assert_eq!(rewrite.packet.src, "203.0.113.50:51000");
    // The site sees the public address, which on this line is the WAN address.
    assert_eq!(request.steps.last().expect("last").packet.src, "203.0.113.50:51000");
    let router = g.report.facts.nodes.iter().find(|n| n.id == "home-router").expect("router facts");
    assert_eq!(router.wan_ipv4, router.public_ipv4);
}

#[test]
fn m1_answers_are_checked_against_the_world_not_a_stored_key() {
    let m = mission("which-address");
    let mut w = m.world.clone();
    dial(&mut w, public());
    answer(&mut w, "line-type", "different");
    assert_eq!(goal(&grade(m, &w), "line-type"), (Status::Fail, Reason::GoalWrongAnswer));
    answer(&mut w, "line-type", "cant-tell");
    assert_eq!(goal(&grade(m, &w), "line-type"), (Status::Fail, Reason::GoalWrongAnswer));
    answer(&mut w, "line-type", "same");
    let g = grade(m, &w);
    assert_eq!(goal(&g, "line-type"), (Status::Pass, Reason::GoalMet));
    assert_eq!(g.status, Status::Pass);

    // Same mission file, different line: the right answer flips.
    w.isp_mut("isp").expect("isp").ipv4 = LineMode::Cgnat;
    assert_eq!(goal(&grade(m, &w), "line-type"), (Status::Fail, Reason::GoalWrongAnswer));
}

// ---------------------------------------------------------------- Mission 2

#[test]
fn m2_no_forward_stops_at_the_router() {
    let m = mission("open-a-door");
    let g = grade(m, &m.world);
    let t = attempt(&g, "open-jellyfin");
    dropped(t, "home-router", Reason::RouterNoForward);
    let last = t.request().expect("leg").steps.last().expect("step");
    assert_eq!(last.table.as_ref().expect("table").kind, TableKind::NoMatch);
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m2_correct_forward_delivers_and_the_reply_returns() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    dial(&mut w, public());
    w.router_mut("home-router").expect("router").port_forwards.push(forward(8096, "192.168.1.20", 8096));
    let g = grade(m, &w);
    let t = attempt(&g, "open-jellyfin");
    assert_eq!((t.outcome, t.ended_at.as_deref()), (Outcome::Delivered, Some("server")));
    assert_eq!(g.status, Status::Pass);

    let request = t.request().expect("request");
    let hit = request.steps.iter().find(|s| s.at == "home-router").expect("router step");
    assert_eq!(hit.reason, Reason::PortForwardHit);
    assert_eq!(hit.table.as_ref().expect("table").kind, TableKind::ForwardHit);
    assert_eq!(hit.before.as_ref().expect("before").dst, "203.0.113.50:8096");
    assert_eq!(hit.packet.dst, "192.168.1.20:8096");
    let reply = t.legs.iter().find(|l| l.role == LegRole::Reply).expect("reply");
    assert_eq!(reply.steps.last().expect("last").at, "phone");

    assert_eq!(finding(&g, Reason::PortForwardUsable).0, Status::Pass);
    // Working also means exposed, but that is kept for the debrief.
    let exposed = g.report.findings.iter().find(|f| f.rule_id == Reason::ServiceExposed).expect("note");
    assert!(exposed.debrief_only);
}

#[test]
fn m2_every_wrong_forward_fails_for_its_own_reason() {
    let m = mission("open-a-door");
    let cases: [(PortForward, &str, Reason); 4] = [
        (forward(8097, "192.168.1.20", 8096), "home-router", Reason::RouterNoForward),
        (forward(8096, "192.168.1.99", 8096), "home-router", Reason::ForwardTargetMissing),
        (forward(8096, "192.168.1.20", 80), "server", Reason::NoService),
        (
            PortForward { proto: Proto::Udp, ..forward(8096, "192.168.1.20", 8096) },
            "home-router",
            Reason::RouterNoForward,
        ),
    ];
    for (rule, at, reason) in cases {
        let mut w = m.world.clone();
        dial(&mut w, public());
        w.router_mut("home-router").expect("router").port_forwards.push(rule);
        let g = grade(m, &w);
        dropped(attempt(&g, "open-jellyfin"), at, reason);
        assert_eq!(g.status, Status::Fail);
    }
    // A forward that is switched off is no forward.
    let mut w = m.world.clone();
    dial(&mut w, public());
    w.router_mut("home-router")
        .expect("router")
        .port_forwards
        .push(PortForward { enabled: false, ..forward(8096, "192.168.1.20", 8096) });
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "home-router", Reason::RouterNoForward);
}

#[test]
fn m2_forward_does_not_help_if_the_phone_dials_the_home_address() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    dial(&mut w, lan());
    w.router_mut("home-router").expect("router").port_forwards.push(forward(8096, "192.168.1.20", 8096));
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "mobile", Reason::PrivateNotRoutable);
}

#[test]
fn m2_times_and_transfer_come_from_the_links() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    dial(&mut w, public());
    w.router_mut("home-router").expect("router").port_forwards.push(forward(8096, "192.168.1.20", 8096));
    let g = grade(m, &w);
    let t = attempt(&g, "open-jellyfin");
    let request = t.request().expect("request");
    // Each hop's parts add up to its total, and the totals add up to the leg.
    let mut sum = 0;
    for step in &request.steps {
        let Some(timing) = &step.timing else { continue };
        let parts = timing.propagation_ns.expect("propagation")
            + timing.serialization_ns.expect("serialization")
            + timing.queue_ns.expect("queue")
            + timing.extra_ns
            + timing.processing_ns;
        assert_eq!(timing.total_ns, Some(parts));
        sum += parts;
        assert_eq!(timing.arrive_ns, Some(sum));
    }
    assert_eq!(request.duration_ns, Some(sum));
    // 840 km of backbone at 4900 ps/m.
    let long_haul = request.steps.iter().find(|s| s.via_link.as_deref() == Some("mobile-internet")).expect("hop");
    assert_eq!(long_haul.timing.as_ref().expect("timing").propagation_ns, Some(4_116_000));
    let rtt = t.summary.round_trip_ns.expect("round trip");
    assert!((30_000_000..60_000_000).contains(&rtt), "round trip {rtt} ns is outside the teaching range");

    let transfer = t.summary.transfer.as_ref().expect("transfer");
    // The film comes down the phone's radio link, which is the slowest hop.
    assert_eq!(transfer.bottleneck_link.as_deref(), Some("phone-mobile"));
    assert_eq!(transfer.bottleneck_bps, Some(16_000_000));
    assert!(transfer.wire_bytes.expect("wire") > transfer.bytes);
    assert_eq!(transfer.duration_ms, Some(770_547));
}

// ---------------------------------------------------------------- Mission 3

#[test]
fn m3_the_same_forward_fails_behind_carrier_nat() {
    let m = mission("the-wall");
    let g = grade(m, &m.world);
    let t = attempt(&g, "open-jellyfin");
    dropped(t, "isp", Reason::CgnatNoMapping);
    let request = t.request().expect("request");
    assert!(request.steps.iter().all(|s| s.at != "home-router"), "the packet must never reach the router");
    assert_eq!(request.steps.last().expect("last").table.as_ref().expect("table").kind, TableKind::NoMatch);
    assert_eq!(finding(&g, Reason::PortForwardUsable), (Status::Fail, Reason::ForwardBehindCgnat));
    assert_eq!(finding(&g, Reason::LineHasPublicIpv4), (Status::Fail, Reason::WanAddressIsCarrierNat));
}

#[test]
fn m3_no_address_gets_in_from_outside() {
    let m = mission("the-wall");
    for (target, at, reason) in [
        (lan(), "mobile", Reason::PrivateNotRoutable),
        (wan(), "mobile", Reason::PrivateNotRoutable),
        (public(), "isp", Reason::CgnatNoMapping),
    ] {
        let mut w = m.world.clone();
        dial(&mut w, target);
        dropped(attempt(&grade(m, &w), "open-jellyfin"), at, reason);
    }
}

#[test]
fn m3_what_is_my_ip_differs_from_the_wan_address() {
    let m = mission("the-wall");
    let g = grade(m, &m.world);
    let t = attempt(&g, "check-ip");
    assert_eq!(t.outcome, Outcome::Delivered);
    let request = t.request().expect("request");
    let carrier = request.steps.iter().find(|s| s.at == "isp").expect("isp step");
    assert_eq!(carrier.reason, Reason::CgnatMappingCreated);
    assert_eq!(carrier.before.as_ref().expect("before").src, "100.64.12.34:51000");
    assert_eq!(carrier.packet.src, "203.0.113.50:62000");
    let router = g.report.facts.nodes.iter().find(|n| n.id == "home-router").expect("router facts");
    assert_ne!(router.wan_ipv4, router.public_ipv4);
    assert_eq!(router.nat_layers, 1);
    let server = g.report.facts.nodes.iter().find(|n| n.id == "server").expect("server facts");
    assert_eq!(server.nat_layers, 2);
}

#[test]
fn m3_wins_by_naming_the_hop_and_the_line_type() {
    let m = mission("the-wall");
    let mut w = m.world.clone();
    answer(&mut w, "blocked-at", "home-router");
    answer(&mut w, "line-type", "same");
    let g = grade(m, &w);
    assert_eq!(goal(&g, "blocked-at"), (Status::Fail, Reason::GoalWrongAnswer));
    assert_eq!(goal(&g, "line-type"), (Status::Fail, Reason::GoalWrongAnswer));
    answer(&mut w, "blocked-at", "isp");
    answer(&mut w, "line-type", "different");
    assert_eq!(grade(m, &w).status, Status::Pass);
}

// ---------------------------------------------------------------- Mission 4

fn join(world: &mut World, node: &str) {
    world.host_mut(node).expect("host").tunnels.iter_mut().for_each(|t| t.enabled = true);
}

#[test]
fn m4_starts_blocked_and_a_mesh_address_needs_both_to_join() {
    let m = mission("dial-out");
    dropped(attempt(&grade(m, &m.world), "open-jellyfin"), "isp", Reason::CgnatNoMapping);

    // Nobody joined: the mesh address does not exist.
    let mut w = m.world.clone();
    dial(&mut w, mesh());
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "phone", Reason::OverlayNotJoined);

    // Only the server joined: the phone has no way onto the mesh.
    let mut w = m.world.clone();
    dial(&mut w, mesh());
    join(&mut w, "server");
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "phone", Reason::OverlayNotJoined);

    // Only the phone joined: the server has no mesh address to dial.
    let mut w = m.world.clone();
    dial(&mut w, mesh());
    join(&mut w, "phone");
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "phone", Reason::OverlayNotJoined);
}

#[test]
fn m4_joining_does_not_open_the_public_address() {
    let m = mission("dial-out");
    let mut w = m.world.clone();
    join(&mut w, "server");
    join(&mut w, "phone");
    dial(&mut w, public());
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "isp", Reason::CgnatNoMapping);
}

#[test]
fn m4_both_dial_out_then_the_relay_joins_them() {
    let m = mission("dial-out");
    let mut w = m.world.clone();
    join(&mut w, "server");
    join(&mut w, "phone");
    dial(&mut w, mesh());
    let g = grade(m, &w);
    assert_eq!(g.status, Status::Pass);
    let t = attempt(&g, "open-jellyfin");
    assert_eq!((t.outcome, t.ended_at.as_deref()), (Outcome::Delivered, Some("server")));

    // Outbound first: both setup legs come before the request.
    let roles: Vec<LegRole> = t.legs.iter().map(|l| l.role).collect();
    assert_eq!(roles, [LegRole::Setup, LegRole::Setup, LegRole::Request, LegRole::Reply]);
    for setup in &t.legs[..2] {
        assert_eq!(setup.steps.first().expect("first").reason, Reason::TunnelDialsOut);
        assert_eq!(setup.steps.last().expect("last").at, "relay");
        assert_eq!(setup.steps.last().expect("last").reason, Reason::TunnelSessionUp);
    }
    // The server's dial-out made a row at its router and at the ISP.
    let server_setup = t.legs.iter().find(|l| l.from == "server").expect("server setup");
    let created: Vec<&str> = server_setup
        .steps
        .iter()
        .filter(|s| s.table.as_ref().is_some_and(|t| t.kind == TableKind::MappingCreated))
        .map(|s| s.at.as_str())
        .collect();
    assert_eq!(created, ["home-router", "isp"]);

    // The request rides in on those rows, wrapped, and is unwrapped at the server.
    let request = t.request().expect("request");
    let at: Vec<&str> = request.steps.iter().map(|s| s.at.as_str()).collect();
    assert_eq!(at, ["phone", "mobile", "internet", "relay", "internet", "isp", "home-router", "server", "server"]);
    let isp = request.steps.iter().find(|s| s.at == "isp").expect("isp step");
    assert_eq!(isp.reason, Reason::CgnatMappingHit);
    assert_eq!(isp.packet.bytes, 1060);
    assert_eq!(isp.packet.inner.as_ref().expect("inner").dst, "10.77.0.11:8096");
    assert_eq!(request.steps[0].action, StepAction::Encapsulate);
    assert_eq!(request.steps[7].action, StepAction::Decapsulate);
    // 1500 on the wire, less 60 bytes of WireGuard over IPv4.
    assert_eq!(t.summary.path_mtu, Some(1440));
}

#[test]
fn m4_keepalive_is_what_keeps_the_rows_alive() {
    let m = mission("dial-out");
    let mut w = m.world.clone();
    join(&mut w, "server");
    join(&mut w, "phone");
    dial(&mut w, mesh());
    // The attempt runs ten minutes after anything was last sent by hand.
    assert_eq!(w.attempts[0].idle_s, 600);
    let g = grade(m, &w);
    let request = attempt(&g, "open-jellyfin").request().expect("request").clone();
    let row =
        request.steps.iter().find(|s| s.at == "isp").and_then(|s| s.table.clone()).and_then(|t| t.row).expect("row");
    // The row is only as old as the last keepalive, well inside the timeout.
    assert_eq!((row.idle_s, row.timeout_s), (25, Some(60)));
    assert_eq!(finding(&g, Reason::TunnelKeepalive).0, Status::Pass);
}

// ------------------------------------------------- Unknown is never a pass

#[test]
fn unknown_line_type_gives_unknown_when_it_matters() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    dial(&mut w, public());
    w.router_mut("home-router").expect("router").port_forwards.push(forward(8096, "192.168.1.20", 8096));
    w.isp_mut("isp").expect("isp").ipv4 = LineMode::Unknown;
    let g = grade(m, &w);
    let t = attempt(&g, "open-jellyfin");
    assert_eq!((t.outcome, t.status, t.reason), (Outcome::Unknown, Status::Unknown, Some(Reason::LineTypeUnknown)));
    assert_eq!(g.status, Status::Unknown);
    assert_eq!(goal(&g, "open-jellyfin"), (Status::Unknown, Reason::GoalUnknown));
    assert_eq!(finding(&g, Reason::PortForwardUsable), (Status::Unknown, Reason::LineTypeUnknown));
    assert_eq!(finding(&g, Reason::LineHasPublicIpv4).0, Status::Unknown);
}

#[test]
fn unknown_line_type_still_answers_when_both_cases_agree() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    w.isp_mut("isp").expect("isp").ipv4 = LineMode::Unknown;
    // A private address fails on the mobile network whatever the home line is.
    dial(&mut w, lan());
    let t = attempt(&grade(m, &w), "open-jellyfin").clone();
    dropped(&t, "mobile", Reason::PrivateNotRoutable);
    assert_eq!(t.assumed.len(), 1);
    // Outbound works either way too.
    let m1 = mission("which-address");
    let mut w = m1.world.clone();
    w.isp_mut("isp").expect("isp").ipv4 = LineMode::Unknown;
    assert_eq!(attempt(&grade(m1, &w), "check-ip").outcome, Outcome::Delivered);
}

#[test]
fn the_learners_line_check_beats_our_data() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    dial(&mut w, public());
    w.router_mut("home-router").expect("router").port_forwards.push(forward(8096, "192.168.1.20", 8096));
    // The teaching profile says this ISP shares addresses.
    w.isp_mut("isp").expect("isp").ipv4 = LineMode::Profile;
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "isp", Reason::CgnatNoMapping);
    // The learner checked their own line and found a public address.
    w.line_checks.push(LineCheck { isp: "isp".into(), result: LineCheckResult::Public });
    let g = grade(m, &w);
    assert_eq!(attempt(&g, "open-jellyfin").outcome, Outcome::Delivered);
    let f = g.report.findings.iter().find(|f| f.rule_id == Reason::LineHasPublicIpv4).expect("finding");
    assert_eq!(f.provenance, sim_core::Provenance::LineCheck);
    // "Can't find it" is a valid answer and yields unknown.
    w.line_checks[0].result = LineCheckResult::Unknown;
    assert_eq!(attempt(&grade(m, &w), "open-jellyfin").status, Status::Unknown);
}

#[test]
fn a_profile_with_no_claim_is_unknown() {
    let m = mission("open-a-door");
    let mut b = bundle().clone();
    b.isps.get_mut("teaching-fibre").expect("profile").claims.clear();
    let mut w = m.world.clone();
    dial(&mut w, public());
    w.isp_mut("isp").expect("isp").ipv4 = LineMode::Profile;
    let g = sim_core::grade(m, Depth::Guided, &w, &Artifacts::default(), &b);
    assert_eq!(g.status, Status::Unknown);
}

#[test]
fn ipv6_is_absent_unless_the_line_has_it() {
    let m = mission("open-a-door");
    let mut w = m.world.clone();
    dial(&mut w, Target::Ipv6Of { node: "server".into() });
    dropped(attempt(&grade(m, &w), "open-jellyfin"), "phone", Reason::Ipv6NotAvailable);
    w.isp_mut("isp").expect("isp").ipv6 = Ipv6Mode::Unknown;
    assert_eq!(attempt(&grade(m, &w), "open-jellyfin").status, Status::Unknown);
}

// ------------------------------------------------------------ Determinism

#[test]
fn identical_input_gives_identical_bytes() {
    let again = data_build::build(&root()).expect("builds twice");
    assert_eq!(serde_json::to_string(bundle()).expect("json"), serde_json::to_string(&again).expect("json"));

    for m in bundle().missions.values() {
        let mut w = m.world.clone();
        join_all(&mut w);
        let first = serde_json::to_string(&grade(m, &w)).expect("json");
        // Through JSON and back, as the browser does it.
        let round: World = serde_json::from_str(&serde_json::to_string(&w).expect("json")).expect("world");
        assert_eq!(round, w);
        assert_eq!(first, serde_json::to_string(&grade(m, &round)).expect("json"));
        // Order of nodes and links in the file must not change any decision.
        let mut shuffled = w.clone();
        shuffled.nodes.reverse();
        shuffled.links.reverse();
        let a = grade(m, &w);
        let b = grade(m, &shuffled);
        assert_eq!(a.goals, b.goals);
        assert_eq!(a.report.findings, b.report.findings);
        assert_eq!(a.report.attempts, b.report.attempts);
    }
}

fn join_all(world: &mut World) {
    for node in &mut world.nodes {
        if let Node::Host(host) = node {
            host.tunnels.iter_mut().for_each(|t| t.enabled = true);
        }
    }
}

// ------------------------------------------------ Mission 5: write the tunnel

// A key pair made with Node's crypto module, not with our own code.
const SERVER_PRIVATE: &str = "EPXV4Q6FxuX4RT/KMQEiBWRYOGE5t/32aTjX944kZ18=";
const SERVER_PUBLIC: &str = "8zk2Okc5d/z9exmtbj/2cGf2/lZdwOfXFU7s4sItJjo=";
const HUB_PUBLIC: &str = "W9J3Hqm8EgeN6UewWxIP6ccD2CQZ8cC6fWREezgTW1U=";
const PHONE_PUBLIC: &str = "SkifF5LojM/Ns7UUYJeZMb/cG4NPREjAS+VB5gCXTW0=";

fn good_conf() -> String {
    format!(
        "[Interface]\nPrivateKey = {SERVER_PRIVATE}\nAddress = 10.8.0.2/24\n\n[Peer]\nPublicKey = {HUB_PUBLIC}\n\
         Endpoint = 192.0.2.50:51820\nAllowedIPs = 10.8.0.0/24\nPersistentKeepalive = 25\n"
    )
}

fn files(conf: Option<&str>, public_key: Option<&str>) -> Artifacts {
    let mut artifacts = Artifacts::default();
    if let Some(conf) = conf {
        artifacts.files.insert("/root/wg0.conf".into(), conf.into());
    }
    if let Some(key) = public_key {
        artifacts.files.insert("/root/publickey".into(), key.into());
    }
    artifacts
}

fn tunnel_grade(conf: Option<&str>, public_key: Option<&str>) -> Grade {
    let m = mission("write-the-tunnel");
    sim_core::grade(m, Depth::Guided, &m.world, &files(conf, public_key), bundle())
}

fn artifact(grade: &Grade, id: &str) -> (Status, Reason) {
    let a = grade.artifacts.iter().find(|a| a.id == id).expect("artifact exists");
    (a.status, a.reason)
}

fn setup<'a>(trace: &'a Trace, id: &str) -> &'a sim_core::trace::Leg {
    trace.legs.iter().find(|leg| leg.id == id).unwrap_or_else(|| panic!("no leg {id}"))
}

fn leg_dropped(leg: &sim_core::trace::Leg, at: &str, reason: Reason) {
    let last = leg.steps.last().expect("steps");
    assert_eq!(last.action, StepAction::Drop);
    assert_eq!((last.at.as_str(), last.reason), (at, reason));
}

#[test]
fn m5_starts_with_no_files_and_no_tunnel() {
    let g = tunnel_grade(None, None);
    assert_eq!(g.status, Status::Fail);
    assert_eq!(artifact(&g, "wg0-conf"), (Status::Fail, Reason::ArtifactMissing));
    assert_eq!(artifact(&g, "server-public-key"), (Status::Fail, Reason::ArtifactMissing));
    // The phone reaches the hub, and the hub has nowhere to send it.
    dropped(attempt(&g, "reach-now"), "hub", Reason::TunnelNoEndpoint);
}

#[test]
fn m5_the_starter_file_is_not_yet_a_config() {
    let m = mission("write-the-tunnel");
    let starter = &m.artifacts[0].starter;
    let g = tunnel_grade(Some(starter), Some(SERVER_PUBLIC));
    let conf = g.artifacts.iter().find(|a| a.id == "wg0-conf").expect("artifact");
    assert_eq!((conf.status, conf.reason), (Status::Fail, Reason::ArtifactInvalid));
    let lines: Vec<u32> = conf.problems.iter().map(|p| p.line).collect();
    assert_eq!(lines, vec![4, 12, 14], "{:?}", conf.problems);
    // With no readable config there is no private key to compare with.
    assert_eq!(artifact(&g, "server-public-key"), (Status::Unknown, Reason::ArtifactKeyUnchecked));
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m5_a_correct_config_wins_through_the_hub() {
    let g = tunnel_grade(Some(&good_conf()), Some(&format!("{SERVER_PUBLIC}\n")));
    assert_eq!(artifact(&g, "wg0-conf"), (Status::Pass, Reason::ArtifactOk));
    assert_eq!(artifact(&g, "server-public-key"), (Status::Pass, Reason::ArtifactOk));
    assert_eq!(goal(&g, "reach-now"), (Status::Pass, Reason::GoalMet));
    assert_eq!(goal(&g, "reach-later"), (Status::Pass, Reason::GoalMet));
    assert_eq!(g.status, Status::Pass);

    let t = attempt(&g, "reach-later");
    assert_eq!(t.outcome, Outcome::Delivered);
    assert_eq!(t.ended_at.as_deref(), Some("server"));
    // The server dialled first. Its handshake went out through both NATs.
    let dial = setup(t, "setup-server-wg0");
    assert_eq!(dial.role, LegRole::Setup);
    assert_eq!(dial.steps.first().expect("step").at, "server");
    assert_eq!(dial.steps.last().expect("step").at, "hub");
    assert_eq!(dial.steps.last().expect("step").reason, Reason::TunnelSessionUp);
    // The phone's packet is passed on by the hub, and gets in on the mapping
    // the server's own packets keep alive.
    let request = t.request().expect("request");
    let path: Vec<&str> = request.steps.iter().map(|s| s.at.as_str()).collect();
    assert_eq!(path, ["phone", "mobile", "internet", "hub", "internet", "isp", "home-router", "server", "server"]);
    assert!(request.steps.iter().any(|s| s.at == "hub" && s.reason == Reason::HubForwards));
    let at_isp = request.steps.iter().find(|s| s.at == "isp").expect("isp step");
    let table = at_isp.table.as_ref().expect("table");
    assert_eq!(table.kind, TableKind::MappingHit);
    assert_eq!(table.row.as_ref().expect("row").idle_s, 25);
    let reply = t.legs.iter().find(|leg| leg.role == LegRole::Reply).expect("reply");
    assert_eq!(reply.steps.last().expect("step").at, "phone");
    assert_eq!(finding(&g, Reason::TunnelKeepalive), (Status::Pass, Reason::TunnelKeepalive));
}

#[test]
fn m5_without_a_keepalive_it_works_at_first_then_the_mapping_expires() {
    let conf = good_conf().replace("PersistentKeepalive = 25\n", "");
    let g = tunnel_grade(Some(&conf), Some(SERVER_PUBLIC));
    assert_eq!(artifact(&g, "wg0-conf"), (Status::Pass, Reason::ArtifactOk));
    assert_eq!(goal(&g, "reach-now"), (Status::Pass, Reason::GoalMet));
    assert_eq!(goal(&g, "reach-later"), (Status::Fail, Reason::GoalNotMet));
    dropped(attempt(&g, "reach-later"), "isp", Reason::MappingExpired);
    assert_eq!(finding(&g, Reason::TunnelKeepalive), (Status::Fail, Reason::TunnelKeepaliveMissing));
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m5_a_keepalive_slower_than_the_isp_timeout_also_expires() {
    // The home ISP forgets after 60 seconds, the router after 120.
    for (seconds, stops_at) in [(59, None), (60, Some("isp")), (90, Some("isp")), (300, Some("isp"))] {
        let conf = good_conf().replace("= 25", &format!("= {seconds}"));
        let g = tunnel_grade(Some(&conf), Some(SERVER_PUBLIC));
        match stops_at {
            None => assert_eq!(g.status, Status::Pass, "{seconds}"),
            Some(node) => {
                assert_eq!(goal(&g, "reach-now"), (Status::Pass, Reason::GoalMet), "{seconds}");
                dropped(attempt(&g, "reach-later"), node, Reason::MappingExpired);
                assert_eq!(finding(&g, Reason::TunnelKeepalive), (Status::Fail, Reason::TunnelKeepaliveTooSlow));
            }
        }
    }
}

#[test]
fn m5_a_wrong_endpoint_never_reaches_the_hub() {
    // Wrong port: the hub is there, but nothing listens on that port.
    let g = tunnel_grade(Some(&good_conf().replace(":51820", ":51821")), Some(SERVER_PUBLIC));
    let t = attempt(&g, "reach-now");
    leg_dropped(setup(t, "setup-server-wg0"), "hub", Reason::NoService);
    dropped(t, "hub", Reason::TunnelNoEndpoint);
    assert_eq!(g.status, Status::Fail);
    // Wrong address: nobody is there at all.
    let g = tunnel_grade(Some(&good_conf().replace("192.0.2.50", "192.0.2.51")), Some(SERVER_PUBLIC));
    let t = attempt(&g, "reach-now");
    let dial = setup(t, "setup-server-wg0");
    assert_eq!(dial.steps.last().expect("step").action, StepAction::Drop);
    assert_ne!(dial.steps.last().expect("step").at, "hub");
    dropped(t, "hub", Reason::TunnelNoEndpoint);
    // The server's own address, as some guides wrongly show.
    let g = tunnel_grade(Some(&good_conf().replace("192.0.2.50", "10.8.0.1")), Some(SERVER_PUBLIC));
    assert_eq!(goal(&g, "reach-now"), (Status::Fail, Reason::GoalNotMet));
}

#[test]
fn m5_allowed_ips_too_narrow_drops_the_phone_at_the_server() {
    // Only the hub's own address: the phone's packets come from 10.8.0.3.
    let g = tunnel_grade(Some(&good_conf().replace("10.8.0.0/24", "10.8.0.1/32")), Some(SERVER_PUBLIC));
    assert_eq!(artifact(&g, "wg0-conf"), (Status::Pass, Reason::ArtifactOk));
    let t = attempt(&g, "reach-now");
    assert_eq!(setup(t, "setup-server-wg0").steps.last().expect("step").reason, Reason::TunnelSessionUp);
    dropped(t, "server", Reason::TunnelAllowedIps);
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m5_a_public_key_from_another_pair_is_refused_by_the_hub() {
    // A real key, from RFC 7748, that belongs to nobody here.
    let g = tunnel_grade(Some(&good_conf()), Some("hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTmo="));
    assert_eq!(artifact(&g, "wg0-conf"), (Status::Pass, Reason::ArtifactOk));
    assert_eq!(artifact(&g, "server-public-key"), (Status::Fail, Reason::ArtifactKeyMismatch));
    let t = attempt(&g, "reach-now");
    leg_dropped(setup(t, "setup-server-wg0"), "hub", Reason::TunnelUnknownPeer);
    dropped(t, "hub", Reason::TunnelNoEndpoint);
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m5_a_missing_or_broken_public_key_never_passes() {
    for key in [None, Some("not a key"), Some("")] {
        let g = tunnel_grade(Some(&good_conf()), key);
        let expected = if key.is_none() { Reason::ArtifactMissing } else { Reason::ArtifactInvalid };
        assert_eq!(artifact(&g, "server-public-key"), (Status::Fail, expected));
        leg_dropped(setup(attempt(&g, "reach-now"), "setup-server-wg0"), "hub", Reason::TunnelUnknownPeer);
        assert_eq!(g.status, Status::Fail);
    }
}

#[test]
fn m5_the_wrong_key_for_the_hub_cannot_be_opened() {
    let g = tunnel_grade(Some(&good_conf().replace(HUB_PUBLIC, PHONE_PUBLIC)), Some(SERVER_PUBLIC));
    assert_eq!(artifact(&g, "wg0-conf"), (Status::Pass, Reason::ArtifactOk));
    let t = attempt(&g, "reach-now");
    leg_dropped(setup(t, "setup-server-wg0"), "hub", Reason::TunnelWrongPeerKey);
    assert_eq!(g.status, Status::Fail);
}

#[test]
fn m5_the_wrong_tunnel_address_is_not_the_one_the_phone_dials() {
    let g = tunnel_grade(Some(&good_conf().replace("10.8.0.2/24", "10.8.0.9/24")), Some(SERVER_PUBLIC));
    assert_eq!(goal(&g, "reach-now"), (Status::Fail, Reason::GoalNotMet));
    assert_eq!(attempt(&g, "reach-now").ended_at.as_deref(), Some("server"));
}

#[test]
fn m5_the_private_key_is_nowhere_in_the_grade() {
    let g = tunnel_grade(Some(&good_conf()), Some(SERVER_PUBLIC));
    let json = serde_json::to_string(&g).expect("json");
    assert!(!json.contains(SERVER_PRIVATE));
}

// ----------------------------------- A new device is one YAML file, no code

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("mkdir");
    for entry in fs::read_dir(from).expect("read dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy");
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    for part in ["data", "content", "missions"] {
        copy_dir(&root().join(part), &dir.join(part));
    }
    fs::create_dir_all(dir.join("data/devices")).expect("mkdir");
    dir
}

fn vm_on(device: &str) -> World {
    let mut w = mission("open-a-door").world.clone();
    w.host_mut("server").expect("server").device = Some(device.into());
    w.nodes.push(Node::Host(Host {
        id: "vm".into(),
        label: "Virtual machine".into(),
        runs_on: Some("server".into()),
        ..Host::default()
    }));
    w
}

#[test]
fn a_new_device_is_one_yaml_file_and_no_code() {
    let dir = scratch("one-yaml");
    let source = "{ source: \"https://example.org/spec\", retrieved_at: \"2026-11-10\" }";
    let sourced = |value: &str| format!("{{ value: {value}, {} }}", &source[2..source.len() - 2]);
    fs::write(
        dir.join("data/devices/test-box.yaml"),
        format!(
            "id: test-box\nkind: device\nlabel: Test box\ncapabilities:\n  arch: {}\n  ram_gb: {}\n  nic: [{{ speed_mbps: {} }}]\n  virtualization: {}\n  igpu_transcode: unknown\npower: {{ idle_w: unknown, load_w: unknown }}\n",
            sourced("arm64"),
            sourced("4"),
            sourced("1000"),
            sourced("false"),
        ),
    )
    .expect("write");
    let b = data_build::build(&dir).expect("builds with the new device");
    assert!(b.devices.contains_key("test-box"));

    let find = |world: &World, bundle: &Bundle| {
        let report = sim_core::evaluate(world, bundle);
        let f = report.findings.into_iter().find(|f| f.rule_id == Reason::HostVirtualization).expect("finding");
        (f.status, f.explanation_id)
    };
    // The rule read the capability. It never saw the name.
    assert_eq!(find(&vm_on("test-box"), &b), (Status::Fail, Reason::HostVirtualization));
    assert_eq!(find(&vm_on("old-laptop"), &b), (Status::Pass, Reason::HostVirtualization));
    // A device we have no pack for is unknown, not a pass.
    assert_eq!(find(&vm_on("never-heard-of-it"), &b), (Status::Unknown, Reason::DeviceDataMissing));
}

#[test]
fn the_data_build_refuses_unsourced_claims() {
    let dir = scratch("unsourced");
    fs::write(
        dir.join("data/devices/bare.yaml"),
        "id: bare\nkind: device\nlabel: Bare\ncapabilities:\n  arch: x86_64\n  ram_gb: unknown\n  nic: []\n  virtualization: true\n  igpu_transcode: unknown\npower: { idle_w: unknown, load_w: unknown }\n",
    )
    .expect("write");
    fs::write(
        dir.join("data/isps/real-net.yaml"),
        "id: real-net\nlabel: Real Net\nkind: real\nclaims:\n  ipv4_cgnat: { value: \"yes\" }\n  static_ipv4: { value: unknown }\n",
    )
    .expect("write");
    let errors = data_build::build(&dir).expect_err("must fail");
    assert!(errors.iter().any(|e| e.contains("bare.yaml") && e.contains("`arch`")), "{errors:?}");
    assert!(errors.iter().any(|e| e.contains("bare.yaml") && e.contains("`virtualization`")), "{errors:?}");
    assert!(errors.iter().any(|e| e.contains("real-net.yaml") && e.contains("ipv4_cgnat")), "{errors:?}");
    assert!(!errors.iter().any(|e| e.contains("static_ipv4")), "an unknown claim needs no source: {errors:?}");
}

#[test]
fn the_data_build_refuses_a_reason_with_no_words() {
    let dir = scratch("no-words");
    let steps = fs::read_to_string(dir.join("content/steps.yaml")).expect("read");
    let cut = steps.replace("cgnat-no-inbound-mapping:", "some-other-id:");
    fs::write(dir.join("content/steps.yaml"), cut).expect("write");
    fs::remove_file(dir.join("content/explanations/cgnat-no-inbound-mapping.md")).expect("remove");
    let errors = data_build::build(&dir).expect_err("must fail");
    assert!(errors.iter().any(|e| e.contains("no text for `cgnat-no-inbound-mapping`")), "{errors:?}");
}

// ------------------------------------------- Mission reports stay the same

/// Grades of every mission, at both depths, from its starting world and from
/// a solved one, plus a world that exercises a device pack. Schema changes
/// must not move a byte of these. To accept a deliberate change, run the test
/// with `UPDATE_MISSION_FIXTURES=1` and review the diff.
#[test]
fn mission_reports_match_the_recorded_fixtures() {
    let mut out = std::collections::BTreeMap::new();
    for m in bundle().missions.values() {
        for depth in [Depth::Guided, Depth::Challenge] {
            let name = |state: &str| format!("{}/{}/{state}", m.id, format!("{depth:?}").to_lowercase());
            let graded = |world: &World| {
                serde_json::to_value(sim_core::grade(m, depth, world, &Artifacts::default(), bundle())).expect("json")
            };
            out.insert(name("start"), graded(&m.world));
            if m.id == "write-the-tunnel" {
                // Solved by its files, not by changes to the world.
                let solved = files(Some(&good_conf()), Some(SERVER_PUBLIC));
                let grade = sim_core::grade(m, depth, &m.world, &solved, bundle());
                out.insert(name("solved"), serde_json::to_value(grade).expect("json"));
                continue;
            }
            let mut solved = m.world.clone();
            join_all(&mut solved);
            if let Some(router) = solved.router_mut("home-router") {
                router.port_forwards = vec![forward(8096, "192.168.1.20", 8096)];
            }
            if solved.attempt_mut("open-jellyfin").is_some() {
                dial(&mut solved, if m.id == "dial-out" { mesh() } else { public() });
            }
            out.insert(name("solved"), graded(&solved));
        }
    }
    for device in ["old-laptop", "single-board-computer", "never-heard-of-it"] {
        let report = sim_core::evaluate(&vm_on(device), bundle());
        out.insert(format!("vm-on/{device}"), serde_json::to_value(report).expect("json"));
    }
    // One entry per line, so a change shows up as one changed line.
    let lines: Vec<String> = out.iter().map(|(name, value)| format!("{name}\t{value}")).collect();
    let text = lines.join("\n") + "\n";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mission-reports.tsv");
    if std::env::var_os("UPDATE_MISSION_FIXTURES").is_some() {
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(&path, &text).expect("write");
    }
    let recorded = fs::read_to_string(&path).expect("fixture file exists");
    assert!(recorded == text, "mission reports changed. Compare against {}", path.display());
}
