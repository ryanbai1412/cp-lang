# Spec domain

| | |
|---|---|
| Kind | Domain charter |
| Owner | spec |
| Describes | `spec/core/`, `spec/lib/`, `spec/tools/`, `spec/conformance/`, `spec/SUBSET.md`, `spec/GLOSSARY.md`, `handbook/language/`, `crates/cpl-conform/`, `scripts/conformance-*.py`, `scripts/spec-vocab.py`, `scripts/spec-check.sh`, `scripts/spec-stub.py`, `scripts/next-decision.py` |

## Mission

Say exactly what CP Lang is, in one place, so that a contestant, a model and
a second implementer all get the same answer to every "does this compile,
what does it print, what traps?" question. Explain why the language is
shaped that way, so a reader can hold the model in their head. Then hold the
implementation to it with conformance cases that catch each compiler bug
before a contestant does.

Example: the spec says a reversed view `a.rev` walks backwards but keeps
`a`'s keys, so `a.rev.drop(1)` over `[10, 20, 30]` is `20 10`.
`spec/conformance/core-07/rev-pieces-cut-in-walk-order.cp` pins that
answer. A compiler that returns `30 20` fails the case, and the case then
blocks Milestone 1 until it is fixed.

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
| Normative text: where `spec/` disagrees with any other doc, `spec/` wins | every domain | this charter; `spec-vocab.py` |
| Rule ids `Rnn.n.n` | compiler, diagnostics (`spec/diagnostics.md` cites them), site, idioms | `cpl-conform --rules`, `conformance-coverage.py --check` |
| The conformance case format (`// expect:`, `// rule:`, `// input:`, `// timeout:`, `// has:` headers; `at N` counts from the first line after the headers) | compiler, stdlib (`std-*` groups), tools, diagnostics | `crates/cpl-conform/tests/ratchet.rs`, `spec/conformance/README.md` |
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
| diagnostics | Stable error codes, so `// expect: E0302 at N` keeps meaning the same thing |
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
| Every rule is tested | rules with no case; forbidding rules with no reject case | ratchet in the gate | `scripts/conformance-coverage.py --check`, `spec/conformance/rule-gaps.txt` |
| Every case cites something that exists | cites to unknown rules | gate | `cpl-conform --rules` |
| Every diagnostic code is reachable | codes with no reject case | ratchet | `scripts/conformance-coverage.py --check` |
| Feature combinations are tested (index types x consumers) | empty matrix cells | generated suite, gate | `scripts/conformance-matrix.py --check`, `spec/conformance/matrix.toml` |
| Every trait method on every claimer is tested | trait methods with no case | generated table, ratchet | `scripts/conformance-traits.py --check` |
| Chapters and `std` agree on named arguments | named args no std fn takes | ratchet | `scripts/conformance-sigs.py --check`, `sig-gaps.txt` |
| What the chapters call is in the prelude | chapter functions missing from the prelude | ratchet | `scripts/conformance-prelude.py --check`, `prelude-gaps.txt` |
| Chapters and `std` agree on return types and existence (e.g. a chapter naming a type `std` doesn't have) | contradictions found per rebuild round | `missing`: found only by rebuild rounds | — |
| The suite catches compiler bugs | share of reverted compiler fixes the suite turns red | mutation suite over history | `scripts/conformance-bugcatch.py`, `plan/domains/spec/bugcatch.md` |
| The suite catches bugs no past fix shows | mutation score on compiler source mutants | `missing` | — |
| Interpreter and C++ agree | cases whose output differs between `cpl run` and the built C++ | differential suite | `cpl-conform diff` |
| Interpreter and C++ agree on programs nobody wrote by hand | divergences on generated programs | `missing`: no program generator yet | — |
| Generated C++ has no undefined behaviour | cases failing under ASan, UBSan and `_GLIBCXX_DEBUG` | differential suite with sanitizer flags | `CPL_CONFORM_CXXFLAGS='-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG' cpl-conform diff` (not in the gate) |
| Every case is itself right | wrong cases found by implementers | gate for new cases; `missing` for known-gap cases, which can hide a wrong header or a misused API because they are expected to fail | `scripts/conform_staged.py` |
| Code examples in the spec run | spec blocks failing that are not `spec-only` | ratchet | `cpl-conform spec-blocks --check` |
| Milestone gaps are visible, not hidden | failing cases in a supported section with no owner | gate | `scripts/conformance-subset.py --check`, `spec/SUBSET.md` |
| One spelling per idea | banned spellings in spec | lint | `scripts/spec-vocab.py` |
| Each decision is recorded once | duplicate decision numbers | check | `scripts/next-decision.py --check` |
| Cases test the language, not the compiler's internals | cases that reach crate internals | design: a case is a `.cp` file run through `cpl` | `crates/cpl-conform/src/conformance.rs` |
| A cold reader can write correct code from `spec/` alone | holes and contradictions per cold rebuild round | process: rebuild rounds | `research/rebuild-rounds/` |
| Handbook language docs and the spec say the same thing | disagreements found | `missing`; process today: an edit to either updates both in one commit | — |
| Handbook docs are written for an outsider | context-dependent phrases, hashes | lint | `scripts/check-handbook.py` |
| Each rule earns what a contestant must remember | rubric score from a cold reader | process: review against the design rules in `AGENTS.md` | — |
| The suite is fast enough to run on every land | wall time of `cpl-conform conformance spec-blocks` | metric only, `missing` as a gate | `plan/domains/spec/metrics.toml` |

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
