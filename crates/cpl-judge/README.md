# cpl-judge

Judge profiles: compiler, flags, platform, stack and source-size limit a submission meets. A file names one with `// judge:`, a project in `cp.toml`, a build with `--judge` (which wins).

## Invariants

- Without any, the judge is `generic`.
- `[judge.<name>]` in `cp.toml` changes a profile's fields or adds one that says what it `extends`.

## Tests

`src/tests.rs`; builds against a profile in `cpl-cli/tests/`.

Where it sits among the crates: `handbook/design/architecture.md`.
