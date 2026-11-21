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
code. Enums order by variant, then payload (decision 28.2).
`for f in fields(x)` and `for (a, b) in fields(x, y)` walk struct, record
and tuple fields, unrolled in lowering (28.1); a variadic `..xs: Ts` is the
tuple of its arguments (28.3); `read<(Int, Int)>()` takes type arguments
on a lowercase fn (28.4); `lazy d: () -> T` takes an expression and passes
`() => e` (28.5; free fns, generic fns and methods). A projection is
already a writable place through a method: `fn at(mut self, i: Int) lends
mut Int: mut self.xs[i]` then `b.at(1) += 10` works. Enum variants are not
reflected yet. Not built: postfix `x?` expression-extent lifting,
opaque `struct List<T>` / struct type-param
defaults / one-sided range types /
`@droppable` / `labels(...)` / `Str(len:)`, `@derive` (`E0501`), `print`
and `read` as library fns over packs, `assert`/`debug`/`??` as library fns
(`assert` also feeds the range prover its condition, so moving it needs that
as a general rule), `Niche`, library diagnostics. 0 of the 35 `design/spec/*.md`
```cp blocks pass `cpl check`; most stop at parse or stdlib gaps.
**Bar.** Every code block in the design spec passes `cpl check`; the stdlib
needs no compiler knowledge beyond the documented intrinsics; each
mechanism above has programs that the interpreter and the C++ agree on.
**Next.** Postfix `?` extent; remaining 0.3 syntax (opaque `struct`,
type-param defaults, one-sided ranges, `@droppable`,
`labels`, `Str(len:)`); variant reflection and derives as trait
defaults on `fields`; `print`/`read` on variadic packs; `assert`/`debug`/`??` in the stdlib; `Niche`; library diagnostics.
**Coordination.** Core-extensions lane (`49fcf312`) owns derives, variadic packs,
and lazy parameters; this lane (§1 core syntax/semantics) owns `Self` in
trait/impl scope, associated types, `trait`/`impl` headers, opaque structs,
type-param defaults, one-sided range types, postfix `?` extent, and remaining
0.3 syntax.

**Spec-block scoreboard** (`scripts/spec-blocks.py`, **2/35** passing on `design/spec/*.md`: `13-arrays.md#0`, `23-optionals.md#2`):

| Block | First blocker (`cpl check`) | Owner |
|---|---|---|
| `00-layers.md#0` | `E0103`: `fn Domain.of(n: 1..) -> (domain D: Index)` (dotted fn name + `1..`) | §1 core syntax + §2 domains |
| `00-layers.md#1` | `E0103`: `impl<D: Counted> Read for D uses io:` (`uses` clause on `impl`) | §1 core syntax (contexts) |
| `12-scoped-types.md#0` | `E0106`: `let (domain Piece, pieces) = ...` (`domain` binder in pattern) | §2 domains + §1 syntax |
| `13-arrays.md#0` | **PASS** | §1 core syntax |
| `14-functions-traits.md#0` | `E0107`: ellipsis `...` body placeholder in `@memo fn count(...)` | spec snippet (placeholder `...`) |
| `14-functions-traits.md#1` | `E0107`: ellipsis `...` body placeholder in `@memo fn count(...)` | spec snippet (placeholder `...`) |
| `16-state.md#0` | `E0107`: `with io(base: 1)` bare context statement | §1 core syntax (contexts) |
| `16-state.md#1` | `E0102`: `.read` enum-variant shorthand default (`count: Count = .read`) | §1 core syntax |
| `17-contexts.md#0` | `E0107`: `context io:` declaration | §1 core syntax (contexts) |
| `17-contexts.md#1` | `E0107`: `with io(base: 1)` bare context statement | §1 core syntax (contexts) |
| `17-contexts.md#2` | `E0103`: `fn read<T>() -> T uses io` (bodyless signature + `uses`) | §1 core syntax (contexts) |
| `21-index-spaces.md#0` | `E0103`: `impl Index for (A, B):` / `impl Index for Counted` header | §1 core syntax + §2 domains |
| `21-index-spaces.md#1` | `E0201`: `e` unbound (fragment assuming prior declarations) | spec snippet (fragment) |
| `21-index-spaces.md#2` | `E0204`: no type or domain named `Bits` | §2 domains / stdlib |
| `23-optionals.md#0` | `E0205`: `let depth = depth!` (fragment without prior `depth`) | spec snippet (fragment) |
| `23-optionals.md#1` | `E0201`: `dp` unbound (fragment without prior `dp`/`a`) | spec snippet (fragment) |
| `23-optionals.md#2` | **PASS** | §1 core syntax |
| `25-io.md#0` | `E0204`: no type or domain named `Out` (`&mut Out` now parses) | stdlib (I/O) |
| `25-io.md#1` | `E0201`: `parse_int` not in scope (`impl Read for Int`) | stdlib (I/O) |
| `27-algebra-segtrees.md#0` | `E0204`: `D` unbound (fragment without prior `D`/`M`/`A`) | spec snippet (fragment) |
| `27-algebra-segtrees.md#1` | `E0204`: `Monoid` is a struct, not a trait | stdlib (algebra) |
| `30-stress.md#0` | `E0107`: `@alt(..., n: ..=8)` + `...` body placeholder | stress lane + spec snippet |
| `31-interaction.md#0` | `E0201`: no `Wire` in scope | stdlib / interaction lane |
| `31-interaction.md#1` | `E0204`: no trait named `Print` (`impl Read + Print` now parses) | stdlib (I/O) |
| `31-interaction.md#2` | `E0204`: no trait named `Limited` (then `labels(asks)`) | interaction lane + §1 syntax |
| `31-interaction.md#3` | `E0107`: `...` body placeholder in `@gen fn hidden(...)` | stress lane + spec snippet |
| `32-graders-bundling.md#0` | `E0107`: `...` body placeholder in `@cpp(...) fn pdep(...)` | codegen lane + spec snippet |
| `40-signatures.md#0` | `E0204`: no type or domain named `Ordering` (`trait Ord(Eq)` & `&mut` now parse) | stdlib |
| `40-signatures.md#1` | `E0204`: no type or domain named `Seq` (then `where` bounds on methods) | stdlib + §1 core syntax |
| `40-signatures.md#2` | `E0103`: bodyless `fn push(...)` in `impl List<T>` (opaque `struct List<T>` now parses) | §1 core syntax |
| `40-signatures.md#3` | `E0107`: `id: Self` associated const/field in `trait Monoid` | §1 core syntax |
| `40-signatures.md#4` | `E0105`: `(type Z: Field)` scoped-type return (`2..` one-sided range now parses & checks) | §2 domains / §1 syntax |
| `40-signatures.md#5` | `E0103`: bodyless `fn undirected(...)` in `impl Graph<V, W>` (`W = ()` default now implemented) | §1 core syntax |
| `40-signatures.md#6` | `E0103`: bodyless `fn len(self) -> Int` in `impl Str` (opaque `struct Str` now parses) | §1 core syntax |
| `40-signatures.md#7` | `E0103`: `fn read<T: Read>() -> T uses io` (bodyless fn + `uses` + variadics) | §1 core syntax + core-extensions |

## 2. Index spaces and fresh domains (`plan/12`, `plan/19`)
**North star.** Every index-space bug in the P0 list is unwritable: indices from one
space never reach another, off-by-one and base mix-ups are type errors or
one-local-edit conversions, and every range check the compiler can prove
disappears. Runtime-sized and fresh spaces (compressed values, components,
rooted trees, subsets, grids, intervals, cuts) compose freely, cost nothing
over raw `int`, and are shown in the contestant's own base by every tool.
**Where we are** (updated 2026-11-21). Counted domains, ranged reads,
arbitrary runtime-expression domains (`domain V = read(1..=n)`,
`domain C = compress(xs)`), existential tuple and record returns
(`fn f(..) -> (domain P, [P]Int)`, `let (domain P, part) = f(..)` including
multiple existential domains and renamed local domains), recursive
save/restore (`cpl::DomGuard` + interpreter restore stack), fresh-domain
escape diagnostics (`E0313` on return, outer assignment, outer container
store, and closure capture), heap storage for runtime-sized arrays
(`std::vector` / `cpl::Grid`), and runtime-bounded fields in `read()`
records (`w: 0..=m`) all work across checker, interpreter, C++ codegen,
formatter, linter, LSP, and docs. Symmetric triangular tables `[Sym<V>]T`
(indexed by `adj[u, v]` or `adj[(u, v)]`, packed into `n*(n+1)/2` cells),
derived pair domains (`domain P = V.pairs`), and zero-overhead domain iteration
helpers (`V.after(i)`, `V.before(i)`, `for (i, j) in V.pairs`) also work across
checker, IR, interpreter, C++ codegen, trace, editor syntax, and docs. Seven
ports converted (`ucup4s25c`, `cf2222f`, `cf2101b`, `cf2268c`, `cf2150f`,
`cf2147h`, `cf2129b`). Remaining: algebraic product domains (`domain State = (V, K)`,
`research/frictions/cf2178h.md`), graph edge domains (`domain E = edges(V)`),
eliminating redundant runtime bounds checks when indexing `[0..=m]` by a `0..=m`
record field in C++ codegen, and explicit `domain` field syntax on
domain-carrying structs.
**Bar.** Settled semantics in `plan/OPERATING.md` implemented across
checker, interpreter and C++; three ports converted. (Met: 7 ports
converted and passing `gate.sh`.)
**Next.** Algebraic product domains (`domain State = (A, B)` with packing/unpacking,
from `dream/cf2178h.cp`) and `edges(V)` derived domains; propagate runtime
range-field facts (`w: 0..=m`) into C++ bounds elimination (`cpl-codegen/src/facts.rs`);
migrate remaining stdlib existential signatures to tuple syntax where appropriate;
explicit `domain` field syntax for value-owned structs.

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
two ports. Rollback union-finds share a typed `Mark`; `DsuOf<V, T>`
rolls a value per set back with the sets and `ParityDsu<V>` keeps sides
(`cf2222f`, `cf2237h` pass 150 random cases each against the originals). Present but thin: RNG (seeded stream, no pick/perm/sample), bitsets (set algebra, shift,
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
wave-11 frictions (`research/frictions/cf2237h.md`, `cf2222f.md`): an
interval run map;
offline divide and conquer over time; predicate counting; link-cut tree.

## 5. Diagnostics and front end (`plan/30` 3.7)
**North star.** Every error is the message an expert coach would give: what is wrong,
why, and the exact fix, as an applicable edit, in the contestant's terms.
Recovery is so good that a half-written file still gives full types, hover
and completion; there are zero cascades; every planned feature names itself;
and libraries can attach their own errors. Measured against a benchmark of
real contestant mistakes, the first error points at the real cause almost
always.
**Where we are.** Statement recovery keeps the names a broken declaration
declares (a broken fn header is bound with an unknown type), and an
unreadable character no longer drops its line. Broken-ports benchmark
(`cpl-cli/tests/recovery.rs`, six single edits per port): 0.04 cascades per
copy, first error on the right line 99.8%, ratchet at 0.05 per copy.
Planned 0.3 forms go through one table (`cpl-syntax/src/planned.rs`, `E0504`
"not implemented yet: ... (see design/...)"); two entries so far. Golden
rendered diagnostics in `cpl-cli/tests/diagnostics/`.
**Bar.** Golden diagnostics for every error code; a broken-ports benchmark
with first-error accuracy and cascade counts.
**Next.** Count cascades by root cause instead of "every error after the
first"; struct/enum headers that fail to parse should also bind with an
unknown shape; fn stubs `-> T: ...` and the rest of `design/DELTA.md` into
the planned table or implemented; quick fixes on every syntax error;
top-level tuple-destructured names in LSP completion.

## 6. Interpreter, debugger and traces (`plan/50`)
**North star.** Doubt becomes evidence in seconds. A time-travel debugger over
DAP in every editor that steps backward and forward through any run, including
a run replayed from a failing judge input; an HTML trace that shows only what
matters, filtered by function, multitest case, watched variable, call depth,
index space, or ring buffer before a crash; snapshots that let you iterate on
one stage; every runtime error stops with the concrete values, the call path,
the test case number, and the input line that caused it; never a segfault; the
interpreter fast enough to run full-size tests: within 10x of C++ `-O2` on
ports, and ultimately within 5x.
**Where we are.** Debug interpreter with runtime checks (36 trap sites in
`trap_parity.cp`, 5 open in `known_traps.cp`) and `debug(...)` output;
`cpl run --trace` writes a self-contained HTML trace (`tools/trace/index.html`,
37 Node unit tests, not yet browser-tested) and `cpl stress --trace` aligns
solution vs `@brute` on `print`/`debug` checkpoints. Gaps against `plan/50`:
(1) `Trap` now carries call stacks (summarized past 32 frames), multitest case
number, and concrete values on arithmetic/bounds/domain/input traps (golden
tests in `crates/cpl-cli/tests/runtime_errors.rs`); (2) trace filtering now
supports `--trace` globs/exclusions, `--test`, `--watch`, `--last` ring buffer,
`--depth`, and 1-based `Bound(d)` cut labels (`crates/cpl-cli/tests/trace_filters.rs`);
(3) `cpl dap` serves a time-travel Debug Adapter Protocol server over stdio
(breakpoints, call stacks, domain-labeled locals/globals, `next`/`stepIn`/`stepOut`/`continue`,
backward `stepBack`/`reverseContinue`, hover/watch `evaluate`, replay of failing inputs,
exception stops on traps; `crates/cpl-cli/tests/dap.rs`) wired into `editors/vscode`
(`type: "cp"`, `cp.debug`); (4) interpreter speed on compute-heavy ports is 44x–261x slower than C++ `-O2`
after unboxing `?Int`/1-field `Int` structs and removing hot-path allocations
(`dph` 884 -> 693 ms, `dpl` 542 -> 483 ms, `dpk` 770 -> 728 ms, `dpm` 1288 ->
1028 ms; closing the gap to <= 10x needs a typed bytecode VM);
(5) no stage snapshots (`--stage`) or interactive exchange trace.
**Bar.** Every runtime error class has a golden message with values and call
path; trace filtering and index-space views tested end to end; DAP step
debugger wired into VS Code and tested over stdio; interpreter within 10x of
C++ `-O2` on ports (north star 5x).
**Next.** (1) Typed register bytecode VM in `cpl-interp` (unboxed slots, resolved
locals, monomorphized calls) to bring interpreter speed to <= 10x of C++ `-O2`; (2) stage snapshots (`--stage`) and interactive exchange traces;
(3) headless-browser test for `tools/trace/index.html`.

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
(`(u: V, v: V, w: W)`) was generated as random pairs, not a tree; there
were no simple or connected graphs. `Perm<V>` (`std.seq`), `Tree<V>`,
`SimpleGraph<V, E>`, `ConnectedGraph<V, E>` and `Dag<V, E>` (`std.graph`)
are now stdlib aliases every tool accepts; stress makes a weighted tree as
a tree with its weights, and the three graphs with no loop or repeated
edge, connected or acyclic as named, and shrinks each by deleting a vertex
with its edges and every value read over its vertices renamed to match
(`deleting_a_graph_vertex_renames_the_vertices_read_beside_it`). The seeded-bug benchmark
(`bench/stress/`, `tests/seeded_bugs.rs`) runs in the gate: 9 bugs
(sentinel, item reuse, memo bound, heap direction, off-by-one under a
recursive `@brute`, multitest leak, DP order over permutations, update
order over a weighted tree, a reachability step over a graph that checks
one end of an edge), all shrunk to their known minimal input, median
0.49 s. A recursive `@brute` works; a
failure that needs several tests shrinks each of them; a smaller size drops
elements from nested arrays (a list of permutations), not just top-level
ones; letter bounds (`'A' + k`) evaluate. Values that only fail together
(two tests that must repeat one number) still shrink one at a time, so they
stop at the first such value, not at 1.
**Bar.** A seeded-bug benchmark in the gate: classic bugs injected into
correct solutions, each found and shrunk to a known minimal case, with the
median time to a minimal counterexample at most 10 s (`GOALS` 4).
**Next.** Lowering equal values together; generators keyed to the
stdlib declarations, not their names; worst-case and anti-hash
generators; grid and string-family shapes (`[R, C]Char` mazes, palindromes).

## 8. Formatter, linter, profiler, coverage, bench, doc (`plan/45`)
**North star.** Every tool a contestant reaches for exists, is instant, and
understands the language's ideas, and each answers a doubt in one command
from the facts the compiler uses: a formatter with one style that never
moves twice; a linter whose every rule comes from a real friction or bug,
with zero false positives on the corpus and a fix where one exists; a
profiler that explains time and memory per line and per index space, in the
interpreter and in the judge's build, with predicted (`@cost`) versus actual
complexity; coverage of lines and branches the samples never reach;
benchmarks against limits on worst-case inputs; a documentation generator;
a REPL; and refactorings through the LSP. Every new language feature arrives
with its line in each of these tools.
**Where we are.** fmt, lint (8 rules), doc and bench exist with tests.
`cpl profile` counts each statement's starts and time in the interpreter
and, with `--native`, in the C++ built with a counter at each statement;
the counts agree on all 183 programs of `crates/cpl-cli/tests/programs`.
`cpl cover` lists unreached lines and one-way `if`s over a problem's cases.
fmt is stable on every parseable `.cp` file at any depth under `proto`,
`examples` and `stdlib` (`formats_the_whole_corpus`; 3 files do not lex). lint on the 1,031
corpus files that check: 535 warnings (424 `L0414`, 51 `L0201`, 48
`L0203`, 6 `L0412`, 5 `L0101`, 1 `L0202`). Of the 424 `L0414`, 337 sit
on an index the C++ really checks (`cpl::at`); the other 87 are in fns
the build leaves out (library files built alone, unused helpers), so
none is known false, but no test holds lint to the code generator yet. No lint is yet mined from `scripts/friction_lint.py`.
Profile time is not yet compared with `@cost`.
**Bar.** Each tool has tests and a doc page; lint has zero false positives
on the corpus; fmt is idempotent on every port; interpreter and native
profiles agree on every count.
Taste lints L0501–L0506 back the `plan/PORTING.md` checklist (search
loops, `let _ =` discards, symmetric writes, loop reductions, index-only
loops, repeated bounds and moduli), each with a doc entry and a test from
a real port; their fixes change 136 corpus files and break none.
**Next.** A test that every `L0414` is an index
the C++ checks (codegen to report its checked indexes); lints from the friction
detectors (`wa_trap` first); profile against `@cost` and the time limit.

## 9. Judges and submission (`plan/70`)
**North star.** What passed here passes on the judge, and the contestant
never thinks about it. Each named judge (Codeforces, AtCoder, CSES, ICPC
on DOMjudge, Kattis, DMOJ, USACO, IOI-style graders, a generic floor, or a
project's own) is a profile fixing compiler, flags, platform, stack, widths,
libraries and limits; function-style and interactive judges included. `cpl build` writes one deterministic, readable C++17
file for it, within its source limit, free of UB, quick to compile; local
runs (`run --release`, `test`, `stress`, `bench`) build with the same flags
and stack; the recursion contract and the size report say what the judge
will do before it does. Every port is compiled, run and size-checked under
every profile on every gate, on the judge's own compiler version, and a
submit command checks samples, size and profile before sending.
**Where we are.** Seven built-in profiles (`crates/cpl-judge`), chosen by
`--judge`, a `// judge:` line or `cp.toml`, which can also change or add
profiles; `cpl judges` lists them with their sources (the Codeforces facts
are unconfirmed). The profile sets the source limit (warning at 75%), the
header stamp, `__int128` support and whether deep recursion needs its own
256 MB stack. All 257 ports fit every profile (max 59.4 KB) and compile
with every profile's flags (1,799 compiles, `--full`); a million nested
calls run on each judge's stack and agree with the interpreter. `run --release`, `test`,
`stress` and `hunt` build a `.cp` file with its judge's flags (those g++
here accepts) and run it under `ulimit -s` of the judge's stack; `bench`
and `profile` still use `g++ -std=c++17 -O2`. The compilers here are g++
11, clang 14 and MinGW 10, not the judges' versions.
**Bar.** Every profile: every port builds within its limit, compiles with
its flags (as accepted here), and the deep-recursion run passes on its
stack; each fact in a profile cites a source or says it is unconfirmed.
**Next.** `bench` and `profile` with the profile's flags; the judge's
time and memory limits on local runs; a size report (bytes per part) and a readable minimisation rung
before dropping the source header; clang `-Wunused-value` in 6
ports; USACO and grader profiles; judge-version containers.

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
rename within a file, document symbols, document highlights, folding
(blocks, comment runs, imports), semantic tokens (each resolved name as a
type, fn, method, field, variable or parameter, marking declarations and
immutable bindings), workspace symbols (fuzzy, over open files and the
public names of every module they use), formatting, completion (members
after `.`, names in scope, `use std.`), signature help, inlay hints
(unwritten types, parameter names of literal arguments) and quick fixes
from diagnostic fixes; a panic becomes an error reply plus an
internal-error diagnostic. Unit tests in `src/tests.rs`; `tests/latency.rs`
measures 6.1 (492 lines, debug build: keystroke to diagnostics median
6 ms, hover 0.06 ms, completion 1.8 ms). The VS Code extension has a
TextMate grammar, snippets, the client, Run/Stress/Trace lenses, live
sample values, counterexample overlays and a trace webview; its unit
tests run in `--full`, its extension-host suite in no gate.
Hover shows the index space and its bound for arrays and positions
(`xs: [V]Int`, `domain V = 0..n, n <= 100`, storage per element and in
total), appends the declaration's `///` doc, which in the stdlib states the
cost, and definition on a field or method jumps to it, in the stdlib too.
Gaps against `plan/60`: no computed cost on hover; no proven range of an
expression; no inlay hints for index spaces or ranges;
 no cost or
overflow warnings; rename is one file; no refactorings or "fix all"; no
expected-type ranking or auto-import; run/test/trace exist only in the VS
Code client; no JetBrains plugin, no fixture tests, fuzzer, DAP or
tree-sitter grammar, so Helix has no highlighting and Zed no setup.
`editors/README.md` documents Neovim (runtime folder: filetype, Vim syntax,
`cpl lsp` autostart), Emacs (`cp-mode` with eglot and lsp-mode) and Helix
(`languages.toml`); `editors/smoke.sh` checks their keyword lists against the
compiler and drives each against the real server (Neovim: diagnostic and
hover; Emacs: mode, faces, eglot entry; Helix: `--health`), in `--full`.
**Bar.** LSP fixture tests per `plan/60` 6.7 for every request; each editor
integration built and smoke-tested in `scripts/check.sh --full`.
**Next.** Fixture tests; hover index spaces and ranges; inlay hints for
index spaces and ranges; semantic tokens; server code lenses and commands
for run, test and trace; JetBrains plugin; tree-sitter grammar (Helix
highlighting, a Zed extension, Neovim tree-sitter); fuzzer.

## 11. Docs, site and playground (`plan/90`)
**North star.** The language is writable from the docs alone: a one-afternoon tour, a
guide with one chapter per idea, a complete reference, a stdlib reference
with costs and tested doc examples, an error index with one page per
diagnostic and lint code (each with verified trigger and fix), a cookbook
built from hundreds of real validated ports, interactive lessons that teach
by solving, and a browser playground that runs the interpreter, C++17
preview, trace viewer and a judge — every single code block and example
executed in the gate so documentation never drifts.
**Where we are.** Static site generator (`site/build.py`, 26 chapters + 63
individual error/lint pages + playground + trace = 91 HTML pages: 10-lesson
course with 51 checked exercises, 15 reference pages including canonical
`docs/one-page-cheat-sheet.md`, `errors.html`, `cookbook.html`, and
`ports.html`), WASM playground (23 presets with verified sample cases,
URL-hash `#share=` links, multi-case sample runner, verbatim CLI `stderr`
rendering, and `Playground::check` warning exit-code parity), canonical trace
viewer, and offline bundle (`dist/cp-lang.html`, ~2.0 MB < 2.6 MB cap).
105/105 doc `cp` fences, 22/22 `site/examples/*.cp`, 10/10 course write-code
refs, 63/63 diagnostic/lint codes, 22/22 cookbook recipes, 71 gallery ports
(19 with side-by-side CP / dream / C++ panes from `research/PORTS.md` and
`proto/ports/<group>/dream/`), 24/24 non-prelude `stdlib/*.cp` modules, and
playground WASM-vs-CLI parity (`crates/cpl-wasm/tests/parity.rs` +
`site/test_playground_parity.py`) are verified in `scripts/check.sh --fast`
via `site/check_docs.py`. Writable-from-docs harness
(`scripts/writable-from-docs.py` + `scripts/test_writable_from_docs.py`,
report in `research/writable-from-docs.md`) audited 10 unseen statements from
`corpus/statements/` (`abc380b`–`abc387c`), raising `cpl check` pass from 7/10
to 8/10 after closing 5 docs gaps.
Gaps: `cpl doc stdlib` still has undocumented items and 0 doc-comment
examples; playground does not yet have a generated C++17 diff view.
**Bar.** A cold reader writes a correct solution from the docs alone; every
doc `cp` block, example fixture, cookbook recipe, ports gallery entry, stdlib
module, error-index trigger/fix, and playground CLI parity test passes in
`scripts/check.sh --fast`; 100% of diagnostic/lint codes have an error-index
page; offline bundle stays under 2.6 MB.
**Next.** Add docstrings and tested `cpl doc --test` examples across
`stdlib/*.cp` and wire into the docs gate; expand the writable-from-docs
benchmark to harder Codeforces/AtCoder D-F problems and fix remaining gaps;
add generated C++17 diff view to the playground.

## 12. Validation: ports, scoreboard, differential testing (`plan/80`)
**North star.** Thousands of hard accepted solutions across CF, AtCoder,
ICPC/UCup, IOI, NOI and DMOJ rethought in the language, discovered
automatically and differential-tested against their originals (interpreter,
sanitized C++ and `-O2` C++), measured on every push in a single deterministic
scoreboard (`scripts/scoreboard.py` + `research/scoreboard.json`) that blocks
regressions across every goal: correctness (gate pass, zero trap-parity drift),
expressiveness (contestant-written tokens vs raw and library-stripped original,
median <= 0.6x, none > 1.0x), performance (runtime p50 <= 1.1x / max <= 1.5x,
peak RSS p50 <= 1.0x / max <= 1.2x on max-size inputs, interpreter within 10x
of C++, north star 5x), submission viability (emitted C++ <= 64 KB, p95 <= 32 KB, warning-free
GCC/Clang), and friction (structured `research/frictions/*.md` reports with
zero open blockers and `friction_lint` zero `wa_trap`). Backed by a replayed
P0 bug corpus (>= 90% prevented or trapped), static `sizeof` and `-O2` assembly
zero-cost proofs, and real contest use by strong contestants showing faster
accepted solutions and fewer wrong attempts than in C++.
**Where we are.** Unified scoreboard (`scripts/scoreboard.py` + `scripts/test_scoreboard.py`,
16 unit tests) and committed full-mode baseline (`research/scoreboard.json`) are live and
wired into `scripts/check.sh --fast` (`--fast --check --quiet`, ~5 s with debug `cpl`).
At `46be8a59`, **275/275** active `proto/ports/ecnerwala/` ports pass `--full` (16 interactive,
4 custom checkers), automatically picking up concurrently pushed ports:
contestant-written tokens median 0.422x raw / 0.504x library-stripped (0 > 1.0x raw, 1 > 1.0x
stripped: `ucup5s05f` at 1.054x), dream gap across 27 `dream/*.cp` ports `cp=11,825` vs
`dream=7,639` (median 1.326x, p90 2.532x, max 4.676x on `cf2187f1`; strict per-port ratchet
so no port's `cp/dream` ratio may grow), emitted C++ median 9,893 B / p95 26,248 B / max
59,202 B (`cf2255f`; 3 > 48 KB, 0 > 64 KB), `friction_lint` 0 `wa_trap` / 6 `real_gap` /
54 `stale_port`, and `research/frictions/*.md` 56 files / 173 open items (0 blocker, 34 high,
64 medium, 44 low, 2 closed; every port over 1.5x now has a performance friction naming its
codegen/stdlib/layout cause).
Full-mode perf across all 275 ports (`research/scoreboard.json`, 258 measured, 122 >= 10 ms):
runtime **p50 0.993x** (goal <= 1.1x: **PASS**), p95 3.546x, **max 310.196x** (goal <= 1.5x:
**FAIL**, 20 > 1.5x), peak RSS p50 0.930x / p95 2.356x / max 6.195x (`cf2268c`).
Interpreter vs C++ `-O2` on matched inputs (271/275 ok; 159 >= 20 ms): **p50 7.17x** (bar
<= 10x: **PASS**, north star <= 5x: **FAIL**), **max 117.80x** (`cf2262e`, 61 > 10x);
ratcheted at >10% regression on `p50` and per-port (`>= 20 ms`).
Taste scores from `research/reviews/<name>.md` are wired into `scripts/scoreboard.py`
(`totals.taste`) and `research/PORTS.md` (`Taste` column; ready for `research/reviews/`).

Worst 10 runtime ratios (`research/scoreboard.json`, full mode at `46be8a59`):

| Port | Ratio | C++ / Orig | Input | Cause (`research/frictions/`) |
| --- | --- | --- | --- | --- |
| `cf2096e` | 310.20x | 2078.3 / 6.7 ms | `gen size=25000` | `s.rev.items` re-reverses `s` every iteration ($O(n^2)$) |
| `ucup4s25h` | 26.06x | 221.8 / 8.5 ms | `gen size=25000` | `s[lo..hi]` lowers to $O(n^2)$ char concatenation |
| `cf2187d` | 15.95x | 116.1 / 7.3 ms | `gen size=1000` | Bitset knapsack overhead |
| `cf2245f` | 8.65x | 1782.3 / 206.2 ms | `gen size=1000` | 64-bit `Mod<P>` + checked 2D DP in $O(n^3)$ loop |
| `cf2255e2` | 4.48x | 241.7 / 54.0 ms | `gen size=25000` | Idiomatic container/codegen overhead |
| `cf2245a` | 4.40x | 50.0 / 11.4 ms | `gen size=25000` | `s[lo..hi]` $O(k^2)$ char concatenation |
| `cf2261g` | 3.55x | 357.4 / 100.8 ms | `gen size=25000` | Un-niched `Row<?Int, 2>` + per-node temp `vector` |
| `cf2147f` | 3.38x | 679.1 / 200.8 ms | `gen --max` | `std::function` in `LazySegTree` + un-niched `?Node` |
| `cf2066e` | 3.17x | 6162.4 / 1945.9 ms | `gen --max` | `SortedBag` lacks $O(1)$ ordered cursor |
| `cf2178h` | 3.03x | 151.2 / 49.9 ms | `gen size=200` | `dijkstra` / `?Int` 16 B layout overhead |

Full-mode `--check` ratchets any >= 10 ms port whose runtime ratio regresses > 10%.
`gen/NAME.py --max` is preferred when present (added to `cf2066e`, `cf2147f`, `cf2150g`).
`research/PORTS.md` (via `scripts/ports-page.py`) displays cached `Runtime` and `Taste`
columns without re-running. Constructive-problem checker hook
(`checkers/NAME.py INPUT OUTPUT ANSWER`) is live in `proto/ports/ecnerwala/gate.sh`,
`proto/ports/ecnerwala/README.md`, and `scripts/scoreboard.py`. Legacy: 30 `wave2` ports and
DMOJ registry (`bench/corpus-500-registry.json`: 594 unique problems). Gaps: `known_traps.cp`
has 5 open traps; 20 ports exceed the <= 1.5x runtime ceiling; 61 ports exceed the 10x
interpreter-ratio bar; static `sizeof` / `-O2` assembly zero-cost metrics and replayed P0
bug corpus are not yet wired into the scoreboard.
**Bar.** 500+ Rust-gated ports; `scripts/scoreboard.py --fast --check` green in
`scripts/check.sh --fast`; full-mode perf and interpreter baseline refreshed regularly with
runtime p50 <= 1.1x / max <= 1.5x; zero open blocker frictions; P0 bug corpus >= 90% handled.
**Next.** Add static `sizeof` / `-O2` assembly zero-cost metrics and replayed P0 bug-corpus
coverage to `scripts/scoreboard.py`; refresh `research/scoreboard.json` after each major wave
and perf fix (investigating `cf2096e` 300x and `ucup4s25h` 27x outliers); add checkers for
remaining constructive ports; close the 5 open traps in `known_traps.cp`.
