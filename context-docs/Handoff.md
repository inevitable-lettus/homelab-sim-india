# Handoff

Living state and build order. **Read this first in every session, and update it at the end of every phase.** Any session can pick up at the first phase not done. Phases are dependency-ordered, and each ends with its tests green.

## Current status (2026-11-10)

**Phase 0 is done.** Repo: <https://github.com/inevitable-lettus/homelab-sim-india>. Preview: <https://inevitable-lettus.github.io/homelab-sim-india/>. CI and deploy passed on the first push, and the deployed page runs the Rust core, loads the data bundle and links to its exact source commit. Spike 3 of Phase 1 is also done. **Next action: the rest of Phase 1, using [Phase1-Brief.md](Phase1-Brief.md).**

`docs/ideation.md` is a local-only snapshot of the original ideation doc, deliberately kept out of the public repo. **Where it conflicts with these context docs, these win.** It predates the India focus and most decisions below.

## Product in one paragraph

A free, open-source, India-specific homelab learning simulator for college students and existing hobbyists. Learners complete missions with a deterministic Rust logic sim and a real browser Linux terminal. The first track teaches remote access on Indian ISP lines, including CGNAT. It is a static site with no backend or accounts.

## Decision log

Owner-confirmed on 2026-11-10:

| Area | Decision |
|---|---|
| Positioning | India-specific full learning sim, building on what HLBuilder does |
| HLBuilder relationship | New repo, reuse selected parts. Not a fork, not upstream PRs |
| Learners | College students and existing hobbyists |
| Depth | One track, two depths (guided and challenge) |
| Build order | Remote-access track first |
| Curriculum | Document the entire curriculum, as Markdown in the repo |
| Core language | Rust compiled to WASM |
| Mission type | Logic sim plus real terminal: real Linux, the sim grades the files |
| Hardware data | Archetypes only at launch; adding device data must be very easy and modular |
| ISP data | Dated sourced profiles plus the learner's own line check |
| India at launch | ISP behaviour and power realities |
| Data and curriculum licence | CC BY-SA 4.0 |
| Code licence | AGPL-3.0, because reused HLBuilder code requires it |
| Principles kept from ideation | Static, no backend, no accounts, no LLM in the runtime, minimal visuals, sourced claims |
| Revenue | Future only |

Pushed to later by the owner:

| Item | Note |
|---|---|
| Community ISP reports | Next phase, on GitHub Discussions, promoted to the Homelab India community on GitHub |
| Rupee prices and sellers | No open data source and goes stale quickly. Needs its own innovation |
| OverTheWire model | Borrow nothing now |
| Hosted SSH servers | Possible future option only |

**Adopted from the 2026-11-10 critique, not separately confirmed.** Tell the owner if any of these becomes costly, and change them if the owner objects:

| Decision | Replaces |
|---|---|
| NetSim dropped: mostly Cisco/Huawei CLI handlers, about 13 KB of non-CLI Rust, last pushed January 2026 | Ideation decision 12 |
| YAML in git for data | Postgres authoring (decision 10) |
| One world graph with fixed-point evaluation; layers are curriculum and UI only | Strict layer gating |
| Three-valued verdicts (pass, fail, unknown) | Implicit pass on missing data |
| Per-verdict provenance wording | Global strict/community toggle |
| v86 as default terminal engine, pending spike | Open question in ideation |
| Progress in browser storage plus JSON export | Not previously specified |
| DCO for contributions, no CLA | Not previously specified |
| Mocked Proxmox and router UIs parked | "Recommended hybrid" item 3 in ideation |
| Tap-to-connect and mobile-friendly mission screen | Not previously specified |

## Build phases

### Phase 0: Repository and licences (done 2026-11-10)

Create git repo, Cargo workspace (`sim-core`, `sim-wasm`, `data-build`), Vite + React + TypeScript app, CI, `LICENSE` (AGPL-3.0), CC BY-SA 4.0 licence files for `data/`, `content/`, `missions/`, `curriculum/`, `CONTRIBUTING.md` with DCO, `THIRD_PARTY.md`, and a source link in the app footer.

**Done =** a static build of a page that calls a Rust `evaluate` stub through WASM, deployed to a preview URL, with CI running `cargo test`, type checks and lint.

### Phase 1: Spikes (throwaway code, written conclusions)

1. **Terminal:** boot a small v86 Linux image in the browser, write a WireGuard config in it, read it back through `read_file`. Measure download size and boot time.
2. **Trace playback:** a moving labelled packet along React Flow edges on a low-end Android phone.
3. **HLBuilder audit:** read the candidate files in [Architecture.md](Architecture.md), and list what is actually worth copying, with commit hash.
4. **Budget check:** confirm or revise the budgets, and test the phone and metered-data assumption.

**Done =** each result recorded in this file, the engine chosen, and budgets updated.

#### Spike 3 result: HLBuilder audit (done 2026-11-10, commit `2bf4c28736a99d45c6440411ee6fac8f3c3f5c89`)

Based on sizes, imports and exports, not a line-by-line read. **Nothing needs copying for Phase 1.** Its value is as a reference, so `THIRD_PARTY.md` stays empty for now.

| File or area | Finding | Decision |
|---|---|---|
| `features/builder/components/custom-edge.tsx` (23 KB) | React Flow custom edge. No packet-motion code | Read for edge-path patterns in Phase 5 |
| `features/builder/lib/layout/` (about 70 KB) | Pure auto-layout with tests | Skip. Missions are pre-placed. Revisit for sandbox |
| `features/builder/lib/connection-rules.ts` | Validity logic in the UI | Do not copy (boundary 1). Reference for Rust rules in Phase 4 |
| `features/builder/lib/port-config.ts` | Pure helpers tied to HLBuilder's types | Skip. Small enough to write in our model |
| `lib/hardware-config.ts` | Flags keyed by hardware type | Skip. Our rules read capabilities |
| `features/setup-guide/lib/setup-plan.ts` | Structured checklist, coupled to its taxonomy and gaming features | Borrow the section/step shape later |
| `features/builder/lib/planner/` | Mostly LAN-party and game hosting | Skip |

Consequence: if no HLBuilder code is ever copied, AGPL is a choice rather than a requirement. The decision log still stands at AGPL.

### Phase 2: Curriculum documentation

`curriculum/README.md` maps all tracks. One file per track covers goals, prerequisites, concepts and missions. The remote-access track is specified fully, mission by mission, at both depths, with India-specific content. Other tracks are outlined (at least physical build, home network, services, storage and backups, power).

**Done =** every planned track has a document, and all remote-access missions are detailed enough to build from without further decisions.

### Phase 3: Data packs and build pipeline

Rust schema types, `tools/data-build`, generated JSON Schema, the versioned bundle, six archetypes, initial ISP profiles (Jio, Airtel, BSNL, ACT), state tariff presets, explanation loader.

**Done =** the bundle builds in CI, invalid packs fail with a clear message, a test proves a new device works from **one YAML file with no code change**, and every ISP claim has a date and source or is `unknown`.

### Phase 4: Simulation core

World graph, fixed-point evaluation, three-valued findings, NAT and carrier NAT, mappings, forwards, firewall, DNS, tunnels, IPv6, traces, grading.

**Done =** fixture worlds cover every Mission 1–4 win and failure path, identical input gives byte-identical output, and missing data yields `Unknown` under test.

### Phase 5: Mission UI and playback

Mission screen, board, findings, trace player and step log, depth switch, local progress, per [Design.md](Design.md) and [Animations.md](Animations.md).

**Done =** a full mission playable at 360px and desktop, by keyboard and in reduced motion. Playwright covers win and failure.

### Phase 6: First learner test (gate)

Ship Missions 1–3 without the terminal. Put them in front of about five real learners from the target groups, watch them, and check whether they can explain CGNAT afterwards.

**Done =** findings are written here and the owner decides whether to continue, change or stop before the terminal work. No beginner has tested this concept yet.

### Phase 7: Terminal missions

Guest image, `TerminalHost`, file parsers, `grade` on artifacts, file-editor fallback, guest image source publication.

**Done =** Mission 5 passes and fails correctly from both terminal and editor.

### Phase 8: Complete the track

Remaining missions at both depths, line check, ISP profile pages, power scenarios, concept pages.

**Done =** the track is complete end to end, and every explanation has been reviewed.

### Phase 9: Launch

Production deploy, accessibility pass, final licence check, README. Owner announces to the Homelab India community.

**Done =** public URL live, source link correct, no learner data leaves the browser (verified by network inspection).

### Next phase after launch

Community ISP reports through GitHub Discussions with a structured form (ISP, circle, plan, results, and never an IP address), real device packs, and the other curriculum tracks.

## Open questions

- [ ] Project name.
- [ ] Do target learners mostly use phones? (Phase 1 and 6)
- [ ] Which reports count as enough to change an ISP profile's confidence? (Before community reports)
- [ ] Are HLBuilder's maintainer and Homelab India worth contacting early? AGPL doesn't require it, but it is a courtesy and could help reach.

## Known risks

- **ISP claims:** current evidence is forum-level. Verify before publishing, and keep "unknown" honest.
- **Effort:** Rust core, real terminal and a full curriculum are the heavier choices, on one person's time. Phase 6 exists to catch a wrong bet early.
- **Direct competitor:** [HLBuilder](https://github.com/Butterski/homelab-builder) is active. Our difference is teaching, sourced verdicts, failure simulation and the India focus.
- **Community channels:** [homelabindia/wiki](https://github.com/homelabindia/wiki) was active in November 2026, and a separate [homelab-india](https://github.com/homelab-india) organisation exists. Which one is the main community is unconfirmed.

## Session context

- **Canonical commands:** `npm run setup` (once), `npm run dev`, `npm run check` (everything CI runs), `npm run build`.
- **Commits:** this repo's local Git config uses the GitHub no-reply address on purpose, because the repo is public. Sign off with `git commit -s`.
- **PATH:** Rust came from Homebrew's `rustup`, which isn't on the shell PATH. The npm scripts handle it through `scripts/env.sh`. For direct `cargo` use: `export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"`.
- **Toolchain:** Rust 1.99.1 pinned in `rust-toolchain.toml`. `wasm-bindgen` CLI pinned to the crate version in `Cargo.toml`, used directly instead of wasm-pack. TypeScript is 6.x because `typescript-eslint` does not support 7 yet.
- **Generated, gitignored:** `app/src/wasm-pkg/`, `app/public/data/`. Run `npm run wasm` and `npm run data` after pulling Rust changes.
- **Terminal image:** `bash spikes/terminal/build-image.sh` needs Docker running. **Not run or tested yet.**
- **Known:** `npm audit` reports 10 moderate findings, all one `minimatch` issue inside ESLint tooling. Dev-only, not shipped.
- **Update rule:** at the end of each phase, update Current status, mark the phase done with the date, and record gotchas here. Change Architecture, Design or Animations only when the design itself changes.
