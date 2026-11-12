# Handoff

Living state and build order. **Read this first in every session, and update it at the end of every phase.** Any session can pick up at the first phase not done. Phases are dependency-ordered, and each ends with its tests green.

## Current status (2026-11-12)

**Phase 7 is done (2026-11-12): Mission 5, Write the tunnel, is playable from a file editor and from a real Linux terminal in the browser. Next action: the owner's review, then sandbox phase S1 ([Sandbox.md](Sandbox.md) §14), stopping for review after each of S1 to S5.** Phase 6's learner test has still not been run.

**Phases 0, 1, 4 and 5 are done**, with the minimum Phase 2 and 3 material they needed. Repo: <https://github.com/inevitable-lettus/homelab-sim-india>. Preview: <https://inevitable-lettus.github.io/homelab-sim-india/>. The deployed site has a landing page, the track map and Missions 1 to 4, playable on desktop and at 360px, with trace playback, a step log, findings, debriefs, concept pages, settings and local progress. Phase 1 confirmed v86 as the terminal engine and React Flow for the board.

**Owner's build order (2026-11-10), which overrides the phase numbering:** Part 1 is Phases 1, 4, 5, 6, 7, with only the minimum Phase 2 and 3 material the code needs (Rust schema types, short specs for Missions 1 to 5, a few archetypes, one clearly labelled teaching ISP profile with no real-ISP claims). Part 2 is Phases 2, 3 and 8. Part 3 is Phase 9. The owner approves between parts.

**Owner's change of order (2026-11-11, later the same day):** the sandbox is parked until Phases 6 and 7 are done and something is online for people to see. The site says "Build your own: coming soon". **Next action: Phase 7 (terminal missions)**, then the Part 1 report and the owner's approval, then S1 to S5 (spec in [Sandbox.md](Sandbox.md)). Data for the sandbox is listed in [Data-Backlog.md](Data-Backlog.md) and is worked by agents slowly, in parallel, one pack at a time.

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

Owner-confirmed on 2026-11-11:

| Area | Decision |
|---|---|
| Sandbox | "Build your own" sandbox with parts found in India, including unconventional machines (Apple silicon laptops, old laptops, old phones). In the first release. Replaces Design.md's "sandbox not in the first release" |
| Builder | Fable 5.2 builds Phases S1 to S5 from [Sandbox.md](Sandbox.md) |
| Data | Agents fill [Data-Backlog.md](Data-Backlog.md) slowly, one pack per change |

Taken in the sandbox spec without the owner, cheap to change (marked **(default)** in [Sandbox.md](Sandbox.md)):

| Decision | Note |
|---|---|
| Prices are learner-typed only, stored on the device, stripped from share links by default | Keeps "rupee prices deferred" intact |
| Learner-entered custom devices, `Provenance::Learner`, "Based on what you entered" | Lets anyone model their own old laptop |
| No CPU performance model | No open, sourced benchmark data |
| Slab tariffs billed on top of the household's own use, default 150 kWh/month | The India-specific power lesson |
| Busy share 10% by default; UPS and laptop usable-battery constants as stated teaching assumptions | Shown next to every number they affect |
| Outage scenarios assume the ISP's side stays up unless the scenario says otherwise | Stated as an assumption |
| Sandbox slot: after Phase 5, before Phase 6, so the learner test can include it | Owner may reorder |

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

### Phase 5: Mission UI and playback (done 2026-11-11)

Mission screen, board, findings, trace player and step log, depth switch, local progress, per [Design.md](Design.md) and [Animations.md](Animations.md).

**Done =** a full mission playable at 360px and desktop, by keyboard and in reduced motion. Playwright covers win and failure.

#### Phase 5 result

Built: landing page with one blocked trace and the track map, the mission screen (brief, goals, controls, board, inspector, player bar, step log, result, findings, debrief), concept pages, settings (depth, motion, theme, export, import, clear) and credits. File map in [Architecture.md](Architecture.md#app-structure-as-built-phase-5), behaviour in [Design.md](Design.md) and [Animations.md](Animations.md). Bundle schema is now 3 (`tracks`, from `content/tracks.yaml`).

| Check | Result |
|---|---|
| Browser tests | 13 pass (`npm run e2e`): 7 at 1280x800, 6 at 360x740. Failure then win in Mission 2, Mission 3 by answers, reduced-motion stepping, autoplay to the end, keyboard open and close of the inspector, no sideways scroll on five pages |
| Step log equals the trace | Asserted step for step. The test loads the same WASM in Node to get the expected trace |
| Unit tests | 16 app tests (8 core through WASM, 8 timeline), 36 Rust tests |
| Playback | 60 fps at 6x CPU throttle and 58 fps at 20x, desktop and 360px at 3x pixel ratio, on Mission 4's 27 steps. Headless Chrome on an Apple M4, so script cost only |
| Size | 350,616 B gzipped of the 512,000 B budget: WASM 193 KB, JS 149 KB, CSS 9 KB |
| By eye | Missions 2, 3 and 4 and the landing page, in light and dark, at desktop and 375px, in the app's browser pane |

Not verified: a real phone, a screen reader, visual snapshots, Mission 1 by hand (its paths are covered by the Rust tests only), and cable drag feel on a touch device (dragging is off on narrow boards).

Gotchas for later phases:

- **Card size is in two places**: `NODE_W` and `NODE_H` in `app/src/mission/geometry.ts` and `.device` in `app.css`. Mission layouts assume 160 by 64: wide rows are 190 apart, narrow rows 84 apart, with side branches at x 185.
- **A travelling label does not work on a stacked board.** It covers the card above the packet. The narrow board docks the label at the top instead. The sandbox board (S4) should reuse `Board` with `mode`, not fork it.
- The player pauses when the tab is hidden, and `requestAnimationFrame` stops in a hidden pane. A playback that "hangs" in an embedded preview is usually that.
- `@playwright/test` uses the installed Chrome (`channel: 'chrome'`), so nothing is downloaded. CI runs `npm run e2e -w app` after `npm run check`, on the build that check made. It is not part of `npm run check`.
- The `impeccable` skill's project setup interview was skipped on purpose. [Design.md](Design.md) is the product context, and its rules win over the skill's defaults (no web fonts, no decorative motion, system font stack for metered data).
- React Flow is 12.x: `fitView` padding takes per-side pixel strings, which is how room is kept for the packet label.
- Decisions taken without the owner, all cheap to change: hash routes; one `localStorage` key `homelab-sim:v1`; the debrief shown in the dock and not as a dialog; results hidden until an attempt has been run with the current setup; cable length labels left off the board (the step log carries distance and time); the line-check screen left for Phase 8.

### Phases S1 to S5: Build your own (sandbox)

Specified in full in [Sandbox.md §14](Sandbox.md#14-build-phases-s1-to-s5), with each phase's done criteria. In short: **S1** schema v4 and data build, **S2** world additions, intents and fit rules, **S3** power, **S4** sandbox UI, **S5** templates and first real data. S1 to S3 are core-only and can start once the uncommitted Phase 5 work is committed. S4 needs the Phase 5 board. S5 needs the P0 items of [Data-Backlog.md](Data-Backlog.md).

**Parked on 2026-11-11 before any schema code was written.** What exists for it:

- `tools/data-build/tests/fixtures/mission-reports.tsv` and the test `mission_reports_match_the_recorded_fixtures`: every mission's grade at both depths, from its starting and a solved world, plus three device-pack worlds. S1's "mission reports byte-identical" check is this test. Update it only with `UPDATE_MISSION_FIXTURES=1` and a reviewed diff.
- `data-drafts/` (gitignored): P0 data drafted by four research agents on 2026-11-11, each pack with an `.evidence.md` file. **Unreviewed.** Nothing in it may enter `data/` until S1 can validate it and a reviewer has opened every source. `data-drafts/README.md` holds the conventions the agents were given, including these refinements of Sandbox.md §4 that S1 should adopt or reject: `power.input` is tagged by `kind`; `adapter.provides` is one struct with `nic`, `bays` and `usb` (a hub provides more than one); `client.plays`, `client.apps` and `adapter.provides` are `Cap` values so a real device's claim has a source.
- Planned for S1 and not built: `schemars` behind a `schema` feature of `sim-core` for JSON Schema, `serde_path_to_error` in the data build for field paths in errors, and a macro that declares each all-`Cap` section struct together with its "every value sourced" check.

**Done =** S5's done criteria, and this file updated with measured catalog size and gotchas.

### Phase 6: First learner test (gate): kit written 2026-11-11, test not run

**Status:** Missions 1 to 4 are live on the preview URL, and [curriculum/learner-test-kit.md](../curriculum/learner-test-kit.md) has the facilitator script, the tasks, what to observe, a five-point rubric for "can they explain CGNAT", the note sheet and the gate table. **No learner has been tested.** An agent cannot do that part. The gate below is still open, and building continued past it on the owner's instruction.

Ship Missions 1–3 without the terminal, and the sandbox if S1 to S5 are done. Put them in front of about five real learners from the target groups, watch them, and check whether they can explain CGNAT afterwards, and whether they can plan a build and read its report.

**Done =** findings are written here and the owner decides whether to continue, change or stop before the terminal work. No beginner has tested this concept yet.

### Phase 7: Terminal missions (done 2026-11-12)

Guest image, `TerminalHost`, file parsers, `grade` on artifacts, file-editor fallback, guest image source publication.

**Done =** Mission 5 passes and fails correctly from both terminal and editor. **Met**, by the tests below.

What exists:

- **Core.** `x25519.rs` (Curve25519 and key base64, checked against RFC 7748 and a Node key pair), `wireguard.rs` (`wg-quick` parser with line-numbered problems; the private key is never stored or echoed), `artifacts.rs` (a config sets the tunnel on `spec.node`; a public key file sets the key the `register_on` host has on record, an empty string when missing, which never matches). Reasons added: `artifact-ok`, `artifact-key-unchecked`, `tunnel-wrong-peer-key`.
- **Engine changes in `walk.rs`.** `Payload` carries the sender's peer index. A receiver drops a packet locked to a key that is not its own. A failed handshake on a hand-made tunnel is shown as a setup leg.
- **Mission 5**, `missions/write-the-tunnel/mission.yaml`: a hub at a public address with forwarding on, a phone that is already set up, and a server whose tunnel comes only from `/root/wg0.conf` and `/root/publickey`. Two attempts, at `idle_s` 0 and 600.
- **UI.** `mission/Files.tsx` (editors, verdicts, key tool), `lib/keys.ts`, `lib/terminal.ts` (v86 host), `mission/TerminalPanel.tsx` and `TerminalView.tsx` (xterm.js, lazy chunk). WASM export `wireguard_public_key`.
- **Image.** `tools/guest/` (`versions.env`, `build-image.sh`, `build-state.mjs`, `build.sh`), run by `npm run guest`. CI and the deploy workflow build it with Docker and cache it on the hash of `tools/guest/`.

Results (2026-11-12, on the M4 MacBook):

| Check | Result |
|---|---|
| Rust tests | 39 in data-build (13 new for Mission 5), 21 in sim-core. Win, no keepalive, slow keepalive (60, 90, 300 s), wrong endpoint port and address, AllowedIPs too narrow, key from another pair, missing or broken key file, wrong hub key, wrong tunnel address, starter file |
| Missions 1 to 4 fixtures | byte-identical. Four lines added for Mission 5 |
| Browser tests (Chrome, 1280 and 360 wide) | 16 pass, 2 skipped. Mission 5 from the editor at both widths. Mission 5 from the real terminal on desktop: keys from `wg genkey`, config written with `printf`, fail without keepalive, win with it, deleted file fails |
| Screenshots looked at | 360px brief with problems and key tool; 360px board; 1280px terminal running `nano` on the starter file |
| Terminal, saved state to prompt | under 5 s in the test, including the download from localhost |
| First download for the terminal | 13.8 MB uncompressed (state 11.3 MB, engine 2.4 MB, index 0.1 MB), under the 20 MB budget |
| App size | 452 KB gzipped of 512 KB. xterm.js is 84 KB of that and loads only when the terminal opens |
| Guest | Alpine 3.21.0, kernel 6.12.115-0-virt, 74 packages, 1,430 rootfs files, 58 MB on disk in total |

**Decisions made here (the owner can overrule):**

- **No keepalive.** A peer with an endpoint and no keepalive is modelled as having last dialled when the tunnel was last used, `idle_s` ago. So it works at `idle_s: 0` and fails with `nat-mapping-expired` at 600, as the curriculum says. Real WireGuard only speaks when it has something to send, so "works at first" assumes the learner tested from the server once.
- **GPL.** The image ships with a **written offer** plus pointers to Alpine's `aports` recipes, in `guest/SOURCES.md`, linked from the credits page and described in `THIRD_PARTY.md`. This binds whoever publishes the site for three years. The alternative is to publish the source archives beside the image. Owner to confirm.
- **Challenge depth starts with empty files.** `starter` is the guided text.
- **Files are graded as last checked**, not as typed, so playback is not rebuilt on every keystroke.
- **Lesson keys are saved in this browser's localStorage** with the rest of the draft. The UI says not to use them on a real server.

Gotchas:

- The state, the engine, the index and the rootfs are one set. `build-state.mjs` refuses to run if `node_modules/v86` is not the commit in `versions.env`. Bump both together.
- Alpine packages are pinned to the 3.21.0 image but not to exact versions, so a rebuild can pick up newer package builds. `manifest.json` records what was installed.
- Locally the image was assembled from the Phase 1 rootfs (`GUEST_IMAGES=spikes/terminal/images GUEST_BIOS=spikes/terminal/.v86/bios bash tools/guest/build.sh`), because Docker was not running. **The Docker path of `build-image.sh` has only run in CI**: check the first CI run after this push.
- The built-in browser pane freezes animation frames while hidden, so xterm draws nothing there. Use Playwright for terminal checks.
- A learner-registered key equal to another peer's key makes the hub match the wrong peer entry first. The tests use an unrelated key for the mismatch case.
- ESLint must ignore `app/public/guest` (the engine is minified JS).

After the push (commit `ab43c60`): CI and the deploy both passed, with the image built by Docker on the runner (kernel 6.12.117-0-virt there, 6.12.115 locally: the package drift noted above). The live site serves `guest/manifest.json`, the state (11.3 MB), the engine and `SOURCES.md`, and the bundle has Mission 5.

**Known flaky test:** the phone run of "The wall" timed out once in CI and passed on retry. The same test is what made the CI run for the Phase 6 commit (`cd23474`) fail, which was not noticed at the time. It has never failed locally. CI now uploads Playwright traces (`playwright-traces` artifact) so the next failure can be read. Not yet diagnosed.

Not verified: a real phone, a screen reader on the editor or the terminal, the terminal on a slow connection or a low-memory device, Firefox and Safari, `vi`. GitHub Pages limits (1 GB site, 100 GB a month soft bandwidth, 10 minute deploy) are quoted from memory of GitHub's documentation, not re-read; the site is about 60 MB.

Mission 5's brief, hints and debrief were written by the coding session and are due a pass by the content author, like the other missions.

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
- [ ] Do learners trust a sandbox report with many "can't tell yet" answers, or does it read as broken? (Phase 6)
- [ ] Which ISP-supplied routers are actually issued today, per ISP and circle? (Data-Backlog D1)
- [ ] Do target learners mostly use phones? (Phase 1 and 6)
- [ ] Which reports count as enough to change an ISP profile's confidence? (Before community reports)
- [ ] Are HLBuilder's maintainer and Homelab India worth contacting early? AGPL doesn't require it, but it is a courtesy and could help reach.

## Known risks

- **ISP claims:** current evidence is forum-level. Verify before publishing, and keep "unknown" honest.
- **Effort:** Rust core, real terminal and a full curriculum are the heavier choices, on one person's time. Phase 6 exists to catch a wrong bet early.
- **Sandbox data volume:** real device packs need a source for every value. Unconventional devices will show many unknowns at first. The archetypes and learner-entered devices cover the gap; the "help add this data" loop is how it improves.
- **Sandbox claims about real products:** a wrong sourced value now tells a learner what to buy. Review every real pack like code.
- **Direct competitor:** [HLBuilder](https://github.com/Butterski/homelab-builder) is active. Our difference is teaching, sourced verdicts, failure simulation and the India focus.
- **Community channels:** [homelabindia/wiki](https://github.com/homelabindia/wiki) was active in November 2026, and a separate [homelab-india](https://github.com/homelab-india) organisation exists. Which one is the main community is unconfirmed.

## Session context

- **Canonical commands:** `npm run setup` (once), `npm run dev`, `npm run check` (run before every commit), `npm run build`, `npm run e2e` (builds, then browser tests; needs Chrome installed).
- **Commits:** this repo's local Git config uses the GitHub no-reply address on purpose, because the repo is public. Sign off with `git commit -s`.
- **PATH:** Rust came from Homebrew's `rustup`, which isn't on the shell PATH. The npm scripts handle it through `scripts/env.sh`. For direct `cargo` use: `export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"`.
- **Toolchain:** Rust 1.99.1 pinned in `rust-toolchain.toml`. `wasm-bindgen` CLI pinned to the crate version in `Cargo.toml`, used directly instead of wasm-pack. TypeScript is 6.x because `typescript-eslint` does not support 7 yet.
- **Generated, gitignored:** `app/src/wasm-pkg/`, `app/public/data/`. Run `npm run wasm` and `npm run data` after pulling Rust changes.
- **Terminal image:** `npm run guest` builds it into `app/public/guest/` and needs Docker, `zstd` and Python 3 (see Phase 7). The Phase 1 copy: `bash spikes/terminal/build-image.sh` needs Docker running. Built on 2026-11-10 into `spikes/terminal/images/` (gitignored): 1,430 files, 43 MB compressed on disk, of which a learner fetches about 12 MB. `MEM_MB=128 node spikes/terminal/build-state.mjs` builds the saved state next to it (needs `zstd`). Both are local only until Phase 7 publishes them through CI.
- **Spikes:** `spikes/` has its own `package.json` and is not an npm workspace, so `npm ci` at the root ignores it. Pages: `cd spikes && npm run dev`, then `/terminal/` and `/trace/`. Nothing in `app/src` imports from it. Keep it until Phase 7 has what it needs.
- **Known:** `npm audit` reports 10 moderate findings, all one `minimatch` issue inside ESLint tooling. Dev-only, not shipped.
- **Update rule:** at the end of each phase, update Current status, mark the phase done with the date, and record gotchas here. Change Architecture, Design or Animations only when the design itself changes.



## Sandbox data drafts (2026-11-11, unreviewed)

Four data agents drafted P0 packs into `data-drafts/` (gitignored), each with an `.evidence.md` file. Nothing is in `data/`. A reviewer must open every source before S1 uses them.

- **Drafted:** 6 OS packs, 3 runtime packs, 5 device packs (MacBook Air M1, ThinkPad T480, ThinkCentre M720q, Raspberry Pi 4, TP-Link UE300), 5 service packs, `reference/media.yaml`, battery and UPS notes, 3 tariffs (BESCOM, MSEDCL, BSES Rajdhani), 1 UPS (APC BX600C-IN).
- **Not drafted:** the DC mini UPS and the ISP router pack. The maker pages would not load.
- **Weak spots the agents flagged:** the T480 is not pinned to one configuration; India availability is `unknown` for the Pi and the MacBook Air; the BSES tariff may be out of date; `battery_wh` for the APC unit is computed, not published; MSEDCL slabs include the wheeling charge.
- **Schema gaps they hit**, for S1 to settle: no `armv6`, `ppc64el` or `s390x` arch; `host_network` cannot say "localhost only"; no per-kW fixed charge in a tariff; no optional or outbound-only ports; no minimum OS version on an install; `battery.wh` is a whole number.

## Sandbox S1: schema v4 and data build (done 2026-11-12)

- `SCHEMA_VERSION` is 4. Pack types live in `crates/sim-core/src/packs.rs`: `Cap` with `basis` and `note`, `PackKind::Learner`, `Category`, every device section, OS, runtime, service, tariff, outage, media format and template packs. `bundle.rs` re-exports them.
- The three archetypes are migrated (`category`, `compute`, `network`). Missions did not change.
- **Fixture:** `mission-reports.tsv` changed in one token per line only, `"schema_version":3` to `4` (23 replacements on 23 lines, then the fixture test passed unchanged). Nothing else in any report moved.
- Data build: loads `data/os`, `data/runtimes`, `data/services`, `data/power/tariffs`, `data/power/outages`, `data/reference/media.yaml`, `content/sandbox/templates`. Checks: category section allow-list, sourcing for every real pack (the `Audit` trait walks every `Cap`), id references, tariff slab order, tracking parameters in source links, templates settle. Errors name the file and the field (`serde_path_to_error`).
- Output is split: `bundle.json` keeps what missions name, `catalog.json` holds the rest. `Bundle::merge` joins them.
- JSON Schemas are generated into `schemas/` (`cargo run -p data-build -- schemas`). A test fails when they are stale.
- TypeScript mirror: `app/src/lib/packs.ts`.

Decisions taken without the owner, cheap to change:

- OS, runtime, service, tariff and media packs take `teaching: true` for generic teaching packs. Outages use `kind: teaching | real` as the spec says. A real pack needs sourced values and at least one entry in `sources`.
- Schema gaps the data agents hit: added `armv6`, `ppc64le`, `s390x`; `host_network` takes `yes | limited | no` (a plain true or false still reads); tariffs have `fixed_paise_per_kw_month`; ports have `optional` and `direction`; an install has `os_min_version` (shown, not read by rules). `battery.wh` stays a whole number.
- Drafts use `Cap` in more places than the spec (`needs.host_network_for`, `needs.static_lan_ip`, `needs.transcode_apis.*`, `paid_features`, `adapter.provides.*`). The schema follows the drafts.

Gotchas:

- **WASM size.** Parsing the pack types took the module from 206 KB to 293 KB gzipped and broke the 512 KB budget. Fix: `opt-level = "z"` and `wasm-opt -Oz` (npm dev dependency `binaryen`, run by `scripts/build-wasm.sh`). Total is now 492,667 B. There is about 19 KB of headroom for S2 and S3.
- `Cap` is deserialized by hand. A map whose first key is one of `value, source, retrieved_at, basis, note` is a sourced value; any other map is the value itself. No section type may use those names for its own top-level fields.
- One-file tests in S1 prove a pack of every kind and category loads and lands in the catalog. Rule-level one-file tests come with the rules in S2 and S3.


## Sandbox S2: run, fit and connect in the core (done 2026-11-12)

- World additions (`intents`, `power_feeds`, `home`, `custom_devices`, switches, power units, host `os`/`runtime`/`guest`/`alloc`/`parts`/`settings`/`role`/`price`). A world with none of them is a mission world and its report is unchanged: the mission fixture did not move.
- `Report.build` (only for builds): goal results, per-machine budgets (RAM, bays, USB, Ethernet ports), parts list with the learner's own prices, the list of unknowns with pack and field.
- Goals expand to attempts named `intent:<goal id>`, so the trace decides reach and stream goals. A goal fails if any finding under it is a fail and an error, is "can't tell" if one is unknown (or nothing spoke about it), else passes.
- Switches are walked (`switch-forwards`) and looked through when finding a host's gateway.
- WASM: `add_catalog_json` (merge the catalog), `validate_world_json` (list of `{kind, at}` problems; empty means loadable).
- Tests: `crates/sim-core/tests/builds.rs`, 11 tests, table-driven, pass/fail/unknown for each rule in Sandbox.md 7.1 to 7.3, with an in-test bundle of teaching packs.

Deviations from Sandbox.md, on purpose:

- The rules live in `crates/sim-core/src/build/` (`run.rs`, `fit.rs`, `connect.rs`, `expand.rs`, `check.rs`, `power.rs`), not `rules/`.
- Tests are Rust tables, not fixture files under `tests/builds/`.
- Extra reasons: `intent-reach`, `intent-not-checked`, `learner-value-used`, `switch-forwards` and six `assumed-*`.
- Warnings are `(Fail, Warning)` and do not fail a goal. Unknown OS requirements are `(Unknown, Info)` and do not make a goal "can't tell".
- A `Cap` map is a sourced value when it has a `value` key (it used to be "when its first key is a sourced key"). No pack struct held by a `Cap` may have a field named `value`.
- The step text for the 48 new reasons in `content/steps.yaml` is a short first draft. The content author owns the final wording.

### Size debt (owner's decision, 2026-11-12)

The site is 548,763 B gzipped against the 512,000 B in Architecture.md. The owner said to get the sandbox working first and not tune for phones yet, so `scripts/check.sh` allows 700,000 B for now. Where the bytes are (twiggy, before `wasm-opt`): derived readers for packs about 216 KB, world model 117 KB, missions 91 KB, tagged enums read through serde's buffer 80 KB. Reading `Cap` through one untyped step saved about 45 KB raw. Reading everything from a `serde_json::Value` did not help by itself. Ideas not yet tried: hand-written readers for the tagged enums (`Node`, `Intent`, `Target`), a lighter reader generated by the `section!` macro, splitting the sandbox rules into a second WASM module loaded on `#/build`.
