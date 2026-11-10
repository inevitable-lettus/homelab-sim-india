# Homelab Sim

A free, open-source homelab learning simulator for India. Learners complete missions on a simulated network and see exactly why a connection works or fails. The first track covers remote access on Indian ISP lines, including CGNAT.

**Status: early setup. There is nothing to learn from yet.** Current state and build order are in [context-docs/Handoff.md](context-docs/Handoff.md).

## Run it

Needs Node 24+, [Rust via rustup](https://rustup.rs) and a POSIX shell.

```bash
npm run setup
```

```bash
npm run dev
```

Before sending a change, run the same check CI runs:

```bash
npm run check
```

## How it fits together

A deterministic Rust core compiled to WebAssembly decides every outcome. A React app renders the results. Data lives as YAML in git and ships as a static JSON bundle. There is no backend and there are no accounts. Details are in [context-docs/Architecture.md](context-docs/Architecture.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

- Code: [AGPL-3.0-only](LICENSE)
- `data/`, `content/`, `missions/` and `curriculum/`: [CC BY-SA 4.0](LICENSES/CC-BY-SA-4.0.txt)
- Reused third-party code: [THIRD_PARTY.md](THIRD_PARTY.md)
