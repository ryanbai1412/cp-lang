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
spelling per idea, no special cases, every P0 bug class unwritable or
trapped, and every feature working identically across check, interpreter,
C++ codegen, formatter, linter, LSP, trace, and docs. Any algorithm in the
corpus reads like its editorial: types carry the proofs, revisions stay
local, and the obvious program is the correct and fast one. A contestant
learns the whole core in an afternoon and never meets a corner case.
Concretely, the compiler knows no trait by name: `print`, `read`, `assert`,
`debug`, `??` and every derive are library functions built on four core
mechanisms (compile-time reflection over fields and variants, variadic
generics as tuple packs, lazy parameters, writable places returned from
methods), user optionals reach 8 bytes through a library `Niche`, and a
library can say in its own words why a call does not type-check.
**Where we are** (audited 2026-11-21 at `68ab60c`+). Working: `repeat`,
`loop`, one-line enums and structs (paren and colon forms), multi-binding `let`/`var`, unary
`+`, counted domains, `read(lo..=hi)`, `D.count`, `for ... if`/`where`,
`if let`/`while let` with `where` and multi-binding `if let a = x, b = y`,
`let ... else`, `if x max= y:`, `?T` arithmetic and bitwise lifting (`+ - * / % ** & | ^ << >>`, unary `-`, `~`),
`max(?T, T)`, `max=`/`min=` on optionals, `[A, B]T`, `[(A, B)]T` and `[A][B]T`,
`pairs()`/`with_prev()` in `std.seq`, `Self` (including dotted range bounds `Self.min..=Self.max`)
and associated types in traits. Structs get `Eq`, `Ord`, `Hash`, `Show` and
`Read` from the compiler, structurally, with `impl Show/Eq/Ord` as hooks
(`cpl-ir` `show_fn`/`eq_fn`/`lt_fn`); that is stdlib privilege, not library
code. Enums, even fieldless ones, have no order (`E0309` on `d < Dir.S` and
on sorting). Not built: postfix `x?` expression-extent lifting,
opaque `struct List<T>` / struct type-param
defaults / one-sided range types /
`@droppable` / `labels(...)` / `Str(len:)`, `@derive` (`E0501`), variadic
parameters (checked, then `E0501` in lowering), `read<(Int, Int)>()`, lazy
parameters, `Niche`, library diagnostics, a method call as an assignment
target (`b.at() += 1` is `E0401`). 0 of the 35 `ideal-spec/spec/*.md`
```cp blocks pass `cpl check`; most stop at parse or stdlib gaps.
**Bar.** Every code block in the design spec passes `cpl check`; the stdlib
needs no compiler knowledge beyond the documented intrinsics; each
mechanism above has programs that the interpreter and the C++ agree on.
**Next.** Postfix `?` extent; remaining 0.3 syntax (opaque `struct`,
type-param defaults, one-sided ranges, `@droppable`,
`labels`, `Str(len:)`); reflection (`for f in fields(x)`, variants) and
derives as trait defaults; variadic packs, then `print`/`read` on them;
lazy parameters, then `assert`/`debug`/`??`; `Niche`; writable places;
library diagnostics.

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

## 4. Standard library (`plan/STDLIB.md`, `plan/20`)
**North star.** A standard library that covers everything ever asked at the hardest
contests (ICPC WF, IOI, AtCoder WTF, Codeforces div 1), every module
written only with public language features, coherent and guessable: the same
name for the same idea everywhere, every structure customisable through
general traits (monoids, actions, orders), every signature documented with
its cost and checked by a port or test, and every one as fast as the best
known C++ library implementation (AtCoder Library, KACTL, ecnerwala's
library) on matched benchmarks.
The local library (`examples/local-lib/`) is the staging area: a module
is promoted once it meets the stdlib bar.
**Where we are.** 25 modules, 280 distinct public names (389 `pub`
declarations including methods), 160 `@test`s passing in
`cpl test stdlib` (164 after number theory). Number theory is complete
for single numbers (Miller-Rabin and Pollard's rho exact for every
`Int`, `phi`, `primitive_root`, `floor_sum`, O(1) `mul_mod`), used by
two ports. Present but thin: RNG (seeded stream, no pick/perm/sample), bitsets (set algebra, shift,
next; no prev/first/last), flows (Dinic, min cut, Hungarian; no
Hopcroft-Karp, no min-cost flow), strings (Z, prefix function, suffix
array, LCP, rolling hash; no Manacher, no Aho-Corasick), geometry
(integer points, turns, segments, area, point location; no hull).
Absent: NTT and convolution, matrices, search over reals and
unimodal functions. NTT, matrices, primes, matching and min-cost flow
exist in `examples/local-lib/`; ports import only `lib.math.poly` (9
files) and `lib.math.powers` (1). 37 public names appear in no port or
example (only in stdlib's own tests). `plan/STDLIB.md` is the design of
the whole surface; a test fails when a public name is missing from it.
**Bar.** Every item of `plan/STDLIB.md` marked [new] or [rename] has
landed with tests; every public name is used by a port or an example;
the [rename]s have codemods and the ports are moved.
**Next.** `std.ntt`; `std.matrix`;
`bsearch_real` and `ternary_max`; RNG `pick`/`perm`/`sample`; bitset
`first`/`last`/`prev`; `max_matching` and `CostFlow`; `manacher` and
`AhoCorasick`; `convex_hull`; then the `_by` renames. From the
wave-11 frictions (`research/frictions/cf2237h.md`, `cf2222f.md`): DSU
payloads, typed rollback checkpoints and parity; an interval run map;
offline divide and conquer over time; predicate counting; link-cut tree.

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
(1) `Trap` now carries call stacks (summarized past 32 frames), multitest case
number, and concrete values on arithmetic/bounds/domain/input traps (golden
tests in `crates/cpl-cli/tests/runtime_errors.rs`); (2) trace filtering only
supports a single `--trace FN` — missing multi-fn/glob/exclusion, `--test N`,
`--watch VAR`, `--last N`, `--depth N`, and `Bound(d)` cuts are not shifted to
domain base in traces; (3) no step debugger / DAP server or VS Code debugger
wiring; (4) interpreter speed on compute-heavy ports is 44x–261x slower than C++ `-O2`
after unboxing `?Int`/1-field `Int` structs and removing hot-path allocations
(`dph` 884 -> 693 ms, `dpl` 542 -> 483 ms, `dpk` 770 -> 728 ms, `dpm` 1288 ->
1028 ms; closing the remaining gap to <= 30x requires a bytecode VM);
(5) no stage snapshots (`--stage`) or interactive exchange trace.
**Bar.** Every runtime error class has a golden message with values and call
path; trace filtering and index-space views tested end to end; DAP step
debugger wired into VS Code and tested over stdio; interpreter within 30x of
C++ `-O2` on ports.
**Next.** (1) Profile and optimize interpreter hot paths toward <= 30x of C++;
(2) trace filtering (`--trace` globs/exclusions, `--test`, `--watch`, `--last`,
`--depth`) and index-space cut/domain fixes; (3) DAP step debugger (`cpl dap`)
with time-travel + VS Code integration; (4) browser test for trace viewer and
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
**North star.** An editor that knows everything the compiler knows, in every
major editor from one language server: a VS Code extension, a JetBrains
plugin (IntelliJ Platform LSP API, highlighting, run configurations, the
trace view), and documented, smoke-tested Neovim, Zed, Emacs and Helix
setups. Every client gets the same server features, because they live in
the server, not the client: diagnostics as you type, hover with types,
proven ranges, docs and costs, inlay hints for index spaces and inferred
bounds, completion ranked by the expected type that knows the stdlib and
adds the `use`, signature help, rename and refactorings across files, go to
definition into the stdlib, a code action for every fixable diagnostic, and
run, test, stress and trace as server code lenses and commands. VS Code and
JetBrains add the test explorer, the trace panel beside the source and a
DAP debugger. Fixture tests with cursor markers cover every request, a
fuzzer proves the server never dies, and a latency benchmark guards 6.1.
**Where we are** (audited 2026-11-21). `cpl lsp` (crates/cpl-lsp) answers
diagnostics on open and on incremental change, hover, definition (into
`stdlib/` files too: `Fenwick` jumps to `stdlib/fenwick.cp`), references,
rename within a file, document symbols, formatting, completion (members
after `.`, names in scope, `use std.`), signature help, inlay hints
(unwritten types, parameter names of literal arguments) and quick fixes
from diagnostic fixes; a panic becomes an error reply plus an
internal-error diagnostic. Unit tests in `src/tests.rs`; `tests/latency.rs`
measures 6.1 (492 lines, debug build: keystroke to diagnostics median
6 ms, hover 0.06 ms, completion 1.8 ms). The VS Code extension has a
TextMate grammar, snippets, the client, Run/Stress/Trace lenses, live
sample values, counterexample overlays and a trace webview; its unit
tests run in `--full`, its extension-host suite in no gate.
Hover appends the declaration's `///` doc, which in the stdlib states the
cost, and definition on a field or method jumps to it, in the stdlib too.
Gaps against `plan/60`: no computed cost on hover; no hover for the index space of a subscript or for a
proven range; no inlay hints for index spaces or ranges; no semantic
tokens, workspace symbols, document highlights or folding; no cost or
overflow warnings; rename is one file; no refactorings or "fix all"; no
expected-type ranking or auto-import; run/test/trace exist only in the VS
Code client; no fixture tests, fuzzer, DAP, tree-sitter grammar, JetBrains
plugin, or setups for other editors.
**Bar.** LSP fixture tests per `plan/60` 6.7 for every request; each editor
integration built and smoke-tested in `scripts/check.sh --full`.
**Next.** Fixture tests; hover index spaces and ranges; inlay hints for
index spaces and ranges; semantic tokens; server code lenses and commands
for run, test and trace; JetBrains plugin; other-editor setups; fuzzer.

## 11. Docs, site and playground (`plan/90`)
**North star.** The language is writable from the docs alone: a one-afternoon tour, a
guide with one chapter per idea, a complete reference, a stdlib reference
with costs and tested doc examples, an error index with one page per
diagnostic and lint code (each with verified trigger and fix), a cookbook
built from hundreds of real validated ports, interactive lessons that teach
by solving, and a browser playground that runs the interpreter, C++17
preview, trace viewer and a judge — every single code block and example
executed in the gate so documentation never drifts.
**Where we are.** Static site generator (`site/build.py`, 24 chapters + 61
individual error/lint pages + playground + trace = 87 HTML pages: 10-lesson
course with 51 checked exercises, 13 reference pages including canonical
`docs/one-page-cheat-sheet.md` and `errors.html`), WASM playground (23 presets),
canonical trace viewer, and offline bundle (`dist/cp-lang.html`, 1.87 MB < 2.6
MB cap). 100/100 doc `cp` fences, 22/22 `site/examples/*.cp`, 10/10 course
write-code refs, and 61/61 diagnostic/lint codes (`52 cpl-diag + 9 cpl-lint`,
with 57 active trigger/fix pairs + 3 reserved + 1 internal) are verified in
`scripts/check.sh --fast` via `site/check_docs.py`. Reference pages and syntax
highlighter are updated for the landed 0.3 batch (`where` guards, `repeat`,
`loop`, multi-binding, unary `+`, one-line enums, counted domains,
`read(lo..=hi)`, `read(none: ...)`, stepped domains, 1-based/grid slices,
`rfind`/`Str in Str`, `@!interactive`, `debug` purity). Gaps: no cookbook from
the 239 validated ecnerwala ports (only 5 showcase cards on `index.html`);
`cpl doc stdlib` finds 715 items across 25 modules (334 undocumented, 0
doc-comment examples); course lessons still use pre-0.3 idioms in places.
**Bar.** A cold reader writes a correct solution from the docs alone; every
doc `cp` block, example fixture, cookbook recipe, and error-index trigger/fix
passes in `scripts/check.sh --fast`; 100% of diagnostic/lint codes have an
error-index page; offline bundle stays under 2.6 MB.
**Next.** Build the cookbook from validated ports; sync `site/content/stdlib.md`
with `stdlib/*.cp` and add tested `cpl doc` examples; refresh course lessons for
0.3.

## 12. Validation: ports, scoreboard, differential testing (`plan/80`)
**North star.** Thousands of hard accepted solutions across CF, AtCoder,
ICPC/UCup, IOI, NOI and DMOJ rethought in the language, discovered
automatically and differential-tested against their originals (interpreter,
sanitized C++ and `-O2` C++), measured on every push in a single deterministic
scoreboard (`scripts/scoreboard.py` + `research/scoreboard.json`) that blocks
regressions across every goal: correctness (gate pass, zero trap-parity drift),
expressiveness (contestant-written tokens vs raw and library-stripped original,
median <= 0.6x, none > 1.0x), performance (runtime p50 <= 1.1x / max <= 1.5x,
peak RSS p50 <= 1.0x / max <= 1.2x on max-size inputs, interpreter within 30x
of C++), submission viability (emitted C++ <= 64 KB, p95 <= 32 KB, warning-free
GCC/Clang), and friction (structured `research/frictions/*.md` reports with
zero open blockers and `friction_lint` zero `wa_trap`). Backed by a replayed
P0 bug corpus (>= 90% prevented or trapped), static `sizeof` and `-O2` assembly
zero-cost proofs, and real contest use by strong contestants showing faster
accepted solutions and fewer wrong attempts than in C++.
**Where we are.** 237 ecnerwala ports gated in `scripts/check.sh --fast`
(`cpl check`, `cpl fmt --check`, `cpl lint` with 0 warnings, `cpl build <= 64 KB`;
227 have `gen/*.py`, 218 have `samples/`, 13 have `interactors/*.py`), plus 30
`wave2` Rust-era ports (30/30 check + lint, 7 unformatted) and legacy DMOJ
registry (`bench/corpus-500-registry.json`: 594 unique problems, 566 verified
at `87ce801`). `bench/corpus_p95_bench.py` last ran at `6145b0b` (343 ports,
runtime p50 0.98x / p95 2.57x, RSS p50 1.07x / p95 3.64x, max emitted 34.4 KB).
`scripts/friction_lint.py` reports 0 `wa_trap`, 3 `real_gap`, 31 `stale_port`
on ecnerwala; `research/frictions/` has 2 reports (5 open items: 0 blocker, 2
high, 2 medium, 1 low). Gaps: no unified `scripts/scoreboard.py` or committed
`research/scoreboard.json` regression baseline, no contestant-written token
ratio tracking, no automated friction-report summarizer, no `--fast` scoreboard
gate in `scripts/check.sh`, and `known_traps.cp` still has 5 open traps.
**Bar.** 500+ Rust-gated ports; `scripts/scoreboard.py --fast --check` in
`scripts/check.sh --fast` against `research/scoreboard.json`; full-mode perf
and interpreter baseline tracked; zero open blocker frictions; P0 bug corpus
>= 90% handled.
**Next.** Build `scripts/scoreboard.py` + `scripts/test_scoreboard.py` (per-port
gate, contestant tokens, emitted bytes, max-size C++ vs original runtime/RSS,
interpreter time, friction summary over `research/frictions/*.md`, baseline
diff failing on regressions, `--fast` mode); generate and commit
`research/scoreboard.json`; wire `--fast` into `scripts/check.sh`; add static
zero-cost and bug-corpus metrics to the scoreboard.
