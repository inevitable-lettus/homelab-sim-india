//! Reads `data/`, `content/` and `missions/`, validates them against the Rust
//! schema in sim-core, and returns one bundle. Every problem is reported with
//! the file it came from, and the build fails if there is any.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use sim_core::bundle::{
    Bundle, ClaimValue, Control, DevicePack, Explanation, GoalCheck, IspProfile, Layout, MediaFormat, Mission, OsPack,
    Outage, PackKind, ProfileKind, RuntimePack, SCHEMA_VERSION, ServicePack, Tariff, Template, Track,
};
use sim_core::model::{Node, World};
use sim_core::packs::{Audit, Leaf, LeafState};
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

/// Query-string parameters that track the reader. Sources may not carry them.
const TRACKING: [&str; 3] = ["utm_", "tag=", "affid"];

fn read_yaml<T: DeserializeOwned>(path: &Path, errors: &mut Errors) -> Option<T> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            errors.push(format!("{}: cannot read: {e}", path.display()));
            return None;
        }
    };
    for (n, line) in text.lines().enumerate() {
        let query = line.split_once("://").and_then(|(_, rest)| rest.split_once('?')).map(|(_, query)| query);
        if let Some(marker) = query.and_then(|q| TRACKING.iter().find(|marker| q.contains(**marker))) {
            errors.push(format!(
                "{}: line {}: a source link carries the tracking parameter `{marker}`",
                path.display(),
                n + 1
            ));
        }
    }
    match serde_path_to_error::deserialize(serde_norway::Deserializer::from_str(&text)) {
        Ok(value) => Some(value),
        Err(e) => {
            // Name the field, so a contributor knows where to look.
            let field = e.path().to_string();
            let field = if field == "." { String::new() } else { format!("`{field}`: ") };
            errors.push(format!("{}: {field}{}", path.display(), e.inner()));
            None
        }
    }
}

fn check_id(path: &Path, expected: &str, id: &str, errors: &mut Errors) {
    if id != expected {
        errors.push(format!("{}: id `{id}` must match the file or folder name `{expected}`", path.display()));
    }
}

/// Values with no source, in a pack that claims to describe something real.
fn check_sourced(path: &Path, pack: &impl Audit, errors: &mut Errors) {
    let mut leaves: Vec<Leaf> = Vec::new();
    pack.audit("", &mut leaves);
    for leaf in leaves.iter().filter(|leaf| leaf.state == LeafState::Bare) {
        errors.push(format!(
            "{}: `{}` needs `{{ value, source, retrieved_at }}` in a real pack, or `unknown`",
            path.display(),
            leaf.path
        ));
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
            let allowed = pack.category.allowed_sections();
            for section in pack.sections().into_iter().filter(|section| !allowed.contains(section)) {
                errors.push(
                    format!("{}: `{section}` is not a section of category `{:?}`", path.display(), pack.category)
                        .to_lowercase(),
                );
            }
            if pack.kind == PackKind::Device {
                // A real product may not carry a value without saying where it came from.
                check_sourced(&path, &pack, errors);
            }
            if devices.insert(pack.id.clone(), pack).is_some() {
                errors.push(format!("{}: duplicate device id", path.display()));
            }
        }
    }
    devices
}

/// One YAML file per pack, named after its id.
fn load_packs<T: DeserializeOwned>(
    root: &Path,
    folder: &str,
    errors: &mut Errors,
    mut check: impl FnMut(&Path, &T, &mut Errors) -> String,
) -> BTreeMap<String, T> {
    let mut packs = BTreeMap::new();
    for path in files(&root.join(folder), "yaml") {
        let Some(pack) = read_yaml::<T>(&path, errors) else {
            continue;
        };
        let id = check(&path, &pack, errors);
        check_id(&path, &stem(&path), &id, errors);
        packs.insert(id, pack);
    }
    packs
}

fn needs_sources(path: &Path, teaching: bool, sources: usize, pack: &impl Audit, errors: &mut Errors) {
    if teaching {
        return;
    }
    check_sourced(path, pack, errors);
    if sources == 0 {
        errors.push(format!("{}: a real pack needs at least one entry in `sources`", path.display()));
    }
}

fn load_os(root: &Path, errors: &mut Errors) -> BTreeMap<String, OsPack> {
    load_packs(root, "data/os", errors, |path, pack: &OsPack, errors| {
        needs_sources(path, pack.teaching, pack.sources.len(), pack, errors);
        pack.id.clone()
    })
}

fn load_runtimes(root: &Path, errors: &mut Errors) -> BTreeMap<String, RuntimePack> {
    load_packs(root, "data/runtimes", errors, |path, pack: &RuntimePack, errors| {
        needs_sources(path, pack.teaching, pack.sources.len(), pack, errors);
        pack.id.clone()
    })
}

fn load_services(root: &Path, errors: &mut Errors) -> BTreeMap<String, ServicePack> {
    load_packs(root, "data/services", errors, |path, pack: &ServicePack, errors| {
        needs_sources(path, pack.teaching, pack.sources.len(), pack, errors);
        pack.id.clone()
    })
}

fn load_tariffs(root: &Path, errors: &mut Errors) -> BTreeMap<String, Tariff> {
    load_packs(root, "data/power/tariffs", errors, |path, tariff: &Tariff, errors| {
        let dated = usize::from(!tariff.source.is_empty() && !tariff.retrieved_at.is_empty());
        needs_sources(path, tariff.teaching, dated, tariff, errors);
        // Slabs climb, and exactly the last one is open-ended.
        let open = tariff.slabs.iter().filter(|slab| slab.upto_kwh.is_none()).count();
        let last_open = tariff.slabs.last().is_some_and(|slab| slab.upto_kwh.is_none());
        let limits: Vec<u32> = tariff.slabs.iter().filter_map(|slab| slab.upto_kwh).collect();
        if open != 1 || !last_open || limits.windows(2).any(|pair| pair[0] >= pair[1]) {
            errors.push(format!(
                "{}: `slabs` must rise by `upto_kwh` and end with exactly one slab that has no `upto_kwh`",
                path.display()
            ));
        }
        tariff.id.clone()
    })
}

fn load_outages(root: &Path, errors: &mut Errors) -> BTreeMap<String, Outage> {
    load_packs(root, "data/power/outages", errors, |path, outage: &Outage, errors| {
        if outage.kind == ProfileKind::Real && (outage.source.is_none() || outage.retrieved_at.is_none()) {
            errors.push(format!("{}: a real outage scenario needs `source` and `retrieved_at`", path.display()));
        }
        if outage.duration_min == 0 || outage.per_day == 0 {
            errors.push(format!("{}: `duration_min` and `per_day` must be at least 1", path.display()));
        }
        outage.id.clone()
    })
}

fn load_media(root: &Path, errors: &mut Errors) -> BTreeMap<String, MediaFormat> {
    let path = root.join("data/reference/media.yaml");
    if !path.exists() {
        return BTreeMap::new();
    }
    let formats = read_yaml::<BTreeMap<String, MediaFormat>>(&path, errors).unwrap_or_default();
    for (id, format) in &formats {
        if !format.teaching && format.kbps.is_unsourced_value() {
            errors.push(format!("{}: `{id}.kbps` needs a source, or `unknown`", path.display()));
        }
    }
    formats
}

fn load_templates(root: &Path, errors: &mut Errors) -> BTreeMap<String, Template> {
    load_packs(root, "content/sandbox/templates", errors, |_, template: &Template, _| template.id.clone())
}

/// Ids one pack uses to name another must exist.
fn check_references(bundle: &Bundle, errors: &mut Errors) {
    for (id, device) in &bundle.devices {
        for os in device.os_support.keys().filter(|os| !bundle.os.contains_key(*os)) {
            errors.push(format!("device `{id}`: `os_support` names unknown OS `{os}`"));
        }
        let apps = device.client.iter().flat_map(|client| &client.apps);
        for app in apps.filter(|app| !bundle.services.contains_key(*app)) {
            errors.push(format!("device `{id}`: `client.apps` names unknown service `{app}`"));
        }
        let isp = device.router.as_ref().and_then(|router| router.supplied_by.as_ref());
        if let Some(isp) = isp.filter(|isp| !bundle.isps.contains_key(*isp)) {
            errors.push(format!("device `{id}`: `router.supplied_by` names unknown ISP profile `{isp}`"));
        }
    }
    for (id, os) in &bundle.os {
        for runtime in os.provides.native_runtimes.iter().filter(|r| !bundle.runtimes.contains_key(*r)) {
            errors.push(format!("os `{id}`: `provides.native_runtimes` names unknown runtime `{runtime}`"));
        }
    }
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

/// Checks that hold for any authored world: a mission's or a template's.
fn check_world(path: &str, world: &World, layout: &Layout, bundle: &Bundle, errors: &mut Errors) {
    let mut fail = |message: String| errors.push(format!("{path}: {message}"));
    let has_node = |id: &str| world.node(id).is_some();

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
        for (name, layout) in [("wide", &layout.wide), ("narrow", &layout.narrow)] {
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
            Node::Router(router) => {
                if let Some(device) = &router.device
                    && !bundle.devices.contains_key(device)
                {
                    fail(format!("`{}` uses unknown device `{device}`", router.id));
                }
            }
            Node::Switch(sim_core::model::Switch { id, device, .. })
            | Node::Power(sim_core::model::PowerUnit { id, device, .. }) => {
                if let Some(device) = device
                    && !bundle.devices.contains_key(device)
                {
                    fail(format!("`{id}` uses unknown device `{device}`"));
                }
            }
            Node::Internet(_) => {}
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
}

fn check_mission(path: &str, mission: &Mission, bundle: &Bundle, errors: &mut Errors) {
    let world = &mission.world;
    check_world(path, world, &mission.layout, bundle, errors);
    let mut fail = |message: String| errors.push(format!("{path}: {message}"));
    let has_node = |id: &str| world.node(id).is_some();
    let has_attempt = |id: &str| world.attempts.iter().any(|a| a.id == id);
    let has_question = |id: &str| mission.questions.iter().any(|q| q.id == id);

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
        os: load_os(root, &mut errors),
        runtimes: load_runtimes(root, &mut errors),
        services: load_services(root, &mut errors),
        tariffs: load_tariffs(root, &mut errors),
        outages: load_outages(root, &mut errors),
        media: load_media(root, &mut errors),
        templates: BTreeMap::new(),
        isps: load_isps(root, &mut errors),
        explanations: load_explanations(root, &mut errors),
        missions: BTreeMap::new(),
        tracks: Vec::new(),
    };
    check_references(&bundle, &mut errors);
    let templates = load_templates(root, &mut errors);
    for (id, template) in &templates {
        let path = format!("content/sandbox/templates/{id}.yaml");
        check_world(&path, &template.world, &template.layout, &bundle, &mut errors);
        let report = sim_core::evaluate(&template.world, &bundle);
        if report.findings.iter().any(|f| f.rule_id == Reason::EvaluationDidNotSettle) {
            errors.push(format!("{path}: the world does not settle"));
        }
    }
    bundle.templates = templates;
    let missions = load_missions(root, &mut errors);
    let mut orders = BTreeMap::new();
    for (id, mission) in &missions {
        check_mission(&format!("missions/{id}/mission.yaml"), mission, &bundle, &mut errors);
        if let Some(other) = orders.insert(mission.order, id) {
            errors.push(format!("missions `{id}` and `{other}` both have order {}", mission.order));
        }
    }
    bundle.missions = missions;
    let tracks_path = root.join("content/tracks.yaml");
    if let Some(tracks) = read_yaml::<Vec<Track>>(&tracks_path, &mut errors) {
        for track in &tracks {
            for id in &track.missions {
                if !bundle.missions.contains_key(id) {
                    errors.push(format!("content/tracks.yaml: track `{}` lists unknown mission `{id}`", track.id));
                }
            }
        }
        for id in bundle.missions.keys() {
            if !tracks.iter().any(|track| track.missions.contains(id)) {
                errors.push(format!("content/tracks.yaml: mission `{id}` is in no track"));
            }
        }
        bundle.tracks = tracks;
    }
    if errors.is_empty() { Ok(bundle) } else { Err(errors) }
}

/// Split what was built into the bundle missions need and the sandbox
/// catalog, which is fetched only when the sandbox opens.
pub fn split(full: Bundle) -> (Bundle, Bundle) {
    let mut devices: Vec<&str> = Vec::new();
    let mut isps: Vec<&str> = Vec::new();
    for mission in full.missions.values() {
        for node in &mission.world.nodes {
            match node {
                Node::Host(host) => devices.extend(host.device.as_deref()),
                Node::Router(router) => devices.extend(router.device.as_deref()),
                Node::Isp(isp) => isps.extend(isp.profile.as_deref()),
                Node::Switch(switch) => devices.extend(switch.device.as_deref()),
                Node::Power(unit) => devices.extend(unit.device.as_deref()),
                Node::Internet(_) => {}
            }
        }
    }
    let (mission_devices, catalog_devices) =
        full.devices.clone().into_iter().partition(|(id, _)| devices.contains(&id.as_str()));
    let (mission_isps, catalog_isps) = full.isps.clone().into_iter().partition(|(id, _)| isps.contains(&id.as_str()));
    let catalog = Bundle {
        devices: catalog_devices,
        isps: catalog_isps,
        os: full.os,
        runtimes: full.runtimes,
        services: full.services,
        tariffs: full.tariffs,
        outages: full.outages,
        media: full.media,
        templates: full.templates,
        ..Bundle::empty()
    };
    let missions = Bundle {
        devices: mission_devices,
        isps: mission_isps,
        explanations: full.explanations,
        missions: full.missions,
        tracks: full.tracks,
        ..Bundle::empty()
    };
    (missions, catalog)
}

fn write_json(json: &str, output: &Path) -> Result<(), Errors> {
    let io = |e: std::io::Error| vec![format!("{}: {e}", output.display())];
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    fs::write(output, json).map_err(io)
}

pub fn write(bundle: &Bundle, output: &Path) -> Result<String, Errors> {
    let mut json = serde_json::to_string(bundle).map_err(|e| vec![e.to_string()])?;
    json.push('\n');
    write_json(&json, output)?;
    Ok(format!(
        "{} devices, {} OS, {} runtimes, {} services, {} tariffs, {} templates, {} ISP profiles, {} explanations, {} missions, {} bytes",
        bundle.devices.len(),
        bundle.os.len(),
        bundle.runtimes.len(),
        bundle.services.len(),
        bundle.tariffs.len(),
        bundle.templates.len(),
        bundle.isps.len(),
        bundle.explanations.len(),
        bundle.missions.len(),
        json.len()
    ))
}

/// JSON Schema for every kind of pack file, so editors can check a file as
/// it is written. File name first, then the schema as pretty JSON.
pub fn schemas() -> Vec<(&'static str, String)> {
    fn pretty<T: schemars::JsonSchema>() -> String {
        let mut json = serde_json::to_string_pretty(&schemars::schema_for!(T)).expect("a schema is plain JSON");
        json.push('\n');
        json
    }
    vec![
        ("device.schema.json", pretty::<DevicePack>()),
        ("os.schema.json", pretty::<OsPack>()),
        ("runtime.schema.json", pretty::<RuntimePack>()),
        ("service.schema.json", pretty::<ServicePack>()),
        ("tariff.schema.json", pretty::<Tariff>()),
        ("outage.schema.json", pretty::<Outage>()),
        ("media.schema.json", pretty::<BTreeMap<String, MediaFormat>>()),
        ("template.schema.json", pretty::<Template>()),
    ]
}

pub fn write_schemas(dir: &Path) -> Result<String, Errors> {
    let all = schemas();
    for (name, json) in &all {
        write_json(json, &dir.join(name))?;
    }
    Ok(format!("{} schemas", all.len()))
}
