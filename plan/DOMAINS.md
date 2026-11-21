# Domains: where we are and where we are going

One section per domain. Each has the north star (the maximally ambitious,
complete version), where we are today, the bar that says "done", and the
next steps. The lane that owns a domain keeps its section true: correct
"where we are" with evidence, sharpen the north star whenever you see
further, and move finished steps out. Every few rounds a fresh reader asks,
for every domain, "what is the maximal version, and what is the next step?"
and edits this file. Goals and principles are in `AGENTS.md`; the loop is
in `plan/OPERATING.md`; the detailed design for each domain is in the
`plan/` chapter named in its heading.

## 1. Core language and types (`plan/1x-*`)
**North star.** A small, general core in which the whole stdlib is ordinary
library code: no stdlib privilege, one spelling per idea, no special cases.
Every P0 bug class is either unwritable or trapped, and the obvious program
is the correct and fast one.
**Where we are.** 0.2.2 plus the first 0.3 syntax batch (`repeat`, `loop`,
one-line enums, multi-binding `var`, unary `+`, counted domains,
`read(lo..=hi)`). Missing from the 0.3 design: comptime reflection over a
type's fields and variants (for derives: Read, Print, Gen, Hash, Ord),
variadic generics (`read<(Int, [n]Int)>()`, `print(a, b, c)`), lazy
parameters (`??`, `assert`, `debug` as library), a niche trait for 8-byte
user optionals, library-authored diagnostics, writable places through
methods (`seg.at(i) += 1`), `for x in s where c`, 2-D index spaces.
**Bar.** Every code block in the design spec passes `cpl check`; the stdlib
needs no compiler knowledge beyond the documented intrinsics.
**Next.** Reflection-based derives; variadics; lazy parameters; niche trait;
writable places; remaining syntax.

## 2. Index spaces and fresh domains (`plan/12`, `plan/19`)
**North star.** Wrong-index-space and out-of-range bugs are unwritable in
the happy path and one local edit on the cold path; runtime-sized domains
cost nothing over raw `int` indices.
**Where we are.** Counted domains and ranged reads work; domains from any
runtime expression are refused by the back ends; existential returns are
missing. (Lane running.)
**Bar.** Settled semantics in `plan/OPERATING.md` implemented across
checker, interpreter and C++; three ports converted.
**Next.** Fresh domains end to end; escape diagnostics; stdlib functions
that return a new index space.

## 3. Codegen and performance (`plan/30`)
**North star.** Generated C++ is as fast as a strong contestant's
hand-written C++ (runtime p50 <= 1.1x, max <= 1.5x, always tunable to
1.1x), small (hard 64 KB), readable, and every safety feature has a
zero-cost form.
**Where we are.** C++17 output passes every port under ASan/UBSan; no
big-stack thread; `?Int` is 16 bytes outside proven tables; no systematic
runtime benchmark against the originals. (Lane running.)
**Bar.** Per-port runtime and memory on max inputs, measured on every push.
**Next.** Big-stack thread from the judge profile; heap for runtime-sized
arrays; niches; a matched-algorithm benchmark harness.

## 4. Standard library (`plan/20`)
**North star.** Everything a contest needs, written once, coherent and
guessable: the same name for the same idea everywhere, every module built
only from public language features, every signature documented with cost.
**Where we are.** 25 modules, about 322 public items. Absent or thin:
number theory (sieve, factorization, CRT), NTT/convolution, matrices,
binary search on answers, RNG and hashing, bitsets as a real type, sparse
tables generalised, flows and matchings beyond the basics, strings beyond
the basics, geometry. No single design document of the whole surface.
**Bar.** A full stdlib design doc (every module, signature, naming rule,
core features relied on); every function exercised by a port or test.
**Next.** Write the design doc from 0.2.2 and what the corpus uses; fill
the missing modules; renames to the 0.3 names.

## 5. Diagnostics and front end (`plan/30` 3.7)
**North star.** Every error says what is wrong, why, and the fix, in the
contestant's terms; recovery is good enough that an editor works in a
half-written file; no cascades.
**Where we are.** Line-level recovery; cascades from failed declarations;
generic messages for planned syntax. (Lane running.)
**Bar.** Golden diagnostics for every error code; a broken-ports benchmark
with first-error accuracy and cascade counts.
**Next.** Error nodes, unknown-type bindings, "not implemented yet" for
planned syntax.

## 6. Interpreter, debugger and traces (`plan/50`)
**North star.** Doubt becomes evidence in seconds: time-travel debugging,
an HTML trace filtered to what matters, stage snapshots, watching index
spaces in the contestant's own base, every runtime error with the values
and the call path, never a segfault.
**Where we are.** Debug interpreter with checks; `cpl run --trace` writes
an HTML trace; no step debugger; interpreter speed limits large inputs.
**Bar.** Every runtime error class has a golden message; the trace viewer
is tested in a browser; interpreter within 30x of C++ on ports.
**Next.** Audit against `plan/50`; DAP debugger; interpreter speed.

## 7. Stress testing and shrinking (`plan/50` 3, `plan/40` 7b)
**North star.** One command finds a minimal counterexample against a brute
or checker, shrinking structurally (graphs stay graphs), with generators
derived from the input types.
**Where we are.** `cpl stress` with shrinking exists; generators are
hand-written per port.
**Bar.** Seeded-bug benchmark: time to a minimal counterexample.
**Next.** Type-derived generators; structural shrinking; benchmark.

## 8. Formatter, linter, profiler, coverage, bench, doc (`plan/45`)
**North star.** Every tool a contestant reaches for exists, is fast, and
understands the language's ideas (domains, optionals, costs).
**Where we are.** fmt, lint, doc, bench exist; profiler and coverage do
not.
**Bar.** Each tool has tests and a doc page; lint has zero false positives
on the corpus.
**Next.** Profiler (per-line time and counts); coverage; lint rules from
friction reports.

## 9. Judges and submission (`plan/70`)
**North star.** Submission is boring: one file per judge profile, under
64 KB, that compiles on every target compiler and behaves identically.
**Where we are.** One C++17 output; 64 KB check in the gate; no judge
profiles.
**Bar.** Profiles for Codeforces, AtCoder, CSES, ICPC-style; each tested
with the target compiler flags.
**Next.** Judge profiles (stack, compiler, I/O); profile tests.

## 10. Editor and LSP (`plan/60`)
**North star.** An editor that knows everything the compiler knows:
diagnostics as you type, hover with types and costs, inlay hints for index
spaces, completion, rename, run/test/trace from the editor.
**Where we are.** `cpl lsp` and a VS Code extension with highlighting and
snippets.
**Bar.** LSP feature tests per `plan/60` 6.7.
**Next.** Audit against `plan/60`; hover, inlay hints, completion quality.

## 11. Docs, site and playground (`plan/90`)
**North star.** The language is writable from the docs alone: tour, guide,
reference, stdlib reference, error index, cookbook, and a playground.
**Where we are.** Site generator, course, playground page, cheat sheet;
docs lag the 0.3 changes.
**Bar.** A cold reader writes a correct solution from the docs alone; every
example in the docs is run in the gate.
**Next.** Update to 0.3; error index; cookbook from ports.

## 12. Validation: ports, scoreboard, differential testing (`plan/80`)
**North star.** Hundreds of hard accepted solutions rethought in the
language, all differential-tested, with a scoreboard on every push that
blocks regressions in correctness, speed, size and friction.
**Where we are.** 235 ecnerwala ports gated; friction lint; no single
scoreboard; port waves running.
**Bar.** 500+ ports; scoreboard history; zero open blocker frictions.
**Next.** Scoreboard script; continuous port waves; friction triage.
