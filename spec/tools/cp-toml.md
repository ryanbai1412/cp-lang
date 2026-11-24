# `cp.toml`

<!-- Generated from `SCHEMA` in crates/cpl-config/src/lib.rs; `BLESS=1 cargo test -p cpl-config` rewrites it. -->

The nearest `cp.toml` at or above a program's folder configures the tools. `cpl lint` reports a table or key not listed here.

## `[fmt]`

The layout `cpl fmt` writes. Read by `cpl-fmt`.

| Key | Meaning |
|---|---|
| `indent_width` | spaces per indent, 2 to 8 (default 4) |
| `line_width` | the width a line wraps at, 40 to 200 (default 100) |

## `[lints]`

Lint levels: each lint or group is "allow", "warn" or "deny". Read by `cpl-lint`.

| Key | Meaning |
|---|---|
| `memory_mb` | the memory a solution may use, for memory lints |
| `<name>` | a lint or group name: its level |

## `[libs]`

Libraries a program may `use`, each by the folder it is in. Read by `cpl-check`.

| Key | Meaning |
|---|---|
| `<name>` | a library name: `"path"` or `{ path = "path" }`, relative to this file or `~/` |

## `[limits]`

A problem's limits, under its own `// limits:` line. Read by `cpl-contest`.

| Key | Meaning |
|---|---|
| `time_ms` | the time limit in milliseconds |
| `memory_mb` | the memory limit in megabytes |

## `[judge]`

The judge profile a build targets, under `--judge` and `// judge:`. Read by `cpl-judge`.

| Key | Meaning |
|---|---|
| `judge` | a profile's name in quotes |

## `[judge.<name>]`

Changes to a judge profile; one that isn't built in says what it `extends`. Read by `cpl-judge`.

| Key | Meaning |
|---|---|
| `extends` | the built-in profile it starts from |
| `<name>` | a profile key (spec/tools/judge-profiles.md) |
