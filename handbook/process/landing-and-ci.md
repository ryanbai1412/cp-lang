# Landing and CI

| | |
|---|---|
| Kind | Process |
| Owner | infra |
| Status | Active |
| Describes | `scripts/land.sh`, `scripts/check.sh`, `.githooks/`, `.github/workflows/ci.yml`, `culprit.yml` |

How a commit gets onto main, what checks it, and what happens when main goes
red.

## Land a commit

```sh
scripts/install-hooks.sh        # once per clone
git commit -m "stdlib: add interval map" -m "Domain: stdlib"
scripts/land.sh
# land: pushed 1a2b3c4 to origin main in 7s (try 1)
```

- Every commit has a subject `area: summary` (the areas are in `AGENTS.md`)
  and a `Domain: <lane>` trailer. `land.sh` refuses a commit without one.
- `land.sh` rebases onto `origin/main` and pushes, retrying when someone
  else pushed first. It runs no tests: CI checks every main tip, and infra
  keeps main green. It takes about 5-10 s.
- Work lands straight on main. There are no pull requests and no review
  gate.

## What runs before a commit

The pre-commit hook runs `scripts/check.sh --commit`, the commit tier: only
what the staged paths can affect.

| You changed | It runs |
|---|---|
| A crate's `.rs` files | fmt, clippy and the tests of that crate and every crate depending on it |
| `std/`, `Cargo.toml`, toolchain files | every crate's tests |
| `corpus/problems/*/spec.cp` or `port.cp` | the corpus lint ratchet and `cpl check` on each touched file |
| `spec/conformance/*.cp` | those conformance cases |
| `handbook/` | the handbook checker |
| A script with a `scripts/test_<name>.py` | that test |
| Prose (`plan/`, `research/`, `decisions.md`) | nothing beyond the secrets scan |
| Anything else | the whole fast gate |

`python3 scripts/affected.py <paths>` prints the selection. The tier has a
25 s budget: past it the hook warns, logs the slow step to
`.git/cpl-commit-slow.log` and lets the commit through, since CI runs the
rest.

Every commit tier also scans added lines for secrets, home paths and
unlisted hosts (`scripts/public-hosts.txt`): the history is public.

## What CI runs

`ci.yml` runs on every push to main.

| Job | What it checks |
|---|---|
| `gate` (12 shards) | `scripts/check.sh --fast`: clippy, nextest split 6 ways by hash, conformance and interpreter-vs-C++ diff split 3 ways, corpus, docs, generated files, ratchets |
| `cli` (Linux, macOS, Windows) | `cpl-cli` end-to-end tests on each OS |
| `vscode`, `neovim`, `jetbrains`, `editors` | each editor integration against the real `cpl lsp` |
| `acceptance` (3 shards) | every committed acceptance solution still gets AC with the release build |
| `site build` | the site builds and every snippet on it runs |
| `regen` | rewrites stale generated files and commits them |
| `edge` | the published edge release is installable (not blocking) |

`ci-slow.yml` runs reports that never block main: wall-clock budgets and
LSP latency.

Run what CI runs with `scripts/check.sh --fast` (or `just check`);
`--full` adds fuzzing, judge flags and wasm and editor smoke.

### The queue

```text
push A ──► run A starts
push B ──► B waits
push C ──► C replaces B (B shows "cancelled", it never ran)
run A ends ──► run C starts, testing A..C's newest tip
```

A run that has started always finishes; a newer push never cancels it. At
most one run waits, so CI always tests the newest tip next. The commits in
between show "cancelled" on GitHub: they were never run, and the next run's
result covers them. Every long step has a `timeout-minutes` cap, so a hung
job ends instead of holding the queue.

## When main goes red

Infra is the sheriff. For each red run:

1. Read the failing job's log and the commits since the last green run;
   the failing test and the changed paths usually name the culprit within
   minutes.
2. If not, `culprit.yml` (it starts by itself after each red main run)
   bisects the failing tests over that range with `scripts/culprit.py`,
   checks that reverting the culprit makes them pass, and pushes a revert
   branch.
3. Fix forward when the fix is small and obvious. Otherwise tell the
   culprit's lane, with the failing test and its error; if main is still
   red 20 minutes later, land the verified revert and tell them.

Before reporting a red, reproduce it on a fresh `origin/main` build
(`git pull && cargo build`): a stale checkout or a `target/` built from
another checkout gives false reds.

Never weaken or delete a test to turn main green. A slow test moves to
`--full`, never away.

## Rules for infra changes

- No blocking pre-merge gate, and no new cost before a push.
- Landing stays under 30 s at p95.
- After any change to landing, hooks or CI, check with the lanes 5-15
  minutes later and compare commits per 15 minutes with the hour before
  (`python3 scripts/devex_stats.py`). If it hurts them, revert first and
  diagnose after.

## Changes

- 2026-11-23: first version, folded from the infra plan and the gate-tier
  design.
