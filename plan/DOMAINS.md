# The areas of CP Lang

CP Lang is one language plus the tools around it. This page lists every
area: what it is, where it lives, where it stands, and what comes next.
The long per-area audit log, with evidence and history, is
`plan/DOMAINS-LOG.md`. Goals and rules are in `AGENTS.md`.

| # | Area | Lives in | One line |
|---|---|---|---|
| 1 | Spec and conformance | `spec/`, `scripts/conformance.py` | What the language is, and the tests that hold implementations to it |
| 2 | Core language | `crates/cpl-syntax`, `cpl-check`, `cpl-ir` | Parsing, types, ownership, domains |
| 3 | Proofs and range analysis | `crates/prover` | Proves ranges, overflow and laws so checks disappear |
| 4 | Standard library | `spec/std/` (stubs), `stdlib/` (code) | Everything a contest needs, as ordinary library code |
| 5 | C++ codegen and performance | `crates/cpl-codegen` | One fast, readable C++17 file per program |
| 6 | Diagnostics | `crates/cpl-diag`, `spec/diagnostics.md` | Errors that name the cause and the fix |
| 7 | Interpreter, debugger, traces | `crates/cpl-interp`, `tools/trace` | Runs programs with checks; shows what went wrong |
| 8 | Stress testing | `crates/cpl-stress` | Finds and shrinks a counterexample from the types alone |
| 9 | Contest tools | `cpl-fmt`, `cpl-lint`, `cpl-doc`, `cpl-contest` | fmt, lint, doc, profile, cover, `new`/`test`/`watch` |
| 10 | Judges and submission | `crates/cpl-judge` | What passes here passes on the judge |
| 11 | Editor and LSP | `crates/cpl-lsp`, `editors/` | Everything the compiler knows, in the editor |
| 12 | Docs, site, playground | `docs/`, `site/`, `crates/cpl-wasm` | Writable from the docs alone |
| 13 | Corpus and hill-climb | `corpus/problems/` | Real problems that drive the design |
| 14 | Validation and scoreboard | `proto/ports/`, `scripts/scoreboard.py`, `crates/cpl-verify` | Ports gated against their C++ originals, measured on every push |

## 1. Spec and conformance

**Goal.** Two independent implementers reading only `spec/` build the same
language. Every normative rule has an id and a test.

**Now.** v0.3.0 spec: 14 core chapters, library guides, stubs for every
std module, tool contracts. Chapters 1–3 carry rule ids `(R01.2.3)` with
cases in `spec/conformance/`; the runner meets about half of them, and
each miss is a checker or interpreter bug. Library docs follow the
rustdoc-style standard in `spec/std/README.md`, ratcheted by
`scripts/spec-stub.py --check`.

**Next.** Rule ids and cases for chapters 4–14; finish the library docs;
a strict checker mode that rejects anything the spec doesn't define; an
independent rebuild test (a fresh agent with only `spec/`).

## 2. Core language

**Goal.** A small core in which the whole stdlib is ordinary code a user
could write. One spelling per idea; every P0 bug class unwritable or
trapped. Domains (typed index spaces, positions vs cuts, compressed and
product domains) are part of the core, not a separate feature.

**Now.** Most of 0.3 parses and checks: domains, optionals with niches,
`is` patterns, `let ... else`, record update, existential domains, field
reflection, variadic packs, lazy parameters. The checker is laxer than
the spec in places (it accepts forms the spec doesn't define).

**Next.** Remaining 0.3 syntax, derives as library code, close the gaps
the conformance suite finds.

## 3. Proofs and range analysis

**Goal.** Every range check, overflow check and `@laws` claim the compiler
can prove disappears; the rest are checked in debug builds.

**Now.** A dependency-free prover answers "proved / counterexample / not
proved" in microseconds; it drives range elimination and `invariant`.

**Next.** Range facts on expressions and `var`s for the LSP; feed runtime
range fields into C++ bounds elimination.

## 4. Standard library

**Goal.** Covers everything asked at the hardest contests, coherent and
guessable, every structure customizable through general traits, as fast
as the best C++ libraries.

**Now.** `spec/std/` holds the 0.3 design (about 2,000 documented
signatures, bodies `todo`); `stdlib/` holds the working 0.2 library
(38 modules) that the ports use.

**Next.** Implement the 0.3 stubs in `stdlib/`, turning each doc example
into a test.

## 5. C++ codegen and performance

**Goal.** The C++ a world-class contestant would hand-write: runtime p50 at
or below 1.0x the originals, never above 1.5x, tight layouts with niches
and narrow storage, one file under 64 KB.

**Now.** Every port passes under ASan/UBSan; deep recursion runs on a
big thread stack; runtime p95 about 3x the originals.

**Next.** Narrow storage from domains and ranges, niches everywhere, a
matched-algorithm benchmark.

## 6. Diagnostics

**Goal.** Every error is what an expert coach would say: what, why, and the
fix as an edit. No cascades.

**Now.** Strong recovery (first error on the right line 99.8% on the
broken-ports benchmark); every code has a catalogue entry and an error
page. Runtime traps now carry their specific `R` code.

**Next.** Make checker codes match `spec/diagnostics.md` everywhere the
conformance suite disagrees; quick fixes on every syntax error.

## 7. Interpreter, debugger, traces

**Goal.** Doubt becomes evidence in seconds: a time-travel debugger, HTML
traces filtered to what matters, every trap with values and call path.

**Now.** Debug interpreter with runtime checks, call stacks and values on
traps; `cpl run --trace` writes an HTML trace.

**Next.** Interpreter speed (within 10x of C++), stage snapshots, a
browser test for the trace viewer, DAP debugging.

## 8. Stress testing

**Goal.** `cpl stress d.cp` turns "WA on test 37" into a minimal
counterexample in seconds, with nothing written but the solution and a
brute.

**Now.** Generates from read types (ranges, domains, arrays, records,
multitest, interactors) and shrinks structurally; a 21-bug benchmark
finds and minimizes every bug, median 0.6 s.

**Next.** Reads in shapes a row can't describe, graph/tree/grid
generators, anti-hash and worst-case inputs.

## 9. Contest tools

**Goal.** Every tool a contestant reaches for exists, is instant, and
understands the language.

**Now.** fmt (stable on the whole corpus), lint (taste and bug lints from
real ports), doc, profile (interpreter and native agree), cover, bench,
and the `new`/`test`/`watch` problem workflow.

**Next.** Lints from the friction detectors; profile against `@cost`
and the time limit.

## 10. Judges and submission

**Goal.** What passed here passes on the judge, and the contestant never
thinks about it.

**Now.** Seven judge profiles (compiler, flags, stack, source limit);
every port fits every profile.

**Next.** Line markers that map judge errors back to source; USACO and
IOI grader profiles.

## 11. Editor and LSP

**Goal.** One language server giving every editor everything the compiler
knows, including storage sizes, layouts and peak memory per judge profile.

**Now.** Diagnostics, hover, definition, references, rename, completion,
semantic tokens; VS Code, JetBrains, Neovim, Emacs, Helix setups.

**Next.** Fixture tests for every request; proven ranges and layout
hovers; the VS Code extension suite in the gate.

## 12. Docs, site, playground

**Goal.** A cold reader writes correct solutions from the docs alone.

**Now.** A static site with a 10-lesson course, guides, a cheat sheet, an
error index, a cookbook, a ports gallery and a browser playground, all
checked in the gate.

**Next.** Move the docs to 0.3 once the implementation follows the spec.

## 13. Corpus and hill-climb

**Goal.** The design is driven by real hard problems: for each one, the
ideal program (`dream.cp`) and the spec-compliant one (`spec.cp`), with
every gap fixed in the spec.

**Now.** 836 problems with `meta.json`; 100 have dream and spec ports, 99
pass `cpl check`. Findings are in `spec/delta/hillclimb.md`.

**Next.** Run the 100 spec ports against judge tests once the 0.3 stdlib
runs; extend the hill-climb to more problems.

## 14. Validation and scoreboard

**Goal.** Thousands of accepted solutions, differential-tested against
their originals and measured on every push across correctness, length,
speed and memory.

**Now.** 285 of 285 active ports pass; contestant-written tokens median
0.42x the C++; the scoreboard runs in the fast gate.

**Next.** Zero-cost metrics (`sizeof`, assembly) in the scoreboard;
grow the bug-replay and revision suites.
