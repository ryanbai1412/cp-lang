# idioms architecture

The code and data this domain owns, as they are on main (2026-11-22, read
from the tree). Updated in the same commit as any structural change.

Paths marked *(infra)* live under `scripts/`, which infra owns; idioms is
their main user and asks infra before changing them (see QUESTIONS.md).

## Modules

| Path | What it is for |
|---|---|
| `corpus/submissions/`, `submissions/info.json` | Raw accepted judge submissions (C++, Java, ...) and judge metadata. Never hand-edited. |
| `corpus/problems/<id>/` | One record per problem (1160): `meta.json`, `orig.cpp`, `port.cp`, `dream.cp`, `spec.cp`, `tests/`, `gen.py`, checkers. |
| `corpus/sets/ecnerwala.txt` | The 305 tracked ports: each has `port.cp`, `dream.cp`, `orig.cpp`; 286 have `tests/`; none has `spec.cp`. Their `port.cp` is pre-0.3 code. |
| `corpus/sets/sketches.txt` | 86 sketch problems. |
| `corpus/hillclimb.json` | The reproducible 100-problem v0.3 hill-climb set; each has `dream.cp` and `spec.cp`. This is what `cpl-conform corpus` runs. |
| `corpus/gold/` | 11 gold programs (`GOLD.txt` says why). |
| `corpus/lib/{real,scout}`, `corpus/cp.toml` | Libraries some ports import; `[libs]` points at `examples/local-lib`, which is gone with v0.2 (debt). |
| `corpus/index.json`, `scripts/corpus.py` *(infra)* | Generated index; `corpus.py build/check` regenerates and validates `meta.json`. |
| `research/dryrun/` | The 20-problem contest dry run: `pNN_*_ref.cpp`, `_gen.py`, `.tests`, checker/interactor. Top-level `pNN_*.cp` are v0.2 solutions; `v03/pNN_*.cp` are v0.3 (`_w` = workaround variant); `v03/loop.sh` and `verify.sh` run check, test, stress vs ref, then the codeforces C++ vs ref. `LOG.md`, `LOG-v03.md` are the run logs. |
| `research/reviews/<id>.md`, `baseline.json`, `scripts/review-check.py` *(infra)* | Cold taste reviews (port and dream score 1-5) and their ratchet, run in the fast gate (`review-check.py --check`). |
| `research/frictions/` | 173 friction reports (parsed by the gate: each `##` is a friction with Severity/Goal). |
| `research/design-history/EXEMPLARS-PORTS.md` | The gold bar the taste reviewer scores against (`plan/PORTING.md` cites it). |
| `spec/delta/` | Non-normative gap logs between dreams and the spec: `hillclimb.md` (every dream/spec diff), `parser-gaps.md` (forms `spec.cp` may fail on), `dream-comparisons.md`, `blind-dreams-first-ten.md`, `rebuild-*`, `0.2.md`. |
| `crates/cpl-judge` | Judge profiles (785 + 312 test lines, no deps): `Profile` (compiler, flags, OS, stack, thread stack, size limit), `builtins()`, `builtin(name)`, `named_in(source)` (`// judge:`), `resolve(path, source, chosen)` (flag > file > `cp.toml` > `generic`), `project_config`/`Config` (`[judge.<name>]` overrides and `extends`), flag probing with a memo (`CPL_JUDGE_PROBE_DIR`), `TARGET_BYTES` 32 KB / `HARD_BYTES` 64 KB. |
| `research/scoreboard.json` (corpus suite) | The committed corpus results `cpl-conform all --check` ratchets against. The crate `cpl-conform` runs it; its owner is not named in DOMAINS.md (see QUESTIONS.md). |
| `research/writable-from-docs*.{json,md}`, `scripts/writable-from-docs.py` *(infra)* | The docs-only eval: a cold model writes programs from the docs; last run 49/55 passed (`latest` in the JSON). Shared with site. |
| Skill (`skills/cp-lang/`, `scripts/skill-gen.py`) | **Not on main.** On `devin/skill-v2`: `SKILL.md` (123 lines, cap 160), topic pages (`idioms.md`, `io.md`, `domains-and-cuts.md`, ...), `stdlib/*.md` generated from `spec/std` doc comments, `skill-gen.py --check` (drift, front matter, every cp block checks). |
| Principles, idiom catalog | **Not on main.** On `devin/idioms-docs`: `spec/idioms/principles.md` (beliefs B1-B6, practices 1-17), `spec/idioms/lints.md` (catalogue). |

## Interfaces other domains use

| Interface | Consumers | Held by |
|---|---|---|
| `cpl_judge::{resolve, builtin, builtins, named_in, Profile, Os, project_config}` | tools (`cpl-cli` lib, profile, report; `cpl-contest`), lsp (`server.rs`), debugger (`cpl-stress/src/program.rs`), site (`cpl-wasm`) | `crates/cpl-judge/src/tests.rs`; `crates/cpl-cli/tests/judges.rs` (tools) |
| The corpus as a test set: `corpus/problems/*/{spec,port}.cp` and `tests/` | compiler and spec (`cpl-conform corpus`), tools (`judges.rs` `SPEC_PORTS`, 30 hill-climb ids; LSP `tests/ports.rs` floor), lint-fmt (`formats_the_whole_corpus` floor, lint false-positive gate), perf (bench) | corpus-size floors in those tests |
| `research/scoreboard.json` corpus cases | `judges.rs` `passing_corpus_ports()` (parses the JSON by hand), fast gate ratchet | `cpl-conform all --check` |
| Idiom catalog (future `spec/idioms/`) | lint-fmt (every idiom lint cites a practice), site (docs), skill | not yet |
| Skill (future `skills/cp-lang/`) | site (could publish it), users' agents | `skill-gen.py --check` (on branch) |

## Invariants

- A port sits beside its accepted original (`orig.cpp`); nobody edits `submissions/`.
- `dream.cp` is written without the checker and is never edited to pass it; `spec.cp` must pass `cpl check` except on forms in `spec/delta/parser-gaps.md`; every dream/spec difference is logged in `spec/delta/hillclimb.md`.
- Reviews score without reading `orig.cpp`; a port without a dream caps at 3.
- Judge resolution order: `--judge` > `// judge:` > `cp.toml` > `generic`; output is deterministic and under the profile's byte limit.
- Corpus test floors only fall with a deliberate commit that says why.

## Known debt

- The 305 tracked ports are pre-0.3 `port.cp`; review scores were given under 0.2 (`research/reviews/*.md`). 175 of 305 `port.cp` and 145 of 305 `dream.cp` pass `cpl check` on today's v0.3 default (metrics.toml `ports_check_v03`).
- `judges.rs` proves every judge only on 30 hill-climb `spec.cp` + 7 corpus passes; 0 of the 305 tracked ports are in either list.
- `judges.rs` `passing_corpus_ports()` parses `scoreboard.json` with string slicing (breaks on a format change); should read it through `cpl-json`.
- `corpus/cp.toml` `lib = "../examples/local-lib"` points at a deleted path.
- Two port generations side by side: `research/dryrun/pNN_*.cp` (v0.2) beside `v03/`; 877 `port.cp` files corpus-wide are v0.2 (deletion in flight, `children/v02-ports-delete.md`).
- Skill and idiom docs live only on branches, so main has no skill at all.
- Review, corpus and docs-eval scripts sit in infra's `scripts/` though idioms is their only writer.
