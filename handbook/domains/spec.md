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
| Every rule is tested ([C1](../guidelines/conformance.md#c1-every-rule-has-a-case-and-every-forbidding-rule-has-a-reject-case)) | rules with no case; forbidding rules with no reject case | ratchet in the gate | `scripts/conformance-coverage.py --check`, `spec/conformance/rule-gaps.txt` |
| Every case cites something that exists | cites to unknown rules | gate | `cpl-conform --rules` |
| Every diagnostic code is reachable | codes with no reject case | ratchet | `scripts/conformance-coverage.py --check` |
| Feature combinations are tested (index types x consumers) ([C5](../guidelines/conformance.md#c5-where-a-rule-covers-a-space-generate-the-cases)) | empty matrix cells | generated suite, gate | `scripts/conformance-matrix.py --check`, `spec/conformance/matrix.toml` |
| Every trait method on every claimer is tested ([C5](../guidelines/conformance.md#c5-where-a-rule-covers-a-space-generate-the-cases)) | trait methods with no case | generated table, ratchet | `scripts/conformance-traits.py --check` |
| Chapters and `std` agree on named arguments | named args no std fn takes | ratchet | `scripts/conformance-sigs.py --check`, `sig-gaps.txt` |
| What the chapters call is in the prelude | chapter functions missing from the prelude | ratchet | `scripts/conformance-prelude.py --check`, `prelude-gaps.txt` |
| Chapters and `std` agree on return types and existence (e.g. a chapter naming a type `std` doesn't have) | contradictions found per rebuild round | `missing`: found only by rebuild rounds | — |
| The suite catches compiler bugs | share of reverted compiler fixes the suite turns red | mutation suite over history | `scripts/conformance-bugcatch.py`, `plan/domains/spec/bugcatch.md` |
| The suite catches bugs no past fix shows | mutation score on compiler source mutants | `missing` | — |
| Interpreter and C++ agree ([C6](../guidelines/conformance.md#c6-results-agree-across-engines-with-and-without-sanitizers)) | cases whose output differs between `cpl run` and the built C++ | differential suite | `cpl-conform diff` |
| Interpreter and C++ agree on programs nobody wrote by hand | divergences on generated programs | `missing`: no program generator yet | — |
| Generated C++ has no undefined behaviour ([C6](../guidelines/conformance.md#c6-results-agree-across-engines-with-and-without-sanitizers)) | cases failing under ASan, UBSan and `_GLIBCXX_DEBUG` | differential suite with sanitizer flags | `CPL_CONFORM_CXXFLAGS='-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG' cpl-conform diff` (not in the gate) |
| Every case is itself right | wrong cases found by implementers | gate for new cases; `missing` for known-gap cases, which can hide a wrong header or a misused API because they are expected to fail | `scripts/conform_staged.py` |
| Code examples in the spec run ([C7](../guidelines/conformance.md#c7-code-in-the-spec-runs)) | spec blocks failing that are not `spec-only` | ratchet | `cpl-conform spec-blocks --check` |
| Milestone gaps are visible, not hidden | failing cases in a supported section with no owner | gate | `scripts/conformance-subset.py --check`, `spec/SUBSET.md` |
| One spelling per idea | banned spellings in spec | lint | `scripts/spec-vocab.py` |
| Each decision is recorded once | duplicate decision numbers | check | `scripts/next-decision.py --check` |
| Cases test the language, not the compiler's internals | cases that reach crate internals | design: a case is a `.cp` file run through `cpl` | `crates/cpl-conform/src/conformance.rs` |
| A cold reader can write correct code from `spec/` alone | holes and contradictions per cold rebuild round | process: rebuild rounds | `research/rebuild-rounds/` |
| Handbook language docs and the spec say the same thing | disagreements found | `missing`; process today: an edit to either updates both in one commit | — |
| Handbook docs are written for an outsider | context-dependent phrases, hashes | lint | `scripts/check-handbook.py` |
| Each rule earns what a contestant must remember | rubric score from a cold reader | process: review against the design rules in `AGENTS.md` | — |
| A behaviour the user can observe has exactly one specified answer | rules whose text a cold reader had to guess (silent holes per round) | process: rebuild rounds; `missing`: a check that each `std` function a chapter names has a cost, a trap list and an example | `research/rebuild-rounds/` |
| Generated programs agree across engines (the Csmith and YARPGen approach) ([C6](../guidelines/conformance.md#c6-results-agree-across-engines-with-and-without-sanitizers)) | divergences between `cpl run`, `cpl build` with `-O2`, and a sanitized build, on random well-typed programs | generated suite with a Python model of each program's output: catches wrong answers both engines share, shrinks failures by dropping statements | `scripts/gen-programs.py` (views, folds, Euclidean `/` `%`, bit operators, in-place verbs, loops, closures writing captures; `--cxxflags` adds a sanitized leg) |
| A meaning-preserving edit never changes the output (equivalence-modulo-inputs testing) | cases whose output changes after renaming, reordering independent `let`s, inlining a `let`, wrapping in a `fn`, or `cpl fmt` | `missing`: metamorphic transforms over every passing case | — |
| Checked and release builds agree on valid programs | cases whose output differs between the checked build and the release build | `missing` as a suite; hand cases under `core-09/release-*` | `spec/conformance/core-09/` |
| Every syntactically valid program parses, every invalid one gets one error | grammar productions never generated; parser crashes on generated input | `missing`: grammar fuzzing from the EBNF in `spec/core/01-lexical-grammar.md` | — |
| Integer widths and overflow are right at every boundary | range types x operations without a boundary case (min, max, max+1, products near \(10^{18}\)) | generated matrix covers index types; `missing` for width boundaries | `scripts/conformance-matrix.py` |
| Large output is complete and ordered (over 64 KiB, mixed `print` forms, multitest) | cases over 64 KiB passing in both engines | hand suite | `spec/conformance/core-10/` |
| Interactive I/O flushes correctly | interactive cases passing under an interactor in both engines | hand suite | `spec/conformance/core-06/interactor-send-recv.cp`, `// tool:` cases |
| Ownership and lending rules accept every safe program and reject every unsafe one | false E06xx on safe programs found per rebuild round; sanitizer failures on accepted programs | hand suite plus sanitized `cpl-conform diff`; `missing`: generated alias and lend programs | `spec/conformance/core-05/` |
| What `cpl explain` claims about copies and allocations is true | explain claims contradicted by the generated C++ | hand cases with `// tool: explain` and `// lacks:`; `missing` as a rule over every case | `spec/conformance/core-05/` |
| Tool behaviour a journey relies on is specified and tested (`check`, `run`, `test`, `stress`, `build`, `fmt`, `lint`, LSP) | journey steps with no rule id and case | hand suite with `// tool:` cases | `spec/tools/`, `spec/conformance/` |
| Trace, debugger and explain output is specified | tool features with no rule id | `missing` | — |
| The same source behaves the same on Linux, macOS and Windows (CRLF source and input, paths) | platform cases passing on all three | hand suite; `missing` on macOS and Windows runners | `spec/conformance/core-01/` |
| Known gaps get fixed, not just listed | known-gap cases; age of the oldest; entries in `sig-gaps.txt` and `prelude-gaps.txt` | ratchet: lists only shrink; `missing`: an age report per owner | `spec/SUBSET.md`, `spec/conformance/*-gaps.txt` |
| A known-gap case fails for the reason it names ([C3](../guidelines/conformance.md#c3-a-known-gap-fails-for-the-reason-it-names)) | known-gap cases whose current error isn't the gap's subject (a wrong `at N`, a misused API) | check: a failure with the case's own expected code means a wrong `at N` or `has:`; an E0606 on a printed `?T` means a missing `!` | `scripts/conformance-gaps.py` |
| The suite is deterministic | cases that pass and fail on the same build | per-case `// timeout:`; `missing`: a flake report | `crates/cpl-conform/tests/timeout.rs` |
| Cross-references in the spec resolve | `(see Rnn.n.n)` naming no rule; links to missing files | `missing` for chapter text; `cpl-conform --rules` covers cases only | — |
| The spec says what changed between versions | normative changes with no changelog line | `missing` | — |
| Old programs keep their meaning (pins and `cpl migrate`, R11.4.3) | rule ids for compatibility with cases | `missing` | — |
| Each chapter teaches its model first | supported chapters with no model paragraph | process | `spec/core/` |
| Every design decision gives its reason and the alternatives that lost | decisions cited in `spec/` with no rationale in `handbook/language/` | `missing` | — |
| The suite is fast enough to run on every land | wall time of `cpl-conform conformance spec-blocks` | metric only, `missing` as a gate | `plan/domains/spec/metrics.toml` |

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
