# Contributing

Thanks for helping. Run `npm run setup` once, then `npm run check` before opening a pull request. CI runs the same command.

## Rules that matter most

- The Rust core decides every outcome. The UI only displays results.
- Missing data must produce `Unknown`, never a pass.
- Every device or ISP claim needs a source URL and a date, or it stays `unknown`.
- Never add code that collects or sends a learner's IP address or results.
- A new device is one YAML file and must not need a code change.

The full list is in [CLAUDE.md](CLAUDE.md) and [context-docs/Architecture.md](context-docs/Architecture.md).

## Add a device in one file

A device is one YAML file in `data/devices/`, named after its `id`. There is no code to change: the data build reads the file, checks it against [`schemas/device.schema.json`](schemas/device.schema.json), and the device appears in the catalog.

A small, real example, a hard disk:

```yaml
id: example-4tb-hdd
kind: device
category: drive
label: Example 4 TB hard disk
drive:
  capacity_gb: { value: 4000, source: "https://example.org/datasheet", retrieved_at: "2026-11-12" }
```

- `kind: device` is a real product, so every capability value carries its own `source` and `retrieved_at`, or is the word `unknown`. A link may not carry tracking parameters such as `utm_source`.
- `category` is one of `computer`, `router`, `switch`, `access_point`, `ups`, `inverter`, `dc_ups`, `drive`, `enclosure`, `usb_nic`, `usb_hub` or `client`. Only the sections that belong to that category are accepted, for example `drive:` for a drive and `compute:` for a computer.
- A generic teaching model is `kind: archetype`, with plain values and no sources. See [`data/archetypes/old-android-phone.yaml`](data/archetypes/old-android-phone.yaml) for a fuller file.
- Anything you leave out is `unknown`, and `unknown` never passes a check.

Run `npm run check`. If the file is wrong, the data build names the file and the field. `tools/data-build/tests/packs.rs` has what the build accepts and refuses, and a test that adds one file of each kind.

Data is contributed under CC BY-SA 4.0, like everything in `data/`.

## Sign your commits (DCO)

This project uses the [Developer Certificate of Origin](https://developercertificate.org). It means you wrote the change, or have the right to submit it under this project's licences. Sign each commit:

```bash
git commit -s
```

There is no CLA.

## Licensing of contributions

- Code is contributed under AGPL-3.0-only.
- Anything in `data/`, `content/`, `missions/` or `curriculum/` is contributed under CC BY-SA 4.0.

Report facts in your own words and link to the source. Do not paste other people's forum posts.

If you copy code from another project, keep its notices and add it to [THIRD_PARTY.md](THIRD_PARTY.md).

## Privacy in reports

Never include your real IP address, router password, or anything else that identifies your connection in an issue or pull request.
