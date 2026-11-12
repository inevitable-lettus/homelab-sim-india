//! The sandbox templates, built from the repo's own packs: each goal comes
//! out the way the template is meant to teach.

use std::path::PathBuf;

use sim_core::{Reason, Status};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_template_teaches_what_it_says() {
    use Status::{Fail, Pass, Unknown};
    let bundle = data_build::build(&root()).expect("the repo builds");
    let want: &[(&str, &[(&str, Status)])] = &[
        (
            "old-laptop-media-server",
            // Behind carrier NAT, the phone on mobile data cannot get in.
            &[("run-media", Pass), ("watch-on-tv", Pass), ("watch-away", Fail), ("power-cut", Pass)],
        ),
        // No Ethernet port for the cable, until a USB adapter is added.
        ("macbook-air-as-server", &[("run-media", Fail), ("watch-on-tv", Fail)]),
        // We know nothing about the ISP's router.
        ("pi-hole-whole-house", &[("run-dns", Pass), ("adblock-house", Unknown)]),
        ("mini-pc-proxmox", &[("run-media", Pass), ("run-dns", Pass)]),
        // The DNS server has no Android build.
        ("old-phone-tiny-server", &[("run-sync", Pass), ("run-dns", Fail)]),
        // The UPS feeds the server and not the router.
        ("power-cut-proof", &[("reach-away", Pass), ("power-cut", Fail), ("running-cost", Pass)]),
    ];
    assert_eq!(bundle.templates.len(), want.len(), "a template with no expectations here");
    for (id, goals) in want {
        let template = bundle.templates.get(*id).unwrap_or_else(|| panic!("template {id}"));
        let report = sim_core::evaluate(&template.world, &bundle);
        let build = report.build.as_ref().expect("a template is a build");
        assert!(!report.findings.iter().any(|f| f.rule_id == Reason::IntentNotChecked), "{id}: a goal nothing checked");
        assert_eq!(build.intents.len(), goals.len(), "{id}");
        for (goal, status) in *goals {
            let got = build.intents.iter().find(|i| i.id == *goal).unwrap_or_else(|| panic!("{id}: goal {goal}"));
            let against: Vec<_> = report
                .findings
                .iter()
                .filter(|f| f.intents.iter().any(|i| i == goal) && f.status != Pass)
                .map(|f| (f.rule_id.id(), f.status, f.severity, f.involved.clone()))
                .collect();
            assert_eq!(got.status, *status, "{id}: {goal}: {against:?}");
        }
    }
}

#[test]
fn the_fixes_the_templates_point_at_work() {
    let bundle = data_build::build(&root()).expect("the repo builds");
    let status = |world: &sim_core::World, goal: &str| {
        let report = sim_core::evaluate(world, &bundle);
        report.build.unwrap().intents.iter().find(|i| i.id == goal).unwrap().status
    };

    // A USB Ethernet adapter gives the laptop a port for its cable.
    let mut world = bundle.templates["macbook-air-as-server"].world.clone();
    let part = serde_json::from_value(serde_json::json!({ "id": "nic", "pack": "usb-ethernet-adapter" })).unwrap();
    world.host_mut("server").unwrap().parts.push(part);
    assert_eq!(status(&world, "run-media"), Status::Pass);

    // A small DC UPS on the router keeps the way in open through the cut.
    let mut world = bundle.templates["power-cut-proof"].world.clone();
    let json =
        serde_json::json!({ "kind": "power", "id": "router-ups", "label": "Router UPS", "device": "dc-mini-ups" });
    world.nodes.push(serde_json::from_value(json).unwrap());
    world
        .power_feeds
        .push(serde_json::from_value(serde_json::json!({ "from": "router-ups", "to": "router" })).unwrap());
    assert_eq!(status(&world, "power-cut"), Status::Pass);
}
