# Phase 1 brief

Self-contained task sheet. **Read this file only**, plus the Architecture sections named below when a step needs them. Do not re-read the ideation doc or research prior art. That work is done.

**Phase 1 is done (2026-11-10). Results are in Handoff.md. This file is kept for reference.**

Goal: answer three risky questions with throwaway code, write the answers into [Handoff.md](Handoff.md), and stop. **Do not build Phase 2 onward. Do not refactor Phase 0.**

## Already done, do not redo

- Phase 0 scaffold passes `npm run check`, and the page runs the Rust core through WASM in a real browser.
- Spike 3 (HLBuilder audit) is finished and recorded in Handoff.md.
- `@xyflow/react` and `v86` are installed in `app/`.
- [`spikes/terminal/build-image.sh`](../spikes/terminal/build-image.sh) has built the Alpine image into `spikes/terminal/images/` (`alpine-fs.json` and `alpine-rootfs-flat/`, gitignored, local only). **It has never been booted.** If the folder is missing, ask the owner to rerun the script with Docker running.
- [`spikes/fixtures/cgnat-traces.json`](../spikes/fixtures/cgnat-traces.json) is a ready trace fixture.
- `app/src/lib/motion.ts` holds the motion tokens.

## Rules

- Spike pages live under `spikes/`, and may be served with a throwaway Vite config. They must not be imported by `app/src` or added to the production build.
- Prefer the smallest code that answers the question. No tests, polish or abstractions in spike code.
- Stop a spike once its question is answered. If one is stuck after two distinct approaches, write down what failed and move on.
- Run `npm run check` before finishing, to prove the spikes did not disturb the app.

## Spike 1: real terminal, graded files

**Question:** can browser Linux let a learner write a real file that the app reads back, at a size and speed learners will accept?

1. Use the image from `build-image.sh` (status in Handoff.md). v86's `tools/docker/alpine/build-state.js` shows the working boot options, and `examples/alpine.html` in `spikes/terminal/.v86` is the reference page.
2. Boot it from a spike page with `new V86({...})`, using `bzimage_initrd_from_filesystem` and a `host9p` root. Connect `serial0_send` and `serial0-output-byte` to a basic terminal.
3. Inside the guest, create `/root/wg0.conf` with `wg genkey` for a real key.
4. Read it back with `read_file` and print it on the host page.
5. Try a saved state (`build-state.js`) and compare time to prompt.

**Report:** works or not, first-download KB, cold boot and saved-state seconds, memory used, any need for cross-origin isolation. Check `read_file` sees guest writes, since a 9p cache mode can matter here.

## Spike 2: trace playback

**Question:** can a labelled packet follow React Flow edges smoothly on a weak phone, under the rules in [Animations.md](Animations.md)?

1. Render `cgnat-traces.json` nodes and edges with `@xyflow/react`.
2. Implement a minimal `buildTimeline` and a player: play, pause, step, speed.
3. Move one overlay element with `transform`, using `getPointAtLength` on the edge path. No per-frame React state.
4. Show address rewrites and the carrier-NAT drop.

**Report:** frame rate on a real low-end Android if the owner has one, otherwise Chrome with 6× CPU throttling (say which). Bundle cost of React Flow, gzipped. Whether pan and zoom during playback stay correct. Read `custom-edge.tsx` at the HLBuilder commit in Handoff.md only if edge paths are unclear.

## Spike 4: budgets

Using the numbers above, confirm or revise the budget table in `Architecture.md`, "Hosting and budgets", and the phone assumption in `Design.md`.

## Finish

Record in Handoff.md, Phase 1 section: each result, terminal engine chosen, and any change these findings force in Architecture, Design or Animations. Apply those doc changes. Delete nothing in `spikes/` until Phase 7 has what it needs. Mark Phase 1 done with the date.
