# Domains: where we are and where we are going

One section per domain. Each has the north star (the maximally ambitious,
complete version), where we are today, the bar that says "done", and the
next steps. The north star is the maximally ambitious, complete version,
not a milestone: if it looks reachable soon, it is not ambitious enough.
The lane that owns a domain keeps its section true: correct
"where we are" with evidence, sharpen the north star whenever you see
further, and move finished steps out. Every few rounds a fresh reader asks,
for every domain, "what is the maximal version, and what is the next step?"
and edits this file. Goals and principles are in `AGENTS.md`; the loop is
in `plan/OPERATING.md`; the detailed design for each domain is in the
`plan/` chapter named in its heading.

## 1. Core language and types (`plan/1x-*`)
**North star.** A small, general core in which the entire stdlib, every
derive, every contest helper and every library diagnostic is ordinary
library code a user could have written, with no compiler privilege. One
spelling per idea, no special cases, and every P0 bug class unwritable or
trapped. Any algorithm in the corpus reads like its editorial: types carry
the proofs, revisions stay local, and the obvious program is the correct and
fast one. A contestant learns the whole core in an afternoon and never meets
a corner case. Concretely, the compiler knows no trait by name: `print`,
`read`, `assert`, `debug`, `??` and every derive are library functions built
on four core mechanisms (compile-time reflection over fields and variants,
variadic generics as tuple packs, lazy parameters, writable places returned
from methods), user optionals reach 8 bytes through a library `Niche`, and a
library can say in its own words why a call does not type-check.
**Where we are** (audited 2026-11-21 at `68ab60c`, probes in the commit
message). The first 0.3 syntax batch works: `repeat`, `loop`, one-line
enums, multi-binding `var`, unary `+`, `read(lo..=hi)`. Structs get `Eq`,
`Ord`, `Hash`, `Show` and `Read` from the compiler, structurally, with
`impl Show/Eq/Ord` as hooks (`cpl-ir` `show_fn`/`eq_fn`/`lt_fn`); that is
stdlib privilege, not library code. Enums, even fieldless ones, have no
order (`E0309` on `d < Dir.S` and on sorting). Not built: `@derive`
(`E0501`), variadic parameters (checked, then `E0501` in lowering),
`read<(Int, Int)>()` (does not parse as a call), lazy parameters,
`Niche`, library diagnostics, a method call as an assignment target
(`b.at() += 1` is `E0401`), `for x in s where c` (`E0103`), 2-D index
spaces. 0 of the 52 ```cp blocks of the 0.3 spec pass `cpl check`; most
stop at a parse error, so the count is a floor.
**Bar.** Every code block in the design spec passes `cpl check`; the stdlib
needs no compiler knowledge beyond the documented intrinsics; each
mechanism above has programs that the interpreter and the C++ agree on.
**Next.** Reflection (`for f in fields(x)`, variants) and derives as
trait defaults; variadic packs, then `print`/`read` on them; lazy
parameters, then `assert`/`debug`/`??`; `Niche`; writable places;
library diagnostics; remaining syntax.

## 2. Index spaces and fresh domains (`plan/12`, `plan/19`)
**North star.** Every index-space bug in the P0 list is unwritable: indices from one
space never reach another, off-by-one and base mix-ups are type errors or
one-local-edit conversions, and every range check the compiler can prove
disappears. Runtime-sized and fresh spaces (compressed values, components,
rooted trees, subsets, grids, intervals, cuts) compose freely, cost nothing
over raw `int`, and are shown in the contestant's own base by every tool.
**Where we are.** Counted domains and ranged reads work; domains from any
runtime expression are refused by the back ends; existential returns are
missing. (Lane running.)
**Bar.** Settled semantics in `plan/OPERATING.md` implemented across
checker, interpreter and C++; three ports converted.
**Next.** Fresh domains end to end; escape diagnostics; stdlib functions
that return a new index space.

## 3. Codegen and performance (`plan/30`)
**North star.** The generated C++ is what a world-class contestant would hand-write:
runtime p50 at or below 1.0x the accepted originals and never above 1.5x,
always tunable to 1.0x without leaving the language; memory and layout as
tight as the best C++ (4-byte indices where proven, niches everywhere,
struct-of-arrays on request); readable output that fits 64 KB with room to
spare; and every safety feature with a zero-cost proven form. Compiles in
well under a second.
**Where we are.** C++17 output passes every port under ASan/UBSan; no
big-stack thread; `?Int` is 16 bytes outside proven tables; no systematic
runtime benchmark against the originals. (Lane running.)
**Bar.** Per-port runtime and memory on max inputs, measured on every push.
**Next.** Big-stack thread from the judge profile; heap for runtime-sized
arrays; niches; a matched-algorithm benchmark harness.

## 4. Standard library (`plan/20`)
**North star.** A standard library that covers everything ever asked at the hardest
contests (ICPC WF, IOI, AtCoder WTF, Codeforces div 1), every module
written only with public language features, coherent and guessable: the same
name for the same idea everywhere, every structure customisable through
general traits (monoids, actions, orders), every signature documented with
its cost and checked by a port or test, and every one as fast as the best
known C++ library implementation (AtCoder Library, KACTL, ecnerwala's
library) on matched benchmarks.
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
**North star.** Every error is the message an expert coach would give: what is wrong,
why, and the exact fix, as an applicable edit, in the contestant's terms.
Recovery is so good that a half-written file still gives full types, hover
and completion; there are zero cascades; every planned feature names itself;
and libraries can attach their own errors. Measured against a benchmark of
real contestant mistakes, the first error points at the real cause almost
always.
**Where we are.** Line-level recovery; cascades from failed declarations;
generic messages for planned syntax. (Lane running.)
**Bar.** Golden diagnostics for every error code; a broken-ports benchmark
with first-error accuracy and cascade counts.
**Next.** Error nodes, unknown-type bindings, "not implemented yet" for
planned syntax.

## 6. Interpreter, debugger and traces (`plan/50`)
**North star.** Doubt becomes evidence in seconds. A time-travel debugger over
DAP in every editor that steps backward and forward through any run, including
a run replayed from a failing judge input; an HTML trace that shows only what
matters, filtered by function, multitest case, watched variable, call depth,
index space, or ring buffer before a crash; snapshots that let you iterate on
one stage; every runtime error stops with the concrete values, the call path,
the test case number, and the input line that caused it; never a segfault; the
interpreter fast enough to run full-size tests (within 10x of C++).
**Where we are.** Debug interpreter with runtime checks (36 trap sites in
`trap_parity.cp`, 5 open in `known_traps.cp`) and `debug(...)` output;
`cpl run --trace` writes a self-contained HTML trace (`tools/trace/index.html`,
37 Node unit tests, not yet browser-tested) and `cpl stress --trace` aligns
solution vs `@brute` on `print`/`debug` checkpoints. Gaps against `plan/50`:
(1) `Trap` carries only `{message, span}` — no call stack, no multitest case
number, several traps omit offending values (e.g. overflow, division by zero),
and there are no golden message tests for runtime errors; (2) trace filtering
only supports a single `--trace FN` — missing multi-fn/glob/exclusion,
`--test N`, `--watch VAR`, `--last N`, `--depth N`, and `Bound(d)` cuts are
not shifted to domain base in traces; (3) no step debugger / DAP server or VS
Code debugger wiring; (4) interpreter speed on compute-heavy ports is 46x–323x
slower than C++ `-O2` (measured on `dph` 46.2x, `dpl` 65.5x, `dpk` 147.4x,
`dpm` 322.9x); (5) no stage snapshots (`--stage`) or interactive exchange trace.
**Bar.** Every runtime error class has a golden message with values and call
path; trace filtering and index-space views tested end to end; DAP step
debugger wired into VS Code and tested over stdio; interpreter within 30x of
C++ `-O2` on ports.
**Next.** (1) Call stacks, values, and golden tests for every runtime error
class; (2) profile and optimize interpreter hot paths toward <= 30x of C++;
(3) trace filtering (`--trace` globs/exclusions, `--test`, `--watch`, `--last`,
`--depth`) and index-space cut/domain fixes; (4) DAP step debugger (`cpl dap`)
with time-travel + VS Code integration; (5) browser test for trace viewer and
stage snapshots.

## 7. Stress testing and shrinking (`plan/50` 3, `plan/40` 7b)
**North star.** One command, `cpl stress d.cp`, turns "WA on test 37" into
a minimal counterexample in seconds, side by side with the brute's answer,
with nothing written but the solution and a brute: generators derived from
the input types and constraints (ranges, domains, words, permutations,
trees, weighted and simple graphs, multitest, interactive judges),
constraints types can't state written as ordinary generator code,
structural shrinking that keeps every invariant (a tree stays a tree, a
graph keeps its shape, a permutation stays a permutation), adversarial
generators that hunt hash collisions, TLE inputs and worst cases, and a
seeded-bug benchmark showing it finds classic bugs faster than any hand
workflow. Every structured type works the same in every tool: the
compiler accepts it, `cpl run` and the C++ read it, stress generates and
shrinks it.
**Where we are.** Audited 2026-11-21. `cpl stress` (`crates/cpl-stress`)
generates from top-level `let x: T = read()` types: ranges, `Int` (a
guessed range, narrowed when the program stops on it), `Char`, `Bool`,
`Str`, domains, aliases, tuples, records, N-D arrays, `[read() for ...]`,
`@!multitest`, and the hidden test an `@interactor` reads. Shrinking is
structural (sizes bisected, single elements deleted, values lowered,
multitest cut to the one failing test), and an in-file `@gen` fn shrinks
through its recorded `rand`/`shuffle` choices. Gaps found: `Perm<V>` and
`Tree<V>` are known only to stress (the compiler rejects them with E0204,
so only a local alias named `Perm` or `Tree` works); a weighted tree
(`(u: V, v: V, w: W)`) is generated as random pairs, not a tree; there are
no simple or connected graphs; an edge list shrinks by clamping, which can
make self-loops; there is no benchmark of how fast stress finds bugs.
**Bar.** A seeded-bug benchmark in the gate: classic bugs injected into
correct solutions, each found and shrunk to a known minimal case, with the
median time to a minimal counterexample at most 10 s (`GOALS` 4).
**Next.** Seeded-bug benchmark; `Perm`/`Tree`/graph types every tool
accepts; weighted trees; simple and connected graphs; edge-deleting graph
shrinking.

## 8. Formatter, linter, profiler, coverage, bench, doc (`plan/45`)
**North star.** Every tool a contestant reaches for exists, is instant, and understands
the language's ideas: a formatter with one style; a linter whose every rule
comes from a real bug, with zero false positives; a profiler that explains
time and memory per line and per index space, with predicted versus actual
complexity; coverage; benchmarks against limits on worst-case inputs; a
documentation generator; a REPL; and refactorings through the LSP.
**Where we are.** fmt, lint, doc, bench exist; profiler and coverage do
not.
**Bar.** Each tool has tests and a doc page; lint has zero false positives
on the corpus.
**Next.** Profiler (per-line time and counts); coverage; lint rules from
friction reports.

## 9. Judges and submission (`plan/70`)
**North star.** Submission is boring and always right: one self-contained file per judge
profile (every major judge and ICPC-style local setups), compiled and run in
CI against each judge's real compiler and flags, under 64 KB, identical
behaviour everywhere, function-style and interactive judges included, and a
submit command that checks samples, size and profile before sending.
**Where we are.** One C++17 output; 64 KB check in the gate; no judge
profiles.
**Bar.** Profiles for Codeforces, AtCoder, CSES, ICPC-style; each tested
with the target compiler flags.
**Next.** Judge profiles (stack, compiler, I/O); profile tests.

## 10. Editor and LSP (`plan/60`)
**North star.** An editor that knows everything the compiler knows, in every major
editor from one language server (a VS Code extension, a JetBrains plugin,
documented Neovim, Zed, Emacs and Helix setups): diagnostics as you type,
hover with types, ranges and costs, inlay hints for index spaces and inferred
bounds, completion that knows the stdlib, signature help, rename and
refactorings, code actions for every fixable diagnostic, and run, test,
stress, trace and debug from the editor with the trace view inside it.
**Where we are.** `cpl lsp` and a VS Code extension with highlighting and
snippets; no JetBrains plugin.
**Bar.** LSP feature tests per `plan/60` 6.7; each editor integration
built and smoke-tested in the gate.
**Next.** Audit against `plan/60`; hover, inlay hints, completion quality;
JetBrains plugin.

## 11. Docs, site and playground (`plan/90`)
**North star.** The language is writable from the docs alone: a one-afternoon tour, a
guide with one chapter per idea, a complete reference, a stdlib reference
with costs, an error index with one page per diagnostic, a cookbook built
from hundreds of real ports, interactive lessons that teach by solving, and
a browser playground that runs the interpreter, the trace viewer and a judge,
every example executed in the gate.
**Where we are.** Site generator, course, playground page, cheat sheet;
docs lag the 0.3 changes.
**Bar.** A cold reader writes a correct solution from the docs alone; every
example in the docs is run in the gate.
**Next.** Update to 0.3; error index; cookbook from ports.

## 12. Validation: ports, scoreboard, differential testing (`plan/80`)
**North star.** Thousands of hard accepted solutions rethought in the language, every
one differential-tested against its original, measured on every push for
correctness, runtime, memory, source size, contestant tokens and friction,
with a scoreboard that blocks regressions; zero open blocker frictions; and
real contest use by strong contestants showing faster accepted solutions and
fewer wrong attempts than in C++.
**Where we are.** 235 ecnerwala ports gated; friction lint; no single
scoreboard; port waves running.
**Bar.** 500+ ports; scoreboard history; zero open blocker frictions.
**Next.** Scoreboard script; continuous port waves; friction triage.
