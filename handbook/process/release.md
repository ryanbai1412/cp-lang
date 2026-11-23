# Release

| | |
|---|---|
| Kind | Process |
| Owner | infra |
| Status | Active |
| Describes | `.github/workflows/release.yml`, `scripts/release/`, `web/public/install.{sh,ps1}` |

A release is how `cpl` reaches a contestant's machine. There are two
channels: edge, built at least hourly from the newest main commit that
works, for people who want fixes now; and stable, which Ryan cuts from a
fully green commit. One command installs either on Linux, macOS or Windows,
checks the download's checksum, and needs no root. Every release can be
rolled back to the previous toolchain with one command.

We know a release works because each edge is install-smoked on all three
OSes before users see it. The weakest areas today are that releases aren't
signed yet and builds aren't checked for reproducibility. The rules are in
[the CI checklist](../guidelines/ci-checklist.md#releases).

## Install

```sh
curl -fsSL https://cp-lang.org/install.sh | sh                    # Linux, macOS
irm https://cp-lang.org/install.ps1 | iex                         # Windows
curl -fsSL https://cp-lang.org/install.sh | sh -s -- --channel edge
cpl --version
# cpl 0.3.0-edge+ccf30acf
# toolchain edge-2026-11-23-ccf30acf, chosen by the default channel
```

The installer detects the platform, downloads the archive from the GitHub
release, checks its sha256 against `SHA256SUMS`, unpacks it into
`~/.cpl/toolchains/<id>/` and puts a launcher at `~/.cpl/bin/cpl`. No root
needed. With no flag it installs stable, or edge with a notice while there
is no stable release.

From source, at any commit: `cargo install --locked --git
https://github.com/ryanbai1412/cp-lang cpl-cli --rev <sha>`.

# Reference

## Channels

| Channel | Built from | When | Version |
|---|---|---|---|
| edge | the newest main commit whose shipping checks pass | at least hourly | `0.3.0-edge+<sha>` |
| stable | a `v*` tag on a fully green commit | by hand, Ryan decides | `0.3.0` |

**Edge's shipping checks** are the CI jobs a user would feel if they broke:
the gate's conformance shards, `cli` on every OS, and acceptance. Other jobs
(an editor test, a unit-test shard) may be red: edge still ships, since
shipping a working compiler beats waiting for every job.
`scripts/release/shippable.sh` prints the commit it would pick.

**Edge timing.** `release.yml` starts after each main CI run ends, and on
an hourly schedule. When edge moved less than an hour ago, the oldest
waiting run sleeps until the hour is up and then picks the newest shippable
commit, so a commit that becomes shippable just after a publish still ships
within the hour. One dated prerelease per day (`edge-YYYY-MM-DD-<sha>`) is
kept.

**Stable** needs every CI job green on its commit.

## What a release contains

| Asset | Built from |
|---|---|
| `cpl-<target>.tar.gz` / `.zip` for x86_64 and aarch64 Linux (static musl), x86_64 and aarch64 macOS, x86_64 Windows | `crates/cpl-cli`, with the stdlib embedded |
| `SHA256SUMS`, `latest.json` | `scripts/release/manifest.py`; `latest.json` maps each target to `{file, sha256}` plus `toolchain`, `channel`, `commit`, `date` |
| `cp-lang.vsix` | `editors/vscode` |
| `cp-lang-jetbrains.zip` and its plugin-repository XML | `editors/jetbrains` |
| `editors.tar.gz`: Neovim/Vim, Helix, Zed, Emacs, tree-sitter | `editors/` |

One binary is one version: compiler, stdlib and judge profiles together.
`cpl --version` prints channel, version and commit, and every generated C++
file carries the same stamp so an old submission can be rebuilt. Archives
are reproducible: `--locked`, pinned Rust, `SOURCE_DATE_EPOCH` set to the
commit time.

## Pipeline

```text
plan ──► build (5 targets) ──► editors ──► publish ──► install-smoke (Linux, macOS, Windows)
```

- **plan** picks the channel, commit and version (`scripts/release/plan.sh`).
- **build** compiles each target and smoke-runs `cpl run` on a hello program
  on the three native ones.
- **publish** replaces the `edge` release's assets (or creates the stable
  one) and writes `latest.json`.
- **install-smoke** runs the real one-liners on each OS, with and without a
  channel flag, and checks `cpl --version` names the published commit.

Dry run without publishing: dispatch `release.yml` with `publish: false`.

## Update and roll back

```sh
cpl self update                  # newest release of your channel
cpl self channel edge            # switch the default channel
cpl self install edge-2026-11-23-ccf30acf
cpl +stable run a.cp             # one command on another toolchain
```

A bad edge build: install a dated edge or switch to stable, then report it
with `cpl report`.

## Compatibility before 1.0

A minor bump (`0.3` to `0.4`) may break programs and ships `cpl fix`
rewrites; a patch never breaks a valid program. Every release lists its
breaking changes in `CHANGELOG.md`.

## Changes

- 2026-11-23: first version, folded from the infra release plan and updated
  to the hourly edge from the newest shippable commit.
