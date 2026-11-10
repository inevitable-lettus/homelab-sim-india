//! Reads `data/`, `content/` and `missions/`, validates them against the Rust
//! schema in sim-core, and returns one bundle. Every problem is reported with
//! the file it came from, and the build fails if there is any.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use sim_core::bundle::{
    Bundle, Cap, ClaimValue, Control, DevicePack, Explanation, GoalCheck, IspProfile, Mission, PackKind, ProfileKind,
    SCHEMA_VERSION,
};
use sim_core::model::Node;
use sim_core::{Artifacts, Depth, Reason, Status};

pub type Errors = Vec<String>;

fn files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == extension))
        .collect();
    found.sort();
    found
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn read_yaml<T: DeserializeOwned>(path: &Path, errors: &mut Errors) -> Option<T> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            errors.push(format!("{}: cannot read: {e}", path.display()));
            return None;
        }
    };
    match serde_norway::from_str(&text) {
        Ok(value) => Some(value),
        Err(e) => {
            errors.push(format!("{}: {e}", path.display()));
            None
        }
    }
}

fn check_id(path: &Path, expected: &str, id: &str, errors: &mut Errors) {
    if id != expected {
        errors.push(format!("{}: id `{id}` must match the file or folder name `{expected}`", path.display()));
    }
}

fn load_devices(root: &Path, errors: &mut Errors) -> BTreeMap<String, DevicePack> {
    let mut devices = BTreeMap::new();
    for (folder, kind) in [("data/archetypes", PackKind::Archetype), ("data/devices", PackKind::Device)] {
        for path in files(&root.join(folder), "yaml") {
            let Some(pack) = read_yaml::<DevicePack>(&path, errors) else {
                continue;
            };
            check_id(&path, &stem(&path), &pack.id, errors);
            if pack.kind != kind {
                errors
                    .push(format!("{}: files in `{folder}` must have `kind: {kind:?}`", path.display()).to_lowercase());
            }
            if pack.kind == PackKind::Device {
                // A real product may not carry a value without saying where it came from.
                let caps = &pack.capabilities;
                let mut unsourced = vec![
                    ("arch", caps.arch.is_unsourced_value()),
                    ("ram_gb", caps.ram_gb.is_unsourced_value()),
                    ("virtualization", caps.virtualization.is_unsourced_value()),
                    ("igpu_transcode", caps.igpu_transcode.is_unsourced_value()),
                    ("power.idle_w", pack.power.idle_w.is_unsourced_value()),
                    ("power.load_w", pack.power.load_w.is_unsourced_value()),
                ];
                unsourced.extend(caps.nic.iter().map(|nic| ("nic.speed_mbps", nic.speed_mbps.is_unsourced_value())));
                for (name, _) in unsourced.into_iter().filter(|(_, bad)| *bad) {
                    errors.push(format!(
                        "{}: `{name}` needs `{{ value, source, retrieved_at }}` in a real device, or `unknown`",
                        path.display()
                    ));
                }
            }
            if devices.insert(pack.id.clone(), pack).is_some() {
                errors.push(format!("{}: duplicate device id", path.display()));
            }
        }
    }
    devices
}

fn load_isps(root: &Path, errors: &mut Errors) -> BTreeMap<String, IspProfile> {
    let mut isps = BTreeMap::new();
    for path in files(&root.join("data/isps"), "yaml") {
        let Some(profile) = read_yaml::<IspProfile>(&path, errors) else {
            continue;
        };
        check_id(&path, &stem(&path), &profile.id, errors);
        if profile.kind == ProfileKind::Real {
            // Every claim about a real ISP carries a source and a date, or is `unknown`.
            for (name, claim) in &profile.claims {
                if claim.value != ClaimValue::Unknown && (claim.sources.is_empty() || claim.checked.is_none()) {
                    errors.push(format!(
                        "{}: claim `{name}` needs `sources` and `checked`, or `value: unknown`",
                        path.display()
                    ));
                }
            }
        }
        isps.insert(profile.id.clone(), profile);
    }
    isps
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StepText {
    title: String,
    line: String,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct FrontMatter {
    title: Option<String>,
    line: Option<String>,
}

fn load_explanations(root: &Path, errors: &mut Errors) -> BTreeMap<String, Explanation> {
    let mut explanations = BTreeMap::new();
    let steps_path = root.join("content/steps.yaml");
    if steps_path.exists()
        && let Some(steps) = read_yaml::<BTreeMap<String, StepText>>(&steps_path, errors)
    {
        for (id, step) in steps {
            explanations.insert(id, Explanation { title: step.title, line: step.line, ..Explanation::default() });
        }
    }
    for path in files(&root.join("content/explanations"), "md") {
        let id = stem(&path);
        let text = fs::read_to_string(&path).unwrap_or_default();
        let (front, body) = match text.strip_prefix("---\n").and_then(|rest| rest.split_once("\n---\n")) {
            Some((front, body)) => match serde_norway::from_str::<FrontMatter>(front) {
                Ok(front) => (front, body),
                Err(e) => {
                    errors.push(format!("{}: front matter: {e}", path.display()));
                    continue;
                }
            },
            None => (FrontMatter::default(), text.as_str()),
        };
        let entry = explanations.entry(id.clone()).or_default();
        if let Some(title) = front.title {
            entry.title = title;
        }
        if let Some(line) = front.line {
            entry.line = line;
        }
        let mut heading: Option<String> = None;
        let mut sections: BTreeMap<String, String> = BTreeMap::new();
        for line in body.lines() {
            if let Some(title) = line.strip_prefix("## ") {
                heading = Some(title.trim().to_lowercase());
            } else if let Some(heading) = &heading {
                let section = sections.entry(heading.clone()).or_default();
                section.push_str(line);
                section.push('\n');
            } else if !line.trim().is_empty() {
                errors.push(format!("{}: text before the first `## ` heading", path.display()));
                break;
            }
        }
        for (heading, text) in sections {
            let text = Some(text.trim().to_owned()).filter(|t| !t.is_empty());
            match heading.as_str() {
                "what happened" => entry.what_happened = text,
                "the concept" => entry.concept = text,
                "why it matters" => entry.why_it_matters = text,
                "how to fix it" => entry.how_to_fix = text,
                "on a real line" => entry.real_line = text,
                "go deeper" => entry.go_deeper = text,
                other => errors.push(format!("{}: unknown section `## {other}`", path.display())),
            }
        }
    }
    for (id, explanation) in &explanations {
        if explanation.title.trim().is_empty() || explanation.line.trim().is_empty() {
            errors.push(format!("content: explanation `{id}` needs a title and a line"));
        }
    }
    // The core can say each of these, so each needs words.
    for reason in Reason::ALL {
        if !explanations.contains_key(reason.id()) {
            errors.push(format!("content: no text for `{}`. Add it to content/steps.yaml", reason.id()));
        }
    }
    explanations
}

fn load_missions(root: &Path, errors: &mut Errors) -> BTreeMap<String, Mission> {
    let mut missions = BTreeMap::new();
    let mut folders: Vec<PathBuf> = fs::read_dir(root.join("missions"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    for folder in folders {
        let path = folder.join("mission.yaml");
        let Some(mission) = read_yaml::<Mission>(&path, errors) else {
            continue;
        };
        let name = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        check_id(&path, &name, &mission.id, errors);
        missions.insert(mission.id.clone(), mission);
    }
    missions
}

fn check_mission(path: &str, mission: &Mission, bundle: &Bundle, errors: &mut Errors) {
    let world = &mission.world;
    let mut fail = |message: String| errors.push(format!("{path}: {message}"));
    let has_node = |id: &str| world.node(id).is_some();
    let has_attempt = |id: &str| world.attempts.iter().any(|a| a.id == id);
    let has_question = |id: &str| mission.questions.iter().any(|q| q.id == id);

    let mut ids: Vec<&str> = world.nodes.iter().map(Node::id).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        fail("two nodes share an id".into());
    }
    for link in &world.links {
        for end in [&link.a, &link.b] {
            if !has_node(end) {
                fail(format!("link `{}` names unknown node `{end}`", link.id));
            }
        }
    }
    for node in &world.nodes {
        for (name, layout) in [("wide", &mission.layout.wide), ("narrow", &mission.layout.narrow)] {
            if !layout.contains_key(node.id()) {
                fail(format!("layout.{name} has no position for `{}`", node.id()));
            }
        }
        match node {
            Node::Host(host) => {
                if let Some(device) = &host.device
                    && !bundle.devices.contains_key(device)
                {
                    fail(format!("`{}` uses unknown device `{device}`", host.id));
                }
                if let Some(parent) = &host.runs_on
                    && !has_node(parent)
                {
                    fail(format!("`{}` runs on unknown node `{parent}`", host.id));
                }
            }
            Node::Isp(isp) => {
                if let Some(profile) = &isp.profile
                    && !bundle.isps.contains_key(profile)
                {
                    fail(format!("`{}` uses unknown ISP profile `{profile}`", isp.id));
                }
            }
            Node::Router(_) | Node::Internet(_) => {}
        }
    }
    for attempt in &world.attempts {
        if !has_node(&attempt.from) {
            fail(format!("attempt `{}` starts at unknown node `{}`", attempt.id, attempt.from));
        }
    }
    for overlay in &world.overlays {
        if !has_node(&overlay.relay) {
            fail(format!("overlay `{}` names unknown relay `{}`", overlay.id, overlay.relay));
        }
    }
    for control in &mission.controls {
        let ok = match control {
            Control::Dial { attempt, .. } => has_attempt(attempt),
            Control::PortForwards { router } => matches!(world.node(router), Some(Node::Router(_))),
            Control::OverlayJoin { network, nodes } => {
                world.overlays.iter().any(|o| o.id == *network) && nodes.iter().all(|n| has_node(n))
            }
            Control::Question { question } => has_question(question),
            Control::Tunnel { node, .. } => has_node(node),
        };
        if !ok {
            fail(format!("control {control:?} refers to something that does not exist"));
        }
    }
    for goal in &mission.goals {
        let ok = match &goal.check {
            GoalCheck::Delivered { attempt, to, .. } => has_attempt(attempt) && to.as_ref().is_none_or(|n| has_node(n)),
            GoalCheck::Reaches { attempt, node } => has_attempt(attempt) && has_node(node),
            GoalCheck::Answer { question } => has_question(question),
            GoalCheck::RulePasses { rule } => Reason::ALL.iter().any(|r| r.id() == rule),
        };
        if !ok {
            fail(format!("goal `{}` refers to something that does not exist", goal.id));
        }
    }
    if mission.goals.is_empty() {
        fail("a mission needs at least one goal".into());
    }
    if !bundle.explanations.contains_key(&mission.debrief.concept) {
        fail(format!("debrief concept `{}` has no explanation", mission.debrief.concept));
    }
    // A mission that is already solved when it opens teaches nothing.
    for depth in [Depth::Guided, Depth::Challenge] {
        let grade = sim_core::grade(mission, depth, world, &Artifacts::default(), bundle);
        if grade.status == Status::Pass {
            fail(format!("the starting world already passes at {depth:?} depth"));
        }
        if grade.report.findings.iter().any(|f| f.rule_id == Reason::EvaluationDidNotSettle) {
            fail("the starting world does not settle".into());
        }
    }
}

pub fn build(root: &Path) -> Result<Bundle, Errors> {
    let mut errors = Errors::new();
    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION,
        devices: load_devices(root, &mut errors),
        isps: load_isps(root, &mut errors),
        explanations: load_explanations(root, &mut errors),
        missions: BTreeMap::new(),
    };
    let missions = load_missions(root, &mut errors);
    let mut orders = BTreeMap::new();
    for (id, mission) in &missions {
        check_mission(&format!("missions/{id}/mission.yaml"), mission, &bundle, &mut errors);
        if let Some(other) = orders.insert(mission.order, id) {
            errors.push(format!("missions `{id}` and `{other}` both have order {}", mission.order));
        }
    }
    bundle.missions = missions;
    if errors.is_empty() { Ok(bundle) } else { Err(errors) }
}

pub fn write(bundle: &Bundle, output: &Path) -> Result<String, Errors> {
    let io = |e: std::io::Error| vec![format!("{}: {e}", output.display())];
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    let mut json = serde_json::to_string(bundle).map_err(|e| vec![e.to_string()])?;
    json.push('\n');
    fs::write(output, &json).map_err(io)?;
    Ok(format!(
        "{} devices, {} ISP profiles, {} explanations, {} missions, {} bytes",
        bundle.devices.len(),
        bundle.isps.len(),
        bundle.explanations.len(),
        bundle.missions.len(),
        json.len()
    ))
}

// Unused import guard: `Cap` is part of the schema this crate validates.
const _: fn(&Cap<u32>) -> bool = Cap::is_unsourced_value;
