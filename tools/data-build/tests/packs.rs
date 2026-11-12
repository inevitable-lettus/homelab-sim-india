//! Schema v4 packs: what the data build accepts, what it refuses and why,
//! and that a new pack of any kind is one YAML file.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("mkdir");
    for entry in fs::read_dir(from).expect("read dir").flatten() {
        let (path, target) = (entry.path(), to.join(entry.file_name()));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy");
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    for part in ["data", "content", "missions"] {
        copy_dir(&root().join(part), &dir.join(part));
    }
    dir
}

fn put(dir: &Path, file: &str, text: &str) {
    let path = dir.join(file);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, text).expect("write");
}

fn errors_with(name: &str, file: &str, text: &str) -> Vec<String> {
    let dir = scratch(name);
    put(&dir, file, text);
    data_build::build(&dir).expect_err("must fail")
}

const SRC: &str = "source: \"https://example.org/spec\", retrieved_at: \"2026-11-12\"";

#[test]
fn a_section_that_does_not_belong_to_the_category_is_refused() {
    let text = "id: sw\nkind: archetype\ncategory: switch\nlabel: Switch\ncompute: { ram_gb: 4 }\n";
    let errors = errors_with("bad-section", "data/archetypes/sw.yaml", text);
    assert!(
        errors.iter().any(|e| e.contains("sw.yaml") && e.contains("`compute`") && e.contains("switch")),
        "{errors:?}"
    );
}

#[test]
fn a_bad_value_is_reported_with_its_file_and_field() {
    let text = "id: pc\nkind: archetype\ncategory: computer\nlabel: PC\ncompute:\n  arch: pdp11\n";
    let errors = errors_with("bad-field", "data/archetypes/pc.yaml", text);
    assert!(errors.iter().any(|e| e.contains("pc.yaml") && e.contains("`compute.arch`")), "{errors:?}");
}

#[test]
fn a_source_link_may_not_track_the_reader() {
    let text = "id: pc\nkind: device\ncategory: computer\nlabel: PC\ncompute:\n  ram_gb: { value: 8, source: \"https://example.org/p?utm_source=x\", retrieved_at: \"2026-11-12\" }\n";
    let errors = errors_with("tracking", "data/devices/pc.yaml", text);
    assert!(errors.iter().any(|e| e.contains("pc.yaml") && e.contains("utm_")), "{errors:?}");
}

#[test]
fn tariff_slabs_must_rise_and_end_open() {
    let head = "id: t\nteaching: true\nstate: S\ndiscom: D\ncategory: LT domestic\neffective_from: \"2026-04-01\"\n";
    for (name, slabs) in [
        ("slabs-closed", "slabs: [{ upto_kwh: 100, paise_per_kwh: 300 }]\n"),
        (
            "slabs-fall",
            "slabs: [{ upto_kwh: 200, paise_per_kwh: 300 }, { upto_kwh: 100, paise_per_kwh: 500 }, { paise_per_kwh: 700 }]\n",
        ),
        ("slabs-two-open", "slabs: [{ paise_per_kwh: 300 }, { paise_per_kwh: 500 }]\n"),
    ] {
        let errors = errors_with(name, "data/power/tariffs/t.yaml", &format!("{head}{slabs}"));
        assert!(errors.iter().any(|e| e.contains("t.yaml") && e.contains("`slabs`")), "{name}: {errors:?}");
    }
}

#[test]
fn a_real_pack_of_any_kind_needs_its_sources() {
    let os = "id: some-os\nlabel: Some OS\nkind: os\nfamily: linux\nbase_ram_mb: 300\n";
    let errors = errors_with("os-unsourced", "data/os/some-os.yaml", os);
    assert!(errors.iter().any(|e| e.contains("some-os.yaml") && e.contains("`base_ram_mb`")), "{errors:?}");
    assert!(errors.iter().any(|e| e.contains("some-os.yaml") && e.contains("`sources`")), "{errors:?}");

    let tariff =
        "id: t\nstate: S\ndiscom: D\ncategory: C\neffective_from: \"2026-04-01\"\nslabs: [{ paise_per_kwh: 500 }]\n";
    let errors = errors_with("tariff-unsourced", "data/power/tariffs/t.yaml", tariff);
    assert!(errors.iter().any(|e| e.contains("t.yaml") && e.contains("`sources`")), "{errors:?}");

    let outage = "id: o\nlabel: O\nkind: real\nduration_min: 60\n";
    let errors = errors_with("outage-unsourced", "data/power/outages/o.yaml", outage);
    assert!(errors.iter().any(|e| e.contains("o.yaml") && e.contains("`source`")), "{errors:?}");
}

#[test]
fn a_pack_may_not_name_a_pack_that_does_not_exist() {
    let text = format!(
        "id: pc\nkind: device\ncategory: computer\nlabel: PC\nos_support:\n  no-such-os: {{ value: {{ level: full }}, {SRC} }}\n"
    );
    let errors = errors_with("dangling", "data/devices/pc.yaml", &text);
    assert!(errors.iter().any(|e| e.contains("`pc`") && e.contains("no-such-os")), "{errors:?}");
}

/// One file each, no code: the pack is in the build, and in the catalog
/// rather than the bundle missions load.
#[test]
fn a_new_pack_of_every_kind_is_one_yaml_file() {
    let dir = scratch("one-file-each");
    let before = data_build::build(&dir).expect("builds");
    let cap = |value: &str| format!("{{ value: {value}, {SRC} }}");
    let sources = "sources: [{ url: \"https://example.org/doc\", retrieved_at: \"2026-11-12\" }]\n";

    put(
        &dir,
        "data/os/new-os.yaml",
        &format!(
            "id: new-os\nlabel: New OS\nkind: os\nfamily: linux\narches: [x86_64]\nbase_ram_mb: {}\n{sources}",
            cap("400")
        ),
    );
    put(
        &dir,
        "data/runtimes/new-runtime.yaml",
        &format!("id: new-runtime\nlabel: New runtime\nos_families: [linux]\nvia_vm: {}\n{sources}", cap("false")),
    );
    put(
        &dir,
        "data/services/new-service.yaml",
        &format!(
            "id: new-service\nlabel: New service\ncategory: media\nram_min_mb: {}\nports: [{{ proto: tcp, port: 8096, purpose: web }}]\n{sources}",
            cap("1024")
        ),
    );
    put(
        &dir,
        "data/power/tariffs/new-tariff.yaml",
        "id: new-tariff\nstate: S\ndiscom: D\ncategory: LT domestic\neffective_from: \"2026-04-01\"\nslabs: [{ upto_kwh: 100, paise_per_kwh: 300 }, { paise_per_kwh: 700 }]\nsource: \"https://example.org/order\"\nretrieved_at: \"2026-11-12\"\n",
    );
    put(
        &dir,
        "data/power/outages/new-outage.yaml",
        "id: new-outage\nlabel: Two hours\nkind: teaching\nduration_min: 120\n",
    );
    put(
        &dir,
        "data/reference/media.yaml",
        &format!("new_format:\n  label: New format\n  codec: h264\n  kbps: {}\n", cap("6000")),
    );
    for (category, id, section) in [
        ("router", "new-router", format!("router:\n  port_forwarding: {}\n", cap("false"))),
        ("switch", "new-switch", format!("network:\n  nic: {}\n", cap("[{}, {}, {}, {}, {}]"))),
        ("access_point", "new-ap", format!("network:\n  wifi: {}\n", cap("wifi6"))),
        ("ups", "new-ups", format!("ups:\n  max_w: {}\n", cap("360"))),
        ("inverter", "new-inverter", format!("ups:\n  waveform: {}\n", cap("square"))),
        ("dc_ups", "new-dc-ups", format!("ups:\n  battery_wh: {}\n", cap("30"))),
        ("drive", "new-drive", format!("drive:\n  capacity_gb: {}\n", cap("4000"))),
        ("enclosure", "new-enclosure", format!("adapter:\n  uas: {}\n", cap("true"))),
        ("usb_nic", "new-usb-nic", format!("adapter:\n  provides:\n    nic: {}\n", cap("[{ speed_mbps: 1000 }]"))),
        ("usb_hub", "new-hub", format!("adapter:\n  needs_power: {}\n", cap("true"))),
        ("client", "new-client", "client:\n  plays: { h264: true }\n".to_owned()),
    ] {
        let text = format!("id: {id}\nkind: device\ncategory: {category}\nlabel: {id}\n{section}");
        put(&dir, &format!("data/devices/{id}.yaml"), &text);
    }
    put(
        &dir,
        "content/sandbox/templates/new-template.yaml",
        "id: new-template\ntitle: New\nsummary: A start.\nworld:\n  nodes: [{ kind: internet, id: internet, label: Internet }]\nlayout:\n  wide: { internet: [0, 0] }\n  narrow: { internet: [0, 0] }\n",
    );

    let full = data_build::build(&dir).expect("builds with the new packs");
    assert_eq!(full.devices.len(), before.devices.len() + 11);
    assert_eq!(full.os.len(), before.os.len() + 1);
    assert_eq!(full.runtimes.len(), before.runtimes.len() + 1);
    assert_eq!(full.services.len(), before.services.len() + 1);
    assert_eq!(full.tariffs.len(), before.tariffs.len() + 1);
    assert_eq!(full.outages.len(), before.outages.len() + 1);
    assert!(full.media.contains_key("new_format"));
    assert!(full.templates.contains_key("new-template"));

    let (missions, catalog) = data_build::split(full);
    assert!(catalog.devices.contains_key("new-router") && !missions.devices.contains_key("new-router"));
    assert!(missions.devices.contains_key("old-laptop"), "missions keep the packs they name");
    assert!(missions.os.is_empty() && catalog.os.contains_key("new-os"));
    assert!(catalog.missions.is_empty() && !missions.missions.is_empty());

    // Merged back, nothing is lost.
    let mut merged = missions.clone();
    merged.merge(catalog);
    assert!(merged.devices.contains_key("new-router") && merged.devices.contains_key("old-laptop"));
}

/// `schemas/` is generated. Run `cargo run -p data-build -- schemas` after a
/// schema change.
#[test]
fn the_committed_json_schemas_are_current() {
    for (name, json) in data_build::schemas() {
        let path = root().join("schemas").join(name);
        let committed = fs::read_to_string(&path).unwrap_or_default();
        assert!(committed == json, "{} is stale. Run `cargo run -p data-build -- schemas`", path.display());
    }
}
