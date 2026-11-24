# Spec domain

| | |
|---|---|
| Kind | Domain charter |
| Owner | spec |
| Describes | `spec/core/`, `spec/lib/`, `spec/tools/`, `spec/conformance/`, `spec/SUBSET.md`, `spec/GLOSSARY.md`, `handbook/language/`, `crates/cpl-conform/`, `scripts/conformance-*.py`, `scripts/spec-vocab.py`, `scripts/spec-check.sh`, `scripts/spec-stub.py`, `scripts/next-decision.py` |

The spec domain says exactly what CP Lang is, explains why it is shaped
that way, and holds the compiler to it. A contestant reading a chapter, a
model writing a solution, and an engineer building a second compiler should
all get the same answer to "does this compile, what does it print, where
does it trap?". When the implementation gets one of those wrong, a case of
ours should fail before a contestant notices.

## What good looks like

A reader who knows Python or C++ reads `handbook/language/sequences.md` and
learns that a reversed view walks backwards but keeps its source's keys.
They write `a.rev.drop(1)` over `[10, 20, 30]` and expect `20 10`. The
spec chapter says the same thing in one numbered rule. A conformance case
pins that answer, so a compiler that prints `30 20` fails the case and the
release waits for the fix. When a new contestant hits a rule the spec never
covered, the gap becomes a rule and a case the same day, and the next
reader doesn't hit it.

## How it fits together

There are three layers. `handbook/language/` gives the picture: the
problem each area solves, the model to hold in your head, and the decisions
behind it. `spec/` is the exhaustive layer: every rule with an id, every
std signature, every tool command. `spec/conformance/` is the evidence: about
2300 small programs, each citing the rules it tests and stating the one
result the spec promises.

`cpl-conform` runs the cases against the checker, the interpreter and the
generated C++, along with the spec's own code blocks and the corpus. Scripts
around it keep the spec honest. They list rules with no case, chapter
signatures that disagree with `std`, and gaps the Milestone 1 subset has
promised to close. A generator writes random programs, computes their output
in Python, and compares both engines against that answer.

## How we know it is good

Three kinds of check. Coverage: every rule should have a case, and today
most do. Agreement: the interpreter and the C++ must give the same output
on every case, and the generator catches wrong answers both engines share.
Independent readers: fresh models rebuild contest solutions from the spec
alone, and each place they had to guess becomes a fix. The weakest areas
today are combinations of features that no table enumerates yet, mistakes
in the cases themselves (a wrong expected line hides a real bug), and the
rows marked `missing` in the register below.

## Where to start

- To change a rule, read `guidelines/spec-rules.md`, then the area's
  `handbook/language/` doc and the spec chapter it leads.
- To add a case, read `guidelines/conformance.md` and
  `spec/conformance/README.md`.
- To see what is open, read `spec/SUBSET.md` (known gaps and their owners)
  and the `missing` rows below.

# Reference

## Scope

| Owns | Does not own (owner) |
|---|---|
| `spec/core/` chapters and their rule ids `(Rnn.n.n)` | `std/*.cp` signatures and bodies (stdlib) |
| `spec/lib/` library design chapters, `spec/tools/` tool contracts | `spec/diagnostics.md`, the error catalogue (diagnostics) |
| `spec/conformance/`: the cases, `README.md` (the case format), the generators' data, `COVERAGE.md`, the gap lists (`rule-gaps.txt`, `sig-gaps.txt`, `prelude-gaps.txt`, `permits.txt`) | Making failing cases pass (compiler, perf, stdlib, tools) |
| `spec/SUBSET.md`: which sections are supported, documented or a known gap, with an owner per failing case | `handbook/language/std.md` (stdlib) |
| `handbook/language/` design docs, except `std.md` | The gate script and CI (infra) |
| `crates/cpl-conform` (the runner: `conformance`, `spec-blocks`, `corpus`, `docs`, `diff`) | The corpus programs and scoreboard (idioms) |
| `scripts/conformance-*.py`, `spec-vocab.py`, `spec-check.sh`, `spec-stub.py`, `next-decision.py` | Site chapters built from the language docs (site) |
| Settling spec ambiguities, logged in `decisions.md` | |

Spec writes cases, never compiler code. A failing case goes to the owning
domain with its path.

## Interfaces

**Provides**

| Interface | Used by | Held by |
|---|---|---|
| Authority: handbook leads design; spec/ is its downstream exhaustive form. A handbook/spec disagreement is a bug spec resolves; every other doc yields to both | every domain | this charter; `spec-vocab.py` |
| Rule ids `Rnn.n.n` | compiler, diagnostics (`spec/diagnostics.md` cites them), site, idioms | `cpl-conform --rules`, `conformance-coverage.py --check` |
| The conformance case format (`// expect:`, `// rule:`, `// input:`, `// timeout:`, `// has:` headers; a code's line ends with `// <- here`) | compiler, stdlib (`std-*` groups), tools, diagnostics | `crates/cpl-conform/tests/ratchet.rs`, `spec/conformance/README.md` |
| The pass list in `research/scoreboard.json` | compiler (its headline number), infra (the `cpl-conform ratchet` gate step) | `cpl-conform all --check` |
| `spec/SUBSET.md` known-gap list with owners | every implementing domain, the milestone owner | `conformance-subset.py --check` |
| `spec/GLOSSARY.md`: one term per concept | diagnostics, site, idioms | `spec-vocab.py` |
| `cpl-conform corpus` judging, including `checker.py` for problems with several right answers | idioms | `crates/cpl-conform` tests |
| Design docs in `handbook/language/` | site (in-depth chapters), new contributors | `scripts/check-handbook.py` |

**Needs**

| From | What |
|---|---|
| compiler, perf | A build of `cpl` whose interpreter and generated C++ run every case |
| stdlib | `std` signatures that match the chapters; the prelude exporting every type a prelude signature names |
| diagnostics | Stable error codes, so `// expect: E0302` keeps meaning the same thing |
| infra | The gate runs the ratchets listed below |

## Quality register

Each row is a quality the spec must be high on, the number that measures it,
and the strongest mechanism that enforces it. Mechanisms are ranked: a
design that makes the failure impossible, then a generated, fuzzed,
differential or mutation suite, then a hand-written suite, then a lint,
ratchet or gate, then a process. `missing` marks a row with no real check
yet. Targets and current values are in `plan/domains/spec/metrics.toml`.

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Every rule is tested ([conformance/R1](../guidelines/conformance-checklist.md)) | rules with no case; forbidding rules with no reject case | ratchet in the gate | `scripts/conformance-coverage.py --check`, `spec/conformance/rule-gaps.txt` |
| Every case cites something that exists | cites to unknown rules | gate | `cpl-conform --rules` |
| Every diagnostic code is reachable | codes with no reject case | ratchet | `scripts/conformance-coverage.py --check` |
| Feature combinations are tested (index types x consumers) ([conformance/R5](../guidelines/conformance-checklist.md)) | empty matrix cells | generated suite, gate | `scripts/conformance-matrix.py --check`, `spec/conformance/matrix.toml` |
| Every trait method on every claimer is tested ([conformance/R5](../guidelines/conformance-checklist.md)) | trait methods with no case | generated table, ratchet | `scripts/conformance-traits.py --check` |
| Chapters and `std` agree on named arguments | named args no std fn takes | ratchet | `scripts/conformance-sigs.py --check`, `sig-gaps.txt` |
| What the chapters call is in the prelude | chapter functions missing from the prelude | ratchet | `scripts/conformance-prelude.py --check`, `prelude-gaps.txt` |
| Chapters and `std` agree on return types and existence (e.g. a chapter naming a type `std` doesn't have) | chapter fns whose return type differs from std's; chapter fns std lacks | gate on top-level fns ([spec-rules/R5](../guidelines/spec-rules-checklist.md)); prelude and stub checks for existence; methods compared only by rebuild rounds | `scripts/spec-std-signatures.py --check`, `scripts/conformance-prelude.py`, `scripts/spec-stub.py` |
| The suite catches compiler bugs | share of reverted compiler fixes the suite turns red | mutation suite over history | `scripts/conformance-bugcatch.py`, `plan/domains/spec/bugcatch.md` |
| The suite catches bugs no past fix shows ([conformance/R9](../guidelines/conformance-checklist.md)) | share of one-line compiler mutants the suite catches, among those that build | seeded mutants of compiler `src/`, each run against the conformance suite; each survivor gets a case or is shown equivalent | `scripts/conformance-mutants.py` in `ci-slow.yml` (`mutants`), `plan/domains/spec/mutants.md` |
| Interpreter and C++ agree ([conformance/R6](../guidelines/conformance-checklist.md)) | cases whose output differs between `cpl run` and the built C++ | differential suite | `cpl-conform diff` |
| Interpreter and C++ agree on programs nobody wrote by hand | divergences on generated programs | generated differential suite against an independent Python model ([conformance/R6](../guidelines/conformance-checklist.md)) | `scripts/gen-programs.py`, `scripts/check.sh --full` |
| Generated C++ has no undefined behaviour ([conformance/R6](../guidelines/conformance-checklist.md)) | cases failing under ASan, UBSan and `_GLIBCXX_DEBUG` | generated differential suite: every generated program also runs under ASan, UBSan and `_GLIBCXX_DEBUG`; hand cases through `CPL_CONFORM_CXXFLAGS=... cpl-conform diff` | `scripts/gen-programs.py`, `scripts/check.sh --full` |
| Every case is itself right | wrong cases found by implementers | gate for new cases; known-gap cases checked for a wrong `// <- here` line or an unwrapped `?T` ([conformance/R3](../guidelines/conformance-checklist.md)) | `scripts/conform_staged.py`, `scripts/conformance-gaps.py` |
| Code examples in the spec run ([conformance/R7](../guidelines/conformance-checklist.md)) | spec blocks failing that are not `spec-only` | ratchet | `cpl-conform spec-blocks --check` |
| Milestone gaps are visible, not hidden | failing cases in a supported section with no owner | gate | `scripts/conformance-subset.py --check`, `spec/SUBSET.md` |
| One spelling per idea | banned spellings in spec | lint | `scripts/spec-vocab.py` |
| Each decision is recorded once | duplicate decision numbers (`decision_dup_numbers`, hard zero) | gate: a number is reserved on main first | `scripts/next-decision.py --reserve`, `--check-landing` in `scripts/land.sh` |
| Cases test the language, not the compiler's internals | cases that reach crate internals | design: a case is a `.cp` file run through `cpl` | `crates/cpl-conform/src/conformance.rs` |
| A cold reader can write correct code from `spec/` alone | holes and contradictions per cold rebuild round | process: rebuild rounds | `research/rebuild-rounds/` |
| Handbook language docs and the spec say the same thing ([spec-rules/R6](../guidelines/spec-rules-checklist.md)) | handbook `cp` blocks the compiler rejects, or that a `rejects:` marker says it should reject and it accepts | ratchet: every `cp` block in `handbook/language/` runs in the spec-blocks suite with the spec's own blocks; prose claims beyond code are reviewed when a rule changes | `cpl-conform spec-blocks --check`, `crates/cpl-conform/src/spec_blocks.rs` |
| Handbook docs are written for an outsider | context-dependent phrases, hashes | lint | `scripts/check-handbook.py` |
| Each rule earns what a contestant must remember | rubric score from a cold reader | process: review against the design rules in `AGENTS.md` | — |
| A behaviour the user can observe has exactly one specified answer | rules whose text a cold reader had to guess (silent holes per round) | process: rebuild rounds; `missing`: a check that each `std` function a chapter names has a cost, a trap list and an example | `research/rebuild-rounds/` |
| Generated programs agree across engines (the Csmith and YARPGen approach) ([conformance/R6](../guidelines/conformance-checklist.md)) | divergences between `cpl run`, `cpl build` with `-O2`, and a sanitized build, on random well-typed programs | generated suite with a Python model of each program's output: catches wrong answers both engines share, shrinks failures by dropping statements | `scripts/gen-programs.py` (views, folds, Euclidean `/` `%`, bit operators, in-place verbs, loops, closures writing captures; `--cxxflags` adds a sanitized leg) |
| A meaning-preserving edit never changes the output (equivalence-modulo-inputs testing) | cases whose output changes after renaming, reordering independent `let`s, inlining a `let`, wrapping in a `fn`, or `cpl fmt` | metamorphic suite: every passing output case rerun after `cpl fmt`, CRLF line endings, blank lines and comments between lines ([conformance/R6](../guidelines/conformance-checklist.md)); renaming, reordering and wrapping in a `fn` are not transforms yet | `scripts/conformance-metamorphic.py`, `scripts/check.sh --full` |
| Checked and release builds agree on valid programs | cases whose output differs between the checked build and the release build | generated differential suite: every generated program also runs built with `--checks all` ([conformance/R6](../guidelines/conformance-checklist.md)); hand cases under `core-09/release-*` | `scripts/gen-programs.py`, `spec/conformance/core-09/` |
| Every syntactically valid program parses, every invalid one gets one error ([conformance/R5](../guidelines/conformance-checklist.md)) | crashes on programs derived from the EBNF; derived programs the parser rejects (each a chapter/parser disagreement); grammar alternatives no parsing program reaches | generated suite: 300 programs derived from the chapter's own EBNF, no crash allowed, syntax rejections shrink-only; mutated corpus programs never crash | `scripts/grammar-fuzz.py --check` in `scripts/check.sh --full`, `crates/cpl-cli/tests/front_end_fuzz.rs` |
| Integer widths and overflow are right at every boundary | range types x operations without a boundary case (min, max, max+1, products near \(10^{18}\)) | generated matrices: index types, and `Int` `+ - * / %` (expression and compound) and unary `-` over boundary operands, checked and unchecked ([conformance/R5](../guidelines/conformance-checklist.md)); range-typed stores, `U64` and `I128` not generated yet | `scripts/conformance-matrix.py`, `scripts/conformance-widths.py`, `spec/conformance/widths/` |
| Large output is complete and ordered (over 64 KiB, mixed `print` forms, multitest) | cases over 64 KiB passing in both engines | hand suite | `spec/conformance/core-10/` |
| Interactive I/O flushes correctly | interactive cases passing under an interactor in both engines | hand suite | `spec/conformance/core-06/interactor-send-recv.cp`, `// tool:` cases |
| Ownership and lending rules accept every safe program and reject every unsafe one ([conformance/R5](../guidelines/conformance-checklist.md)) | false E06xx on safe programs found per rebuild round; sanitizer failures on accepted programs; generated keep-a-view cases failing | hand suite plus sanitized `cpl-conform diff`; generated suite: every source (named, temporary, lent param) × storage (`let`, `var`, push, assign, `mut` param) × order × scope, verdicts from R05.7.8–10 and R07.5.10 | `spec/conformance/core-05/`, `scripts/conformance-ownership.py` → `spec/conformance/ownership/` |
| What `cpl explain` claims about copies and allocations is true ([conformance/R6](../guidelines/conformance-checklist.md)) | cases where explain lists more copies than the judge C++ makes (373 of 416 at first run) | hand cases with `// tool: explain` and `// lacks:`; over every `ok` case, explain's copy list against the copies counted in the C++, shrink-only | `scripts/explain-copies.py --check --max` in `scripts/check.sh --full`, `spec/conformance/core-05/` |
| Tool behaviour a journey relies on is specified and tested (`check`, `run`, `test`, `stress`, `build`, `fmt`, `lint`, LSP) | journey steps with no rule id and case | hand suite with `// tool:` cases | `spec/tools/`, `spec/conformance/` |
| Trace, debugger and explain output is specified ([spec-rules/R8](../guidelines/spec-rules-checklist.md)) | tool features in `spec/tools/` with no rule id (28 of 28 at first run) | every top-level feature bullet in `spec/tools/` carries an `(Rnn.n.n)` id, shrink-only | `scripts/spec-tool-rules.py --max` in `scripts/check.sh` |
| The same source behaves the same on Linux, macOS and Windows (CRLF source and input, paths) | platform cases passing on all three | CRLF source checked for every passing case by the metamorphic suite; crate tests on macOS and Windows runners; the conformance suite itself runs on Linux only | `scripts/conformance-metamorphic.py`, `.github/workflows/os.yml`, `spec/conformance/core-01/` |
| Known gaps get fixed, not just listed | known-gap cases; age of the oldest; entries in `sig-gaps.txt` and `prelude-gaps.txt` | ratchet: lists only shrink; age report per owner ([conformance/R3](../guidelines/conformance-checklist.md)) | `scripts/conformance-gap-age.py`, `spec/SUBSET.md`, `spec/conformance/*-gaps.txt` |
| A known-gap case fails for the reason it names ([conformance/R3](../guidelines/conformance-checklist.md)) | known-gap cases whose current error isn't the gap's subject (a wrong `// <- here` line, a misused API) | check: a failure with the case's own expected code means a wrong `// <- here` line or `has:`; an E0606 on a printed `?T` means a missing `!` | `scripts/conformance-gaps.py` |
| The suite is deterministic | cases that pass and fail on the same build | per-case `// timeout:`; flake report over repeated runs ([conformance/R6](../guidelines/conformance-checklist.md)) | `scripts/conformance-flakes.py`, `scripts/check.sh --full`, `crates/cpl-conform/tests/timeout.rs` |
| Cross-references in the spec resolve | rule ids anywhere in the repo naming no rule; links to missing files | gate ([spec-rules/R3](../guidelines/spec-rules-checklist.md)) | `scripts/spec-xref.py --check`, `scripts/check-doc-links.py` |
| The spec says what changed between versions ([spec-rules/R7](../guidelines/spec-rules-checklist.md)) | removed spellings (E0503) with no changelog line | every E0503 spelling in `spec/diagnostics.md` is named under the current version in `CHANGELOG.md` | `scripts/spec-changelog.py` in `scripts/check.sh` |
| Old programs keep their meaning (pins and `cpl migrate`, R11.4.3) | R11.4.x cases failing; `cpl migrate` cases whose rewrite differs | cases for each of R11.4.1–3: the stamp, a build with a pinned compiler, a removed spelling's E0503, and `cpl migrate` rewriting it (`// tool: migrate`, `// rewritten:`), with rule coverage from `conformance-coverage.py` | `spec/conformance/core-11/` |
| Each chapter teaches its model first | supported chapters with no model paragraph | process | `spec/core/` |
| Every design decision gives its reason and the alternatives that lost | decisions cited in `spec/` that no handbook doc names | ratchet: the count only goes down ([spec-rules/R6](../guidelines/spec-rules-checklist.md)) | `scripts/spec-decisions-explained.py`, `scripts/check.sh` |
| The suite is fast enough to run on every land ([conformance/R8](../guidelines/conformance-checklist.md)) | CPU seconds of `cpl-conform conformance spec-blocks` (wall time is in `metrics.toml`) | gate: total CPU under a budget of about twice today's | `scripts/conformance-speed.py` in `scripts/check.sh --full` |

### Building the `missing` rows

In order of what a user would feel:

1. Generated programs agree across engines: silent wrong answers are the
   worst failure a contestant can hit.
2. Metamorphic transforms over every passing case: these turn each of the
   2300 cases into many parity and front-end tests.
3. A known-gap case fails for the reason it names: the first two patterns
   are checked; extend them as new kinds of wrong case turn up.
4. The sanitized `cpl-conform diff` as a scheduled job.
5. Chapter vs `std` contradictions: return types and the existence of named
   items, not just named arguments.
6. Width boundary generation.
7. Mutation score on compiler source.
8. Grammar fuzzing.
9. Cross-reference checks, the changelog, and the handbook/spec sync check.

## Invariants

- A case is never deleted or weakened to make an implementation pass. A
  wrong case is fixed, with the reason in the commit
  ([conformance/R2](../guidelines/conformance-checklist.md)).
- Cases test the language through `cpl`, never through crate internals, so
  the suite stays independent of the compiler.
- Each rule id is defined once, where `(Rnn.n.n)` ends its sentence. A case
  cites it as `// rule: nn.n.n`, without the `R`.
- A case expecting a code marks the line it points at with `// <- here`; no case counts lines (decision 506).
- A new case that fails today is recorded as known-failing. A passing case
  that starts failing blocks the land.
- Normative rules live only in `spec/`. The handbook explains them and
  `plan/domains/spec/` tracks work on them; neither restates them.

## Judgment rubric and probes

Numbers miss some things, so a cold reader also scores fixed samples from 1
to 5, with a reason, and the trend is the signal:

1. **Rebuildable:** could an implementer build this rule from the chapter
   alone, with no guess?
2. **Three answers:** does each checked operation say what it means, what
   the compiler proves, and what each build checks?
3. **One spelling:** do the chapter, the `std` signature, the error text and
   the cases say it the same way?
4. **Sharp case:** is each case minimal, about one rule, and would it fail
   on a plausible compiler bug, not just today's?
5. **Worth the rule:** does each rule earn the thing a contestant must
   remember?

Three probes put the rubric to work:

- **Rebuild:** a cold agent with only `spec/` answers snippet questions and
  writes solutions; every guess is a hole (`research/rebuild-rounds/`).
- **Bug-catch:** revert past compiler fixes one at a time; the suite should
  go red (`scripts/conformance-bugcatch.py`).
- **Add a rule:** add a rule with accept and reject cases from
  `spec/conformance/README.md` alone; the gate is green on the first try.

## Suites and tools

| Tool | What it catches | Run it |
|---|---|---|
| `cpl-conform conformance` | Wrong output, wrong diagnostic, missing trap, per case | `cargo run -p cpl-conform -- conformance [--verbose] [filter]` |
| `cpl-conform diff` | Interpreter vs C++ disagreements over every case; set `CPL_CONFORM_CXXFLAGS` for sanitizer builds | `cargo run -p cpl-conform -- diff` |
| `cpl-conform spec-blocks` | Code examples in `spec/core` and `spec/lib` that don't compile or run | `cargo run -p cpl-conform -- spec-blocks` |
| `cpl-conform corpus` | Corpus solutions that fail their tests, judged by `checker.py` when present | `cargo run -p cpl-conform -- corpus` |
| `cpl-conform all --check` | Any case that passed before and fails now | gate step `cpl-conform ratchet` |
| `scripts/conformance-coverage.py` | Untested rules and codes; writes `COVERAGE.md` | `uv run scripts/conformance-coverage.py --check` |
| `scripts/conformance-matrix.py` | Empty feature x consumer cells; generates `spec/conformance/matrix/` | `python3 scripts/conformance-matrix.py --check` |
| `scripts/conformance-traits.py` | Trait methods with no case | `uv run scripts/conformance-traits.py --check` |
| `scripts/conformance-sigs.py`, `conformance-prelude.py` | Chapter vs `std` disagreements on named args and prelude names | `uv run scripts/conformance-sigs.py --check` |
| `scripts/conformance-subset.py` | A failing case in a supported section with no known-gap line | `python3 scripts/conformance-subset.py --check` |
| `scripts/conformance-bugcatch.py` | Past compiler fixes the suite would not have caught | `uv run scripts/conformance-bugcatch.py` |
| `scripts/spec-vocab.py` | Banned spellings | `python3 scripts/spec-vocab.py` |
| Rebuild rounds | Holes and contradictions a cold reader hits; reports in `research/rebuild-rounds/` | process, one cold agent per round |

## Changes

- 2026-11-23: first version, folded from `plan/domains/spec/README.md` and `ARCHITECTURE.md`.
