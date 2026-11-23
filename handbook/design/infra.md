# Infra design: landing, CI, releases

| | |
|---|---|
| Kind | Design |
| Owner | infra |
| Describes | `scripts/check.sh`, `scripts/affected.py`, `scripts/check_cache.py`, `scripts/cxx-cache.py`, `scripts/land.sh`, `.githooks/`, `.github/workflows/`, `scripts/release/`, `web/public/install.{sh,ps1}` |

How the machinery between a commit and a user is built: what each part
does, what it hands on, and the rules every change keeps. What infra must
be good at is its charter, `handbook/domains/infra.md`; the rules are in
`handbook/guidelines/ci-checklist.md`.

## A commit's path to a user

1. **Commit.** The `pre-commit` hook runs `scripts/check.sh --commit`.
   `scripts/affected.py` maps the staged paths to what they can affect: the
   crates they reach (and their dependents), the tests that read a touched
   data file, `cpl check` on a touched corpus program, a script's unit test.
   A path it doesn't know falls back to `--fast`. Past 25 s the hook warns
   and lets the commit through.
2. **Land.** `scripts/land.sh` rebases onto main and pushes, retrying on a
   rejected push. It runs no gate, so it takes seconds, and refuses a commit
   without a `Domain:` trailer.
3. **CI.** Each push to main queues a `ci` run. One runs at a time; the
   newest tip waits and replaces any older waiting run. The run shards
   `check.sh --fast` across jobs, each restoring a `target/` cache that
   `scripts/ci-mtime.sh` makes reusable by backdating unchanged files.
   Every step has a time cap, so a hang frees the queue.
4. **After CI.** `commit-status.yml` marks every commit the run covered
   (`ci/covered`); on a red run `culprit.yml` names the commits that broke
   it; `watchdog.yml` opens an issue if no run has finished in 30 min.
5. **Release.** `release.yml` picks the newest commit whose shipping jobs
   passed (`scripts/release/shippable.sh`), at most hourly, builds every
   target, smoke-tests the archives and publishes edge. The installers read
   the release's `latest.json`, verify checksums and unpack into `~/.cpl`.

# Reference

## Parts

| Part | Path | What it does |
|---|---|---|
| Gate | `scripts/check.sh` | One script, four tiers: `--commit` (affected only), `--fast` (CI), `--full` (slow tests, judge flags, budgets, wasm and editor smoke), `--timing`. Prints step times. |
| Test selection | `scripts/affected.py`, tests `scripts/test_affected.py` | Staged paths to crates, data-file tests, `cpl check` files, script tests, conformance cases, corpus audit. |
| Stage cache | `scripts/check_cache.py`, tests `scripts/test_check_cache.py` | Skips a stage when the hash of everything it can read matches its last pass. An exclude needs strace evidence that the stage never reads the path. |
| C++ compile cache | `scripts/cxx-cache.py`, tests `scripts/test_cxx_cache.py` | A `g++` shim reusing a binary built from the same bytes, flags, compiler and headers. |
| Landing | `scripts/land.sh`, `scripts/land_stats.py` | Rebase and push; logs `{domain, landed, seconds, tries}` to `.git/cpl-land.log`. |
| Hooks | `.githooks/{pre-commit,commit-msg,pre-push}`, `scripts/install-hooks.sh` | The commit tier; the `area: summary` subject check; a pre-push compile check. |
| Public-history hygiene | `scripts/public-hosts.txt`, the staged scan in `check.sh` | Added lines link only listed hosts; no secret patterns, home paths or local sensitive words. |
| Main CI | `.github/workflows/ci.yml` | The sharded fast gate, editors, CLI on 3 OSes, acceptance and differential checks, the corpus audit, the site build. |
| Non-blocking CI | `.github/workflows/ci-slow.yml` | Wall-clock budgets and LSP latency; its own queue. |
| After CI | `commit-status.yml`, `culprit.yml`, `watchdog.yml`; `scripts/commit_status.py`, `scripts/culprit.py`, `scripts/ci-watchdog.sh` | Per-commit status, culprit naming, the stall alert. |
| Release | `.github/workflows/release.yml`, `scripts/release/{plan.sh,shippable.sh,build.sh,package.py,editors.py,manifest.py,publish.sh}`, tests `scripts/test_release.py` | Edge hourly from the newest shippable commit, stable on dispatch; 5 targets, reproducible archives, editor assets, `latest.json` and `SHA256SUMS`. |
| Installers | `web/public/install.sh`, `web/public/install.ps1`; tested by `crates/cpl-cli/tests/toolchain.rs` | Pick the target, read `latest.json`, verify sha256, unpack into `~/.cpl/toolchains/<id>/`, launcher at `~/.cpl/bin/cpl`. |
| Site deploy | `.github/workflows/site.yml`, `scripts/build-wasm.sh` | Builds `web/` and the playground's wasm, deploys the Worker, smoke-tests the live pages. |
| Pins | `rust-toolchain.toml`, `mise.toml`, `justfile`, `uv.lock`, `pnpm-lock.yaml` | One version per tool, one name per task. |

## Invariants

The rules each change keeps are `ci/R1` to `ci/R24` in
`handbook/guidelines/ci-checklist.md`. The structural ones:

- Landing never runs a gate; CI checks every tip that reaches the front of
  the queue (ci/R7).
- A cache hit can never hide a change (ci/R16).
- No test is weakened or removed to go green; slow tests move tiers (ci/R13).
- Workflows grant only the permissions they use and never run
  `pull_request_target` (ci/R24).
- Release archives rebuild from the commit (`SOURCE_DATE_EPOCH` is the
  commit time).
