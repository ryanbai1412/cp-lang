# Language decisions: what's settled, what's open

Every decision still to make, grouped by the six language features plus
cross-cutting ones. **Settled** = agreed in discussion and written into a
doc. **Leaning** = I have a proposal, you haven't confirmed. **Open** = no
proposal yet. "Where" points to the doc that has (or will have) the detail.

Suggested order: state and effects (5) first, since several open items in
2, 4 and 6 depend on the borrow rules.

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
| 1.6 | Strings | Settled 2026-11-19 | `Str` = `[domain S]Char` (bytes), so strings get domains for free and need no string-only indexing rules | 2.x | §8 |
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
| 2.8 | Conversions between related domains | Settled 2026-11-19 | strict: `Sub<V>` → `V` is the only implicit conversion; every other one is a named method (`v.before`, `pre.vertex`, `V(x)`). Full list for `Rev<V>`, `Pre`, `WithMin<V>` still to write | 2.4 | research/RECOMMENDATIONS.md §3 |
| 2.9 | `v + 1` on positions | Settled | error; `v.next` / `v.offset(k)` (optional) or `v.cyc(k)` | | plan/12-domains-and-sequences.md §1.3 |
| 2.10 | Self-reading tables | Leaning | allowed only for cells provably filled already | | plan/12-domains-and-sequences.md §4 |
| 2.11 | Escape hatch from domain checks | Settled | explicit and greppable: `a.raw[i]` with an `Int` index; never implicit. With `unchecked:` (7.1) and raw intrinsics these are the contest escape hatches of G15 | 2.1 | research/RECOMMENDATIONS.md §3 |
| 2.12 | `Name<` generics vs less-than | Leaning | capitalized name and a tentative type-argument parse closed by `>`; otherwise a comparison (`INF < x`) | | compiler/03-parsing.md, plan/17-syntax.md |
| 2.13 | Explicit type arguments on lowercase generic fns | Settled 2026-11-19 | inferred from the expected type by default; `f::<T>()` when you must say it | 2.12 | compiler/03-parsing.md |

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
| 4.8 | Views as read-only array arguments | Open | allow `[V]T` params to accept views via specialization | 3.2 | plan/12-domains-and-sequences.md §9 |

## 5. State and effects

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 5.1 | Ownership model | Settled | variables own, params borrow, no pointers, `mut` at both ends | | plan/17-syntax.md |
| 5.2 | Copies and moves | Settled | big values need `.copy()`; last-use moves are implicit | | plan/17-syntax.md |
| 5.3 | Precise borrow rules | Settled 2026-11-19 | no stored borrows in structs (so no lifetimes); fields borrow separately, indices don't (`swap`, `a.pair_mut(i, j)`); two-phase borrows; generator-lent values live one iteration | 5.1 | research/RECOMMENDATIONS.md §1 |
| 5.4 | Inline closures reusing a caller's `mut` borrow | Settled 2026-11-19 | yes: a lambda passed to an `inline` parameter is checked as part of the caller's body, so `undo` is an ordinary library fn | 5.3 | research/RECOMMENDATIONS.md §1 |
| 5.5 | Moves out of arrays | Open | `swap`, `take` (leaves default), no partial moves | 5.3 | |
| 5.6 | Pools and handles | Leaning | `pool Nodes: Node`; typed handles; columns per field | | plan/13-abstraction.md M9 |
| 5.7 | `defer` | Leaning | core; runs on every exit; evaluated at exit; no unwinding on crash | | state-structure-views.md §1 |
| 5.8 | `Rollback`, `Journaled`, `undo`, `scoped` | Leaning | plain arrays not rollback-able; journaling is opt-in by type | 5.4, 5.7 | state-structure-views.md §1 |
| 5.9 | `@memo` control | Open | table layout from key type; evaluation order; clearing; I/O is an error | | plan/15-program-structure.md §3.3 |
| 5.10 | `mut fn` | Settled | a statement; captures `var`s above it; can't escape | 5.3 | plan/15-program-structure.md §3.2 |
| 5.11 | Keeping the borrow rules understandable | Settled 2026-11-19 (the method; the rules stay open to change) | fresh agents repeatedly search for the smallest programs where a borrow rule surprises; each one changes the rule or its error message and is kept as a test | 5.3 | plan/agent-context.md |

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
