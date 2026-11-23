# infra architecture

The code as it is on 2026-11-22 (main `4b8f504f`), from reading it. Updated in
the same commit as any structural change; reviewed by architecture &
tech-debt.

## Modules

| Module | Path | What it is for |
|---|---|---|
| Fast/full gate | `scripts/check.sh` (376 lines) | The one gate. `--fast`: staged-secret and public-host scan, lockfile registry check, `cargo fmt --check`, clippy, nextest (minus 5 slow tests), doctests, prelude size, debt ratchet, taste reviews, site snippet gate (`web/scripts/gate.mjs`), `cpl-conform all --check`, recovery scoreboard. `--full` adds prover fuzz, judge flags, budgets, wasm smoke, VS Code and editor smoke. Writes step times and `target/gate-fast-time.json`. |
| Stage cache | `scripts/check_cache.py`, tests `scripts/test_check_cache.py` | Skips a gate stage when the hash of the whole tree minus that stage's excludes (plus tool versions and `CPL_*`/`CARGO_*` env) matches its last pass. Stages: `clippy`, `nextest_doc`, `recovery`, `scoreboard_fast`, `site_gate`. |
| C++ compile cache | `scripts/cxx-cache.py` (as a `g++` shim), tests `scripts/test_cxx_cache.py` | Reuses a binary built from the same C++ bytes, flags, compiler and system headers; per user, so worktrees start warm. |
| Gate env | `scripts/gate-env.sh` | Resolves the absolute cargo target dir for `check.sh`. |
| Landing | `scripts/land.sh` (33 lines), stats `scripts/land_stats.py` | Rebase onto main, push, rebase and push again on a rejected push; no gate (decisions.md 234). Refuses a commit without a `Domain:` trailer; appends `{domain, landed, seconds, tries}` to `.git/cpl-land.log`. CI on every main tip plus the infra sheriff keep main green. |
| Hooks | `.githooks/pre-commit`, `commit-msg`, `pre-push`; installer `scripts/install-hooks.sh` (`just hooks`) | pre-commit runs the commit tier (`check.sh --commit`: the crates the staged paths reach, via `scripts/affected.py` and nextest profile `commit`); commit-msg enforces `area: summary` from a fixed area list; pre-commit gives up after 25 s (warns, logs the step to `.git/cpl-commit-slow.log`, allows the commit); pre-push checks markers, JSON validity and a compile check. |
| Public-history hygiene | `scripts/public-hosts.txt`, `staged_guard` in `scripts/check.sh` | Added lines may link only listed hosts; no secrets patterns, home paths, or words in the local `.git/info/sensitive-words`. Scans the index and `origin/main...HEAD`. |
| Task runner and pins | `justfile`, `mise.toml`, `rust-toolchain.toml`, `pyproject.toml`/`uv.lock`, `package.json`/`pnpm-workspace.yaml` | One command name per task; one version pin per tool. |
| CI | `.github/workflows/ci.yml` | On push to main, pull_request, dispatch: job `gate` runs `scripts/check.sh --fast` on `ubuntu-latest`; job `site` builds `web/` with a release `cpl`, typechecks, unit-tests, checks the installers are in `web/out`. |
| Site deploy | `.github/workflows/site.yml` | On push to main: build `web/`, `wrangler deploy` of `web/worker` (static assets + share API), smoke `/ /docs/ /play/ /install.sh` on cp-lang.org. Needs Actions secrets `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`. |
| Release | `.github/workflows/release.yml`, `scripts/release/{plan.sh,build.sh,package.py,editors.py,manifest.py,publish.sh}`, tests `scripts/test_release.py` | Nightly 07:17 UTC schedule (edge) or dispatch (edge/stable, `publish` flag): plan names the version/tags, 5 targets build in parallel (musl via cargo-zigbuild) with `CPL_CHANNEL`/`CPL_COMMIT` stamped, reproducible archives, smoke on 3 native targets, editors (vsix, JetBrains zip, `editors.tar.gz`), publish writes `latest.json` + `SHA256SUMS` and `gh release create`. |
| Installers | `web/public/install.sh`, `web/public/install.ps1`; tested by `cpl-cli` `tests/toolchain.rs` | Pick the target, read `latest.json`, verify sha256, unpack into `~/.cpl/toolchains/<id>/`, launcher at `~/.cpl/bin/cpl`. Served by the site. |
| Wasm build | `scripts/build-wasm.sh` | Builds `cpl.wasm` + worker for the site and the `--full` smoke. |
| Other lanes' scripts kept here | `scripts/{review-check,dream-gaps,spec-stub,spec-vocab,spec-check.sh,conformance-coverage,corpus,corpus-tests,ports-page,writable-from-docs,debt-ratchet,next-decision,check-doc-links}.py` | Infra owns the directory and the gate wiring; the content belongs to the domain that uses each (idioms, spec, tech-debt, site). See debt. |

## Interfaces other domains use

| Interface | Contract | Consumers | Held by |
|---|---|---|---|
| `scripts/check.sh --fast` exit code | 0 = landable tree; prints step times | every domain, pre-commit (`--commit`), CI | `scripts/test_check_cache.py` (cache), CI |
| `scripts/land.sh [remote]` | lands HEAD on main within seconds or exits non-zero; refuses commits without `Domain:`; success line `land: pushed <sha> to <remote> main in <n>s (try <k>)` | every domain | `scripts/test_check_cache.py` (`test_land_*`) |
| Stage excludes in `scripts/check_cache.py` `STAGES` | a path excluded from a stage is never read by it (strace evidence) | domains adding inputs to a stage | `scripts/test_check_cache.py` |
| Commit subject areas (`.githooks/commit-msg`) | the area list in AGENTS.md | every domain | the hook |
| `scripts/public-hosts.txt` | the only hosts added lines may link | every domain | `staged_guard` |
| `just <task>` names | stable names: `check`, `check-full`, `test`, `conform`, `bench*`, `hooks`, `spec-check`, `debt-ratchet` ... | contributors, docs, other lanes' READMEs | none (debt) |
| Release assets + `latest.json` schema | `target -> {file, sha256}`, `toolchain`, `channel`, `commit`, `date` | installers, `cpl self` (tools) | `scripts/test_release.py`, `tests/toolchain.rs` |

## Invariants

- Every main tip gets a full `check.sh --fast` in CI (one running, one waiting); a red tip is the sheriff's (infra) to revert or fix. Commits are checked before landing only by the commit tier.
- A cached stage is skipped only when every input it can read is unchanged; an exclude needs strace evidence (`check_cache.py` header).
- No test is weakened or removed to make the gate faster; slow tests move to `--full`, never away.
- CI never uses `pull_request_target`; workflows default to `permissions: contents: read`, and only `release.yml`'s publish job gets `contents: write`.
- Release archives are reproducible from the commit (`SOURCE_DATE_EPOCH` = commit time).

## Known debt

- **Gate cache is all-or-nothing per stage.** Any change under `crates/` reruns all 1116 nextest tests (`check_cache.py` `STAGES`: `nextest_doc` excludes only prose). A no-change warm run is seconds, a one-line crate edit is the full ~70-80 s. This is the main obstacle to the 10 s target.
- **End-to-end tests sit in the commit gate.** Slowest in the fast gate (warm run, 2026-11-22): `cpl-cli::stress_corpus` 40 s, `toolchain` self-update/install 13-19 s, `lsp_parity` 15 s, `lsp-ports` 14 s, `judges` 13 s, `lsp fuzz` 13 s, `catalogue` 12 s, `seeded_bugs` 11 s.
- **CI is red and its failures are not local failures.** Run 45067797421 (4703550d): 6 failures, 3 of them emitted C++ using `std::exchange` without `<utility>` (passes on the VM's libstdc++, fails on the runner's), 3 timing-sensitive tests (`profile`, `stress ... TLE`) on a 4-vCPU runner. Codegen fix is perf's; the timing tests are a gate-design issue (here).
- **`check.sh` header is stale**: it says "There is no hosted CI on this repository" and "about 10 s warm"; CI exists and warm is 77 s after a one-line crate edit (metrics.toml).
- **`check.sh` is a hotspot**: 56 commits in 3 days (`git log --since=3.days -- scripts/check.sh`), mostly other lanes adding steps. No per-step registry, so every lane edits the same file.
- **`cargo fmt --all --check` is in both `check.sh` and `land.sh`**, while the repo rule says the tree is not globally rustfmt-clean; it passes today, so either the rule or the doc is stale.
- **`scripts/` mixes infra machinery with other domains' tools** (table above); ownership by directory hides who reviews what.
- **No CONTRIBUTING.md or SECURITY.md** at the root.
- **Site deploy has never succeeded in CI**: 5/5 `site.yml` runs failed on missing Actions secrets; cp-lang.org is live from a manual deploy.
- **0 releases published**; one `release.yml` dry run passed (run 45068359486, 3.5 min).
