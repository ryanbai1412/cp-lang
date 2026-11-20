# Language decisions: what's settled, what's open

Every decision still to make, grouped by the six language features plus
cross-cutting ones. **Settled** = agreed in discussion and written into a
doc. **Leaning** = I have a proposal, you haven't confirmed. **Open** = no
proposal yet. "Where" points to the doc that has (or will have) the detail.

Suggested order: state and effects (5) first, since several open items in
2, 4 and 6 depend on the borrow rules.

**Update, 2026-11-20.** Ryan approved the unified bounded-index, slice and
lexical borrow design (plan/19-index-and-lends.md) with three changes, and
locked in Euclidean division. §9 lists what he approved and the rows it
changed.

**Update, 2026-11-19.** Ryan reviewed the open proposals and approved the
defaults; §8 lists what he approved and the rows it changed. Rows changed
that day say so in their Status. For rows still Leaning or Open, the
current lean is the working default: build on it, and change it when
porting shows friction.

Three rules for this file, from his feedback that day (plan/00-vision.md
G7, G8):

- **Every construct is justified here.** A new row says why, or points to
  the doc that does. "It was convenient for one case" is not a reason.
- **General before special.** Before adding a construct, show that a
  general mechanism (types, domains, traits, borrows, generics, the listed
  intrinsics) can't provide it as library code. Bespoke compiler support
  is the last resort and the row must say why it was needed.
- **Nothing here is protected.** "we shouldn't be afraid to massively
  change things". When porting reveals a cleaner single abstraction,
  replace the rows it covers and record the reason.

## 1. Data modeling

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 1.1 | `Int` width and range | Settled | 64-bit, `MIN = -(2^63-1)`, `LLONG_MIN` is the `?Int` niche | | plan/00-vision.md G9 |
| 1.2 | Range types pick storage width | Settled | `0..=1e9` stores in 4 bytes; `Packed<...>` only when explicit | | plan/13-abstraction.md M7b |
| 1.3 | Optionals and niches | Settled | `?T`, niche when the type has a spare value, else a flag | 1.1 | plan/13-abstraction.md M7b |
| 1.4 | Structs, records, enums | Settled | auto constructors, `with`, payload enums, exhaustive `match` | | plan/17-syntax.md |
| 1.5 | Float semantics | Open | `F64`; `==` allowed but linted; `close(a, b, eps)` in std | | |
| 1.6 | Strings | Settled 2026-11-19, defined 2026-11-20 | `Str` is the alias `[domain]Char`: a flat array of `Char` packaged with its own fresh index type, opened where it is bound (`s: [s.Idx]Char`). **ASCII only**: a `Char` is one byte, there is no UTF-8 anywhere, and a non-ASCII literal is a compile error. `List<Char>` builds one. No string-only indexing rules | 2.x | plan/19-index-and-lends.md §4.4 |
| 1.7 | `I128` / `U64` / `I64` rules | Leaning | `U64` wraps and allows bit ops; `I128` only where declared; `I64` full range, no niche | 1.1 | |
| 1.8 | Bit operations on `Int` | Leaning | shifts that can reach bit 63 are errors with "use `U64`" | 1.7 | |
| 1.9 | Integer division and modulo | Settled 2026-11-20 | **Euclidean everywhere**: `a / b` and `a % b` satisfy `a == (a / b) * b + a % b` with `0 <= a % b < abs(b)`, so `a % b` is never negative, for negative `a` and for negative `b`. (`//` is not an operator: it starts a comment.) When the range checker proves `a >= 0` and `b > 0`, both lower to the native C++ `/` and `%` with no extra code; otherwise to the two-instruction fix-up (`cp_ediv` / `cp_emod` in the prototype). The law holds at the ends of `Int` too (`b = Int.min`, `a % -1 == 0`); `b == 0` and `Int.min / -1` stop the program with a message in every build | 1.1 | plan/11-values-and-types.md |

## 2. Index spaces

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 2.1 | Domains, `[V]T`, unchecked indexing by `V` | Settled, revised 2026-11-20 | `domain V = 1..=n` is sugar for a sealed nominal struct over a range type with `impl Index` and `impl Seq`; its width follows the range (4 bytes at most for input-sized domains). Checks only where positions are created; the no-check guarantee is now derived from `to_idx: 0..K`, not a rule about domains | | plan/19-index-and-lends.md §3.1 |
| 2.2 | Bounds (gaps between positions) | Settled, revised 2026-11-20 | `V.bounds`; **`v.l` / `v.r`** are the boundaries left and right of `v`, and `v.l(k)` / `v.r(k)` the far end of the `k`-wide window ending / starting at `v` (so `v.l == v.l(1)`). `v.before` / `v.after` stay as the long names. Prefix sums are indexed by bounds: `pre[v.r] - pre[v.l(k)]` | | plan/19-index-and-lends.md §3.1 |
| 2.3 | `Range<D>` / `Range1<D>` | Settled | ranges stored as two bounds (`s.lo`, `s.hi`); views yield `Range1`. Library structs over `Seq` | | plan/12-domains-and-sequences.md §2, plan/19-index-and-lends.md §4.1 |
| 2.4 | Slices | Settled, revised 2026-11-20 | `a[s]` is a library projection that lends a slice. A slice is indexed by the positions of its array (`V`), with a window check unless proven; **`Sub<V>` is removed**. The window is a value, the slice is a lend: fns return ranges, callers apply them | 5.12 | plan/19-index-and-lends.md §4.2 |
| 2.5 | Product and fresh domains | Settled, revised 2026-11-20 | `(A, B)` is an `Index`, so `a[i, j]` is sugar for `a[(i, j)]` on any container; `domain D` in a signature or binding introduces a fresh index type that can't leave the block that introduced it (`compress`, `scc`, Euler order) | | plan/19-index-and-lends.md §3.2, §4.3 |
| 2.6 | Domains that depend on a value | Open | e.g. "children of `v`" as a domain; candidate: `domain Kids(v: V) = ...` | 3.x | design-v2 C2 |
| 2.7 | Growable domains | Leaning | pools grow; `freeze()` turns them into ordinary domains | 5.6 | design-v2 |
| 2.8 | Conversions between related domains | Settled, revised 2026-11-20 | strict, and now **no implicit conversion at all**: `Sub<V>` is gone (2.4), so every conversion is a named method (`v.l`, `pre.vertex`, `V(x)`). Full list for `Rev<V>`, `Pre`, `WithMin<V>` still to write | 2.4 | research/RECOMMENDATIONS.md §3, plan/19-index-and-lends.md §4.2 |
| 2.9 | `v + 1` on positions | Leaning, revised 2026-11-20 (was Settled: an error) | `v + k` and `v - k` are the optional step, a `?V` that is `none` outside the domain: `if let u = v - k`, `v + k ?? V.last`, `(v + 1)!`. `v.offset(k)` stays as the named form of the same step, `v.next` / `v.prev` for one step, and `v.cyc(k)` wraps. Six ports did the step on raw `.idx` with a hand-written guard | | §11; research/ergonomics-audit.md §3 |
| 2.10 | Self-reading tables | Leaning | allowed only for cells provably filled already | | plan/12-domains-and-sequences.md §4 |
| 2.11 | Escape hatch from domain checks | Settled | explicit and greppable: `a.raw[i]` with an `Int` index; never implicit. `a.raw` is a library projection lending the same storage indexed by `0..K`. With `unchecked:` (7.1) and raw intrinsics these are the contest escape hatches of G15 | 2.1 | research/RECOMMENDATIONS.md §3 |
| 2.12 | `Name<` generics vs less-than | Leaning | capitalized name and a tentative type-argument parse closed by `>`; otherwise a comparison (`INF < x`) | | compiler/03-parsing.md, plan/17-syntax.md |
| 2.13 | Explicit type arguments on lowercase generic fns | Settled 2026-11-19 | inferred from the expected type by default; `f::<T>()` when you must say it | 2.12 | compiler/03-parsing.md |
| 2.14 | `[domain]T` packages | Settled 2026-11-20 | an array stored with a fresh index type of its own (a pointer and a length); binding one opens it as `[x.Idx]T`. Serves `Str`, jagged rows and `List<[domain]Int>` with no string or CSR rule | 2.5 | plan/19-index-and-lends.md §4.4 |
| 2.15 | Indexing is a trait | Settled 2026-11-20 | `a[i]` is `Indexed<I>.at`, a projection; `[I]T` is sugar for the library struct `Array<I, T>`, and grids, strings and permutations are library code over it. The compiler knows no domain, string, grid or permutation rule | 3.5, 5.12 | plan/19-index-and-lends.md §3.3, §7 |

## 3. Abstraction mechanisms

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 3.1 | Traits vs operation values | Settled | trait for the one canonical behavior, value when there are several | | plan/13-abstraction.md M2/M3 |
| 3.2 | Specialization | Settled | per type and per fn identity; runtime-chosen code is an error | | plan/13-abstraction.md M3 |
| 3.3 | `dyn` escape hatch | Open | keep `dyn (T, T) -> T` for explicit indirect calls, or error only | 3.2 | plan/13-abstraction.md M3 |
| 3.4 | Laws | Settled | declared on operation types, checked by `cp fntest` and debug runs | | plan/13-abstraction.md M4 |
| 3.5 | Representation traits | Settled, revised 2026-11-20 | `Niche`, `Index`, `Seq`, `Bits`, `Small`, `Relocate`, `Zero`, `OrdKey`, `Bounded`. `Seq` is `Index + Ord` for positions in a line (`next`, `offset`, `cyc`, `l`, `r`, `bounds`); enums, tuples and masks are `Index` but not `Seq` | | plan/13-abstraction.md M7b, plan/19-index-and-lends.md §3.1 |
| 3.6 | Implicit representation choices | Settled | only when no worse on any op or memory, and visible (G9) | | plan/00-vision.md G9 |
| 3.7 | Structural field bounds | Leaning | `A has to: V` as a general feature, not graph-specific | | plan/13-abstraction.md G9 |
| 3.8 | Associated types, consts and frozen lets | Settled 2026-11-20 | yes. Associated types (`Indexed.Elem`, `Seq.bounds`). An associated size is `const K = 10` when known at compile time and **`let K = n`** when frozen from a run-time value where the impl is declared; **`const K = n` with a run-time `n` is an error**, so `const` stays strictly compile-time (7.10). Sizing by a `var` is an error | 7.10 | plan/19-index-and-lends.md §3.1 |
| 3.9 | Coherence (who may `impl` what) | Settled 2026-11-19 | orphan rule relaxed inside one program, strict for libraries | 6.4 | §8 |
| 3.10 | `ModInt` moduli | Settled 2026-11-19 | compile-time `Mod<P>` and runtime `ModR` as two types | 3.2 | plan/13-abstraction.md |
| 3.11 | User-defined decorators | Leaning | a `decorator` declaration that rewrites the item hygienically during name resolution; type-dependent choices stay in generic stdlib code | 3.1 | compiler/04-names-and-modules.md |

## 4. Control and iteration

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 4.1 | Blocks as expressions, `match`, `?` | Settled | indentation blocks, `=>` arms, `?` on optionals | | plan/17-syntax.md |
| 4.2 | Generators | Settled, revised 2026-11-19 | inlined; the compiler picks push or pull per use: push for a plain `for`, pull for `zip` or when stored as `Iterator<T>` (so a non-recursive generator can be stored); a recursive generator where pull is needed is an error suggesting `List(g)`; no hidden explicit stacks | | research/RECOMMENDATIONS.md §2, plan/12-domains-and-sequences.md §5 |
| 4.3 | Sequence views | Settled | `runs(a, by:)`, `pairs`, `windows`, `chunks`, `split`, `prefix`, `scan`, `sliding` with explicit state | 4.2 | plan/12-domains-and-sequences.md §6 |
| 4.4 | `zip` of two generators | Settled 2026-11-19 | push the outer, pull the inner; only non-recursive generators can be pulled | 4.2 | research/RECOMMENDATIONS.md §2 |
| 4.5 | `return` / `break` / `?` in `do:` blocks and lambdas | Open | `?` exits the `do:` block; `return` always exits the enclosing fn | | design-v2 Part 10 |
| 4.6 | Labeled `break` | Settled 2026-11-19 | `break 'outer` | | §8 |
| 4.7 | Exhaustiveness for range patterns | Open | `match x: 0 => .., 1..=9 => ..` checked against `x`'s range type | 1.2 | |
| 4.8 | Views as read-only array arguments | Settled 2026-11-20 | a lent row or slice is a `[B]T` to the callee, with no adapter; a fn that wants any indexable thing opts in with `A: Indexed<V, Elem = T>`; concrete `[V]T` parameters stay concrete | 3.2 | plan/19-index-and-lends.md §3.3, §3.5 |

## 5. State and effects

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 5.1 | Ownership model | Settled | variables own, params borrow, no pointers, `mut` at both ends | | plan/17-syntax.md |
| 5.2 | Copies and moves | Settled | big values need `.copy()`; last-use moves are implicit | | plan/17-syntax.md |
| 5.3 | Precise borrow rules | Settled 2026-11-19, revised 2026-11-20 | lexical exclusivity: no stored and no returned borrows (so no lifetimes); fields borrow separately; two lends of one projection are separate when its `disjoint` law is proven from flow facts (5.13), which replaces "indices don't borrow separately"; index arguments are evaluated before any lend begins (two-phase); generator-lent values live one iteration | 5.1 | plan/19-index-and-lends.md §3.4 to §3.6, §5 |
| 5.4 | Inline closures reusing a caller's `mut` borrow | Settled 2026-11-19 | yes: a lambda passed to an `inline` parameter is checked as part of the caller's body, so `undo` is an ordinary library fn. Projections (5.12) are its second client | 5.3 | research/RECOMMENDATIONS.md §1 |
| 5.5 | Moves out of arrays | Settled 2026-11-20 | `swap`, `take` (leaves default) and `pair_mut` are library fns over `at`, allowed by its `disjoint` law; no partial moves | 5.13 | plan/19-index-and-lends.md §3.6 |
| 5.6 | Pools and handles | Leaning | `pool Nodes: Node`; typed handles; columns per field | | plan/13-abstraction.md M9 |
| 5.7 | `defer` | Leaning | core; runs on every exit; evaluated at exit; no unwinding on crash | | state-structure-views.md §1 |
| 5.8 | `Rollback`, `Journaled`, `undo`, `scoped` | Leaning | plain arrays not rollback-able; journaling is opt-in by type | 5.4, 5.7 | state-structure-views.md §1 |
| 5.9 | `@memo` control | Open | table layout from key type; evaluation order; clearing; I/O is an error | | plan/15-program-structure.md §3.3 |
| 5.10 | `mut fn` | Settled | a statement; captures `var`s above it; can't escape | 5.3 | plan/15-program-structure.md §3.2 |
| 5.11 | Keeping the borrow rules understandable | Settled 2026-11-19 (the method; the rules stay open to change) | fresh agents repeatedly search for the smallest programs where a borrow rule surprises; each one changes the rule or its error message and is kept as a test | 5.3 | plan/agent-context.md |
| 5.12 | Projections: `lends` / `lend` | Settled 2026-11-20 | `fn row(self, r: A) lends mut [B]T` with a body that reaches one `lend place`. Meaning: a fn with one extra `inline` callback, called once at the `lend`; the caller's continuation (the rest of the statement, or of the block for a `let`) is that callback. No reference is returned. Justification: it is what makes slices, rows and element access library code without lifetimes | 5.3, 5.4 | plan/19-index-and-lends.md §3.4 |
| 5.13 | `law disjoint` | Settled 2026-11-20 | a projection may state when two of its lends don't overlap (`law disjoint(i, j) = i != j`). Two live lends of one projection on one receiver, one of them `mut`, need the law proven from flow facts (`if`, `while`, `assert`, `requires`, value ranges); otherwise a compile error that prints the law. Never a silent check; a wrong law is caught by debug runs and `cpl fntest` | 3.4, 5.12 | plan/19-index-and-lends.md §3.6 |
| 5.14 | Lent values are second-class | Settled 2026-11-20 | a value built from lent parts is lent and lives only in the block that made it: it may be passed, bound by `let`, iterated and lent onward; not put in a `var`, a field or a container, captured by a non-`inline` lambda, or returned. Borrowed-ness is a mode of the binding, not a type, so there is no `view` keyword | 5.3 | plan/19-index-and-lends.md §3.5 |

## 6. Program structure

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 6.1 | File as function body | Settled | top-level fns see the body's `let`s; calling before the `let`s are set is an error | | plan/15-program-structure.md §2 |
| 6.2 | Program vs library mode | Settled | decided by contents; `use` accepts only libraries | | plan/15-program-structure.md §2.4 |
| 6.3 | Multitest | Settled | file-level `@multitest`; `static let` for shared data; `return` ends a test | 6.1 | plan/15-program-structure.md §6 |
| 6.4 | Modules beyond `pub` | Open | re-exports, private-to-folder, name clashes | | plan/40-tooling.md |
| 6.5 | Parts and purity | Settled | any fn is a part; tools record I/O; no `pure` keyword | | plan/15-program-structure.md §4 |
| 6.6 | Brutes, subtasks, checkers, interactors | Settled | brutes on real calls; `@solution(f, when:)`; checker gets real answers only | | plan/15-program-structure.md §5, §8, §11 |
| 6.7 | Staying under 64 KB | Open | tree-shaking plus a per-instance size report; maybe dedupe identical instances | 3.2 | plan/40-tooling.md |
| 6.8 | Grader-style (IOI) problems | Open | how top-level code becomes the graded function | 6.1 | plan/16-io.md |
| 6.9 | Top-level `var` | Leaning | allowed, invisible to fns except `mut fn` | 5.10 | plan/15-program-structure.md §15 |

## 7. Cross-cutting

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 7.1 | Checks in judge vs local builds | Settled 2026-11-19 | domain indexing never checked; `List` and `a.raw[i]` checked in both builds unless proven or hoisted; `assert` stays; input validation local only; `unchecked:` blocks drop the remaining index checks in the judge build | | research/RECOMMENDATIONS.md §4 |
| 7.2 | Overflow in release builds | Settled 2026-11-19 | proven safe: no check; provably reachable: a warning with the bound; unknown: trapped in local builds, unchecked in the judge build | 1.1 | research/RECOMMENDATIONS.md §4, plan/30-compiler.md |
| 7.3 | Trace data contract | Open | spans, typed values, writes old → new, call tree, test and part boundaries | 6.5 | plan/50-debugging.md |
| 7.4 | Zero-cost guarantees | Open | list of constructs promised to match hand-written C++, checked by benchmarks | 4.2, 3.2 | |
| 7.5 | C++ interop | Open | `@cpp` blocks with checks at the boundary | 1.1 | |
| 7.6 | Remaining syntax | Leaning | lambdas, inline `if`, numeric literals (decisions 21–33) | | plan/17-syntax.md |
| 7.7 | TLE / MLE / overflow estimates | Settled | from typed input bounds and domain sizes; **warnings only**, never errors | 6.3, 7.2 | |

| 7.8 | Typed input drives generators and shrinkers | Settled 2026-11-19 | adopted: one input type gives reading, validation, random generation and shrinking | 6.6 | plan/18-key-features.md §1 |
| 7.9 | Contracts | Settled 2026-11-19 | adopted | 3.4 | plan/18-key-features.md §2 |
| 7.10 | Compile-time `const` | Settled 2026-11-19 | adopted, with a size warning for large baked tables | 6.7 | plan/18-key-features.md §3 |
| 7.11 | REPL on a loaded test | Deferred 2026-11-19 | not now; cheap to add once the debug interpreter exists | | plan/18-key-features.md §4 |
| 7.12 | Command name and file extension | Settled 2026-11-19 (name is a placeholder) | the binary is `cpl` until Ryan picks a final name, because `cp` clashes with Unix `cp`; source files are `.cp` | | plan/b0-distribution.md |
| 7.13 | Editor support in the core | Settled 2026-11-19 | LSP with partial-parse recovery, VS Code extension and syntax highlighting are P0 (G13) | | plan/60-editor.md |
| 7.14 | Trace viewer design | Settled 2026-11-19 | designed fresh with its own design system; the old `mock/trace-mock.html` is rejected and not in the repository. What the trace must contain (7.3) is unaffected | 7.3 | plan/50-debugging.md |
| 7.15 | Sample local library | Settled 2026-11-19 | a fresh one, inspired by popular contest libraries, not a port of `ryanbai1412/cp-library`; it also proves G8 and tests library pinning (G11) | 6.4 | plan/40-tooling.md |
| 7.16 | `test` and `doc` for libraries | Settled 2026-11-19 | `cpl test` and `cpl doc` work the same on the stdlib and on a user's own library | 6.4 | plan/45-dev-tools.md |
| 7.17 | How conciseness is measured | Settled 2026-11-19 | two numbers per port: % fewer lines than the original, and % fewer than the original with library-able code stripped | | plan/80-validation.md §8.1 |

## 8. Approved on 2026-11-19

Ryan answered the open proposals in one pass. In his words where he gave
them, otherwise as the default he approved.

**Goals** (plan/00-vision.md). Added as P0: zero-UB compiler trust (G12),
zero compiler fights under contest pressure (G14), low mental overhead
(G16), and "it just works" (G17). Promoted to P0: the editor with
partial-parse recovery (G13), pinned local libraries (G11), the
general-purpose core (G8), and a fast toolchain (G10). Added as P1:
clean contest escape hatches (G15). G7 now also requires every decision to
be justified.

**Key features** (plan/18-key-features.md). Typed input generators and
shrinkers, contracts and `const` are in (7.8 to 7.10). The REPL is deferred
(7.11).

**Research recommendations** (research/RECOMMENDATIONS.md). All four are
adopted:

- no stored borrows in structs, with the borrow rules that go with it (5.3,
  5.4);
- the compiler picks push or pull for each generator use (4.2, 4.4);
- strict domain conversions, with `a.raw[i]` as the way out (2.8, 2.11);
- `unchecked:` blocks, and unproven overflow trapped in local builds only
  (7.1, 7.2).

**Leans taken.** `Str` is `[domain S]Char` (1.6); `Mod<P>` and `ModR` are
two types (3.10); `f::<T>()` gives explicit type arguments (2.13);
`break 'outer` is the labeled break (4.6); the orphan rule is relaxed
inside a single program (3.9).

**Project.** The command is `cpl` for now and files are `.cp` (7.12). The
repository is `ryanbai1412/cp-lang`; work is pushed straight to `main`,
which must stay green. The trace viewer is designed fresh (7.14). The
sample library is new (7.15). Conciseness is measured two ways (7.17).

## 9. Approved on 2026-11-20

Ryan approved `plan/19-index-and-lends.md`: one mechanism for `domain`,
`Str`, `Grid` and `Perm`, and lexical exclusivity as the borrow model.

**The design.** A bounded index is a sealed struct with `impl Index`
(2.1, 2.15). Indexing is the `Indexed` trait and `[I]T` is a library
struct (2.15). Slices, rows and element access are projections that lend
part of their receiver (5.12), lent values are second-class (5.14), and
two lends of the same container are allowed when the projection's
`disjoint` law is proven (5.13). No borrow is ever stored or returned
(5.3). `Sub<V>` is removed (2.4, 2.8). Decisions 3.8, 4.8 and 5.5, which
were open, are settled by it.

**His three changes.**

- `let K = n`, not `const K = n`, for a size frozen from a run-time
  value. `const` stays strictly compile-time (3.8, 7.10).
- `v.l` / `v.r`, with `v.l(k)` / `v.r(k)`, as the short spellings of
  `v.before` / `v.after` (2.2).
- `Char` and `Str` are ASCII only: one byte per `Char`, a flat `[S]Char`
  or `List<Char>`, and no UTF-8 (1.6).

**Also locked in.** `/` and `%` are Euclidean on integers, and cost
nothing when the operands are proven non-negative and positive (1.9).

**Prototype.** `proto/cpc/` implements the surface rules of the above and
keeps the 30 corpus ports passing; `plan/19-index-and-lends.md` §11 lists
what it covers and what it leaves out. One call made there and not yet
confirmed by Ryan: `v.l(k)` and `v.r(k)` count `v` itself, so `v.l` is
`v.l(1)`.

## 10. Calls made while writing the standard library (2026-11-20)

`stdlib/` and `examples/local-lib/` were written against the approved
design. Where the plan was silent, these calls were made. None is
confirmed by Ryan yet; each is small and local to change.

- **`|=`, `&=`, `^=`.** Compound assignment for the bit operators, like
  `+=`. `Bitset` needs them on every line (`w[i] |= bit(k)`), and their
  absence was the first friction hit.
- **A struct may own a domain.** `Rooted<V>` declares `domain Pre = 0..V.K`
  in its body, and outside it is `Rooted<V>.Pre`. Preorder positions and
  vertices are then different types, so `tin[v]` cannot be used as a
  vertex. This is the associated-type half of 3.8 used for a domain.
- **A function may return a fresh domain.** `scc` returns
  `(domain C, comp: [V]C)`: the component count is only known at run
  time, and the caller binds it with `let (C, comp) = scc(...)`. Same
  mechanism as `let K = n` (3.8).
- **Graph algorithms take the neighbour function**, `adj: V -> Iter<A>`,
  not a graph type. `Graph<V, E>` is one provider; a closure over a grid
  or a number line is another. The vertex domain is given by turbofish
  when it cannot be inferred: `dijkstra::<V>(...)` (2.13).
- **String algorithms take `List<T>` and return `Int` positions.** They
  only compare elements, and their outputs (`z`, the prefix function, a
  suffix array) are lengths and offsets that get added and subtracted
  constantly. Typing them over the `[domain]Char` package (1.6, 2.14)
  is left for when the Rust checker can show whether it reads better.
- **`Bitset` words are `Int`s using all 64 bits.** The helpers `bit`,
  `low`, `shl` and `shr` in `bitset.cp` avoid shifts that reach bit 63,
  so the file obeys the 1.8 lean without `U64`. When `U64` lands, the
  four helpers collapse to plain shifts.
- **The library runs on a reference interpreter**, `proto/cpi/`, until
  the Rust compiler can run it. It checks at run time and does no static
  typing; `proto/cpi/README.md` lists what that does and does not prove.
  The Rust front end has checked the library statically since §13.

## 11. Leaning since 2026-11-20: the ergonomics audit

A read of all 30 ports as code (`research/ergonomics-audit.md`) changed
the prototype in the ways below. They are implemented in `proto/cpc/`,
eleven ports use them, and `proto/tests/ergo/` locks them. None is
confirmed by Ryan yet, so each is **Leaning**. Per the rules at the top of
this file, each says what is library code and what is not.

- **The optional step (2.9, revised).** `i + k` and `i - k` on a position
  are a `?I`. In the library this is `impl Add<Int>` on any `Seq`, the
  body of the `offset` that 12 §1.3 already had. It revises a settled row:
  `v + 1` was an error. The reason is the six ports that did the step on
  raw `.idx` with a separate guard; the step can still never produce a
  position outside the domain.
- **`if let a = x and let b = y`.** Syntax. It extends the existing
  `if let a = x and cond`, so two optional steps need no nesting.
- **Open ranges `..i`, `..=i`, `i..`.** Syntax: a range literal may omit
  an end, and takes its type from the end it has. `Range<I>` is the
  library struct of plan/19 §4.1.
- **A boundary may end a range.** Library: `Range<I>` is two `Bounds<I>`,
  so `..b` and `b..` are the cells before and after boundary `b`. `..=b`
  is an error.
- **`span(a, b)`, `r.prefix(k)`, `r.suffix(k)`, `b.left`, `b.right`.**
  Library functions and methods on `Range` and `Bounds`; `left` and
  `right` are already in plan/19 §4.1.
- **Code after a `lend` (write-back).** Already designed in plan/19 §3.4;
  now prototyped. It is what lets a container keep a derived value right
  on every write.
- **`Counter<K>` and `counts(xs)`.** A library struct whose `at` is a
  projection with a write-back; `proto/tests/lend/writeback.cp` is that
  struct in `.cp`. The prototype compiles it in, as it does `SegTree`.
- **`prefix(a)` with `pre.sum(range)` / `pre.sum(rows, cols)`.** A library
  struct over `[I.bounds]T`, compiled in for the same reason.
- **`x f= y` for any two-argument fn.** Syntax, and one rule in place of
  two: `min=` and `max=` are now instances of it.
- **`[domain S]Letter = read()`.** Library: `read` for an array whose
  element is a domain of `Char`.

## 12. Calls made while hillclimbing hard DP, graph and tree problems (2026-11-20)

Fifteen hard corpus problems were re-solved from scratch in
`proto/ports/hillclimb_dp_graph/` (bitmask, subset, digit, interval, tree
and rerooting DP; SCC, bridges, heavy-light paths, shortest paths;
constructive). Each change below removes boilerplate that showed up in
more than one of them, and each is a general rule rather than a case. The
ports as they had to be written before are kept in `before/`;
`python3 proto/loc.py hillclimb_dp_graph` prints both. On the twelve ports
a change touched: 364 lines before, 235 after; the library-stripped
reduction against the C++ originals went from 43% to 63%.

- **`Subset<D>` is a set, so it has set algebra.** `s | t`, `s & t`,
  `s - t`, `s ^ t`, and `for g in s.subsets` (every subset of `s`, `s`
  first, the empty set last). Before, a submask walk dropped to integers:
  `g = (g - 1) & s.idx` and `Group.nth(s.idx ^ g)`. Now the Grouping DP is
  `for g in s.subsets: best[s] max= score[g] + best[s - g]`. No new type:
  the operators are the ones `Set` already has.
- **`v.cyc(k)` on every domain** (specified in plan/19 §3.2, now built):
  `k` steps round the domain, Euclidean, so `k` may be negative. Ring DP
  and ring input stop spelling `Bead((i.idx + k) % n)`.
- **A read range is the range that was written.** `let m: n - 1..=5e5 =
  read()` and `let k: 1..=n = read()` used to check and generate against
  the static envelope only (`1..=5e5`), so ports added a generator and a
  `where` to say the same thing twice. The bounds are now evaluated at the
  read: input outside them stops with the line number, and stress inputs
  are drawn from them: one rule, the same as for constant bounds.
- **`xs + ys` joins two lists.** Constructive answers are often "this run,
  then that one": `print(odds + [5, 4] + evens)`.
- **`Graph.undirected(edges)` and `Graph.directed(edges)` build the
  adjacency from the edge list as read** (`[E](u: V, v: V)`, or with a
  third field that rides along: `for (v, w) in g[u]`). Every graph port
  had the same four-line loop. This is `stdlib/graph.cp`'s constructor;
  the prototype compiler now has the stand-in.
- **`Rooted(adj, root)` is the depth-first spanning tree.** On a tree that
  is the tree. On any connected graph, every edge it leaves out joins a
  vertex to one of its ancestors, which is the one fact bridges and cut
  vertices need: `low` is four lines over `tree.preorder.rev`, with no
  hand-written DFS, entry and exit clocks or child lists. No new type and
  no new function; `Rooted` only stopped assuming its input was a tree.
  `stdlib/tree.cp`'s `Rooted.new` still searches breadth-first and is to
  follow.
- **The `Rooted` stand-in grew to the stdlib surface**: `children(v)`,
  `size(v)`, `is_anc(a, b)`, `dist(a, b)`, `ancestor(v, k)`,
  `step(a, b, k)`, and the heavy-light layout as `tree.Pre`, `tree.sub(v)`
  and `tree.chains(a, b)`. A subtree is one range of positions and a path
  a few, so a path or subtree update is a loop over ranges on any range
  structure. Tree DP reads `for c in tree.children(v)` instead of
  filtering neighbours by parent.
- **`scc` and `components` return a fresh domain**, `let (C, comp) =
  scc(g)`, numbered so that arcs lead to later components, and
  `condense(g, comp)` is the DAG. A DP over the condensation is then
  `for c in C.rev`, with no topological sort. (Mechanism: section 10, "a
  function may return a fresh domain".)
- **`Dsu<V>()` and `AssignSum<D>()`** are stand-ins for `stdlib/dsu.cp`
  and for `LazySegTree` with the new `AssignSum` action in
  `stdlib/lazy.cp` (range assign, range sum), next to the existing
  `AddMax<D>()`.

Left as friction, not fixed here: the prototype compiler has no `Float`
(one candidate problem was swapped out for it); `let (a, b): (T, U) =
read()` and `var (lo, hi) = ...` are not accepted; every tree port writes
its own two-line random-tree generator; a tree given as a parent array
needs a three-line loop before `Rooted`; rerooting is still a hand-written
prefix and suffix pass (a `reroot` combinator is the next candidate).

## 13. Leaning since 2026-11-20: scout batch 1

The first scout batch (`research/friction-backlog.md`) re-solved ten
corpus problems and listed what the prototype made awkward. The changes
below are implemented in `proto/cpc/`, eight of the ten sketches now pass
against their originals as ports, and `proto/tests/ergo/` and
`proto/tests/bad/` lock them. None is confirmed by Ryan yet, so each is
**Leaning**, and each says what is library code and what is not.

- **A compound assignment on a missing map key starts from the identity
  of its operation.** A rule, and a bug fix: `m[k] min= x` on a key that
  was not there started from 0. Now `m[k] f= x` on a missing key stores
  `f(id, x)`, where `id` is the identity of `f` in `std.algebra`: 0 for
  `+`, `-`, `|`, `^` and `gcd`, 1 for `*` and `lcm`. `min` and `max` have
  no identity over `Int`, so they store `x`, the same "insert if none"
  that `min=` on a `?Int` already had. An operation with no known identity
  is a compile error that asks what a missing key means:

      m[d] min= hops                      // first write stores hops
      m[d] /= 2                           // error: `/` has no identity to start it from
      m[d] = (m.get(d) ?? 1) / 2          // what the error suggests

  This is the monoid rule the library already uses (`opt` of a semigroup),
  applied to one place, not a special case for `min`.
- **`e?` works inside an expression.** Syntax, widening 4.1: `?` was only
  allowed on a whole `let`. Now `i.ord + on_foot[line[i]]?` leaves the fn
  with `none` when the operand is `none`. The operands before it are still
  evaluated first, left to right. Inside a branch, a lambda or a
  comprehension it stays an error for now, because "leave the fn" from
  there is not obvious to a reader.
- **`let a = x and let b = y else:` and `let (a, b) = (x, y) else:`.**
  Syntax. The same `and` chain `if let` already has, on the early-exit
  form, so two optional steps need neither nesting nor a second `else`.
- **`min` and `max` skip `none`.** Typing: with every argument a `T` the
  result is a `T` as before; with some `?T` the `none`s are skipped, and
  the result is a `T` when at least one argument is a plain `T`, a `?T`
  otherwise. It is the `opt` monoid again: `none` is the identity.
- **`.field` is the fn `x => x.field`.** Syntax, only where a
  one-argument fn is expected: `sorted(links, by: .cost)`,
  `Rooted(roads, root, weight: .len)`.
- **An operator names its two-argument fn.** Syntax, only as a call
  argument: `scan(a, +)`, `SegTree<V, Int>.of(+, f)`. With it `x f= y`,
  `scan(a, f)` and `.of(f, ...)` all take the same thing, an operator or a
  named fn.
- **`[k: v, ...]` is a map literal** and `[n - 1]T` takes any `Int`
  expression as a count. Syntax; both were parse errors.
- **A read range is the range that was written, inside records too.**
  Section 12 made `let k: 1..=n = read()` check and generate against the
  value `n` has. The same now holds for a range that is a field or an
  element: `let friends: [n](x: l..=r, v: 1..=100) = read()`.
- **`read(columns)`.** Library: an array of records is read one field at
  a time (all the outcomes, then all the points) instead of one record at
  a time. The data keeps the shape the solution wants; only the input
  order differs.
- **An enum whose `@tags` are single characters reads packed.**
  `@tags("W", "L")` reads `WLWW` as four values, the way `Char` already
  does. Library (`read` for such an enum).
- **`use std.module.{Name}` in a port.** The prototype compiler still
  cannot compile the library's own `.cp` (that needs structs with
  methods and generics in `cpc`), so it compiles in a stand-in for each
  name, as it already did for `SegTree`. The `use` is checked against the
  real module: the name must be `pub` in `stdlib/module.cp`. The new
  stand-ins are `bfs`, `choose`, `Steps`, `scan` and `running_sum`, and
  using one of them without its `use` is an error. Each has the same
  name, arguments and meaning as the `.cp` definition, which
  `proto/cp.py test stdlib` runs on the interpreter. The stand-ins that
  were there before (`SegTree`, `Rooted`, `Dsu`, `Graph`, `scc`) still
  work without a `use`, because the existing ports use them that way; a port may
  import them, and `Dsu<V>.new()` and `.count` are accepted as the
  library spells them.
- **Library additions.** All plain library code in `stdlib/`:
  `choose(n, k)` in `std.math`; `scan(xs, f)` and `running_sum(xs)` in
  the new `std.seq`; `Steps` in the new `std.steps` (`add(range, x)`,
  then `runs(clip)` gives the maximal runs of equal value, which is the
  sorted-events sweep); `SegTree.of(m, f)` with `refresh(i)` and `.all`,
  so a tree over a formula states the formula once; `Rooted.weighted` and
  a `dist` that adds edge lengths (it still counts edges when none are
  given). In the prototype compiler only:
  `Set(xs)`, `take(place)`, `a.swap(i, j)` (plan/19 §4.5),
  `D.windows(k)`, `first_where` on any range,
  `pre.sum(from: v, len: k)` wrapping around the domain, `prefix` over a
  `Bool` array or fn (it counts), and on the `Rooted` stand-in of
  section 12 `weight:`, with `tree.kids` and `tree.root` as the library
  names them.

Two things were looked at and not done. `m[k] f= x` could have required
an explicit start value for every operation; that makes the common
`m[k] += 1` longer to protect the rare `m[k] /= x`, so only the
operations without an identity ask. And `?` could have been allowed
everywhere; see above.
||||||| parent of 7a2691c (Document what the Rust front end now checks)

## 14. Calls made bringing the Rust front end up to the libraries (2026-11-20)

The Rust checker now accepts `stdlib/`, `examples/local-lib/` and
`proto/ports/` unchanged. Where the source needed a rule the plan does
not state, these calls were made. None is confirmed by Ryan yet.

- **14.1 A value with more fields fits where fewer are asked for.** The
  library passes a `Group<T>` where a `Monoid<T>` is expected, and a
  `(u, v, w)` edge where `(u, v)` is:

  ```
  struct Monoid<T>(id: T, op: (T, T) -> T)
  struct Group<T>(id: T, op: (T, T) -> T, inv: T -> T)
  let sums = SegTree<I, Int>.from(Sum<Int>, a)    // Sum<Int> is a Group<Int>
  ```

  The checker accepts a struct for another struct when it has every field
  the other has, at the same types, and a record for a narrower record.
  The alternative is a conversion at every such call.
- **14.2 An impl's type parameters are read off its header.**
  `impl Add for Mod<P>` does not declare `P`; a type argument in the
  header that names nothing in scope is a parameter of the impl.
- **14.3 Top-level statements see the lends and asserts before them.**
  A file's top-level statements are checked one at a time. A slice bound
  by a top-level `let` still keeps its array lent in the statements after
  it, and a top-level `assert i != j` holds until `i` or `j` is assigned:

  ```
  let part = a[l..=r]
  a[l] = 0          // error: `a` is lent to `part` until its last use
  print(part[l])
  ```
- **14.4 The prototype's built-in names are accepted without a type.**
  The ports use `SegTree`, `Mod`, `dijkstra` and a dozen more names with
  no `use`, because `proto/cpc` builds them in. The checker lists them in
  `crates/cpl-check/src/prelude.rs` and does not type what goes through
  them. This is a stopgap: the fix is for each port to `use` the `std`
  module, at which point the name leaves the list.
