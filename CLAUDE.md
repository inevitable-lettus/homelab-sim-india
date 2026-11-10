# Homelab Sim (India)

Free, open-source, India-specific homelab learning simulator. Static site, Rust/WASM logic core, React UI.

**Start every session by reading [context-docs/Handoff.md](context-docs/Handoff.md).** For Phase 1, read only [context-docs/Phase1-Brief.md](context-docs/Phase1-Brief.md).

Commands: `npm run dev`, `npm run check` (run before every commit).

| File | Holds |
|---|---|
| [context-docs/Handoff.md](context-docs/Handoff.md) | Living state, decision log, phased build order |
| [context-docs/Architecture.md](context-docs/Architecture.md) | Components, boundaries, world model, data packs, licensing |
| [context-docs/Design.md](context-docs/Design.md) | Screens, missions, depths, verdict wording, India behaviour |
| [context-docs/Animations.md](context-docs/Animations.md) | Motion rules, trace playback, catalogue, reduced motion |
| `docs/ideation.md` | Local only, not in the repo. Snapshot of the original ideation, superseded where it conflicts |

## Hard rules

1. No backend, accounts or runtime API.
2. The UI never decides validity. The Rust core does, deterministically.
3. Missing data is `Unknown`, never a pass.
4. Rules read capabilities, never product names. A new device is one YAML file and no code change.
5. Never collect or transmit a learner's IP address or results.
6. Every ISP or device claim carries a source and date, or is `unknown`.
7. No LLM in the runtime.
8. Code is AGPL-3.0. `data/`, `content/`, `missions/` and `curriculum/` are CC BY-SA 4.0. Record reused HLBuilder files in `THIRD_PARTY.md`.

Update `Handoff.md` at the end of every phase.
