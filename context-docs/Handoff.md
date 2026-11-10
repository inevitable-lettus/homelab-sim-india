# Handoff

Living state and build order. **Read this first in every session, and update it at the end of every phase.** Any session can pick up at the first phase not done. Phases are dependency-ordered, and each ends with its tests green.

## Current status (2026-11-10)

**Phases 0, 1 and 4 are done**, with the minimum Phase 2 and 3 material Phase 4 needed. Repo: <https://github.com/inevitable-lettus/homelab-sim-india>. Preview: <https://inevitable-lettus.github.io/homelab-sim-india/>. The deployed page runs the Rust core, loads the data bundle (four missions) and links to its exact source commit. It has no mission UI yet. Phase 1 confirmed v86 as the terminal engine and React Flow for the board. Results are under Phase 1 below.

**Owner's build order (2026-11-10), which overrides the phase numbering:** Part 1 is Phases 1, 4, 5, 6, 7, with only the minimum Phase 2 and 3 material the code needs (Rust schema types, short specs for Missions 1 to 5, a few archetypes, one clearly labelled teaching ISP profile with no real-ISP claims). Part 2 is Phases 2, 3 and 8. Part 3 is Phase 9. The owner approves between parts.

**Next action: Phase 5 (mission UI and playback).** Then Phases 6 and 7.

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

### Phase 1: Spikes (done 2026-11-10)

1. **Terminal:** boot a small v86 Linux image in the browser, write a WireGuard config in it, read it back through `read_file`. Measure download size and boot time.
2. **Trace playback:** a moving labelled packet along React Flow edges on a low-end Android phone.
3. **HLBuilder audit:** read the candidate files in [Architecture.md](Architecture.md), and list what is actually worth copying, with commit hash.
4. **Budget check:** confirm or revise the budgets, and test the phone and metered-data assumption.

**Done =** each result recorded in this file, the engine chosen, and budgets updated.

**Outcome:** engine is **v86**, started from a **saved state** with 128 MB guest RAM. The board is **React Flow** with the packet overlay inside its viewport. No real phone was available: every "low-end" number below is headless Chrome 6x CPU throttling on an Apple M4, which slows script but not the GPU, storage or memory. Rerun on a real phone before trusting them. Reproduce with `cd spikes && npm install && node measure.mjs` (needs installed Chrome and the local image).

#### Spike 1 result: terminal (done 2026-11-10)

Works. The guest ran `wg genkey`, wrote `/root/wg0.conf`, and `read_file` returned it on the host.

| Measure | Cold boot | Saved state |
|---|---|---|
| Time to prompt, unthrottled | 18.1 s | 0.13 s |
| Time to prompt, 6x CPU throttle | 83.4 s | 0.79 s |
| Image download to reach a prompt and run the test | 16.6 MB in 828 requests | 12.4 MB in 14 requests |
| With engine (`libv86.mjs` 94 KB + `v86.wasm` 380 KB, gzipped) | 17.0 MB | **12.9 MB** |

- Times exclude download, because the test server was local. Arithmetic only: 12.9 MB takes about 10 s at 10 Mbit/s and about 52 s at 2 Mbit/s.
- The saved state is 11.4 MB (zstd -19) and is nearly the same size for 128 MB and 256 MB guests. A state boot needs no BIOS and no kernel fetch, and then pulls about 0.9 MB of files on demand.
- **Memory:** the renderer process used 400 to 815 MB RSS across five runs (noisy, measured with `ps` after the test). That is a real risk on phones with 2 to 3 GB of RAM, and is not tested on one.
- **Cross-origin isolation is not needed.** `crossOriginIsolated` was false and everything worked, so GitHub Pages is enough.
- **`read_file` sees guest writes at once** with `cache=loose`, before any `sync`, for a 92-byte and a 405 KB file. No cache-mode change needed.
- The same test passed in Node (`spikes/terminal/build-state.mjs`), which is how the state is built. CI can build it without a browser.

Gotchas for Phase 7:
- The prompt is `(none):~# ` followed by `ESC[6n` (a cursor query), not `localhost:~# ` as v86's own script expects. Waiting for the wrong prompt looks like a hang. Set the hostname in the image and use a real terminal emulator that answers the query.
- A state only works with the exact engine build, `alpine-fs.json` and rootfs it was saved with. Version them as one set. `build-image.sh` is now pinned to v86 commit `315b2b0`, which matches the npm package `0.5.473+g315b2b0`.
- The spike terminal is a bare `<pre>`. Phase 7 needs a terminal emulator such as xterm.js for `nano`. Its size is not measured yet.
- The local image is 1,430 files; the largest are 7.6 MB (rootfs chunk) and 11.4 MB (state). Check these against the host's per-file and file-count limits in Phase 7.

#### Spike 2 result: trace playback (done 2026-11-10)

Works. A labelled packet follows the fixture's edges with play, pause, step and speed, shows the NAT rewrites and stops at the carrier NAT.

| Run (headless Chrome, production build) | Playback | Playback while panning and zooming | Script per frame |
|---|---|---|---|
| Desktop 1280x800 | 60 fps, worst frame 16.8 ms | 60 fps | 0.012 ms |
| 360x740 at 3x pixel ratio, touch | 60 fps | 60 fps | 0.012 ms |
| Same, 6x CPU throttle | 60 fps, worst frame 16.8 ms | 60 fps | 0.069 ms |
| Same, 20x CPU throttle | 60 fps | 60 fps | 0.262 ms |

- The throttle was real: a fixed busy loop took 18 ms, 98 ms and 347 ms at 1x, 6x and 20x.
- **React Flow costs 69.2 KB of JS and 3.0 KB of CSS, gzipped.** The whole spike page (React, React DOM, React Flow, spike code) is 129 KB of JS gzipped.
- **Pan and zoom stay correct with no extra work.** The packet element sits in React Flow's `ViewportPortal`, so it is positioned in flow coordinates and moves with the board. Measured error between the packet and its point on the edge was 0 px over about 240 frames per run, with mouse drag and wheel on desktop and touch drag and pinch on mobile.
- Edge path selector: `.react-flow__edge[data-id="…"] .react-flow__edge-path`. `getPointAtLength` gives flow coordinates. No HLBuilder code was needed.

Findings that change the design:
- **A five-node horizontal board does not work at 360px.** It fits only at about 0.33 zoom, where node labels are unreadable. Phone boards need their own layout: stacked, with at most three nodes across.
- The packet label is counter-scaled by `1 / zoom` so it stays readable at any zoom. It clips at the board edge and must flip sides there.
- Cross-fading the old and new address in the same place is unreadable halfway. Show the old value struck through above the new one.
- `ViewportPortal` mounts after the first effect, so refs into it are null at first.
- In a Node script, Vite's `createServer` sets `NODE_ENV=development`, and a later `build` in the same process then bundles the development React. Set it back before building.

#### Spike 4 result: budgets (done 2026-11-10)

| Budget | Before | Now | Measured |
|---|---|---|---|
| App + core, gzipped, before any mission | under 500 KB | unchanged | 104 KB today; about 176 KB once React Flow is in |
| `evaluate` on a mission-sized world | under 16 ms on a low-end Android phone | unchanged | Not measurable until Phase 4 |
| Guest image first download | under 30 MB | **under 20 MB, saved state only** | 12.9 MB |
| Terminal time to prompt after download | none | **under 3 s on a low-end phone** | 0.8 s at 6x throttle |
| Packet playback | 30 fps on a low-end phone | unchanged | 60 fps at 6x throttle |

The phone and metered-data assumption is **still untested**: no phone was used and no learner was asked. It stays an open question for Phase 6. What changed is the plan for phones: the terminal's memory use means the file editor should be the default there, with the terminal as an opt-in.

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

### Phase 4: Simulation core (done 2026-11-10)

World graph, fixed-point evaluation, three-valued findings, NAT and carrier NAT, mappings, forwards, firewall, DNS, tunnels, IPv6, traces, grading, and network physics.

**Done =** fixture worlds cover every Mission 1–4 win and failure path, identical input gives byte-identical output, and missing data yields `Unknown` under test.

#### Phase 4 result

Built: `crates/sim-core` (`model`, `bundle`, `facts`, `walk`, `trace`, `rules`, `grade`, `physics`, `ip`, `reason`), `tools/data-build` as a real validator, `crates/sim-wasm` with `set_bundle`, `evaluate_json`, `grade_json`, `resolve_json`, and `app/src/lib/types.ts` as the TypeScript mirror. Design is in [Architecture.md](Architecture.md).

Minimum Phase 2 and 3 material written for it: three archetypes, one teaching ISP profile (`teaching-fibre`, `kind: teaching`, no real-ISP claim), Missions 1 to 4 as `missions/<id>/mission.yaml`, text for all 73 reasons in `content/steps.yaml`, 15 long explanations in `content/explanations/`, and [curriculum/remote-access/missions-1-5.md](../curriculum/remote-access/missions-1-5.md).

| Check | Result |
|---|---|
| Rust tests | 36 pass: 10 unit (IP classes, physics), 26 over the real mission files |
| Mission 1–4 paths | Every win and every intended failure asserted, including each wrong port forward failing for its own reason |
| Determinism | Bundle built twice is byte-identical; a grade is byte-identical after a JSON round trip; reversing node and link order changes no decision |
| Unknown | Unknown line type, missing profile claim, "can't find it" line check, missing device pack and unknown IPv6 all give `Unknown`, under test |
| One-YAML device | Test adds one file under `data/devices/`, rebuilds, and the virtualization rule changes its answer with no code change |
| Unsourced data | The build fails for a real device with a bare value and for a real ISP claim with no source and date |
| Size | 260,759 B gzipped of the 512,000 B budget. The WASM is 191 KB of that |

Teaching-model numbers for the solved Mission 2: 42.3 ms round trip phone to server, and 12 min 51 s for a 1.5 GB film over the phone's 16 Mbit/s of spare downlink. Mission 4 through the relay: 42.7 ms, path MTU 1440.

Gotchas for later phases:

- **The WASM is 191 KB gzipped**, mostly serde code for the many types. It fits, but Phase 5 adds React Flow (about 70 KB). If the budget gets tight, try `opt-level = "z"` and `wasm-opt` before cutting features.
- A trace can have **no legs**: the attempt failed before any packet left (no address of that kind, mesh not joined, name not found). The UI must show the reason on the origin node.
- Mesh addresses are stable seats (`.10`, `.11`, in node-id order) whether or not a device has joined, so the dial option can show the address before joining.
- `GoalCheck::RulePasses` fails when the rule produced no finding. A rule that said nothing has not passed.
- `rustfmt.toml` sets `max_width = 120`. Run `cargo fmt --all` before `npm run check`.
- Decisions taken without the owner, all cheap to change: the mesh overlay is always relayed; a line check of "can't find it" overrides profile data with `Unknown`; schema version is 2; `serde_norway` is the YAML parser (`serde_yaml` is unmaintained).

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
- **Terminal image:** `bash spikes/terminal/build-image.sh` needs Docker running. Built on 2026-11-10 into `spikes/terminal/images/` (gitignored): 1,430 files, 43 MB compressed on disk, of which a learner fetches about 12 MB. `MEM_MB=128 node spikes/terminal/build-state.mjs` builds the saved state next to it (needs `zstd`). Both are local only until Phase 7 publishes them through CI.
- **Spikes:** `spikes/` has its own `package.json` and is not an npm workspace, so `npm ci` at the root ignores it. Pages: `cd spikes && npm run dev`, then `/terminal/` and `/trace/`. Nothing in `app/src` imports from it. Keep it until Phase 7 has what it needs.
- **Known:** `npm audit` reports 10 moderate findings, all one `minimatch` issue inside ESLint tooling. Dev-only, not shipped.
- **Update rule:** at the end of each phase, update Current status, mark the phase done with the date, and record gotchas here. Change Architecture, Design or Animations only when the design itself changes.
