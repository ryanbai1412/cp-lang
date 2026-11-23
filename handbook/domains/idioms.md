# Idioms

| | |
|---|---|
| Kind | Domain charter |
| Owner | idioms |
| Status | Active |
| Describes | `corpus/`, `crates/cpl-judge`, `skills/cp-lang/`, `spec/idioms/`, `research/acceptance/`, `research/dryrun/`, `research/reviews/`, `research/frictions/`, `research/skill-eval/`, `plan/domains/spec/delta/`, `handbook/process/porting.md` |

The idioms domain decides what good CP Lang looks like and proves it on
real problems. It keeps a corpus of accepted contest solutions rewritten in
CP Lang, the skill that teaches models to write CP Lang, and the acceptance
runs where a stranger tries to go from the website to accepted submissions.
If you want to know whether CP Lang actually works for contests, this
domain's evidence is the answer.

## What good looks like

Take a Codeforces problem. Its accepted C++ sits in
`corpus/problems/<id>/orig.cpp`. Beside it, `port.cp` solves the same
problem in CP Lang: it opens with a short statement and the editorial idea,
reads like that idea, uses about half the C++'s tokens, prints the same
answers on every test, and runs about as fast. Next to it, `dream.cp` shows
how the program would read if the language had everything we wish for, and
every difference between the two is a friction report sent to the team that
can remove it.

Now take a stranger who has never seen CP Lang. They install it from the
site, open a fresh contest in their own editor, and solve problems without
help. Whatever slows them down becomes a ranked blocker for its owner, and
every solution they get accepted is rebuilt and judged on every push, so a
fixed problem stays fixed.

## How it fits together

- **The corpus** is the evidence. Each problem folder holds the original,
  the port, the dream, tests whose answers come from an accepted solution,
  and a checker or interactor where the problem needs one. Every other
  domain uses it as a test set: the compiler for agreement between the
  interpreter and C++, lint for false positives, perf for benchmarks.
- **The rules for good code** live in
  `handbook/guidelines/idiomatic-code.md`; the rules for good problem
  folders live in `handbook/guidelines/problem-packages.md`. Porting
  follows `handbook/process/porting.md`, and every new port gets a cold
  taste review.
- **The skill** (`skills/cp-lang/`) and the idiom catalog (`spec/idioms/`)
  teach the same style to models and to lints.
- **Judge profiles** (`crates/cpl-judge`) say how each judge compiles and
  runs a submission, so local runs match it.
- **Acceptance runs** (`research/acceptance/`) put cold agents through the
  whole contest journey on each operating system, on problems they have
  never seen, and turn what they hit into `plan/product/BLOCKERS.md`.

## How we know it is good

Ports are checked on every commit, linted against a ratchet, and compiled
with every judge's flags. The corpus suite judges each hill-climb problem
by its tests or checker. Reviews score every port, and the count under 4/5
can only go down. Acceptance solutions are judged on every push.

The weakest areas today are agreement between the interpreter and the C++
across every port on every test, inputs that nobody validates against the
statement, ports that drop the statement's bounds, and a requirement change
staying local, which no suite measures yet. The register below lists every
quality with its check; rows marked `missing` are the work queue, in order
of user impact.

## Where to start

- **Port a problem:** read `handbook/process/porting.md`, then copy the
  layout of `corpus/problems/cf2147f/`.
- **Fix a corpus failure:** run `target/release/cpl-conform corpus <id>`
  and read the problem's `tests/`.
- **Run an acceptance round:** read `research/acceptance/PROTOCOL.md`.

# Reference

## Scope

**Owns:**

- The corpus: `corpus/problems/<id>/` (`meta.json`, `orig.cpp`, `port.cp`,
  `dream.cp`, `spec.cp`, `tests/`, `gen.py`, `checker.py`, interactors), the
  sets in `corpus/sets/` (`ecnerwala.txt`, the judge-proven `v03.txt`), the
  hill-climb set `corpus/hillclimb.json`, `corpus/newcomer/` (the newcomer
  lint set), `corpus/header.py`. Never `corpus/submissions/`, which holds raw
  accepted submissions and is never edited.
- Judge profiles: `crates/cpl-judge` (compiler, flags, OS, stack, size limit
  per judge, resolution order `--judge` > `// judge:` > `cp.toml` >
  `generic`).
- The bundled AI skill `skills/cp-lang/` and its principles and idiom
  catalog in `spec/idioms/` (non-normative: taste, not language rules).
- Acceptance runs: `research/acceptance/` (the protocol, local judges for
  held-out problems, `regress.py`, `blockers.py`, the run records) and the
  generated `plan/product/BLOCKERS.md`.
- Evals and reviews: `research/dryrun/`, `research/skill-eval/`,
  `research/reviews/` (cold taste reviews), `research/frictions/`,
  `research/PORTS.md` (generated), and the corpus suite's entries in
  `research/scoreboard.json`.
- Dream-versus-spec gap logs in `plan/domains/spec/delta/`.
- `handbook/process/porting.md`.

**Doesn't own:**

- Language rules and the conformance runner `crates/cpl-conform` (spec).
  Idioms is the corpus suite's main user and supplies its problems.
- Compiler and codegen bugs the corpus finds (compiler), stdlib APIs and
  their cost (stdlib), runtime and code size of emitted C++ (perf).
- Lint rules and their engine (lint-fmt). Idioms supplies the catalog the
  lints cite and the corpus they must stay quiet on.
- The site and its docs pages (site); `cpl test`, `cpl build` and the
  editors (tools); `cpl stress` (debugger).
- Scripts under `scripts/` such as `port-check.sh`, `review-check.py`,
  `skill-gen.py`, `corpus.py` and `ports-page.py` (infra keeps them running;
  idioms is their main user and writes most changes to them).

## Interfaces

**Provides:**

| Interface | Where | Used by |
|---|---|---|
| The corpus as a test set: `port.cp`, `spec.cp`, `tests/`, checkers | `corpus/problems/<id>/` | compiler and spec (`cpl-conform corpus`), tools (`crates/cpl-cli/tests/judges.rs`, LSP corpus tests), lint-fmt (false-positive gate, formatter corpus), perf (benchmarks), diagnostics (fix tests) |
| Judge profiles: `cpl_judge::{resolve, builtin, builtins, named_in, Profile, Os, project_config}` | `crates/cpl-judge/src/` | tools (`cpl build`, `cpl submit`, `cpl profile`), LSP, debugger (`cpl stress`), site (`cpl-wasm`) |
| The judge-proven port list | `corpus/sets/v03.txt` | `crates/cpl-cli/tests/judges.rs` |
| Accepted acceptance-run solutions, rebuilt and judged on every push | `research/acceptance/solutions/`, `research/acceptance/regress.py` | CI `acceptance` job |
| Ranked blockers per lane from the acceptance runs | `plan/product/BLOCKERS.md` | every domain |
| The idiom catalog and principles | `spec/idioms/` | lint-fmt (every taste lint cites a practice), site, the skill |
| The bundled skill | `skills/cp-lang/` | users' agents, site |
| Friction reports with a repro and an owner | `research/frictions/`, each domain's worklog | every domain |

**Needs:**

| From | What |
|---|---|
| spec | `cpl-conform corpus` judging each problem with its checker; conformance cases for every rule a port relies on |
| compiler | Ports that check also run, in `cpl run` and in the C++, with identical output |
| stdlib | The structures the editorial names (so ports need no hand-rolled copies), each with its cost documented |
| perf | Emitted C++ within 1.1x of the original, under 32 KB |
| lint-fmt | Lints that cite the idiom catalog and whose fixes compile |
| tools | `cpl test` judging by checker and interactor; `cpl build --judge` |
| site | Docs that a cold agent can write correct programs from |
| infra | CI for the acceptance job and the port-check ratchet; edge releases |

## Quality register

Each row is a quality a user of idioms' work could feel: a contestant
reading a port or the docs, a learner using the skill, a stranger following
the site, a library author, a contributor adding a port. The metric says how
high it is, and the mechanism is the strongest one that enforces it.
Mechanisms, strongest first: an architecture that makes the failure
impossible; a generated, fuzz, property, differential or metamorphic suite;
a hand-written suite; a lint, ratchet or gate; a process where no mechanism
can decide. `missing` means no real check exists yet. Rows are grouped by
the journeys in `handbook/product/journeys.md`. Targets and today's numbers
are in `plan/domains/idioms/metrics.toml`.

Where mature projects already measure a quality, the row follows them:
rustc's crater runs over every published crate (here: every port rebuilt on
every release), the Rust and Python API guidelines and their review
checklists (the taste checklist), Kotlin and Swift's idiom catalogs backed by
lints, the Benchmarks Game's matched-algorithm comparisons, Codeforces and
Polygon problem packages (generator, validator, checker, interactor), and
GCC and LLVM's differential testing against a reference.

### Discovering and learning (J1, J2)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| A model writes correct CP Lang from the skill alone | blind pass rate on a fresh unseen 20, first attempt, with no workaround | process: a cold agent per release; generated judging | `research/dryrun/v03/loop.sh`, `research/skill-eval/`; per-release unseen set: `missing` |
| A model or person writes correct code from the docs alone | writable-from-docs tasks passing | process: cold model; generated judging | `scripts/writable-from-docs.py`, `research/writable-from-docs.json` (shared with site) |
| Every skill example compiles and is current | skill `cp` blocks passing `cpl check`; generated stdlib pages in sync | generated suite | `scripts/skill-gen.py --check` in `scripts/check.sh` |
| The skill teaches the normative spelling | skill and corpus lines using a removed spelling (`some(...)`, `none`, `.enumerate()`, `assume`) | `handbook/guidelines/idiomatic-code-checklist.md#r8`; `missing` (a generated scan against the spec's removed-spelling list) | `missing` |
| Showcase ports read like the editorial | taste-review median and count under 3 and under 4 | `handbook/guidelines/idiomatic-code-checklist.md#r2`; process: cold review per port; ratchet on the count under 4 | `scripts/review-check.py --check` |
| The idiom catalog covers what newcomers write | marked newcomer spots a lint flags | generated suite | `corpus/newcomer/score.py`, `crates/cpl-cli/tests/newcomer.rs` |
| Every idiom in the catalog has an example that checks and a lint or a reason | catalog entries with a checked example and a linked lint | `missing` | `missing` |

### A stranger's whole contest attempt (J1 to J9)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| A stranger installs, solves and submits with only the site | steps passing per OS and persona; findings per accepted problem | process: acceptance run per edge, one agent per OS | `research/acceptance/PROTOCOL.md`, `research/acceptance/runs/` |
| Unseen problems, not memorised ones | held-out problems newer than the corpus, fresh per run | process | `research/acceptance/problems/sets/` |
| Every blocker reaches its owner, ranked | open rows by kind, frequency, minutes lost | generated report | `research/acceptance/blockers.py` (in `gen.toml`), `research/acceptance/send_rows.py` |
| A fixed acceptance bug stays fixed | accepted run solutions rebuilt and judged | generated suite on every push | CI `acceptance` job (`research/acceptance/regress.py`) |
| Drafts for open blockers become regression tests once fixed | open repros with a named blocker row | process | `research/acceptance/open-repros/`; automatic promotion when the row closes: `missing` |
| Local judges are right | held-out references accepted on samples, generated cases and the max input; brute or checker agreement | differential (reference against brute) | `research/acceptance/judge.py` |
| Acceptance covers Linux, macOS and Windows | OSes running install and the CLI steps | CI on each OS | `.github/workflows/acceptance.yml` (`install-cli`) |
| Editors work for a stranger | the five LSP features shown per editor per OS | process inside the run; tools owns the smoke | acceptance run records; tools' editor jobs |
| The run measures quality, not wall-clock | findings per accepted problem, hesitations, tool round-trips | process | `research/acceptance/score_solutions.py`; round-trip counting: `missing` |

### Writing and revising (J5)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Solutions are short | contestant tokens, median and max against the C++ | generated over every port | `scripts/ports-page.py --tokens` |
| A change of requirement stays local | chunks touched when one requirement changes (end position, extra DP dimension, map for array) | process: the revision probe | `missing` as a suite |
| Ports don't hand-roll what std or the language provides | ports flagged by the hand-rolled-std lint family | `handbook/guidelines/idiomatic-code-checklist.md#r3`; lint, ratchet | `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`, run in the commit tier for staged ports) |
| Ports keep the statement's constraints as types | ports whose read types match the statement's bounds; max-bound input runs | `handbook/guidelines/idiomatic-code-checklist.md#r5`; `missing` (bounds in `meta.json` and a generated max-bound run) | `plan/domains/idioms/header-audit.md` (manual work list) |
| Every port has the standard header (Source, Problem, Solution) | ports with all three | `handbook/process/porting.md#the-header`; generated count | `corpus/header.py` |
| Every port has a dream and a review | ports newer than the baseline without either | gate | `scripts/review-check.py --check` |
| Every gap between a port and its dream is reported with a repro and an owner | frictions with severity, goal and a run showing the failure | `handbook/guidelines/idiomatic-code-checklist.md#r11`; gate on format; process on content | `research/frictions/` (parsed in the gate) |
| Dream and spec differences are logged | `dream.cp` and `spec.cp` diffs without a log entry | `missing` (a generated diff against `plan/domains/spec/delta/hillclimb.md`) | `missing` |

### Correctness of the corpus (J6, J7)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Every port checks on the current language | ports failing `cpl check` | ratchet, list only shrinks | `scripts/port-check.sh` (`corpus/port-check-failing.txt`) in `scripts/check.sh`; per staged port in the commit tier |
| Every port agrees with its accepted C++ | ports matching `orig.cpp` on samples plus generated inputs | differential | `crates/cpl-cli/tests/judges.rs` (`every_v03_port_checks`) over `corpus/sets/v03.txt`; generated-input agreement on every commit: `missing` (done by hand per port today) |
| The hill-climb problems pass | `cpl-conform corpus` passes, judged by checker where one exists | generated suite, ratchet | `cpl-conform corpus`, `research/scoreboard.json` via `cpl-conform all --check` |
| Every problem has tests, and expected outputs come from an accepted solution | untested problems; expected outputs regenerated from `orig.cpp` | `handbook/guidelines/problem-packages-checklist.md#r1`; generated | `scripts/corpus-tests.py verify`; `corpus/audit.py` (`answer-is-input`, `no-reference`, ratchet `corpus/audit-known.txt`) |
| Multi-answer problems are judged by a checker | problems with several valid answers and no checker | `handbook/guidelines/problem-packages-checklist.md#r3`; differential: two accepted submissions that differ on a test | `corpus/audit.py` (`needs-checker`); `cpl-conform corpus` judges by `checker.py` |
| Interactive problems run with an interactor | interactive ports with an interactor and generator | `handbook/guidelines/problem-packages-checklist.md#r4`; hand-written per problem | `corpus/problems/<id>/interactor.py`; `corpus/audit.py` (`interactor-no-gen`) |
| Generators produce valid inputs | generated inputs accepted by a validator | `handbook/guidelines/problem-packages-checklist.md#r5`; `missing` (a validator per problem, as Polygon requires) | `missing` |
| The interpreter and the C++ agree on every port | ports where `cpl run` and the built C++ differ on any test | differential | `missing` across the whole corpus (compiler owns the mechanism; idioms owns running it on the corpus) |
| Corpus edits keep working ports working | ports whose verdicts change in a commit that edits them | differential before and after | commit-tier `cpl check` and lint ratchet on staged ports; verdict diff: `missing` |

### Performance and size (J8, J9)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Idiomatic ports run as fast as matched C++ | p50, p95 and max runtime ratio on max inputs | `handbook/guidelines/idiomatic-code-checklist.md#r10`; `handbook/guidelines/performance.md`; generated benchmark | perf's corpus bench; per-port max input for every tracked port: `missing` |
| Every port can be tuned to 1.1x in the language | tuned ports at or under 1.1x | process | `missing` |
| Every built solution is under 32 KB | Codeforces builds over 32 KB; none over 64 KB | generated over every port | `crates/cpl-cli/tests/judges.rs` (64 KB hard limit); 32 KB count: `missing` as a ratchet |
| A built solution compiles under each judge's flags | ports failing any judge's compiler and flags | generated matrix | `crates/cpl-cli/tests/judges.rs` (`every_port_compiles_with_every_judges_flags`) |
| What passed locally passes on the judge | ports whose local verdict differs from the judge's | process: a sample of real submissions, by Ryan only | `missing` |

### Judge profiles (J9)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Each profile matches its judge's compiler, flags, stack and size limit | fields cited to the judge's published settings | hand-written suite | `crates/cpl-judge/src/tests.rs` |
| Resolution order is the documented one | cases per source (flag, `// judge:`, `cp.toml`, default) | hand-written suite | `crates/cpl-judge/src/tests.rs` |
| Local runs mimic the profile (stack, widths) | profiles whose stack and integer widths a test exercises | generated matrix | `crates/cpl-cli/tests/judges.rs` |
| A deep recursion that fits the judge's stack fits locally | ports with deep recursion passing under each profile's stack | `missing` | `missing` |

### Long-term use and contributing (J10)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Old ports rebuild after library changes | ports broken by a change, migrated in the same change | ratchet; codemods from stdlib and diagnostics | `scripts/port-check.sh`; byte-for-byte rebuild across releases: `missing` |
| Adding a port is cheap and the bar is clear | porting checklist; time per port | process | `handbook/process/porting.md` |
| Concurrent corpus work can't turn main red | staged ports checked and linted before landing | gate | commit tier (`scripts/check.sh`) |
| One problem-package format for corpus and acceptance | tools that read one format (gen, brute, checker, interactor, limits) | `handbook/guidelines/problem-packages-checklist.md#r8`; `missing` (architecture: shared with `cpl test` and `cpl stress`) | `missing` |
| Generated reports are current | stale `research/PORTS.md`, `plan/STATUS.md`, `BLOCKERS.md` | generated, checked in the gate | `scripts/gen.py` |
| Nothing private enters the corpus | home paths, session links, unknown hosts | gate | pre-commit public-history scan |

## Suites and tools

| Run | What it catches |
|---|---|
| `target/release/cpl-conform corpus` | a hill-climb problem failing its tests or its checker |
| `scripts/port-check.sh target/debug/cpl` | a port that stopped checking |
| `cargo test --release -p cpl-cli --test judges -- --include-ignored every_` | a judge-proven port failing its tests, its judge's flags or the size limit |
| `cargo test -q -p cpl-lint corpus` | new lint warnings on the corpus |
| `python3 research/acceptance/regress.py target/release/cpl -j 4` | a once-accepted acceptance solution no longer accepted |
| `python3 research/acceptance/judge.py <problem> <solution>` | a held-out solution failing samples, generated cases or the max input |
| `python3 research/acceptance/blockers.py` | regenerates `plan/product/BLOCKERS.md` |
| `python3 scripts/review-check.py --check` | a port without a dream or review, or too many under 4 |
| `uv run python scripts/skill-gen.py --check --cpl target/debug/cpl` | a skill example that no longer checks |
| `uv run python scripts/ports-page.py --tokens` | token ratios against the C++ |
| `uv run python corpus/newcomer/score.py --fixes` | newcomer spots no lint flags, or fixes that break |
| `python3 corpus/header.py` | ports without the standard header |
| `research/dryrun/v03/loop.sh <problem> <cases>` | a dry-run solution failing check, test, stress or the C++ build |

## Invariants

- A port sits beside its accepted original; nobody edits `corpus/submissions/`.
- `dream.cp` is written without the checker and never edited to pass it.
  `spec.cp` passes `cpl check`, except on forms listed in
  `plan/domains/spec/delta/parser-gaps.md`, and every difference between a
  dream and its spec is logged in `plan/domains/spec/delta/hillclimb.md`.
- Reviewers score without reading `orig.cpp`; a port without a dream scores
  at most 3.
- Judge resolution is `--judge`, then `// judge:`, then `cp.toml`, then
  `generic`; the emitted C++ is deterministic and under the profile's byte
  limit.
- Corpus floors and ratchets used by other domains' tests only fall in a
  commit that says why.

## Judgment rubric

Numbers miss taste, so reviews and probes ask five questions:

1. **Says the idea.** Does a reader who knows the editorial find each step
   named in the code, in its words?
2. **Every fact once.** Does each constraint, index space and absence appear
   once, as a type or a name?
3. **No hacks.** Did the writer route around the language where a general
   mechanism should exist?
4. **Revises locally.** When one requirement changes, does the edit stay in
   one chunk?
5. **Trust.** Would the contestant submit it without re-reading the C++?

The probes, each run by a cold agent with only the skill: solve a fresh
set of problems (1-3); port random tracked problems from `orig.cpp` without
seeing `port.cp`, scored against `research/design-history/EXEMPLARS-PORTS.md`
(1, 2, 5); solve an unseen problem, then change one requirement and count
the chunks touched (4); find a planted off-by-one with stress and trace
(3, 5).

## Changes

- 2026-11-23: first version, folding in the stable parts of
  `plan/domains/idioms/README.md` and `plan/domains/idioms/ARCHITECTURE.md`.
- 2026-11-23: mechanism cells cite the rule in `handbook/guidelines/idiomatic-code.md` or `problem-packages.md`.
- 2026-11-23: prose overview first, the tables under `# Reference`; cells cite checklist ids.
- 2026-11-23: the package rows name `corpus/audit.py`.
- 2026-11-23: invariants and the judgment rubric folded in from the idioms plan folder.
