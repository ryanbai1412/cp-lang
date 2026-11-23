# Domain: infra

| | |
|---|---|
| Kind | Domain charter |
| Owner | infra |
| Status | Active |
| Describes | `.github/`, `scripts/check.sh`, `scripts/land.sh`, `.githooks/`, `scripts/release/`, `web/public/install.{sh,ps1}` |

Infra is the machinery between a commit and a user: landing a commit on
main, the checks that run on it, and the releases and installers that put
`cpl` on a contestant's machine. Every other domain depends on it, so its
first job is to stay out of their way.

## What good looks like

A lane commits a change and runs `scripts/land.sh`; about 7 seconds later
the commit is on main. Within minutes CI says whether main still works. If
it doesn't, infra names the commit that broke it from the failing log, and
its lane fixes it or infra reverts it. Within the hour, edge ships from the
newest commit no worse than the last edge on the shipping checks, and a stranger installs it with
one line:

```sh
curl -fsSL https://cp-lang.org/install.sh | sh      # Linux, macOS
irm https://cp-lang.org/install.ps1 | iex           # Windows
```

## How it fits together

- **Landing** is cheap on purpose. A pre-commit hook runs only the tests the
  change can affect, within 25 seconds, and `land.sh` only rebases and
  pushes. There is no review gate.
- **CI** checks every main tip with the full gate, sharded across hosted
  runners. One run goes at a time and the newest commit waits; each
  commit's `ci/covered` status says which run covered it.
- **The sheriff** is infra's own loop: read each red run, name the culprit,
  get it fixed or reverted.
- **Releases** build every target from the commit `shippable.sh` picks,
  publish edge hourly, smoke-test the installers on three OSes, and leave
  stable to Ryan.

The ideas behind these, and every rule, are in
[the CI guideline](../guidelines/ci.md) and
[its checklist](../guidelines/ci-checklist.md).

## How we know it is good

`land_stats.py` tracks landing time and commits per hour; `devex_stats.py`
tracks how much lane time goes to tooling. The scripts have their own unit
tests, and every edge release is install-smoked on Linux, macOS and
Windows. The weakest areas today: main CI takes about 8 minutes against a
3-minute goal, there is no alert when edge or the site falls behind, no
release signing, and no automatic flake detection. The quality register
below lists every quality with its check, `missing` where there is none.

## Where to start

- **A red main:** [landing and CI](../process/landing-and-ci.md), "When main
  goes red".
- **A slow commit or CI step:** `python3 scripts/affected.py <paths>`, then
  `.github/workflows/ci.yml`.
- **A release question:** [release](../process/release.md).
- **How the parts are built:** [infra design](../design/infra.md).

# Reference

## Scope

| Infra owns | Not infra's (owner) |
|---|---|
| CI workflows: `.github/workflows/{ci,ci-slow,culprit,release,site,os,acceptance,jetbrains,debugger-loop}.yml` | What each gate step checks: the domain whose test it is |
| The gate `scripts/check.sh` (`--commit`, `--fast`, `--full`, `--timing`), its caches `scripts/check_cache.py`, `scripts/cxx-cache.py`, and test selection `scripts/affected.py` | Emitted-C++ bugs CI finds (perf, compiler) |
| Landing `scripts/land.sh`, `scripts/land_stats.py`; hooks `.githooks/`, `scripts/install-hooks.sh` | The site's content and Worker code (site) |
| Keeping main green: triage, fix forward or revert any red, culprit finding `scripts/culprit.py` | `cpl self` and the launcher (tools) |
| Releases: `release.yml`, `scripts/release/` (plan, build, package, editors, manifest, publish, `shippable.sh`) | Another domain's tool kept in `scripts/`, e.g. `debt-ratchet.py` (tech-debt), `spec-stub.py` (spec), `review-check.py` (idioms): infra keeps only the gate wiring |
| Installers `web/public/install.sh`, `install.ps1` and the install smoke | Repository secrets (coordinator) |
| The site deploy workflow `site.yml` and `scripts/build-wasm.sh` | Editor plugin code (tools, editors) |
| Devex measurement `scripts/devex_stats.py`; toolchain pins `rust-toolchain.toml`, `mise.toml`, `justfile`, lockfiles | |
| Public-history hygiene: `scripts/public-hosts.txt` and the staged scan in `check.sh` | |

## Interfaces

**Infra provides**

| Interface | Contract | Used by |
|---|---|---|
| `scripts/land.sh` | rebases and pushes HEAD to main in seconds, or says why not; refuses a commit without a `Domain:` trailer; prints `land: pushed <sha> to origin main in <n>s (try <k>)` | every lane |
| `scripts/check.sh --commit` (pre-commit) | runs only what the staged paths can affect, within 25 s; past that it warns and lets the commit through | every lane |
| `scripts/check.sh --fast` exit code | 0 means the tree is landable; prints step times | CI, anyone |
| CI on every main tip | one run going, the newest tip waiting; a red run names its culprit | every lane, release |
| Edge release: GitHub release `edge` with `latest.json` and `SHA256SUMS` | `latest.json` maps each target to `{file, sha256}` plus `toolchain`, `channel`, `commit`, `date` | installers, `cpl self` |
| `install.sh`, `install.ps1` | install the newest release of a channel into `~/.cpl`, verifying checksums | users, the site |
| Commit-subject areas (`.githooks/commit-msg`) | the area list in `AGENTS.md` | every lane |

**Infra needs**

| Interface | From |
|---|---|
| Tests that are deterministic on a 4-core hosted runner | every domain |
| `cpl --version` reporting channel and commit; `cpl self update` | tools |
| `web/out/` static export, `web/worker/wrangler.toml` | site |
| The VS Code `.vsix`, JetBrains plugin and Neovim/Vim setup builds | tools, editors |
| Repository secrets for deploys and signing | coordinator |

## Quality register

Each row is a quality a user (contestant, learner, editor user, library
author, contributor) can feel from infra's work, the metric, the strongest
mechanism that enforces it, and the check. `missing` means no real check yet.
Targets and today's numbers live in `plan/domains/infra/metrics.toml`.

### Landing and the commit tier

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Landing is fast | `land.sh` p50, p95, max seconds | Re-architecture: `land.sh` only rebases and pushes; gates run in CI | `scripts/land_stats.py` over the local landing log; `scripts/test_check_cache.py` `test_land_*` |
| Landing never waits on a person | landings needing a human step | Re-architecture: no review gate, retry on a rejected push | `scripts/test_check_cache.py` `test_land_retries_when_the_push_is_rejected` |
| Every commit names its domain | commits on main without `Domain:` | Gate: `land.sh` refuses | `scripts/test_check_cache.py` |
| Commit subjects follow the format | subjects outside `area: summary` | Gate: `commit-msg` hook | `.githooks/commit-msg`; **missing**: CI check for hook-skipped commits |
| The commit tier catches what the change can break | main reds in a test the commit tier would have selected | Generated selection from the cargo graph and data deps (`affected.py`) | `scripts/test_affected.py`; **missing**: every red traced to "selected or not" |
| The commit tier is cheap | pre-commit seconds p95 | Budget: 25 s, then warn and allow | `scripts/check.sh` logs slow commits locally; **missing**: aggregated per lane |
| Lanes spend little time on devex | devex commits per lane | Process: `devex_stats.py` proxy, check-in after each rollout | `scripts/devex_stats.py`, `scripts/test_devex_stats.py`; the proxy is a subject regex and over-counts |
| Commit velocity doesn't fall after an infra change | commits per 15 min before and after a rollout | Process: compare and revert on a drop | `scripts/devex_stats.py`; **missing**: an automatic alert |
| Nothing sensitive reaches public history | added lines with unlisted hosts, home paths, secrets | Gate: staged scan plus a scan of the commits since origin/main | `scripts/check.sh` staged scan; **missing**: CI scan of commits that skipped the hook |

### CI

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| CI answers fast | main run minutes, push to result p50 | Sharded gate, restored `target/`, `ci-mtime.sh`, C++ cache; non-blocking reports in `ci-slow.yml` | GitHub run times; **missing**: a budget that fails when a run exceeds 3 min |
| A hung job can't block the queue | job minutes over its cap | [ci/R28](../guidelines/ci-checklist.md#cir28): every job has `timeout-minutes` | `scripts/test_workflows.py` |
| Every main commit has a result | commits on main with no `ci/covered` status | [ci/R10](../guidelines/ci-checklist.md#cir10): `scripts/commit_status.py`, `.github/workflows/commit-status.yml` | built; first production run pending |
| CI results match a local run | reds that don't reproduce on a fresh `origin/main` build | Pinned toolchains, hermetic std path, deterministic tests | **missing**: a check that tests don't read wall-clock budgets in the gate |
| No flaky tests | tests that both pass and fail on the same commit | **missing**: rerun-on-fail detection and a quarantine list that only shrinks | none |
| A red names its culprit fast | minutes from red to named commit | Bisect of the failing tests over the run's range, then a verified revert (`culprit.yml`); direct log reading first | `scripts/test_culprit.py`; **missing**: a recorded red-to-culprit time per red |
| Main is usually green | % of time the newest finished run is green | Process: the sheriff fixes or reverts within 15 min | **missing**: a recorded metric |
| CI tests all three OSes | jobs per OS that block main | [ci/R29](../guidelines/ci-checklist.md#cir29): `cli`, `vscode` matrices on Linux, macOS, Windows; `os.yml` | `scripts/test_workflows.py` |
| Tests are never weakened to go green | removed or ignored tests per commit | Ratchet: `rust_ignore`, pending tests | `scripts/debt-ratchet.py` |
| Generated files stay current | stale generated files on main | CI `regen` job rewrites them; commit tier warns | `scripts/gen.py --check` |

### Releases and installers

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Edge ships often | hours since the last edge publish | Hourly timer plus CI-completion fallback, from the newest commit no worse than the last edge (ci/R26) | `scripts/release/shippable.sh`, `scripts/test_release.py` `ShippableTest`, `.github/workflows/release.yml`; `scripts/ci-watchdog.sh` alerts past 2 h (ci/R23) |
| Edge is never broken | edge releases that fail install smoke | Shipping checks before publish; install smoke after | `.github/workflows/release.yml` `install-smoke`; **missing**: smoke before marking latest |
| Stable is full green | stable releases from a commit with any red job | Gate in `.github/workflows/release.yml` stable path | `.github/workflows/release.yml`; never published without Ryan |
| One line installs on every OS | install failures per OS | Install smoke on Ubuntu, macOS, Windows after each publish | `.github/workflows/release.yml` `install-smoke` |
| Downloads are verified | archives installed without a matching sha256 | Installers check `SHA256SUMS` | `crates/cpl-cli/tests/toolchain.rs` |
| Releases are reproducible | archives that differ when rebuilt from the same commit | `SOURCE_DATE_EPOCH`, fixed archive order | `scripts/test_release.py`; **missing**: a rebuild-and-compare job |
| Releases are signed | archives without a signature | Sigstore build-provenance attestation per archive (`gh attestation verify`) | ci/R33, `scripts/test_workflows.py` |
| `cpl self update` moves to the newest release | update failures | Tested against local release fixtures | `crates/cpl-cli/tests/toolchain.rs`; **missing** against the live release |
| Editor assets ship and install | releases missing the `.vsix`, JetBrains zip, Neovim/Vim setup | `.github/workflows/release.yml` `editors` job | **missing**: install smoke for each editor asset |
| The binary says what it is | `cpl --version` without channel and commit | Stamped at build (`CPL_CHANNEL`, `CPL_COMMIT`) | `.github/workflows/release.yml` build smoke |
| The live site is current | minutes from a green main commit to cp-lang.org | `site.yml` deploys from the newest commit whose site jobs passed (ci/R25) | `scripts/site-deployable.sh`; `scripts/ci-watchdog.sh` alerts past 2 h (ci/R23), tested in `scripts/test_release.py` `WatchdogTest` |

### Contributors

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| A fresh clone reaches a green gate | minutes, undocumented steps | Process: cold-clone probe from `README.md` | **missing**: not scheduled |
| A gate failure says how to fix it | failures without the failing check and rerun command | `check.sh` `fail` messages name the command | ci/R31, `scripts/test_check_cache.py` |
| One command per task | tasks without a `just` name | `justfile` | none |

## Suites and tools

| Command | What it catches |
|---|---|
| `scripts/land.sh` | Lands HEAD on main; refuses a missing `Domain:` trailer |
| `scripts/check.sh --commit` | Tests the staged paths can affect (pre-commit) |
| `scripts/check.sh --fast` | The CI gate: fmt, clippy, nextest, conformance, generated files, ratchets |
| `scripts/check.sh --full` | Adds fuzzing, judge flags, wasm and editor smoke |
| `python3 scripts/affected.py <paths>` | Which tests a change selects |
| `python3 scripts/culprit.py` | Bisects failing tests over a commit range and verifies a revert |
| `python3 scripts/land_stats.py` | Landing p50/p95, commits per hour per domain |
| `python3 scripts/devex_stats.py` | Devex share per lane, velocity per 15 min |
| `scripts/release/shippable.sh` | The newest main commit no worse than the last edge on the shipping checks |
| `python3 -m unittest scripts/test_*.py` | The scripts' own tests |

## Changes

- 2026-11-23: first charter, folded from the infra plan's README and ARCHITECTURE.
