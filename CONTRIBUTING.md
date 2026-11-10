# Contributing

Thanks for helping. Run `npm run setup` once, then `npm run check` before opening a pull request. CI runs the same command.

## Rules that matter most

- The Rust core decides every outcome. The UI only displays results.
- Missing data must produce `Unknown`, never a pass.
- Every device or ISP claim needs a source URL and a date, or it stays `unknown`.
- Never add code that collects or sends a learner's IP address or results.
- A new device is one YAML file and must not need a code change.

The full list is in [CLAUDE.md](CLAUDE.md) and [context-docs/Architecture.md](context-docs/Architecture.md).

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
