# The areas of CP Lang

CP Lang is one language plus the tools around it. This page lists every
area: what it is, where it lives, where it stands, and what comes next.
Goals and rules are in `AGENTS.md`.

How this file is laid out: everything above "For agents" is the short,
human-readable map and must stay that way (one table, then a few lines
per area). Detail, evidence and history go in "For agents" at the
bottom, under the old numbering.

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
| 14 | Validation and scoreboard | `corpus/problems/`, `scripts/scoreboard.py`, `scripts/corpus-tests.py` | Ports gated against their C++ originals, measured on every push |

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
or below 1.0x the originals, never above 1.5x, and always reachable to
1.1x by more manual tuning in the language; tight layouts with niches and
narrow storage; one file, every solution under 32 KB.

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

**Now.** Eight judge profiles (compiler, flags, stack, source limit,
usual time and memory), USACO among them; every port fits and compiles
under every profile; a warning over 32 KB, an error over 64 KB;
`cpl submit --copy` builds, runs the samples on the judge's build and
copies the file.

**Next.** Line markers that map judge errors back to source; IOI grader
profiles.

## 11. Editor and LSP

**Goal.** One language server giving every editor everything the compiler
knows, including storage sizes, layouts and peak memory per judge profile.

**Now.** Diagnostics, hover, definition, references, rename, completion,
semantic tokens, fixture tests for every request, a fuzzer; hover shows
proven ranges, in-bounds proofs, overflow inputs, loop counts and a fn's
estimated cost; VS Code, JetBrains, Zed, Neovim, Emacs and Helix setups,
each tested in the gate.

**Next.** Estimated cost for more fns (37 of the 1,332 fns in
`corpus/problems` ports, now counting the fns each one calls); memory the bounds allow, as a code lens against
the judge's limit, for locals and lists filled other than by counted
`push`es too (top-level arrays and pushed lists only today);
publish the Zed extension.

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

---

## For agents: detailed audit

The per-area north star, status, bar for done and next steps, with
evidence. Its twelve sections predate the map above (index spaces were
folded into core language; spec, proofs and corpus were added).

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

### 1. Core language and types (`plan/1x-*`)
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
methods), a user struct's optional borrows a field's niche (no flag), and a
library can say in its own words why a call does not type-check.
**Where we are** (audited 2026-11-21 at `815c21e4`+). Working: `repeat`,
`loop`, one-line enums (including explicit integer discriminants `enum Sign: neg = -1, zero = 0, pos = 1` with `Int(s)`/`Sign(n)` conversion) and structs (paren and colon forms), multi-binding `let`/`var`
(tuple and scalar-fill broadcast `var a, b, c: Z = 0`), typed tuple patterns
`let (a: T, b: U) = ...`, named mutable aliases `let st = mut a[i]` (with once-evaluated index temps), unary `+`, counted domains, `read(lo..=hi)`,
`read<[n]T>()` / `read<1..=3>()` (canonical `f<T>(...)` with `L0204` warning + `cpl fix` codemod for legacy `f::<T>(...)`), `x with field: v` record/struct update,
typed lambda params `(x: T) => ...` (typechecked in `cpl-check` and `cpl-ir`), comparison operator sections (`by: <`, `by: >`, etc.), `x is name` pattern-test expressions (`x is v and v > 5`, `if x is v:`, `while cur is n:`), `--- multitest ---` / `--- with multitest() ---`
wrap-lines (provisional; `@!multitest` remains canonical in ports), enum-indexed arrays `[Sign]T`, `Mod<1e9 + 7>` additive value type args,
multi-clause `for a in ..., b in ...:`, `guard let ... else:`, unit-valued
top-level `return print(...)`, `D.count`, `for ... if`/`where`,
`if let`/`while let` (statement and `if let ... else:` expression) with `where`
and multi-binding `if let a = x, b = y`, `let ... else`, `if x max= y:`,
`?T` arithmetic and bitwise lifting (`+ - * / % ** & | ^ << >>`, unary `-`, `~`),
`max(?T, T)`, `max=`/`min=` on optionals, `[A, B]T`, `[(A, B)]T` and `[A][B]T`,
`pairs()`/`with_prev()` in `std.seq`, `Self` (including dotted range bounds `Self.min..=Self.max`)
and associated types in traits. Dream-port audit (`proto/ports/*/dream/*.cp`):
**1/45** passing `cpl check` (`cf2069c.cp`, up from 0/29; 22 of the 44 remaining are now first-blocked on stdlib rather than core syntax). Structs get `Eq`, `Ord`, `Hash`, `Show` and
`Read` from the compiler, structurally, with `impl Show/Eq/Ord` as hooks
(`cpl-ir` `show_fn`/`eq_fn`/`lt_fn`); that is stdlib privilege, not library
code. Enums order by variant, then payload (decision 28.2).
`for f in fields(x)` and `for (a, b) in fields(x, y)` walk struct, record
and tuple fields, unrolled in lowering (28.1); a variadic `..xs: Ts` is the
tuple of its arguments (28.3); `read<(Int, Int)>()` takes type arguments
on a lowercase fn (28.4); `lazy d: () -> T` takes an expression and passes
`() => e` (28.5; free fns, generic fns and methods); `fields(e)` on an enum
walks the active variant's payload (28.6). `std.derive` has `eq_fields`,
`lt_fields` and `show_fields` on `fields` for an `impl` to call (28.7); the
default derive is not on them yet because hooking every struct grew emitted
C++ past the 5% gate. An `impl` gives a built-in type methods, so `Hash`
and `hash_fields` are library code (28.8); `P(e for f in fields(..))`
builds a struct field by field, so `read_fields` is too (28.9). `trait Gen` has static fns (`T.gen(mut r)`), and a generic call takes a return-only type parameter from the type wanted, so `gen_fields` is one line too (28.12). `read()` of a struct calls its `impl Read`, inside arrays and other structs too (28.13). A projection is a writable place through a method
(`b.at(i) += 10`, `b.name(1) = ...`, a read-only `lends` rejected), pinned by
`programs/lends_through_methods`. Not built: postfix `x?` expression-extent lifting,
opaque `struct List<T>` / struct type-param
defaults / one-sided range types /
`@droppable` / `labels(...)` / `Str(len:)`, `@derive` (`E0501`), `print`
and `read` as library fns over packs, `assert`/`debug` as library fns (`??` stays core, 28.14)
(`assert` also feeds the range prover its condition, so moving it needs that
as a general rule), per-law messages. `?S` borrows a field's niche (28.10); a trait's `@on_unimplemented` names the fix when a bound is unmet (28.11). 0 of the 35 `design/spec/*.md`
```cp blocks pass `cpl check`; most stop at parse or stdlib gaps.
**Bar.** Every code block in the design spec passes `cpl check`; the stdlib
needs no compiler knowledge beyond the documented intrinsics; each
mechanism above has programs that the interpreter and the C++ agree on.
**Next.** Postfix `?` extent; remaining 0.3 syntax (opaque `struct`,
type-param defaults, one-sided ranges, `@droppable`,
`labels`, `Str(len:)`); variant reflection and derives as trait
defaults on `fields`; `print`/`read` on variadic packs; `assert`/`debug` in the stdlib; per-law diagnostic messages.
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

**Dream-port audit (`proto/ports/*/dream/*.cp`, 1/45 passing `cpl check`).**
Each dream's first blocker and owning lane (22 stdlib, 22 core syntax/semantics, 1 pass):

| Dream | First blocker | Owner |
|---|---|---|
| `cf2069c.cp` | PASS | — |
| `cf2062e1.cp` | `E0107`: `with io(base: 1)` context directive | §1 core syntax |
| `cf2062e2.cp` | `E0107`: `with io(base: 1)` context directive | §1 core syntax |
| `cf2062g.cp` | `E0107`: `with io(base: 1)` context directive | §1 core syntax |
| `cf2062h.cp` | `E0204`: no type or domain named `Bits` | stdlib |
| `cf2066b.cp` | `E0310`: `count` has no argument named `where` | stdlib |
| `cf2066c.cp` | `E0105`: inline `let a: [read(1..=2e5)]1..=1e9` | §1 core syntax |
| `cf2066d1.cp` | `E0107`: parameterized alias `type M = Mint(mod: ...)` | §1 core syntax |
| `cf2066d2.cp` | `E0107`: parameterized alias `type M = Mint(mod: ...)` | §1 core syntax |
| `cf2066e.cp` | `E0103`: struct field default `field = val` | §1 core syntax |
| `cf2066f.cp` | `E0302`: tuple element widening to `?Tuple` (`PosA.Cut`) | §1 core syntax / §2 domains |
| `cf2069f.cp` | `E0204`: no type or domain named `Pair` | stdlib |
| `cf2084e.cp` | `E0203`: `[Pos]Int` has no method `map` | stdlib |
| `cf2084f.cp` | `E0204`: no type or domain named `Max` | stdlib |
| `cf2084g1.cp` | `E0107`: `@rolling` array attribute | §1 core syntax |
| `cf2084g2.cp` | `E0201`: no `LazyTreap` in scope | stdlib |
| `cf2084h.cp` | `E0108`: `var` field inside `struct` | §1 core syntax |
| `cf2129f1.cp` | `E0501`: generic fn/type lowering gap | §1 core syntax / IR |
| `cf2129f2.cp` | `E0203`: `Range<Int>` has no method `group` | stdlib |
| `cf2135e2.cp` | `E0203`: `Sieve` has no field or method `multiplicative` | stdlib |
| `cf2135f.cp` | `E0206`: no module `std.persistent` | stdlib |
| `cf2138e1.cp` | `E0201`: no `phi` in scope | stdlib |
| `cf2138e2.cp` | `E0201`: no `phi` in scope | stdlib |
| `cf2138f.cp` | `E0401`: `mut` parameter mutability on array element | §1 core syntax |
| `cf2147f.cp` | `E0108`: anonymous/operator `fn (...)` decl | §1 core syntax |
| `cf2147h.cp` | `E0201`: no `choose2` in scope | stdlib |
| `cf2150f.cp` | `E0107`: `with io(...)` / statement form | §1 core syntax |
| `cf2150g.cp` | `E0201`: `z` scope in multi-clause `for` | §1 core syntax |
| `cf2152g.cp` | `E0107`: `record` declaration syntax | §1 core syntax |
| `cf2159e.cp` | `E0107`: comma-separated `const A = ..., B = ...` | §1 core syntax |
| `cf2165f.cp` | `E0302`: open-left range `a[j] <.. a[i]` (passed `by: <`) | §1 core syntax |
| `cf2178h.cp` | `E0201`: no `Zmod` in scope | stdlib |
| `cf2178i.cp` | `E0204`: no type or domain named `Vec2` | stdlib |
| `cf2180f1.cp` | `E0107`: `record` declaration syntax | §1 core syntax |
| `cf2180f2.cp` | `E0107`: `record` declaration syntax | §1 core syntax |
| `cf2183e.cp` | `E0302`: `read()` on `[Int]?Int` | §1 core syntax |
| `cf2183f.cp` | `E0204`: no type or domain named `Tree` | stdlib |
| `cf2183h.cp` | `E0103`: `{ ... }` block/closure argument syntax | §1 core syntax |
| `cf2187d.cp` | `E0107`: `Str of "01?"` bounded alphabet type | §1 core syntax |
| `cf2187f1.cp` | `E0204`: no type or domain named `Perm` | stdlib |
| `cf2187f2.cp` | `E0204`: no type or domain named `Perm` | stdlib |
| `cf2222f.cp` | `E0201`: no `DsuOf` in scope | stdlib |
| `cf2237h.cp` | `E0201`: no `IntervalMap` in scope | stdlib |
| `cf2255e2.cp` | `E0204`: no type or domain named `MaxSub` (passed `enum Sign: neg = -1`) | stdlib |
| `cf2262b.cp` | `E0201`: no `fixpoint` in scope (passed `x is q and ...`) | stdlib |

### 2. Index spaces and fresh domains (`plan/12`, `plan/19`)
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
checker, IR, interpreter, C++ codegen, trace, editor syntax, and docs.
Domain-wide subsets (`V.all`, `Bitset<V>` with `for v in s`, `.without(v)`,
`s - t`, `.find`, `.any`, `.len`, `impl Show`, and `adjacency(edges, into: Bitset)`),
fieldless enum-indexed arrays (`enum Sign: neg = -1, zero = 0, pos = 1`,
`[Sign]T`, `[Sign] x => ...`, `for x in Sign:`), domain-level sequence ops
(`D.partition`, `D.sort(by:)` including array keys, `D.group(by:)`, `D.at`,
`D.first(p)`, `D.filter` with domains satisfying `Seq`/`Indexed`), `D.cuts`
directly in `SegTree`/`LazySegTree`/`Fenwick` (`cf2207f`), algebraic product domains
(`domain State = (V, K)` and named `(v: V, d: Dir, ok: Bool)` with flat `[State]T`,
`for s in State`, `for (v, k) in State`, `dp[(v, k)]`/`dp[v, k]`, `State(v, k)`,
and `s.0`/`s.v` projections across checker, IR, interpreter, C++, LSP, and docs),
algebraic sum domains and anonymous tuple/record index types (`decisions.md` §2.16:
finite-index payload enums like `enum Node: Src / Snk / Cell(Row, Col)` and tuples/records
of index types `(Row, Col)` / `(r: Row, c: Col)` derive `Index` and work in `[Node]T`,
`for v in Node`, `match v:`, `FlowGraph<Node>`, `FlowGraph<(Row, Col)>`, `Dsu<Node>`,
with declaration-time `E0202` when a variant name collides with a domain/type;
migrated `cf1404e` with emitted C++ shrinking `15,338 -> 15,230 B`),
partial-injection and bijection `a.inverse()` (`[D]?E -> [E]?D` trapping on duplicate
images, `[D]E -> [E]D` on bijections, adopted in `cf2084g1`),
auto-loaded `Bitset` for `V.all.without(c)` with zero imports,
range-alias array dimensions (`type Vol = 1..=5; [Vol]Int`),
struct/param array scalar-fill defaults (`count: [Vol]Int = 0`, `delta: [0..=n]Z = 0`),
typed range-indexed sentinel reads (`let target: [0..n]?Int = read(none: -1)`),
optional `.map(f)` / `.filter(p)` (adopted in `cf2084g1`),
and typed `Item`/`Cut` ranges via `minmax` and `choose2` now also work end to end.
Eight ports converted (`ucup4s25c`, `cf2222f`, `cf2101b`, `cf2268c`, `cf2150f`,
`cf2147h`, `cf2129b`, `cf2147f`), with `cf2147h` (`1.631x -> 1.063x`) and
`cf2147f` (`1.467x -> 1.254x`) rewritten toward their dreams. Remaining:
graph edge domains (`domain E = edges(V)`),
eliminating redundant runtime bounds checks when indexing `[0..=m]` by a `0..=m`
record field in C++ codegen, and explicit `domain` field syntax on
domain-carrying structs.
**Bar.** Settled semantics in `plan/OPERATING.md` implemented across
checker, interpreter and C++; three ports converted. (Met: 8 ports
converted and passing `gate.sh`.)
**Next.** `edges(V)` derived domains; propagate runtime
range-field facts (`w: 0..=m`) into C++ bounds elimination (`cpl-codegen/src/facts.rs`);
migrate remaining stdlib existential signatures to tuple syntax where appropriate;
explicit `domain` field syntax for value-owned structs.

### 3. Codegen and performance (`plan/30`)
**North star.** The generated C++ is what a world-class contestant would hand-write:
runtime p50 at or below 1.0x the accepted originals and never above 1.5x,
always tunable to 1.0x without leaving the language; memory and layout as
tight as the best C++ (4-byte indices where proven, niches everywhere,
struct-of-arrays on request); readable output that fits 64 KB with room to
spare; and every safety feature with a zero-cost proven form. Compiles in
well under a second.
**Where we are.** C++17 output passes every port under ASan/UBSan; every
program runs on a thread with the judge profile's thread stack (1 GB by
default; `thread_stack_mb = 0` warns and runs on `main`), so a 10^6-deep
DFS passes on an 8 MB `ulimit -s`;
`Int` stops at `-(2^63 - 1)` in both builds (input, arithmetic and
narrowing reject `-2^63`), so every `?Int` is 8 bytes with `-2^63` as `none`; `bench/corpus_p95_bench.py`
measures runtime and RSS against the originals (p95 about 3x). (Lane
running.)
**Bar.** Per-port runtime and memory on max inputs, measured on every push.
**Next.** Heap for runtime-sized
arrays; niches; a matched-algorithm benchmark harness.
**`std::function` debt.** Every fn value lowers to a monomorphized,
inlinable C++ form except these, which keep `std::function` (26 of the
1,781 corpus and test programs, one of them the intentional parity case of
§174; a fn or `?fn` field nothing reads is dropped, §176; a block that gives
one closure, such as the bound method in `sorted(by: tree.pos)`, is that
closure's own lambda type). `cpl explain` names each one:
- `Monoid<T>.op` when one program builds two monoids over the same `T`
  (`+` and `max` trees): the struct is shared, so the field holds more than
  one closure (8 programs). The static form is a monoid monomorphized per
  op, which is the ranges lane's Monoid/Action work;
- an optional fn field that is read, `SegTree.source` with `refresh`
  (2 programs);
- a fn value whose closure is chosen at run time (3 programs);
- the wrapper of a local `@memo` fn, which its body calls before it is
  assigned (3 programs; the body itself is an `auto` lambda);
- a fn-typed parameter of a closure, a closure returning a closure and a
  recursive closure inside another closure (4 programs);
- a fn-typed parameter of the judge entry, and one of a recursive fn whose
  recursion passes it a different closure (§174);
- a reassigned or hoisted fn slot, and a top-level fn var (2 programs).

### 4. Standard library (`plan/STDLIB.md`, `plan/20`)
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
**Where we are** (updated 2026-11-21 at `df99fea2`). 38 modules; the
docs gate finds all 956 `pub` items documented, each in `plan/STDLIB.md`
with its cost. All the module families the north star names exist:
number theory (one `factor` for every `Int`, decision 92),
NTT/FWHT/subset convolution and Lagrange interpolation, matrices, binary
and ternary search over integers and reals, RNG and hashing, bitsets,
flows and matchings, strings (every algorithm over any `Indexed`/`Seq`,
`lyndon_runs`, lint L0516 for hand-rolled ones), geometry (2D, circles,
3D hull), trees (`Rooted`, `Rooted.virtual`, `PermTree`, `LinkCut`,
`LazyLinkCut`), containers (open-addressing `HashMap`,
`SortedSet`/`SortedBag.range(r)` views, single-walk `IntervalMap.assign`,
`List.reserve`), sequences (`distinct`, `fold(f)`, `sort(cmp:)`,
`reach(while:, cyclic:)`, `[D]T.map` keeping its domain). Range-typed
stores trap in the interpreter and checked C++ (decision 100); ranged 1D
arrays store 32-bit elements (decision 106); segment trees lend a leaf as
a writable place (`st.at(i) += x`). 63 matched C++ benches across nine
areas (geometry 14, flow 10, containers 9, strings 9, ranges 7, trees 5,
dp 4, graph 4, io 1): strings all at most 1.07x, sorted range views
1.00x, cf2237h at the original's speed (0.51 s vs 0.53 s). The `Rooted` build on a 10^6 tree is
0.76x time and 1.14x memory of hand-written C++ (CSR `Adj`, decision
110). Open gaps: `dijkstra` 1.18x, weighted blossom 1.30x, the interpreter on
`mut self` structures, `if let` copying its payload every loop
iteration (cf2021d, 9.5 s vs 0.01 s), and constant-length `[3]T`
literals on the heap (cf2022c, 2.28x until rewritten; in progress).
**Structure.** Ten area owners (graphs, trees, range structures,
strings, number theory/combinatorics, flows/matching, containers,
geometry, DP/search, sequences) each own their part of
`plan/STDLIB.md` section 4, write `spec/lib/<area>.md` and build
it; the table "Who owns what" there lists each area's queue. The
coordinator owns sections 1 to 3, cross-area coherence and the prelude
budget, reviews each design against `spec/lib/rationale/README.md`, and
routes frictions.
**Bar.** Every item of `plan/STDLIB.md` marked [new] or [rename] has
landed with tests; every public name is used by a port or an example;
the [rename]s have codemods and the ports are moved.
**Next (coordinator).** Review the ten area designs as they arrive;
route new frictions; keep section 2 the single vocabulary (the `_by`
renames, `find` vs `first`); bring section 3's remaining core asks (variadic
`min`/`max`, scoped rollback) to the core lane; route the codegen
frictions the ports keep finding (`if let` by const reference, enum
`@memo` keys, iterable `D.cuts`) and keep the browser bundle under
its caps.

### 5. Diagnostics and front end (`plan/30` 3.7)
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
(`cpl-cli/tests/recovery.rs`, six single edits per port): 0.034 cascades per
copy (145 of 4306), first error on the right line 99.7%, 77% of errors carry a
fix-it, ratchet at 0.05 per copy.
Planned forms go through one table (`cpl-syntax/src/planned.rs`, `E0504`
"not implemented yet: ... (see design/...)", or "(not in the design)" for
forms only dreams use); 36 entries, each with a golden. Dreams are a corpus
(`scripts/dream-gaps.py`, summary in the scoreboard): 144 dreams, 97 with
errors; 78/97 first errors name their gap and 240/316 errors are named
(contexts `with io(...)` as deferred, broken checker rules by the rule). Each
unsupported construct class gets one `E0504` at its first use (later uses of
the same feature are dropped as the same root cause): domain parameters, several domains on a line, indented values and continuation
lines, `(a; b)`, `<..`, domains as values, `D.Cut`,
operators as values (`compose: +`),
`(var a, b)` patterns, `match` arms on the `match` line, `let … in`, brace
maps and sets, `m default 0`, `lazy[D]`, filtered domains, patterns as
parameters, `some x`, `[_]T`, `then` chains, `.member`, literal-value types.
Unknown names say "not
in the stdlib (yet)" with a did-you-mean against stdlib names and the `use`
as a fix. Golden rendered diagnostics in `cpl-cli/tests/diagnostics/`.
Fix-its: C spellings (`&&`, `||`, `!x`, `..<`), `let` for a `var`,
did-you-mean on names, fields and methods (built-in, record and UFCS) and a
plain number for a domain (`V(i)`) carry an edit, shown as LSP quick fixes
and applied by `cpl fix`. Common-mistakes benchmark (`fix_it_scoreboard`):
1,874 errors, 94.1% with a fix-it, `cpl fix` restores 91.2% of copies
byte for byte (ratchet 88% and 95%); misspelled members are the gap. A
domain or range mismatch names both index spaces and labels the
declaration each comes from.
**Bar.** Golden diagnostics for every error code; a broken-ports benchmark
with first-error accuracy and cascade counts.
**Next.** Name the last 15 dream first errors (argument-count gaps such as
`gcd` of a list, a missing field, a stdlib type that can't be indexed, `.inverse()`
of a set), then every error (40 unnamed); restore 95% of common-mistake copies (members the back end
knows by hand); count cascades by root cause instead of "every error after the
first"; struct/enum headers that fail to parse should also bind with an
unknown shape; fn stubs `-> T: ...` and the rest of `design/DELTA.md` into
the planned table or implemented; quick fixes on every syntax error and on members the back end knows by
hand (`Str.count`, domain `before`/`after`);
top-level tuple-destructured names in LSP completion.

### 6. Interpreter, debugger and traces (`plan/50`)
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
(`type: "cp"`, `cp.debug`); (4) plain untraced/unprofiled runs now execute on
the typed register bytecode VM (`crates/cpl-interp/src/vm.rs`) with unboxed
`i64`/`(val, some)`/small-tuple (`ArrayData::SmallIntTuple`) slots and flat
typed arrays (`ArrayData` in `crates/cpl-interp/src/value.rs`), structural
modular-arithmetic superinstructions (`ModAddI`/`ModSubI`/`ModMulI`/`ModNegI`/
`Mod::new`), inlined pure leaf helpers, direct VM calls with pooled `Value`
register frames (`vm_pool_v`), fast uncaptured scalar closures (`FastClosure`),
VM lowering for `Fill`/slices/1D+2D+nested arrays/lists/`Map`/`Set`/`Do`/`Give`/
float ops/bit builtins/`U64`/natural sort/indexed-mut calls/fused `?Int`
`min=`/`max=`/list small-tuple loads/indexed `.first`/`.last`, block-local
last-use move semantics (`ways = next`), and `CPL_VM_STATS=1|json|verbose`
hot-loop fallback instrumentation with a zero-fallback ratchet test in
`crates/cpl-cli/tests/interp_ratio.rs`. Across 249 sampled ports, ports with
hot-loop fallbacks dropped from **109 -> 91** (158/249 at 0), and all worst-10
targets now run with `0` hot-loop fallbacks on their samples. On the full
305-port scoreboard (`--full --interp-timeout 10 --jobs 4`, `99290110`),
302/304 passed with 216 ports >= 20 ms: **p50 = 13.07x**, **p95 = 44.28x**,
**max = 93.7x** (`cf2062f`, 154 ports > 10x; `cf2062f` subsequently cut from
93.7x -> 43.5x via fused `?Int` `min=`/`max=`, nested array loads, and list
small-tuple opcodes; bar <= 10x / max <= 30x -> 20x not yet met);
(5) no stage snapshots (`--stage`) or interactive exchange trace.
**Bar.** Every runtime error class has a golden message with values and call
path; trace filtering and index-space views tested end to end; DAP step
debugger wired into VS Code and tested over stdio; interpreter within 10x of
C++ `-O2` on ports (north star 5x).
**Next.** (1) Eliminate remaining hot-loop fallbacks and overhead (`stmt:Expr(Mutate)`
in `convolve_mod`, scalar `expr:Read`/`Print` in tight I/O loops, remaining
`expr:Closure`/HOF calls, and `cpl run` startup/compile time on short ports) to
hit ratchets max <= 30x -> 20x and p50 <= 5x; (2) stage snapshots (`--stage`)
and interactive exchange traces; (3) headless-browser test for
`tools/trace/index.html`.

### 7. Stress testing and shrinking (`plan/50` 3, `plan/40` 7b)
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
0.59 s. A recursive `@brute` works; a
failure that needs several tests shrinks each of them; a smaller size drops
elements from nested arrays (a list of permutations), not just top-level
ones; letter bounds (`'A' + k`) evaluate. Values that fail only while
equal, in one test or across tests, are lowered together
(`values_that_fail_only_while_equal_are_lowered_together`), so two tests
that must repeat one number shrink to `1` and `1`, not `7` and `7`. Trees
(and the tree a connected graph starts from) come as random, long, path or
star shapes: 68 paths and 100 stars in 400 seven-vertex trees, from 27 and
2 (`random_trees_include_paths_and_stars`), and `cpl bench` makes its path
and star cases for `Tree<V>`, which it skipped before
(`trees_are_shaped_as_a_path_and_a_star`). A shrunk graph has its vertex
names swapped and its undirected edges turned low end first toward the
plainest form, so a star shrinks to `1 2, 1 3, 1 4` from any centre.
`std.graph.Digraph` is the directed simple graph: arcs both ways and
cycles are made, and arcs keep their direction while shrinking
(`shrinking_a_digraph_keeps_each_arc_the_way_it_points`). Reads inside
a `for _ in lo..hi` (or a domain) are a field of their own, one row per
pass, nested loops included, when each pass reads the same typed `let`s
(`reads_in_a_loop_are_generated_as_one_row_per_pass`); over the
1,332 ports, those stress can make inputs for rose from 847 to 960. The
scan also found programs that were "supported" with reads left out of
every case: reads inside a plain fn, and `read(gen: f)` / `read(none: x)`.
Each is now an error naming the read
(`reads_no_field_describes_are_refused_not_left_out`). Shrinking now also
lowers a value while it drops an element, which a FIFO memory of `m`
words needs (`a_value_and_an_element_go_together_when_neither_goes_alone`);
the 10th seeded bug, `noip10p1-evict-early`, reads its words in a loop.
A pass may read a count and then that many values (`let s = read()`,
then `[0..s]City`, or a loop over `0..s`): each part of the row binds its
names for the parts after it, and the count shrinks with one of its values
dropped, like a top-level size
(`a_pass_reads_a_count_then_that_many_values`); stress makes inputs for
984 of the 1,332 ports (from 960), and the 11th seeded bug,
`btoi17p3-open-tour`, reads its plans so.
Benchmark: 11 bugs, all minimal, median 0.75 s.
`domain V = read(1..=hi)` is a size read where it is declared, with the
arrays over `V` that long (`a_domain_read_as_a_size_is_generated_with_its_arrays`).
A query whose reads depend on a tag read first (`if op == 1: read l, r
elif op == 2: read i`, `match kind: 'B' => ...`) is a choice: each arm is
generated about as often as the others, a tag that moves to another arm
reads that arm's values, and shrinking tries each arm before lowering
values, so the plainest query wins
(`a_query_reads_what_the_branch_its_tag_takes_reads`). Ports stress makes
inputs for: 1,045 of the 1,389 `.cp` files under `proto/ports` (from
1,014 by the same scan; the 1,332 above counted an older tree), and the
17 query ports this newly covers all give inputs their programs read to
the end.
A struct or an `@tags` enum is read field by field, the enum as a choice
on its tag (`structs_and_tagged_enums_are_read_field_by_field`, 1,053), and
`read(none: x)` writes `x` about a quarter of the time and a value of `T`
otherwise, shrinking to the low end or `x`
(`an_optional_read_with_none_written_as_a_number_is_generated_both_ways`,
1,069). A top-level `read(gen: f)` runs `f` in the interpreter on the
input above it with `rand` seeded, and keeps the value as its seed: when
the values above change it is drawn again, and a smaller case that passes
is retried with other seeds, so `n` shrinks with a word of that length
(`a_read_with_gen_draws_what_its_fn_makes_from_the_input_above`, decision
25.14). Ports stress makes inputs for: 1,112 of the 1,391 `.cp` files
(from 1,069); of the 41 `read(gen: f)` ports newly covered, 39 give inputs
their programs read to the end and 2 trip the port's own `assert` on the
drawn value. The remaining `gen:` refusals are files that do not build.
A `break`, `continue` or `return` that ends only a nested loop or fn no
longer refuses the pass around it
(`a_jump_that_ends_only_a_nested_loop_or_fn_keeps_the_pass_whole`, decision
25.15): 1,182 to 1,193 of the 1,399 files on today's tree.
A typed `read<T>()` inside an expression (a call's argument, an operand, a
`[D] _ =>` table, a `for` over it, a `match` on it) is a field where it
runs a known number of times, in the order the reads run
(`typed_reads_inside_an_expression_are_fields_in_the_order_they_run`,
decision 25.16): 1,193 to 1,198 of 1,402 files. Generic reads were
invisible before, so 4 ports that had been accepted with their input
silently left out are now refused with a message (a read in a branch, a
read in a type's length, `read<T>(base: 1)`).
On today's tree (1,343 files, the batch and `today/` copies counted once
each as the scans above do), five general read shapes take stress from
1,150 to 1,227 files it makes inputs for, 63 of them reading nothing:
fns that read, read where each call is or wherever they read fixed values
(`a_fn_that_reads_is_read_where_each_call_is`,
`a_reader_may_return_once_its_reads_are_done`, decision 25.19);
`U64`, `Mod<P>`, whole-number `F64` and `I128`, and stdlib structs such
as `Pt` (`stdlib_structs_residues_and_unsigned_values_are_generated`,
`floats_and_wide_ints_are_read_as_whole_numbers`, decision 25.20); a tag
read first in a tuple (`a_tag_read_first_in_a_tuple_or_above_other_reads_still_chooses`,
25.21); a loop read whose type uses the loop's name
(`a_loop_read_may_use_the_loop_name`, 25.22); and a `Str` tag compared
with words (`a_word_tag_chooses_by_the_words_the_program_compares_it_with`,
25.23).
A write `a[i] = read()` into an array declared once with its type,
such as `var a: [0..=n]Int`, reads `a`'s element type, so a loop of such
writes is a row per pass (`a_write_of_read_into_a_typed_array_reads_its_element_type`,
25.26): 1,171 to 1,176 of 1,351 files. Bare `read()`s passed to a call
that fixes their type read that type: `Graph<V, E>.undirected(read())`
reads `[E](V, V)` and a struct `Seg(read(), read())` reads a `Seg`
(`bare_reads_take_the_type_the_call_they_are_passed_to_wants`, 25.27),
and a read in one comparison, `read::<Int>() == 1`, is read
(`a_read_in_one_comparison_is_read`): 1,180 files. A pass that starts
`if v == V.first: continue` reads from the second index on, and a `domain`
a pass computes refuses only the reads that use it
(`a_pass_that_skips_the_first_or_last_index_reads_from_the_rest`,
`a_domain_a_pass_computes_refuses_only_the_reads_that_use_it`, 25.28):
1,185 files; a `List<T>` declared once takes writes of reads like an
array (`a_write_of_read_into_a_typed_list_reads_its_element_type`): 1,186.
A table `[0..n] i => if i == 0: 0 else: read::<Int>()` reads `n - 1` values
(`a_table_that_skips_its_first_cell_reads_the_rest`, 25.29): 1,188.
Both new files (cf2124h, cf2138c1) read sizes and parents as untyped
`Int`s, so their inputs still crash them until the ports type the reads.
`let a: List<T> = [read() for _ in 0..n]` reads `n` `T`s
(`a_list_of_bare_reads_reads_the_element_type_it_is_declared_with`,
25.30): 1,189.
A top-level `if` on a tag read above it, as the last line that reads, is a
choice (`a_top_level_branch_on_a_value_read_above_chooses_what_follows`,
25.31): 1,193.
Of the first five, ioi09p7 and noip19p2 read parents as plain `V`s
and fail on the cycles generated, and etsr23r1p3 reads letters past its
alphabet: typing gaps in the ports. Still refused: 27 loops a row can't
describe, 11 reads in a branch or `while true`, 16 fns that read and take
parameters or are called in a branch.
Stress under a judge is a gate regression: all 16 interactive and 5
checker ecnerwala ports pass 20 generated cases each under the
interactor or checker `gate.sh` uses, and a solution printing `1` fails
under every one of the 21 (`stress_judges`, 4.9 s, decision 25.17). These
cases come from each port's `gen/` script, so they are not shrunk: the
hidden input an external interactor reads has no type stress can see.
Time to a counterexample for wrong solutions to all 21 judged ports
(`bench/stress/judged.py`, seed 1, 1,000 cases, one run at a time): a
solution printing `1` is caught on case 1 in all 21, p50 0.06 s, p95
0.12 s. Mutants of the ports themselves (the first site that still checks
of `+ 1`→`+ 2`, `- 1`→`- 2`, `<`↔`<=`, `0..`→`1..`): 36 of 65 caught,
p50 0.71 s, p95 3.92 s including the build; 29 survive 1,000 cases, not
yet sorted into equivalent mutants and inputs the generators never make.
`cpl stress --explain` names the first value the solution and its
`@brute(main)` give a shared variable differently, on the shrunk case:
on a planted sum of squares it reports `s`, its 2nd value, `4` at line 6
against `2` at line 12 (`stress_explain_names_the_first_value_the_solution_and_brute_give_differently`,
decision 25.18). It compares by variable name, so a brute that names
nothing the solution does gets no explanation beyond its `debug(...)` stages.
**Bar.** A seeded-bug benchmark in the gate: classic bugs injected into
correct solutions, each found and shrunk to a known minimal case, with the
median time to a minimal counterexample at most 1 s (`GOALS` 4),
measured and printed by the gate but not asserted, so a loaded machine
can't fail it; the verdict is fixed by seed and case count (decision 25.25).
Benchmark: 21 bugs, all minimal, median 0.60 s, 3.2 s for the test on 4 threads. The ten added cover the
other P0 classes: multitest leaks (`cf2237a-running-min`,
`cf2152a-seen-set`, `cf2124c-lcm-leak`, `noip18p2-shared-table`, each a
value or table above `@!multitest`), overflow at bounds
(`midpoint-overflow`, `mulmod-overflow`, `cube-mod-overflow`) and
off-by-one at a range end (`cf2237c-skip-last`, `cf2183b-mex-from-one`,
`cf2096b-skip-largest`); the slowest is `noip10p1-evict-early` at 3.39 s.
**Next.** loops whose reads a row cannot describe (40 files: nested
loops with counts worked out in the pass, `a[i] = read()`, a `continue`
past a read, `read(gen: f)` in a loop); reads in a branch or a `while
true` that is not a choice (22); fns that read and take parameters,
return early or are called in a branch (15, 6 of them interactive);
generators keyed to the
stdlib declarations, not their names; worst-case values and anti-hash generators; grid and string-family shapes (`[R, C]Char` mazes, palindromes).

### 8. Formatter, linter, profiler, coverage, bench, doc (`plan/45`)
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
L0507 `push_loop` and L0508 `none_check_unwrap` come from the friction
detectors. `cpl lint --taste` counts L05xx per file; the scoreboard and
`research/PORTS.md` show it per port (14 findings over the 281 gated
ports, 342 over all of `proto/ports`). Precision by hand: 29 of 30.
**Next.** A test that every `L0414` is an index
the C++ checks (codegen to report its checked indexes); lints from the friction
detectors (`wa_trap` first); profile against `@cost` and the time limit.

### 9. Judges and submission (`plan/70`)
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
header stamp, `__int128` support and the thread stack the program runs on
(`main`'s stack where the judge allows no threads). All 257 ports fit every profile (max 59.4 KB) and compile
with every profile's flags (1,799 compiles, `--full`); a million nested
calls run on each judge's stack and agree with the interpreter.
A file over its limit loses its C++ comments, then the source header, then
half its indentation, and says so. `cpl build --size` (and any over-limit build) says where the bytes go
by part and piece: across the ports, 3.6% (p50; at most 8.2%) is
namespaces and `main`, the rest is named. `run --release`, `test`,
`stress`, `hunt`, `bench` and `profile --native` build a `.cp` file with
its judge's flags (those g++ here accepts) and run it under `ulimit -s` of
the judge's stack. Time and memory limits are the problem's, not the
judge's (`// limits:` or `[limits]` in `cp.toml`). `cpl submit` builds
for the file's judge and runs its cases on that build; it sends nothing. The compilers here are g++
11, clang 14 and MinGW 10, not the judges' versions.
**Bar.** Every profile: every port builds within its limit, compiles with
its flags (as accepted here), and the deep-recursion run passes on its
stack; each fact in a profile cites a source or says it is unconfirmed.
`cpl test` and `bench` give MLE past the problem's memory limit.
**Next.** `// d.cp:42` markers (`plan/70`) so a judge's line maps back,
kept through every rung; USACO and grader profiles; judge-version containers.

### 10. Editor and LSP (`plan/60`)
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
rename across every open file, the user library that declares the name and each file of the project on disk that imports it
(the stdlib's names refused; refused too when, re-checked with the edits applied, any use of the old or new name would resolve to a different declaration: two locals merged, a use captured by a nearer binding, a top-level name shadowed), document symbols, document highlights, folding
(blocks, comment runs, imports), semantic tokens (each resolved name as a
type, fn, method, field, variable or parameter, marking declarations and
immutable bindings), workspace symbols (fuzzy, over open files and the
public names of every module they use), Run/Trace/Test/Stress code lenses on
the first sample (Test when `<stem>.tests/` has a case) (asked for with `initializationOptions.codeLens`, each a
`cpl` argument list under the client command `cpl.exec`), formatting, completion (members
after `.`, names in scope, `use std.`; what fits the expected type first: the
parameter an argument is for, a `let`'s written type, an assigned variable's), signature help
(defaults and the `;` before named-only parameters, the active one found by name or position), inlay hints
(unwritten types, with the index space of a loop variable (`v: V`,
`i: 0..n`) and the range the input bounds prove for a `let`, in a fn or loop too
(`m: Int in 2..=10001`); parameter names of literal arguments) and quick fixes
from diagnostic fixes (the four most frequent first errors over the broken
ports, an unclosed delimiter, a missing token, a doubled operator and a
stray character, each have one; `recovery.rs` prints the table and how
often the first fix restores the line; a closer dropped inside a call or index
no longer shows first as a wrong argument or index count); the lints
`cpl lint` reports, each with its `cpl fix` fix as a quick fix, and
`cpl fix` on the buffer as a `source.fixAll.cpl` action, removed syntax first (the replaced
spellings, `where`, `--- multitest ---`, `.pow` and getter `()`, each a quick fix tested on a
real port; a getter, built-in or `@get`, completes as a property and hovers its type; hover adds the `cost:` a stdlib or built-in member documents and the range the input bounds prove for a `let` (a header `let (n, q): (1..=N, 1..=Q) = read()` bounds each name, which took ports' loops with a known total from 849 to 984), in a fn or loop too, at its declaration and at each use, and for a `var` the range every assignment in its scope keeps (`best max= 2 * a`; `c += 1` in a `for` loop has none, in `while c < n:` it keeps `0..=n`, since an assignment in a `while` body sees the names its condition bounds), solved together with the `var`s and `let`s it reads (a binary search's `lo`, `hi` and `mid`), and after a loop with no `break` by its failed condition (`while w > 9: w = w / 2` leaves `w` in `1..=9`), narrowed by the comparisons an `if`, `elif` or `while` condition makes, and by the ones each earlier branch failed (`if n > 50:` gives `n - 50: Int in 1..=199950`, its `else` `n - 10: Int in -9..=40`), and for a difference of two names by what a comparison of them proved (`if a < n:` gives `n - a` at least 1; inside a binary search `hi - mid` is at least 1 for `let mid = (lo + hi) / 2`; an assignment `x = y + c` carries `y`'s relations to `x`, the branches of an `if` keep what all of them prove, and a `while` keeps what its body preserves about the names its condition compares, so `lo = mid + 1` / `hi = mid` keep `lo <= hi` and after the search `lo - hi: Int = 0`; the same holds inside a fn for its own locals), and on an index `xs[i]` of a list or string, with its length known until a statement changes it (a breadth-first `while head < q.len: let u = q[head]` before its pushes), why it is in bounds (`ys[mid]: in bounds, 0 <= lo <= mid < hi <= ys.len`, also appended to the index's own hover; `for i in 0..xs.len` gives `0 <= i < xs.len`, and a row the same way, `for j in 0..g[u].len` gives `g[u][j]` `0 <= j < g[u].len` while no row of `g` changes length; 177 indexes in the ports and examples), and on an index of an array declared over a domain, that its type is that domain (`xs[v]: in bounds, v: V, the index space of xs`; `g[r, c]`, `dp[v, k]` over a product domain and `g[p]` with a tuple `p` name each), and on the head of a `for` over a bounded range or domain how often its body runs, alone and with the `for`s around it in the source's terms (`runs at most k = 7 times`, `n * k = 1400000 (1.4e6) times in all`), and on a fn whose loops all have a count, how many iterations one call makes with the fns it calls (`loops: at most n + n * k = 1600000 (1.6e6) iterations per call, with the fns it calls; methods and the stdlib not counted`), and each call in its body whose doc states a cost above `O(1)`, with that cost in the receiver's terms and how often it runs per call (`documented costs: ys.sorted O(ys.len log ys.len) n = 200000 (2e5) times`; 648 such calls in 355 of the 2,002 fns in the ports and examples, 480 with a known count), and for a receiver whose declared array type bounds its size, the steps at max input (`cells.sorted O(n log n), about 3521929 (3.5e6) steps each`), also on the call's own hover anywhere in the file; a list built only by pushes in counted loops is sized by them (`seen has at most n * k items`), so 56 of the 1,278 documented-cost calls get steps; when nothing in a fn is left uncounted, its hover sums these into `estimated cost: O(n + n log n), about 3721929 (3.7e6) steps at max input` (25 of 1,613 fns in the ports and examples; `print`, `read`, `ask` and conversions of numbers, characters and tuples (also `read<Int>()`, a `print` of `if ok: "YES" else: "NO"`, positions in a domain of integers and enums without fields), and `rand`, count one step, of lists and strings none yet; building a struct of the file counts one step; a method's `|D|` is its receiver's domain, `bit.sum O(log |V|)`), and a compiler prelude fn hovers its doc and the cost of its call's shape (`min(a, b)` `O(1)`, `min(xs)` `O(n)`), and on a `+`, `-`, `*`, `/`, `%` or `**`, or a call of `min`, `max` or `abs`, the range of its value (`n * n: Int in 1..=40000000000`) with, when it leaves `Int`, the inputs the prover finds that overflow it (`overflows Int when n = 55109`, also a soft note on the line) or else `may overflow Int`; a lambda passed to a built-in method, block body after `=>` included, hovers and hints its parameters and `let`s; sum domains fold, complete their variants and hover; `++` has its quick fix); a panic becomes an error reply plus an
internal-error diagnostic. Unit tests in `src/tests.rs`; `tests/latency.rs`
measures 6.1 (492 lines, debug build: keystroke to diagnostics median
6 ms, hover 0.06 ms, completion 1.8 ms), and gates the 95th percentile of
keystroke to diagnostics on the 10 largest ports and a 300-line file with
errors (debug build, under load budget 200 ms: worst p95 68 ms, `ioi20p1`). The VS Code extension has a
TextMate grammar, snippets, the client, Run/Stress/Trace lenses, live
sample values, counterexample overlays and a trace view (one panel per
file over `cpl run --trace --format html`; the editor highlights the line
of the step shown, and a click on a line in the viewer jumps there); its unit
tests and its extension-host suite (8 tests, under a virtual display) run in `--full`.
Auto-import: completion offers a stdlib name the file does not import yet, after what is in scope, with its module (`std.math`) and the `use` as an extra edit, and the quick fix for an unresolved stdlib name adds the same `use`: into the file's `use std.<module>.{..}` when it has one, else below its last `use`, else above its first line of code.
Hover shows the index space and its bound for arrays and positions
(`xs: [V]Int`, `domain V = 0..n, n <= 100`, storage per element and in
total), appends the declaration's `///` doc, which in the stdlib states the
cost, and definition on a field or method jumps to it, in the stdlib too.
Gaps against `plan/60`: computed cost on hover covers `for` loops only (not fns or stdlib calls); overflow notes only on names the prover can follow (range-typed inputs and
params and `let`s of arithmetic over them, not narrowed names, loop
variables or `var`s); no cost warnings; no refactorings; Emacs (eglot draws no lenses) and Helix
run from a shell command; no DAP. Fixture tests (`crates/cpl-lsp/tests/fixtures/`: a `.cp` with `$0` cursor markers and the requests to send, checked against a readable `.expected` snapshot, `BLESS=1` to rewrite) cover hover, completion, signature help, definition, references, highlights, rename, code actions, inlay hints, folding and the outline. A fuzz test (`crates/cpl-lsp/tests/fuzz.rs`) sends every request an editor sends on 360 cut, spliced and garbled ports and fails on any internal error. The tree-sitter
grammar (`editors/tree-sitter-cp/`) is token-level: it highlights, but gives
no tree for folding, indents or text objects (the server folds).
The JetBrains plugin (`editors/jetbrains/`, IntelliJ Platform LSP API, 2024.3
or later, commercial IDEs only) has a highlighting lexer, color settings,
comments and braces, `cpl lsp` as its server, run configurations for
`cpl run`, `test` and `stress` on the first sample with a marker on `fn main`,
and the trace viewer in a JCEF tool window, one tab per file, whose current step
is marked in the file like the debugger's execution point and whose clicked
lines move the caret; `--full` builds it and runs its 17 platform tests (lexer,
settings, the viewer's messages, and a headless IDE smoke test of the file type,
highlighting, comments, server, run configuration, run marker, trace and step
marker).
`editors/README.md` documents Neovim (runtime folder: filetype, Vim syntax or
tree-sitter, `cpl lsp` autostart), Emacs (`cp-mode` with eglot and lsp-mode) and Helix
(`languages.toml`, the grammar and its queries) and Zed (an extension in
`editors/zed/`: the language, the grammar at a pinned commit, `cpl lsp`);
`editors/smoke.sh`, in `--full`,
checks their keyword lists against the compiler and runs each editor's own smoke
test, every one of which starts `cpl lsp` the way that editor's setup does and
checks `initialize` and the published error: Neovim also checks hover, the run
lens through `cpl.exec` and tree-sitter captures; Emacs its faces and the eglot
entry, talking to the server over jsonrpc.el; Helix (no headless mode) builds
the grammar, `--health` finds the server and queries, and `editors/lsp-smoke.py`
starts the command its `languages.toml` names; Zed (no headless mode either)
builds the extension to wasm, checks its pinned grammar commit matches the tree,
and `lsp-smoke.py` starts the server. The grammar check regenerates
the parser, parses every port and library without an error, and checks each
editor's queries are the same patterns.
**Bar.** LSP fixture tests per `plan/60` 6.7 for every request; each editor
integration built and smoke-tested in `scripts/check.sh --full`.
**Next.** Publish the Zed extension; more fns with an estimated cost, which needs a cost on the calls that state none (43 of the 152 built-in methods, most of them getters, which already count one step: §6), a cost for `print`, `read` and `ask` of lists and strings by their size, and sizes for lists filled by `read`, comprehensions or `while` loops. Waiting on §1: `Rooted` and graph helpers give their vertices the domain type, after which `white[tree.parent(v)!]` is proven by type with no editor change.

### 11. Docs, site and playground (`plan/90`)
**North star.** The language is writable from the docs alone: a one-afternoon tour, a
guide with one chapter per idea, a complete reference, a stdlib reference
with costs and tested doc examples, an error index with one page per
diagnostic and lint code (each with verified trigger and fix), a cookbook
built from hundreds of real validated ports, interactive lessons that teach
by solving, and a browser playground that runs the interpreter, C++17
preview, trace viewer and a judge — every single code block and example
executed in the gate so documentation never drifts.
**Where we are.** Static site generator (`site/build.py`, 29 chapters + 77
individual error/lint pages + playground + trace = 108 HTML pages: 10-lesson
course with 51 checked exercises, 18 reference/guide pages including canonical
`docs/one-page-cheat-sheet.md`, gold-standard `cpp-guide.md` and `idioms.md`
modeled on `design/EXEMPLARS-PORTS.md`, registry-generated `errors.html`,
`cookbook.html`, generated `exemplars.html` showing Dream vs validated port side
by side + 6 IOI 2025 sketches, generated `stdlib-api.html` from `///` docs in
`stdlib/*.cp`, and `ports.html`), WASM playground (23 presets with verified
sample cases, URL-hash `#share=` links, multi-case sample runner, verbatim CLI
`stderr` rendering, `Playground::check` warning exit-code parity, and a "Try it"
playground run/trace/share button on every rendered docs `cp` fence — 154 verified
links), canonical trace viewer, and offline bundle (`dist/cp-lang.html`, ~2.41 MB
< 2.6 MB cap). All 77 error/lint pages are generated directly from the Rust
diagnostics registry (`crates/cpl-diag/src/registry.rs` +
`crates/cpl-lint/src/lib.rs`) via `cpl explain --json`. 117/117 doc `cp` fences
(154 Try-it links), 22/22 `site/examples/*.cp`, 10/10 course write-code refs,
77/77 registry diagnostic/lint codes (every active code with verified trigger +
fix), 22/22 cookbook recipes, 61 gallery ports (61 with dream), 61 exemplars + 6
IOI 2025 sketches, 36 `stdlib/*.cp` modules with **914/914 public functions
documented** (fast gate fails on any undocumented `pub fn`), canonical-spelling
enforcement (`if` filters, `**` power, parenless getters, `@!multitest`,
method-form folds via `cpl lint` `L0509`–`L0512`, `t.Pre` traversal), and
playground WASM-vs-CLI parity (`crates/cpl-wasm/tests/parity.rs` +
`site/test_playground_parity.py`) are verified in `scripts/check.sh --fast` via
`site/check_docs.py`. Writable-from-docs benchmark (`scripts/writable-from-docs.py`,
pinned 55-problem unseen unique deterministic-output set
`research/writable-from-docs-set.json` v2: 34 CF Div2 B–E, 17 ABC D–E, 4 ARC A–B,
evaluated with `gpt-5.4` `reasoning_effort="medium"` given only the 10-file docs
bundle, with `1e-6` float tolerance) reached **52/55 compile (94.5%) / 48/55
full pass (87.3%)** in Round 9 (up from R0 baseline 40/55 compile 72.7% / 14/55
pass 25.4% and R6 48/55 87.3% / 29/55 52.7%); all 7 remaining Round 9 failures
are classified in `research/writable-from-docs.md` (2 doc gaps fixed post-R9, 3
model syntax/output slips, 2 model algorithmic WAs). Enforced as a ratchet in
`scripts/scoreboard.py --fast --check` (floors 89.5% compile / 82.3% pass) and
run on demand via `.github/workflows/writable-from-docs.yml` (manual dispatch only).
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

### 12. Validation: ports, scoreboard, differential testing (`plan/80`)
**North star.** Thousands of hard accepted solutions across CF, AtCoder,
ICPC/UCup, IOI, NOI and DMOJ rethought in the language, discovered
automatically and differential-tested against their originals (interpreter,
sanitized C++ and `-O2` C++), measured on every push in a single deterministic
scoreboard (`scripts/scoreboard.py` + `research/scoreboard.json`) that blocks
regressions across every goal: correctness (gate pass, zero trap-parity drift),
expressiveness (contestant-written tokens vs raw and library-stripped original,
median <= 0.6x, none > 1.0x), performance (runtime p50 <= 1.1x / max <= 1.5x,
peak RSS p50 <= 1.0x / max <= 1.2x on max-size inputs, interpreter within 10x
of C++, north star 5x), submission viability (every emitted C++ file <= 32 KB, 64 KB the hard judge limit, warning-free
GCC/Clang), and friction (structured `research/frictions/*.md` reports with
zero open blockers and `friction_lint` zero `wa_trap`). Backed by a replayed
P0 bug corpus (>= 90% prevented or trapped), static `sizeof` and `-O2` assembly
zero-cost proofs, and real contest use by strong contestants showing faster
accepted solutions and fewer wrong attempts than in C++.
**Where we are.** Unified scoreboard (`scripts/scoreboard.py` + `scripts/test_scoreboard.py`,
21 unit tests) and committed baseline (`research/scoreboard.json`) are live and
wired into `scripts/check.sh --fast` (`--fast --check --quiet`, ~3–6 s).
At **285/285** active `proto/ports/ecnerwala/` ports passing (16 interactive, 5 custom checkers),
automatically picking up concurrently pushed ports:
contestant-written tokens median 0.417x raw / 0.503x library-stripped (0 > 1.0x raw, 1 > 1.0x
stripped: `ucup5s05f` at 1.054x), dream gap across 45 `dream/*.cp` ports `cp=21,413` vs
`dream=14,881` (median 1.283x, p90 2.532x, max 4.676x on `cf2187f1`; strict per-port ratchet
and worst-20 dream-gap leaderboard published on `research/PORTS.md`), emitted C++ median
10,701 B / p95 26,909 B / max 59,071 B (`cf2255f`; 3 > 48 KB, 0 > 64 KB), `friction_lint`
0 `wa_trap` / 6 `real_gap` / 58 `stale_port`, and `research/frictions/*.md` 66 files / 186 open
items (0 blocker, 43 high, 78 medium, 65 low, 2 closed; **0 schema warnings**, gated in
`--check`, routed into 10 stdlib-area owner queues + `core` and shown in `research/PORTS.md`).
Goal tests (AGENTS.md goals 1 & 2) and spelling-drift tracking are live and ratcheted in `scripts/scoreboard.py` (22 unit tests):
- **Bug-replay suite (`bench/goal_tests/bug_replay/`):** **59/63 handled (93.7%, bar >= 90%: PASS)**
  across all 11 P0 classes (>= 5 cases per class) — 35 `prevented` (`cpl check` + `cpl lint`),
  24 `caught_on_run`, 4 `left_to_diagnose` (`mt_top_var_unreset`, `cx_str_concat_loop`,
  `mod_mix_two_moduli`, `flt_sqrt_neg_nan`), reporting per-class rates and ratcheting handled % and
  per-case outcomes.
- **Revision test (`bench/goal_tests/revision/`):** **10/10 ok, 95 lines touched** across 10 unseen
  problems + 1 changed requirement each (`grid_path_skip_one=33L/4h`, `argmax_one_based=7L/3h`,
  `knapsack_max_to_ways=5L/3h`, `bfs_dist_to_path=10L/3h`, `prefix_sum_query_to_max_window=9L/2h`,
  `interval_merge_count_to_len=11L/3h`, `lis_strict_to_nondec=2L/1h`, `tree_subtree_sum_to_max=2L/1h`,
  `coin_min_to_ways_mod=8L/3h`, `dsu_components_to_largest=8L/2h`), ratcheting `lines_touched` per case.
- **Spelling-drift metric (`totals.spelling_drift`):** split by construct (`multitest`, `sum`, `first`,
  `sort`, `where_filter`, `dot_pow`, `getter_parens`, `turbofish`), each ratcheted strictly downward;
  original 4 constructs locked at **0 active/dream drift** (`scripts/fix-spelling-drift.py`).
- **Dream-compile & dream-score ratchets (`totals.dream_compile`, `totals.dream_scores`):**
  **11/68 dreams (16.2%)** check with zero errors (count never fewer); **61 dreams scored, median 3/5**
  (parsed from `Dream: N/5` in `research/reviews/*.md`).
- **Attributable gates & merge-safe baseline:** `scripts/scoreboard.py` attributes every ratchet to
  changed ports/files or `crates/`/`stdlib/` diffs (naming port and cause; unrelated drift warns),
  `scripts/review-check.py` requires reviews on new/changed ports (scores `< 4` tracked, non-blocking),
  and `research/scoreboard.ports.jsonl` stores 1 sorted line per port with `--update PORT...` for
  conflict-free concurrent rebases.
- **Interpreter ratio ratchets (`totals.interp`):** dedicated `interp-ratio:` section in fast/full
  reports (`p50=5.56x`, `p95=52.43x`, `max=89.04x`, `>10x: 26`; bar `<= 10x`, north star `<= 5x`),
  strictly ratcheting `p50` and `>10x` count (never worse).
- **Fast gate wall time (`totals.gate_fast`):** `scripts/check.sh --fast` now finishes in **84 s**
  (down from 249 s via `[profile.dev.package]` `opt-level = 1` for `cpl-syntax`/`cpl-check`/`cpl-ir`/`cpl-codegen`
  and overlapping `recovery_scoreboard` after `nextest`), recorded to `target/gate-fast-time.json`,
  reported on the scoreboard, and gated at `<= 180 s` (3 min).
Full-mode perf (`research/scoreboard.json`, 241 measured, 109 >= 10 ms):
runtime **p50 0.993x** (goal <= 1.1x: **PASS**), p95 4.40x, **max 310.196x** (goal <= 1.5x:
**FAIL**, 16 > 1.5x), peak RSS p50 0.938x / p95 2.330x / max 6.195x (`cf2268c`).
Interpreter vs C++ `-O2` on matched inputs (251/255 ok; 148 >= 20 ms): **p50 6.91x** (bar
<= 10x: **PASS**, north star <= 5x: **FAIL**), **max 117.80x** (`cf2262e`, 57 > 10x);
ratcheted at >10% regression on `p50` and per-port (`>= 20 ms`).
Taste scores from `research/reviews/<name>.md` are live (10 reviewed, median 4.0/5, 4 under 4)
in `scripts/scoreboard.py` (`totals.taste`) and `research/PORTS.md` (`Taste` column).

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
has 5 open traps; 16 ports exceed the <= 1.5x runtime ceiling; 57 ports exceed the 10x
interpreter-ratio bar; static `sizeof` / `-O2` assembly zero-cost metrics are not yet wired
into the scoreboard.
**Bar.** 500+ Rust-gated ports; `scripts/scoreboard.py --fast --check` green in
`scripts/check.sh --fast`; full-mode perf and interpreter baseline refreshed regularly with
runtime p50 <= 1.1x / max <= 1.5x; zero open blocker frictions; P0 bug corpus >= 90% handled.
**Next.** Add static `sizeof` / `-O2` assembly zero-cost metrics to `scripts/scoreboard.py`;
expand `bench/goal_tests/` bug-replay and revision suites as new P0 classes surface; refresh
`research/scoreboard.json` after each major wave and perf fix (investigating `cf2096e` 310x
and `ucup4s25h` 26x outliers); add checkers for remaining constructive ports; close the 5 open
traps in `known_traps.cp`.
