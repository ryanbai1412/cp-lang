# Language decisions: what's settled, what's open

Every decision still to make, grouped by the six language features plus
cross-cutting ones. **Settled** = agreed in discussion and written into a
doc. **Leaning** = I have a proposal, you haven't confirmed. **Open** = no
proposal yet. "Where" points to the doc that has (or will have) the detail.

Suggested order: state and effects (5) first, since several open items in
2, 4 and 6 depend on the borrow rules.

## 1. Data modeling

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 1.1 | `Int` width and range | Settled | 64-bit, `MIN = -(2^63-1)`, `LLONG_MIN` is the `?Int` niche | | plan/00-vision.md G9 |
| 1.2 | Range types pick storage width | Settled | `0..=1e9` stores in 4 bytes; `Packed<...>` only when explicit | | plan/13-abstraction.md M7b |
| 1.3 | Optionals and niches | Settled | `?T`, niche when the type has a spare value, else a flag | 1.1 | plan/13-abstraction.md M7b |
| 1.4 | Structs, records, enums | Settled | auto constructors, `with`, payload enums, exhaustive `match` | | plan/17-syntax.md |
| 1.5 | Float semantics | Open | `F64`; `==` allowed but linted; `close(a, b, eps)` in std | | |
| 1.6 | Strings | Open | `Str` = `[domain S]Char` (bytes), so strings get domains for free | 2.x | |
| 1.7 | `I128` / `U64` / `I64` rules | Leaning | `U64` wraps and allows bit ops; `I128` only where declared; `I64` full range, no niche | 1.1 | |
| 1.8 | Bit operations on `Int` | Leaning | shifts that can reach bit 63 are errors with "use `U64`" | 1.7 | |

## 2. Index spaces

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 2.1 | Domains, `[V]T`, unchecked indexing by `V` | Settled | positions are 4-byte ordinals; checks only where positions are created | | plan/12-domains-and-sequences.md §1 |
| 2.2 | Bounds (gaps between positions) | Settled | `V.bounds`, `v.before` / `v.after`; prefix sums indexed by bounds | | plan/12-domains-and-sequences.md §1.4 |
| 2.3 | `Range<D>` / `Range1<D>` | Settled | ranges stored as two bounds; views yield `Range1` | | plan/12-domains-and-sequences.md §2 |
| 2.4 | Slices | Settled | borrowed views; positions are `Sub<V>` and convert to `V` | 5.1 | plan/12-domains-and-sequences.md §3 |
| 2.5 | Product and fresh domains | Settled | `(A, B)`; `compress`, `scc`, Euler order return fresh domains | | plan/12-domains-and-sequences.md §1.5 |
| 2.6 | Domains that depend on a value | Open | e.g. "children of `v`" as a domain; candidate: `domain Kids(v: V) = ...` | 3.x | design-v2 C2 |
| 2.7 | Growable domains | Leaning | pools grow; `freeze()` turns them into ordinary domains | 5.6 | design-v2 |
| 2.8 | Conversions between related domains | Leaning | full list for `Sub<V>`, `Rev<V>`, `Pre`, `WithMin<V>` | 2.4 | |
| 2.9 | `v + 1` on positions | Settled | error; `v.next` / `v.offset(k)` (optional) or `v.cyc(k)` | | plan/12-domains-and-sequences.md §1.3 |
| 2.10 | Self-reading tables | Leaning | allowed only for cells provably filled already | | plan/12-domains-and-sequences.md §4 |
| 2.11 | Escape hatch from domain checks | Settled | explicit and greppable, e.g. `a.raw[i]` with an `Int` index; never implicit | 2.1 | |
| 2.12 | `Name<` generics vs less-than | Leaning | capitalized name and a tentative type-argument parse closed by `>`; otherwise a comparison (`INF < x`) | | compiler/03-parsing.md, plan/17-syntax.md |
| 2.13 | Explicit type arguments on lowercase generic fns | Open | default: inferred from the expected type; candidate escape `f::<T>()` | 2.12 | compiler/03-parsing.md |

## 3. Abstraction mechanisms

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 3.1 | Traits vs operation values | Settled | trait for the one canonical behavior, value when there are several | | plan/13-abstraction.md M2/M3 |
| 3.2 | Specialization | Settled | per type and per fn identity; runtime-chosen code is an error | | plan/13-abstraction.md M3 |
| 3.3 | `dyn` escape hatch | Open | keep `dyn (T, T) -> T` for explicit indirect calls, or error only | 3.2 | plan/13-abstraction.md M3 |
| 3.4 | Laws | Settled | declared on operation types, checked by `cp fntest` and debug runs | | plan/13-abstraction.md M4 |
| 3.5 | Representation traits | Settled | `Niche`, `Index`, `Bits`, `Small`, `Relocate`, `Zero`, `OrdKey`, `Bounded` | | plan/13-abstraction.md M7b |
| 3.6 | Implicit representation choices | Settled | only when no worse on any op or memory, and visible (G9) | | plan/00-vision.md G9 |
| 3.7 | Structural field bounds | Leaning | `A has to: V` as a general feature, not graph-specific | | plan/13-abstraction.md G9 |
| 3.8 | Associated types and consts | Open | Rust-style `type Mark`, `const K` in traits | | |
| 3.9 | Coherence (who may `impl` what) | Open | orphan rule relaxed inside one program, strict for libraries | 6.4 | |
| 3.10 | `ModInt` moduli | Open | compile-time `Mod<P>` and runtime `ModR` as two types | 3.2 | plan/13-abstraction.md |
| 3.11 | User-defined decorators | Leaning | a `decorator` declaration that rewrites the item hygienically during name resolution; type-dependent choices stay in generic stdlib code | 3.1 | compiler/04-names-and-modules.md |

## 4. Control and iteration

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 4.1 | Blocks as expressions, `match`, `?` | Settled | indentation blocks, `=>` arms, `?` on optionals | | plan/17-syntax.md |
| 4.2 | Generators | Settled | inlined, can't be stored; generator fns are values; `Iter` param iterated once | | plan/12-domains-and-sequences.md §5 |
| 4.3 | Sequence views | Settled | `runs(a, by:)`, `pairs`, `windows`, `chunks`, `split`, `prefix`, `scan`, `sliding` with explicit state | 4.2 | plan/12-domains-and-sequences.md §6 |
| 4.4 | `zip` of two generators | Leaning | only for non-recursive generators, as stack state machines | 4.2 | plan/12-domains-and-sequences.md §9 |
| 4.5 | `return` / `break` / `?` in `do:` blocks and lambdas | Open | `?` exits the `do:` block; `return` always exits the enclosing fn | | design-v2 Part 10 |
| 4.6 | Labeled `break` | Open | `break 'outer` | | |
| 4.7 | Exhaustiveness for range patterns | Open | `match x: 0 => .., 1..=9 => ..` checked against `x`'s range type | 1.2 | |
| 4.8 | Views as read-only array arguments | Open | allow `[V]T` params to accept views via specialization | 3.2 | plan/12-domains-and-sequences.md §9 |

## 5. State and effects

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 5.1 | Ownership model | Settled | variables own, params borrow, no pointers, `mut` at both ends | | plan/17-syntax.md |
| 5.2 | Copies and moves | Settled | big values need `.copy()`; last-use moves are implicit | | plan/17-syntax.md |
| 5.3 | Precise borrow rules | Leaning | two-phase borrows, field-level borrows, generator-lent values | 5.1 | |
| 5.4 | Inline closures reusing a caller's `mut` borrow | Leaning | if yes, `undo` becomes an ordinary library fn | 5.3 | state-structure-views.md |
| 5.5 | Moves out of arrays | Open | `swap`, `take` (leaves default), no partial moves | 5.3 | |
| 5.6 | Pools and handles | Leaning | `pool Nodes: Node`; typed handles; columns per field | | plan/13-abstraction.md M9 |
| 5.7 | `defer` | Leaning | core; runs on every exit; evaluated at exit; no unwinding on crash | | state-structure-views.md §1 |
| 5.8 | `Rollback`, `Journaled`, `undo`, `scoped` | Leaning | plain arrays not rollback-able; journaling is opt-in by type | 5.4, 5.7 | state-structure-views.md §1 |
| 5.9 | `@memo` control | Open | table layout from key type; evaluation order; clearing; I/O is an error | | plan/15-program-structure.md §3.3 |
| 5.10 | `mut fn` | Settled | a statement; captures `var`s above it; can't escape | 5.3 | plan/15-program-structure.md §3.2 |

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
| 7.1 | Checks in judge vs local builds | Leaning | index checks and `assert` stay; input validation and overflow traps local only | | |
| 7.2 | Overflow in release builds | Leaning | value-range proofs at compile time; unproven arithmetic unchecked in release | 1.1 | plan/30-compiler.md |
| 7.3 | Trace data contract | Open | spans, typed values, writes old → new, call tree, test and part boundaries | 6.5 | plan/50-debugging.md |
| 7.4 | Zero-cost guarantees | Open | list of constructs promised to match hand-written C++, checked by benchmarks | 4.2, 3.2 | |
| 7.5 | C++ interop | Open | `@cpp` blocks with checks at the boundary | 1.1 | |
| 7.6 | Remaining syntax | Leaning | lambdas, inline `if`, numeric literals (decisions 21–33) | | plan/17-syntax.md |
| 7.7 | TLE / MLE / overflow estimates | Settled | from typed input bounds and domain sizes; **warnings only**, never errors | 6.3, 7.2 | |

Rows 2.8, 4.4, 5.3, 5.4, 7.1 and 7.2 have research-backed proposals in research/RECOMMENDATIONS.md.
