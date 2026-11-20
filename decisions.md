# Language decisions: what's settled, what's open

Every decision still to make, grouped by the six language features plus
cross-cutting ones. **Settled** = agreed in discussion and written into a
doc. **Leaning** = I have a proposal, you haven't confirmed. **Open** = no
proposal yet. "Where" points to the doc that has (or will have) the detail.

Suggested order: state and effects (5) first, since several open items in
2, 4 and 6 depend on the borrow rules.

**Update, 2026-11-20 (Wave 2 — v2 Spec Lock).** Stage 2 Wave 2 reconciled Wave 1
(`research/language-architecture-synthesis.md`), Stage 3's 18 subtraction cuts
(`research/subtraction-audit-1.md`), Stage 4's 7 syntax bakeoff winners
(`research/syntax-bakeoff-1.md`), Stage 5's debug/pivot and cheat-sheet audits
(`research/blind-critic-2-debug-pivot.md`, `research/blind-critic-3-cheatsheet-test.md`),
Stage 6's max-N judge audit (`research/max-n-judge-audit-1.md`), the bug-fix
sprint (`6f113c3`, `7d1c976`), and scout batches 15 and 17–22 into one
contradiction-free **CP Lang v2 spec** (`research/language-architecture-v2.md`
and `docs/one-page-cheat-sheet.md`). §16 locks the v2 decisions and lists the
rows revised below.

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
| 1.3 | Optionals and niches | Revised 2026-11-20 (v2) | `?T`, niche when the type has a spare value, else a flag; **`?T` does NOT implement `Ord`** (no `a < b`, no implicit `none`-first sort; use `by:`/`??` explicitly; `min`/`max`/`min=`/`max=` skip `none`) | 1.1, 1.5 | §16.9, plan/13-abstraction.md M7b |
| 1.4 | Structs, records, enums | Revised 2026-11-20 (v2) | auto constructors, payload enums, exhaustive `match`; `with` struct update deleted (use `var` copy + field assign); `Ord` derived only when all fields are `Ord` | | §16, research/language-architecture-v2.md |
| 1.5 | Float semantics | Locked v2 (2026-11-20) | `F64` (8-B IEEE double, no `long double`); quiet-NaN niche (`?F64` is 8 B; NaN & Inf trap locally, making `<` total in checked runs); `1e18` stays `Int`, `.` marks `F64`; integer literals unify, `Int` vars need `F64(x)`; `Int(x)` refused on `F64` (use `floor`/`ceil`/`round`/`trunc`); bare `print(x)` of `F64` is an error (use `"{x:.6f}"`, never `-0.00`); `==`/`!=` lint toward `close(a, b, eps)`/`sign(x, eps)` (exempting `== 0.0`); intrinsics `sqrt`/`sin`/`cos`/`tan`/`atan2`/`exp`/`ln`/`hypot`/`PI` | 1.3 | §16.1, research/friction-backlog-batch-21.md |
| 1.6 | Strings | Revised 2026-11-20 (v2) | **ASCII only** (`Char` = 1 byte, zero UTF-8, non-ASCII literal is an error). Default `Str` is a flat `0..len`-indexed ASCII byte sequence (`s[i]`, `s[l..r]`, `s.push`, `s + t`, `s * k`, `s.starts_with(x, at: i)`, `s.split_once`, `s.parse_int`). Opt-in `[domain S]Char = read()` stays for multi-string nominal-domain DP | 2.14 | §16, research/syntax-bakeoff-1.md Fork 4 |
| 1.7 | Numeric types (`Int`, `U64`, `I128`, `F64`) | Revised 2026-11-20 (v2) | **4 numeric types only**: `Int` (symmetric 64-bit; range types pack to 1/2/4 B), `U64` (wrapping 64-bit), `I128` (explicit 128-bit), `F64`. `I32`, `I64`, `U8`, `U32` deleted | 1.1, 1.2 | §16, research/subtraction-audit-1.md Cut 8 |
| 1.8 | Bit operations & power | Locked v2 (2026-11-20) | `&`, `\|`, `^` (XOR), `~`, `<<`, `>>`, plus `&=`, `\|=`, `^=`, `<<=`, `>>=`; `**` is power (never `^`); `popcount`, `ctz`, `clz`, `bit_len`; shifts reaching bit 63 on `Int` suggest `U64` | 1.7 | §16, docs/one-page-cheat-sheet.md |
| 1.9 | Integer division and modulo | Settled 2026-11-20 | **Euclidean everywhere**: `a / b` and `a % b` satisfy `a == (a / b) * b + a % b` with `0 <= a % b < abs(b)`, so `a % b` is never negative, for negative `a` and for negative `b`. (`//` is not an operator: it starts a comment.) When the range checker proves `a >= 0` and `b > 0`, both lower to the native C++ `/` and `%` with no extra code; otherwise to the two-instruction fix-up (`cp_ediv` / `cp_emod`). `Int` is symmetric (1.1), so `Int.MIN / -1 == Int.MAX` and only `b == 0` stops the program | 1.1 | plan/11-values-and-types.md |
| 1.10 | `??=` null-coalescing assignment | Locked v2 (2026-11-20) | `place ??= expr` evaluates `expr` lazily only when `place` is `none`, stores it, and lends the value (`(m[k] ??= []).push(x)`). No silent auto-vivification on method calls | 1.3 | §16, research/language-architecture-v2.md |

## 2. Index spaces

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 2.1 | Domains, `[V]T`, unchecked indexing by `V` | Settled, revised 2026-11-20 | `domain V = 1..=n` is sugar for a sealed nominal struct over a range type with `impl Index` and `impl Seq`; its width follows the range (4 bytes at most for input-sized domains). Checks only where positions are created; the no-check guarantee is now derived from `to_idx: 0..K`, not a rule about domains | | plan/19-index-and-lends.md §3.1 |
| 2.2 | Bounds (gaps between positions) | Revised 2026-11-20 (v2) | `V.bounds`; **`v.l` / `v.r`** and **`v.l(k)` / `v.r(k)`** (`v.l == v.l(1)`), plus `b.left -> ?V` / `b.right -> ?V`. `v.before` / `v.after` deleted (one spelling). Prefix sums index by bounds or open ranges: `pre.sum(l..=r)` | | §16, research/subtraction-audit-1.md Cut 10 |
| 2.3 | `Range<I>` | Revised 2026-11-20 (v2) | **One `Range<I>` type everywhere** (`Range1<I>` deleted). Stored as two bounds (`l..r`, `l..=r`, `..i`, `..=i`, `i..`, `i.r..`, `l.r..r.l`); `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len`, `.is_empty`, `.rev`, `.step(k)` (`by` keyword deleted). All range DS accept open/empty ranges and return identity | | §16, research/subtraction-audit-1.md Cut 7 |
| 2.4 | Slices | Settled, revised 2026-11-20 | `a[s]` is a library projection that lends a slice. A slice is indexed by the positions of its array (`V`), with a window check unless proven; **`Sub<V>` is removed**. The window is a value, the slice is a lend: fns return ranges, callers apply them | 5.12 | plan/19-index-and-lends.md §4.2 |
| 2.5 | Product and fresh domains | Settled, revised 2026-11-20 | `(A, B)` is an `Index` everywhere (`for p in Cell`, `[Cell]T`, `Dsu<Cell>`, `bfs`, `@memo`), and `a[i, j]` is sugar for `a[(i, j)]`; `domain D` in a signature or binding introduces a fresh index type that can't leave its block (`compress`, `scc`, `freeze`) | 2.16 | plan/19-index-and-lends.md §3.2, §4.3 |
| 2.6 | Domains that depend on a value | Open | e.g. "children of `v`" as a domain; deferred — `tree.children(v)` covers the common case without dependent domain syntax | 3.x | design-v2 C2 |
| 2.7 | Growable arenas & `freeze()` | Revised 2026-11-20 (v2) | **`Pool<T>`, `Pool<Node<_>>`, `PoolMap`, and `Handle` deleted.** Use `type Id = 0..=MAX` (4-byte handle + 4-byte niche `?Id`) with `List<Node>`, `.push_id(x)`, `??=`, and `xs.freeze() -> (domain D, [D]T)` | 1.2, 2.5 | §16, research/subtraction-audit-1.md Cut 6 |
| 2.8 | Conversions between related domains | Settled, revised 2026-11-20 | strict, **no implicit conversion at all**: every conversion is a named method (`v.l`, `pre.vertex`, `V(x)`) | 2.4 | research/RECOMMENDATIONS.md §3, plan/19-index-and-lends.md §4.2 |
| 2.9 | Stepping positions (`v + k`, `b - a`, `v.cyc`) | Locked v2 (2026-11-20) | `v + k` and `v - k` always return `?V`; `b - a -> Int` is same-domain distance; `v.cyc(k) -> V` wraps. `v.offset(k)`, `v.next`, `v.prev` deleted (one spelling). Uniform optionality (flow facts elide runtime check on `!`; diagnostics print column and parenthesized fix) | | §16, research/subtraction-audit-1.md Cut 10 |
| 2.10 | Self-reading tables | Leaning | allowed only for cells provably filled already | | plan/12-domains-and-sequences.md §4 |
| 2.11 | Escape hatch from domain checks | Settled | explicit and greppable: `a.raw[i]` with an `Int` index; never implicit. `a.raw` is a library projection lending the same storage indexed by `0..K`. With `unchecked:` (7.1) and raw intrinsics these are the contest escape hatches of G15 | 2.1 | research/RECOMMENDATIONS.md §3 |
| 2.12 | `Name<` generics vs less-than | Leaning | capitalized name and a tentative type-argument parse closed by `>`; otherwise a comparison (`INF < x`) | | compiler/03-parsing.md, plan/17-syntax.md |
| 2.13 | Generic type arguments & inference | Locked v2 (2026-11-20) | Bidirectional unification of non-closure args (`from:`, `root:`, `g`) before closures removes ~90% of explicit type args; `f::<T>()` is the unambiguous fallback (`f<T>()` stays an error). Generic fn definitions always write `<D>` explicitly (implicit single-letter generics deleted) | 2.12 | §16, research/syntax-bakeoff-1.md Fork 2 |
| 2.14 | `[domain]T` packages | Revised 2026-11-20 (v2) | an array stored with a fresh index type of its own; binding one opens it as `[x.Idx]T`. Serves opt-in `[domain S]Char` and jagged rows (`List<[domain]Int>`); default `Str` is `0..len`-indexed ASCII bytes (1.6) | 1.6, 2.5 | §16, plan/19-index-and-lends.md §4.4 |
| 2.15 | Indexing is a trait | Settled 2026-11-20 | `a[i]` is `Indexed<I>.at`, a projection; `[I]T` is sugar for the library struct `Array<I, T>`, and grids and permutations are library code over it | 3.5, 5.12 | plan/19-index-and-lends.md §3.3, §7 |
| 2.16 | Algebraic `Index` derivation | Locked v2 (2026-11-20) | 5 strictly O(1), `Copy` finite shapes derive `Index` automatically: `Bool`, unit/payload `enum` (computed on demand, solving typed `FlowGraph<Node>`), `?T` (`T: Index`), bounded ranges `lo..hi`/`lo..=hi` (structural equality for identical bounds in scope), and tuples/records `(A, B)`, plus `Subset<D>`. `[D]E: Index` deleted | 2.1, 3.5 | §16, research/subtraction-audit-1.md Cut 5 |

## 3. Abstraction mechanisms

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 3.1 | Traits vs operation values | Revised 2026-11-20 (v2) | Bare operators (`+`, `*`, `^`, `&`, `\|`, `min`, `max`, `gcd`), `(id, op)` pairs, `desc(x)`, and `Action` (for `LazySegTree`); contestant algebraic wrapper zoo (`Sum`/`Max`/`Order`/`Semiring`/etc.) deleted | | §16, research/subtraction-audit-1.md Cut 15 |
| 3.2 | Specialization | Settled | per type and per fn identity; runtime-chosen code is an error | | plan/13-abstraction.md M3 |
| 3.3 | `dyn` escape hatch | Deleted 2026-11-20 (v2) | Removed from the language (`0` uses across 299 ports); static specialization only | 3.2 | §16, research/subtraction-audit-1.md Cut 9 |
| 3.4 | Laws | Settled | declared on operation types, checked by `cpl test` and debug runs | | plan/13-abstraction.md M4 |
| 3.5 | Core & library traits (8 + 3) | Revised 2026-11-20 (v2) | **8 core traits** (`Index`, `Seq`, `Copy`, `Eq`, `Ord`, `Hash`, `Show`, `Read`, all auto-derived when fields support them) + **3 library-author traits** (`Indexed`/`IndexedMut`, `Iterator`, `Gen`). Deleted: `Small`, `Relocate`, `Zero`, `Bits`, `OrdKey`, `Bounded`, `Niche` (internalized), `Shrink`, `Modulus`, `Handle`, `FenwickIndex`, `Window`, `BagWindow`. (`std.algebra.Reversible` is a library-only hook for 1×-memory `Rooted.fold_path`.) | | §16, research/subtraction-audit-1.md Cut 13 |
| 3.6 | Implicit representation choices | Settled | only when no worse on any op or memory, and visible (G9) | | plan/00-vision.md G9 |
| 3.7 | Structural field bounds | Leaning | `A has to: V` as a general feature, not graph-specific | | plan/13-abstraction.md G9 |
| 3.8 | Associated types, consts and frozen lets | Settled 2026-11-20 | yes. Associated types (`Indexed.Elem`, `Seq.bounds`). An associated size is `const K = 10` when known at compile time and **`let K = n`** when frozen from a run-time value where the impl is declared; **`const K = n` with a run-time `n` is an error**, so `const` stays strictly compile-time (7.10). Sizing by a `var` is an error | 7.10 | plan/19-index-and-lends.md §3.1 |
| 3.9 | Coherence (who may `impl` what) | Settled 2026-11-19 | orphan rule relaxed inside one program, strict for libraries | 6.4 | §8 |
| 3.10 | `Mod<P>` (unified moduli) | Revised 2026-11-20 (v2) | **One `Mod<P>` type** accepting both `const` and runtime-frozen `let` `Int` moduli via 3.8 (`ModR` retired). Stores 4-byte `uint32_t` when `P <= 2^31 - 1`, uses `mul_mod` past `3.03e9`; one-way `Int` reduction next to `Mod<P>` | 3.8 | §16, research/language-architecture-v2.md |
| 3.11 | Decorators (`@`) | Revised 2026-11-20 (v2) | User-defined `decorator` macros deleted. **7 contestant decorators** (`@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`/`@interactor`, `@grader`) + **2 library-author** (`@get`, `@test`) | | §16, research/subtraction-audit-1.md Cut 11 |

## 4. Control and iteration

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 4.1 | Blocks as expressions, `match`, `?` | Settled | indentation blocks, `=>` arms, `?` on optionals | | plan/17-syntax.md |
| 4.2 | Generators | Settled, revised 2026-11-19 | inlined; the compiler picks push or pull per use: push for a plain `for`, pull for `zip` or when stored as `Iterator<T>` (so a non-recursive generator can be stored); a recursive generator where pull is needed is an error suggesting `List(g)`; no hidden explicit stacks | | research/RECOMMENDATIONS.md §2, plan/12-domains-and-sequences.md §5 |
| 4.3 | Sequence views, windows & search | Revised 2026-11-20 (v2) | Imperative `Deque<T>` + `MonoDeque<T>` + explicit `while` for stateful two-pointers + stateless `slide_until(r, ok)` yielding `(lo, hi)` pairs (`Window`/`BagWindow`/callback `sliding` deleted). 8 search names: free-fn `bsearch_first`/`bsearch_last`/`bsearch_first_some`, sorted `.first_ge`/`.last_le`, linear `first`/`find`/`min_by`/`max_by` (`first_where` retired). `xs.adjacent` (`pairs` deleted), `a.items` (`enumerate` deleted) | 2.3, 4.2 | §16, research/syntax-bakeoff-1.md Forks 1 & 5 |
| 4.4 | `zip` of two generators | Settled 2026-11-19 | push the outer, pull the inner; only non-recursive generators can be pulled | 4.2 | research/RECOMMENDATIONS.md §2 |
| 4.5 | `do:` blocks & brace blocks | Deleted 2026-11-20 (v2) | `do:`, `{ ... ; ... }`, `;`, `<=>`, and `.field` shorthand deleted; indented block works directly after `=>` | | §16, research/subtraction-audit-1.md Cut 9 |
| 4.6 | Labeled `break` | Settled 2026-11-19 | `break 'outer` / `continue 'outer` | | §8 |
| 4.7 | Exhaustiveness for range patterns | Open | `match x: 0 => .., 1..=9 => ..` checked against `x`'s range type | 1.2 | |
| 4.8 | Views as read-only array arguments | Settled 2026-11-20 | a lent row or slice is a `[B]T` to the callee, with no adapter; a fn that wants any indexable thing opts in with `A: Indexed<V, Elem = T>`; concrete `[V]T` parameters stay concrete | 3.2 | plan/19-index-and-lends.md §3.3, §3.5 |
| 4.9 | Control-flow & expression uniformity | Locked v2 (2026-11-20) | One inline-`if` (`if c: a else: b`), `elif` only, `match` bare `Pattern => expr`, `if let`/`match` expressions, `for ... if`, `while let ... and`, value-copying destructuring `let (a, b) = xs[i]`, `var (a, b) = ...`, parallel `(a, b) = (b, a)`, uniform `?T` optionality, line continuation after binary ops/`=`/open brackets | | §16, research/language-architecture-v2.md |

## 5. State and effects

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 5.1 | Ownership model | Settled | variables own, params borrow, no pointers, `mut` at both ends | | plan/17-syntax.md |
| 5.2 | Copies and moves | Settled | big values need `.copy()`; last-use moves are implicit | | plan/17-syntax.md |
| 5.3 | Precise borrow rules | Settled 2026-11-19, revised 2026-11-20 | lexical exclusivity: no stored and no returned borrows (so no lifetimes); fields borrow separately; two lends of one projection are separate when its `disjoint` law is proven from flow facts (5.13), which replaces "indices don't borrow separately"; index arguments are evaluated before any lend begins (two-phase); generator-lent values live one iteration | 5.1 | plan/19-index-and-lends.md §3.4 to §3.6, §5 |
| 5.4 | Inline closures reusing a caller's `mut` borrow | Settled 2026-11-19 | yes: a lambda passed to an `inline` parameter is checked as part of the caller's body, so `undo` is an ordinary library fn. Projections (5.12) are its second client | 5.3 | research/RECOMMENDATIONS.md §1 |
| 5.5 | Moves out of arrays | Settled 2026-11-20 | `swap`, `take` (leaves default) and `pair_mut` are library fns over `at`, allowed by its `disjoint` law; no partial moves | 5.13 | plan/19-index-and-lends.md §3.6 |
| 5.6 | Pools and handles | Deleted 2026-11-20 (v2) | Replaced by `type Id = 0..=MAX` + `List<Node>` + `.push_id(x)` + `.freeze()` (see 2.7) | 2.7 | §16, research/subtraction-audit-1.md Cut 6 |
| 5.7 | `defer` | Locked v2 (2026-11-20) | **The one scope-cleanup primitive**: runs on every block exit; no unwinding on crash | | §16, research/subtraction-audit-1.md Cut 1 |
| 5.8 | `Rollback`, `undo`, `with` | Revised 2026-11-20 (v2) | `undo` blocks and `with` scope guards deleted; rollback is a plain 2-method struct API paired with `defer` (`let snap = dsu.mark(); defer dsu.undo_to(snap)`) | 5.7 | §16, research/subtraction-audit-1.md Cut 1 |
| 5.9 | `@memo` control | Locked v2 (2026-11-20) | Flat `[Params]?R` (niche-packed) for any algebraic `Index` tuple up to `2^26` slots, else salted `HashMap`; `@memo(flat)` / `@memo(hash)` override; `ask` allowed; nested `@memo` scoped to enclosing call; `read()`/`print` remain errors | 2.16 | §16, research/max-n-judge-audit-1.md §1 |
| 5.10 | Nested `fn` mutation (`mut fn` deleted) | Revised 2026-11-20 (v2) | **`mut fn` deleted.** Any nested `fn` declared after a `var` reads/writes it directly; the compiler tracks captured borrows automatically and forbids escaping | 5.3 | §16, research/subtraction-audit-1.md Cut 3 |
| 5.11 | Keeping the borrow rules understandable | Settled 2026-11-19 (the method; the rules stay open to change) | fresh agents repeatedly search for the smallest programs where a borrow rule surprises; each one changes the rule or its error message and is kept as a test | 5.3 | plan/agent-context.md |
| 5.12 | Projections: `lends` / `lend` | Locked v2 (2026-11-20) | `fn row(self, r: A) lends mut [B]T` with every path reaching **one** `lend place` (no `with`/`lends ()` exception). Post-`lend` write-back lets `SegTree`/`LazySegTree` repair invariants after mutations | 5.3, 5.4 | §16, plan/19-index-and-lends.md §3.4 |
| 5.13 | `law disjoint` | Settled 2026-11-20 | a projection may state when two of its lends don't overlap (`law disjoint(i, j) = i != j`). Two live lends of one projection on one receiver, one of them `mut`, need the law proven from flow facts (`if`, `while`, `assert`, `requires`, value ranges); otherwise a compile error that prints the law. Never a silent check; a wrong law is caught by debug runs and `cpl fntest` | 3.4, 5.12 | plan/19-index-and-lends.md §3.6 |
| 5.14 | Lent values are second-class | Settled 2026-11-20 | a value built from lent parts is lent and lives only in the block that made it: it may be passed, bound by `let`, iterated and lent onward; not put in a `var`, a field or a container, captured by a non-`inline` lambda, or returned. Borrowed-ness is a mode of the binding, not a type, so there is no `view` keyword | 5.3 | plan/19-index-and-lends.md §3.5 |

## 6. Program structure

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 6.1 | File as function body | Settled | top-level fns see the body's `let`s; calling before the `let`s are set is an error | | plan/15-program-structure.md §2 |
| 6.2 | Program vs library mode | Settled | decided by contents; `use` accepts only libraries | | plan/15-program-structure.md §2.4 |
| 6.3 | Multitest | Revised 2026-11-20 (v2) | `@multitest`, `@multitest(until_eof)`, `@multitest(count: t)`; `return` ends a test; **`static let` deleted** (use an explicit `for _ in 0..read():` loop when runtime precomputation precedes test cases) | 6.1 | §16, research/subtraction-audit-1.md Cut 9 |
| 6.4 | Modules beyond `pub` | Open | re-exports, private-to-folder, name clashes | | plan/40-tooling.md |
| 6.5 | Parts and purity | Settled | any fn is a part; tools record I/O; no `pure` keyword | | plan/15-program-structure.md §4 |
| 6.6 | Brutes, checkers, interactors | Revised 2026-11-20 (v2) | `@brute`, `@checker` (inherits input bindings, uses `read_output()` + `assert`), `@interactor` (local stdio judge paired with `ask(..., limit: K)`). Standalone `protocol` DSL and `@solution` deleted | 3.11 | §16, research/subtraction-audit-1.md Cut 11 |
| 6.7 | Staying under 64 KB | Locked v2 (2026-11-20) | Tree-shake all runtime/stdlib helpers and strip `CP_GEN`/`assume`/`debug` in judge builds (target 4–10 KB typical, always < 64 KB) | 3.2 | §16, research/max-n-judge-audit-1.md |
| 6.8 | Grader-style (IOI) problems | Locked v2 (2026-11-20) | `@grader` unifies all IOI ABIs without new keywords: bodyless `@grader fn` = extern judge fn, `@grader fn` with body = exported entry, `@grader struct` = stateful multi-callback grader | 3.11, 6.1 | §16, research/language-architecture-v2.md |
| 6.9 | Top-level `var` | Locked v2 (2026-11-20) | allowed; any nested `fn` declared after a `var` reads/writes it directly (5.10) | 5.10 | §16 |

## 7. Cross-cutting

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 7.1 | Checks & predicates (`assert` vs `assume`) | Revised 2026-11-20 (v2) | **Two predicates only:** `assert` is ALWAYS checked (local + judge; lints on `mut`; feeds proven flow facts); `assume` is local/debug/stress only (stripped on judge; statically rejects `mut`/I/O side effects). Domain indexing never checked; `List`/`Str`/`a.raw` checked unless proven or in `unchecked:` | 7.9 | §16, research/subtraction-audit-1.md Cut 2 |
| 7.2 | Overflow in release builds | Settled 2026-11-19 | proven safe: no check; provably reachable: a warning with the bound; unknown: trapped in local builds, unchecked in the judge build | 1.1 | research/RECOMMENDATIONS.md §4, plan/30-compiler.md |
| 7.3 | Trace data contract | Open | spans, typed values, writes old → new, call tree, test and part boundaries | 6.5 | plan/50-debugging.md |
| 7.4 | Zero-cost & max-N judge guarantees | Locked v2 (2026-11-20) | No `using namespace std;`, 8-B `?Int`/`?F64`, 4-B `Mod<P>` (`P <= 2^31-1`), `vector<uint8_t>` never `vector<bool>`, flat `@memo` over algebraic `Index`, salted flat hash tables, lazy `Rooted` LCA + CSR `kids`, lazy-sorted `Steps` | 1.3, 5.9 | §16, research/max-n-judge-audit-1.md |
| 7.5 | C++ interop | Open | `@cpp` blocks with checks at the boundary | 1.1 | |
| 7.6 | Remaining syntax | Locked v2 (2026-11-20) | 40 keywords (32 contestant + 8 lib-author); single lambda `x => expr`; single inline-`if` `if c: a else: b`; `"{val:.6f}"` format specifiers | | §16, docs/one-page-cheat-sheet.md |
| 7.7 | TLE / MLE / overflow estimates | Revised 2026-11-20 (v2) | from typed input bounds and domain sizes; **warnings only**, never errors; suppressed inside `@brute`, `@gen`, `@checker`, `@interactor`, and `assume` | 6.3, 7.2 | §16, research/blind-critic-2-debug-pivot.md |
| 7.8 | Typed input drives generators and shrinkers | Revised 2026-11-20 (v2) | Dependent bounds (incl. record fields) + `Perm<V>` / `Tree<V>` + `read(gen:)` (with `witness` & coupled tuple reads) + `@gen` + automatic choice-stream shrinking. **`stress ... by ...` DSL and `Distinct`/`Sorted` wrappers deleted** | 6.6 | §16, research/syntax-bakeoff-1.md Fork 6 |
| 7.9 | Contracts | Revised 2026-11-20 (v2) | Collapsed into `assert` (always checked) and `assume` (local-only); `requires`, `ensures`, `invariant`, and `read() where` deleted | 7.1 | §16, research/subtraction-audit-1.md Cut 2 |
| 7.10 | Compile-time `const` | Settled 2026-11-19 | adopted, with a size warning for large baked tables | 6.7 | plan/18-key-features.md §3 |
| 7.11 | REPL on a loaded test | Deferred 2026-11-19 | not now; cheap to add once the debug interpreter exists | | plan/18-key-features.md §4 |
| 7.12 | Command name and file extension | Settled 2026-11-19 (name is a placeholder) | the binary is `cpl` until Ryan picks a final name, because `cp` clashes with Unix `cp`; source files are `.cp` | | plan/b0-distribution.md |
| 7.13 | Editor support in the core | Settled 2026-11-19 | LSP with partial-parse recovery, VS Code extension and syntax highlighting are P0 (G13) | | plan/60-editor.md |
| 7.14 | Trace viewer design | Settled 2026-11-19 | designed fresh with its own design system; the old `mock/trace-mock.html` is rejected and not in the repository. What the trace must contain (7.3) is unaffected | 7.3 | plan/50-debugging.md |
| 7.15 | Sample local library | Settled 2026-11-19 | a fresh one, inspired by popular contest libraries, not a port of `ryanbai1412/cp-library`; it also proves G8 and tests library pinning (G11) | 6.4 | plan/40-tooling.md |
| 7.16 | `test` and `doc` for libraries | Settled 2026-11-19 | `cpl test` and `cpl doc` work the same on the stdlib and on a user's own library | 6.4 | plan/45-dev-tools.md |
| 7.17 | How conciseness is measured | Settled 2026-11-19 | two numbers per port: % fewer lines than the original, and % fewer than the original with library-able code stripped | | plan/80-validation.md §8.1 |
| 7.18 | Honest `cpl stress` & `debug(...)` | Locked v2 (2026-11-20) | One `cpl stress` command defaulting to in-file `@brute`/`@checker`/`@interactor`, escalating sizes (default up to 20), comparing line-by-line unless `@checker` (plus `--close <eps>` for floats), always compiling with `-DLOCAL`, shrinking every failure via choice streams, replaying `debug(...)` side-by-side, and printing max size, rejection rate (>50% warning), diff mode, and output diversity | 6.6, 7.8 | §16, research/blind-critic-2-debug-pivot.md |
| 7.19 | Grids, trees & collection verbs | Locked v2 (2026-11-20) | `type Cell = (Row, Col)` + `std.grid` (`p.adj4`, `p.adj8`, `Dir4`/`Dir8`, `p.step`, `p.ray`, `g.transposed`); `Rooted<V>` with `.fold_path`/`.apply_path` + `Reversible` (retiring `Hld`/`LazyHld`), `.ancestor`, `.postorder`, `.lift_while`, `.virtual_tree`; unified container verbs (`.len`/`.is_empty` properties, `.add`/`.remove`/`in`/`\| & - ^`) | 2.16, 3.1 | §16, research/syntax-bakeoff-1.md Forks 3 & 7 |

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
  The ports use `SegTree`, `Mod`, `Graph`, `dijkstra` and some twenty
  more names with no `use`, because `proto/cpc` builds them in. The checker lists them in
  `crates/cpl-check/src/prelude.rs` and does not type what goes through
  them. This is a stopgap: the fix is for each port to `use` the `std`
  module, at which point the name leaves the list.

## 15. Calls made fixing the prototype bugs of scout batches 5 to 12 (2026-11-20)

Bug fixes only: no syntax was added. Each rule below is the reading the
scouts expected and is locked by a test under `proto/tests/`.

- **15.1 `print` evaluates all its arguments before it writes.**
  Arguments are evaluated left to right, and nothing is written
  until the last one has its value. A later argument that asks the judge
  or prints is then on its own line, before this one:

  ```
  print("!", best(r, c))     // best may `ask`; its question is not written after the "!"
  ```
- **15.2 A range is a value: its ends are read once.**
  `for i in 0..xs.len` goes over the positions `xs` had when the loop
  started, whatever the body pushes or pops. `proto/cpi` already did
  this; `proto/cpc` re-read the end at every step.
- **15.3 `@memo` over a large product stores only what is asked.**
  A dense table has one slot per point of the parameters' domains. Past
  2^26 slots at the domains' declared maximum sizes the function is
  hashed, as `@memo(hash)` is; `into:` and `order:`, which need the
  table, are then a compile error that gives the size.
- **15.4 Destructuring owns what it binds.**
  `let (a, b) = xs[i]`, the value an `if let` unwraps and the subject of
  a `match` are copies. They were C++ references into the list, so a
  `xs.push(...)` in the same block read freed memory (scout batch 12).
  Reading through a view of a container stays the job of `lends`.

  ```
  let (a, b) = xs[i]
  xs.push((a + 1, b))        // fine: a and b are values
  ```
- **15.5 A record of hashable fields is hashable.**
  Numbers, positions, `Bool`, `Char` and records of them can key a
  `HashMap` or `HashSet`; any other key is a compile error.
- **15.6 A `gen:` function's value is written as `read()` reads it.**
  One format per type, shared by the default generator and `gen:`: a
  list after its length, a table row by row.
- **15.7 A generic function is only called.**
  `f<P>(x)` parses as two comparisons, so naming a generic function
  outside a call is an error that says so. How a type argument is written
  out, `f::<P>(x)` (2.13), is not in the prototype yet.
- **15.8 `MinHeap<T>()` takes no arguments.**
  `by:` was accepted and ignored; it is an error until a heap with an
  order is designed.
- **15.9 An input that is not what the type says stops the run.**
  A word where a number is expected stops a local run; it was read as 0.
  An enum with no fields and no `@tags` has no written form, so reading
  one is a compile error that gives the `@tags(...)` line to add; it was
  read as a number, which made every cell the
  first variant.
- **15.10 A declared type takes its name.**
  `struct SegTree:` or `struct Dsu:` in a program is that program's type
  from the declaration on; the built-in of the same name is not consulted
  (it crashed the compiler). A built-in type written without its type
  arguments, `List` for `List<Int>`, is a compile error that gives the form.
- **15.11 `?T` is ordered where `T` is, none first (superseded by 16.9).**
  This was a C++-codegen stopgap in `6f113c3` so records with an optional
  field compiled under `g++`, while `a < b` on two optionals stayed an error.
  §16.9 replaces this with one coherent rule: `?T` does not implement `Ord`,
  and structs derive `Ord` only when every field implements `Ord`.
- **15.12 `var (a, b) = ...` declares vars; a tuple is not assigned to (resolved by 16.9).**
  The names of a `var` pattern are vars, as the one name of `var a = ...`
  is; they were constants. `(a, b) = (b, a + b)` built and assigned to a
  temporary, so `7d1c976` turned it into a compile error pending the language
  pass; §16.9 locks parallel tuple assignment `(a, b) = (x, y)` into v2.

## 16. CP Lang v2 Spec Lock (2026-11-20 — Language Architect Wave 2)

Wave 2 synthesizes Wave 1 (`research/language-architecture-synthesis.md`),
Stage 3's 18 subtraction cuts (`research/subtraction-audit-1.md`), Stage 4's
7 syntax bakeoff verdicts on the 25-problem benchmark suite
(`research/syntax-bakeoff-1.md`), Stage 5's debug/pivot and cheat-sheet audits
(`research/blind-critic-2-debug-pivot.md`, `research/blind-critic-3-cheatsheet-test.md`),
Stage 6's max-N judge audit (`research/max-n-judge-audit-1.md`), the bug-fix
sprint (`6f113c3`, `7d1c976`), and scout batches 15 and 17–22 into one
contradiction-free **CP Lang v2 spec**.
Full before/after snippets and the prototype implementation checklist live in
`research/language-architecture-v2.md`; `docs/one-page-cheat-sheet.md` is the
canonical contestant reference. Every item below replaces earlier leans where
they conflicted and is locked as the single source of truth for the prototype
and Rust compiler sprints.

- **16.1 Concept budget & `F64` semantics locked (Stage 3 Cuts 1–13, Batch 21 E1–E5).**
  - **40 keywords total**: 32 contestant keywords including the 3 literals
    (`let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`,
    `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`,
    `break`, `continue`, `defer`, `and`, `or`, `not`, `assert`, `assume`,
    `mut`, `self`, `unchecked`, `true`, `false`, `none`) plus 8 library-author
    keywords (`trait`, `impl`, `where`, `lends`, `lend`, `law`, `inline`,
    `yield`). Fifteen zombie/speculative keywords are permanently deleted
    (`by`, `with`, `undo`, `requires`, `ensures`, `invariant`, `stress`,
    `protocol`, `limit`, `dyn`, `do`, `pool`, `decorator`, `static`, `yields`),
    and Batch 19's proposed `require`/`hidden`/`export`/`extern`/`interactor`/
    `checker`/`gen` stay ordinary functions or `@` decorators rather than hard
    keywords.
  - **4 numeric types**: `Int` (symmetric 64-bit, with range types packing to
    1/2/4 bytes automatically), `U64` (wrapping 64-bit), `I128`, and `F64`
    (8-byte IEEE double, no `long double`). `I32`, `I64`, `U8`, `U32` are
    deleted (1.7). On `F64` (1.5, Batch 21 E1–E5): `1e18` stays `Int` and only
    literals with `.` are `F64`; integer literals unify with `F64` in typed
    positions while `Int` variables require explicit `F64(x)`; `Int(x)` on an
    `F64` is a compile error requiring `floor(x)`, `ceil(x)`, `round(x)`, or
    `trunc(x)`; NaN and infinity trap at creation in checked runs so quiet NaN
    is the 8-byte `?F64` niche and `<` is a total order; bare `print(x)` of an
    `F64` is a compile error suggesting `"{x:.6f}"` (round half away from zero,
    never `-0.00`, never scientific notation); `==`/`!=` lint toward explicit
    `close(a, b, eps)` and `sign(x, eps)` (exempting literal `== 0.0`); and
    `sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `floor`, `ceil`,
    `round`, `trunc`, `PI` are listed intrinsics.
  - **8 + 3 language traits**: 8 auto-derived core traits (`Index`, `Seq`,
    `Copy`, `Eq`, `Ord` only when all fields are `Ord`, `Hash`, `Show`, `Read`)
    plus 3 library-author traits (`Indexed`/`IndexedMut`, `Iterator`, `Gen`).
    `Small`, `Relocate`, `Zero`, `Bits`, `OrdKey`, `Bounded`, `Niche`
    (internalized), `Shrink` (choice-stream shrinking), `Modulus`, `Handle`,
    `FenwickIndex`, `Window`, and `BagWindow` are deleted (3.5).
  - **7 + 2 `@` decorators**: `@memo`, `@multitest`, `@brute`, `@tags`, `@gen`,
    `@checker`/`@interactor`, `@grader`, plus library-author `@get`, `@test`
    (3.11).
  - **One spelling per construct**: `if cond: a else: b` (no `then`, no postfix
    `if`), `elif` (no `else if`), bare `Pattern => expr` `match` arms (no
    `case`), `x => expr` lambdas (no `.field`), `^` XOR / `**` power, `v.l`/
    `v.r` boundaries (no `v.before`/`v.after`), `v + k`/`v - k`/`v.cyc(k)`
    stepping (no `v.offset`/`v.next`/`v.prev`), `.step(k)` (no `by`),
    `a.items` (no `enumerate`), `xs.adjacent` (no `pairs`), `Bitset<D>`,
    explicit `<D>` on generic `fn` definitions, no `;` or `{ ... }` or `<=>`
    or `do:` or `static let`.

- **16.2 `Str` revised to `0..len`-indexed ASCII bytes + opt-in `[domain S]Char` (1.6, Stage 4 Fork 4, Batch 19 B19-9).**
  Stage 4 proved that auto-opening nominal `s.Idx` as default `Str` fights real
  string algorithms (`4331904`, `noip20p2`, `noi16p1`, `sac22cc3p5`) because
  Z/KMP/SA/hashing constantly add lengths (`i + x.len`, `2 * w`) and land on
  boundary `s.len`. Default `Str` is now a flat `0..len`-indexed ASCII byte
  sequence (`Char` = 1 byte, zero UTF-8) supporting `s[i]`, `s[l..r]`,
  `s.push(ch)` (when `var`), `s + t`, `s * k`, `sep.join`, `s.starts_with(x, at: i)`,
  `s.split`, `s.split_once`, `s.trim_start`, `s.parse_int()`, and `Str(chars)`.
  Nominal `[domain S]Char = read()` remains a 1-line explicit opt-in for
  multi-string DP (LCS, edit distance) using the unchanged `[domain]T` kernel
  mechanism (2.14). Open slices `s[..k]`/`s[k..]` work on `Str` (Batch 22 X2),
  and `var t = s` strictly obeys value semantics (last-use move or explicit
  `s.copy()`, never silent mutable aliasing of `let s`, Batch 22 X4). `Char`
  adds `is_alpha`, `is_alnum`, `digit(base = 10) -> ?Int`, `Char.digit(d)`;
  `Scanner` is dropped (Batch 22 X10, X13).

- **16.3 Sliding windows & one `Range<I>` type (2.3, 4.3, Stage 3 Cuts 7 & 14, Stage 4 Fork 1, Batch 21 E7).**
  `Window<I>`, `BagWindow<I>`, `TwoStackFold`, `GroupWindow`, and callback
  `sliding(D, add:, drop:, ok:)` are all deleted: Fork 1 caught a real bug in
  callback `MonoDeque` (`noi05p1`) and showed imperative `MonoDeque` uses fewer
  keystrokes and 11 fewer symbols. CP Lang v2 ships imperative `Deque<T>`,
  imperative `MonoDeque<T>(min | max)` (`push(at, val)`, `drop_before(min_at)`,
  `clear()`, `best -> ?T`), fixed `D.windows(k)` + `sliding_min`/`sliding_max`,
  explicit `while` loops for stateful two-pointers, and stateless
  `slide_until(r, ok)`. To keep **one `Range<I>` type** (confirming Cut 7 and
  deleting `Range1<I>`) without `!` panics, `slide_until(r, ok)` yields
  non-optional `(lo: I, hi: I)` endpoint pairs directly. Add `(lo..=hi).clamp(D)`
  on integer ranges to clip sweeps cleanly to a domain (Batch 21 E7).

- **16.4 Algebraic `Index` (5 O(1) `Copy` shapes) & no `Pool<T>` (2.7, 2.16, 5.6, Stage 3 Cuts 5 & 6, Batch 15 N1, Batch 18 Collision 1).**
  `Bool`, unit and payload `enum` (on demand), `?T` (`T: Index`), bounded
  ranges `lo..hi`/`lo..=hi` (structural equality for identical written bounds
  in scope — `[0..n]T` is a plain `Int`-indexed array that unifies across
  function boundaries, while `domain D = 1..=n` opts into nominal index safety,
  Blind Critic 3), and tuples/records `(A, B)` derive `Index` automatically
  (alongside `Subset<D>`). Payload enums deriving `Index` solve typed multi-role
  `FlowGraph<Node>` with zero new `domain` sum syntax. `[D]E: Index` is deleted
  (violates O(1) `Copy` invariant; use `Subset<D>` or `@memo(hash)`), and
  `Pool<T>`/`Pool<Node<_>>`/`PoolMap`/`Handle` are deleted (use `type Id = 0..=MAX`
  with 4-byte storage + 4-byte niche `?Id`, `List<Node>`, `.push_id(x)`, `??=`,
  and `.freeze()`). Whole-array reset uses `a.fill(x)` (including `a.fill(none)`),
  and `compress(xs)` returns a `Compressed<T>` object with fresh domain `c.D`,
  value lookup `c[v] -> T`, and value-to-index `c.at(x) -> c.D` / `c.first_ge(x)`
  (Blind Critic 3).

- **16.5 Grids: `type Cell = (Row, Col)` + `p.adj4` / `p.adj8` (7.19, Stage 4 Fork 3, Batch 19 B19-10).**
  No dedicated `Grid<T>` type. Product-domain `type Cell = (Row, Col)` composes
  with `[Cell]T`, `bfs`, `Dsu<Cell>`, and `@memo`, while userland `std.grid`
  provides `Dir4`/`Dir8` (`.cw`, `.ccw`, `.back`, `.from("UDLR", ch)`),
  `p.step(d)`, `p.ray(d)`, explicit **`p.adj4`** and **`p.adj8`** (deleting
  silent-4 `p.around()`), and zero-copy `g.transposed` / `g.flipped` views.

- **16.6 Bidirectional generic inference + `f::<T>()` + plain adjacency (2.13, Stage 4 Fork 2, Batch 15 N8).**
  The type checker unifies concrete non-closure arguments (`from:`, `root:`,
  `g`) before checking closure parameters, and `std.graph`/`std.tree` place
  those arguments first (`bfs(from: [s], next: ...)`, `dijkstra(g, from: s)`,
  `Rooted(root: r, adj: ...)`) while accepting plain `V -> Iter<V>` unweighted
  adjacency without `[(to: v) ...]` wrappers. This removes ~90% of turbofish
  calls while keeping `f::<T>()` unambiguous for return-polymorphic calls like
  `parse::<Int>(tok)`.

- **16.7 Search: 8 canonical names, retiring `first_where` (4.3, Stage 3 Cut 16, Stage 4 Fork 5, Batch 18 SF5).**
  `first_where` / `last_where` are retired after causing silent WAs on
  non-monotone predicates. Binary search uses free functions (also callable as
  methods on domains): `bsearch_first(r, ok) -> ?I`, `bsearch_last(r, ok) -> ?I`,
  and `bsearch_first_some(r, f) -> ?(I, W)` (debug-checked monotone; `bsearch_first`/
  `bsearch_last` also take `F64` ranges with `steps: 80`). Sorted lookup uses
  `.first_ge(x)` / `.last_le(x)` uniformly across `List`, `[D]T`, `Set`, `Map`,
  and `SortedBag`. Linear search uses `first(it)`, `a.find(x)`, and
  `min_by`/`max_by`. `r.mid` stays on `Range<I>`.

- **16.8 Bare ops, unified `Mod<P>`, `LazySegTree.max_right`, `Rooted.fold_path`/`apply_path` + `Reversible`, and signed `Bitset.shifted` (3.1, 3.10, 7.19, Stage 3 Cuts 15 & 18, Stage 4 Fork 7, Batch 15 N2–N5, Batch 18 SF1–SF4, Batch 20 W10, Batch 21 E6).**
  Contestant range structures accept bare operators (`+`, `*`, `^`, `&`, `|`,
  `min`, `max`, `gcd`), `(id, op)` pairs, or `Action` (`LazySegTree` stock tags
  `AddSum`/`AddMin`/`AddMax`/`AssignSum`/`AssignMin`/`AssignMax`). Heaps and
  sorts use `MinHeap<T>`/`MaxHeap<T>`, `by:`, and `desc(x)`. Both `SegTree` and
  `LazySegTree` expose `.max_right`/`.min_left`. `Mod<P>` unifies compile-time
  `const` and runtime-frozen `let` moduli (`ModR` retired). `Bitset<D>.shifted(k)`
  and `Subset<D>.shifted(k)` accept signed `k` (positive up, negative down).
  `Rooted<V>` is the single tree abstraction (DFS spanning tree/forest, flat CSR
  `kids`, lazy LCA sparse table & jump table, zero-alloc `chains`), adding
  generic `tree.fold_path(a, b, ds)` and `tree.apply_path(a, b, mut ds, tag)`
  over any `tree.Pre`-indexed DS (using `std.algebra.Reversible` for 1×-memory
  non-commutative folds — retiring dual-tree `Hld` and `LazyHld`), plus
  `ancestor(v, k)`, `postorder`, `lift_while(v, pred)`, and `virtual_tree(keys)`.
  Core `std.graph`/`std.flow`/`std.dsu`/`std.geometry` add `two_edge_components`,
  `biconnected_components`, `articulation_points`, `two_color`, `topo_prefix`,
  `MinCostFlow`, `max_weight_matching`, `Dsu.mark()`/`undo_to(snap)`, and
  generic `Pt<T = Int>` (exact `turn` stays `Int`-only; `turn_eps` and `unit`
  for `F64`). Niche structures (`Rope`, `RangeKth`, `LiChao`, `PotentialDsu`,
  `Cover`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`, `XorBasis`, `Frac`,
  `Fixed`, `Tri`, `Counter`, etc.) live in `examples/local-lib/`.

- **16.9 Scope cleanup (`defer`), nested `fn` mutation, parallel tuple assignment, coherent `?T` ordering, and uniform optionality (1.3, 4.9, 5.7, 5.8, 5.10, 5.12, Stage 3 Cuts 1 & 3, Stage 5, `6f113c3`, `7d1c976`, Batch 20 W6 & W11).**
  `with` (both forms), `undo` blocks, and `mut fn` are deleted: `defer` is the
  one cleanup primitive, `lends` keeps its unbroken single-`lend` law (with
  post-`lend` write-back), and plain nested `fn` mutates outer `var`s directly.
  `if let` and `match` are expressions, destructuring `let (a, b) = xs[i]`
  always owns/copies values, `var (a, b) = ...` declares mutable `var`s, and
  `??=` handles map-bucket init without silent auto-vivification
  (`(m[k] ??= []).push(x)`).
  - **Parallel tuple assignment `(a, b) = (x, y)` IS locked in (resolving 15.12 & Batch 20 W6):**
    A tuple of distinct mutable places on the LHS evaluates every RHS expression
    first into temporaries and then assigns each place — supporting swaps
    `(a, b) = (b, a)`, extended Euclid `(r0, r1) = (r1, r0 - q * r1)`, and
    rolling DP updates cleanly. Overlapping/duplicate LHS places without proven
    disjointness are a compile error.
  - **Coherent `?T` & `Mod<P>` ordering: neither implements `Ord` (superseding 15.11, Blind Critic 3):**
    `6f113c3` made emitted C++ order `Opt<T>` with `none` first while keeping
    `a < b` an error, which silently makes `MinHeap<(?Int, V)>` pop unreachable
    (`none`) entries first! Meanwhile `List<(Int, Z, Int)>` failed in `g++`
    because tuples unconditionally emitted `operator<` even on `Mod<P>`. In v2,
    neither `?T` nor `Mod<P>` implements `Ord`, and structs/records/tuples
    derive `Ord` (and emit C++ `operator<`) **only when every field implements
    `Ord`**. To sort or heap-order optional keys, state where `none` belongs
    explicitly via `by:` or `??` (e.g. `sort(mut xs, by: x => (x.d == none, x.d ?? 0))`).
    `min`/`max`/`min=`/`max=` continue to skip `none` on either side (so
    `ans min= dist[i]` works when `ans: Int` and `dist[i]: ?Int`, Blind Critic 3).
  - **Uniform optionality & diagnostics (Stage 5, Batch 20 W11):** `i + k`,
    `xs.last`, and `min(gen)`/`max(gen)` ALWAYS return `?T` (removing the
    inconsistent constant-range `min` exception that forced guessing `!`).
    Open-range DS calls (`st.fold(..h[i])`), `if let` expressions, and
    `while let ... and` eliminate most `!` sites; flow facts elide proven-safe
    `!` runtime checks; and optional-mismatch diagnostics print the column,
    name the optional sub-expression, and print the parenthesized fix
    (`a + (max(...) ?? 0)`).

- **16.10 Predicates, stress hooks, checkers/interactors/graders, symmetric I/O, and honest `cpl stress` (6.6, 6.8, 7.1, 7.7–7.9, 7.18, Stage 3 Cuts 2 & 11, Stage 4 Fork 6, Stage 5, Batch 19, Batch 20 W7/W13/W15, Batch 21 E9).**
  Predicates collapse to `assert` (always checked local + judge; lints on `mut`)
  and `assume` (local/debug/stress only; statically rejects `mut`/I/O side
  effects so nothing is ever silently erased on the judge). The `stress ... by ...`
  DSL and standalone `protocol` DSL are deleted. Non-typeable stress constraints
  use dependent bounds (incl. record fields) + `Perm<V>`/`Tree<V>` + typed
  `read(gen:)` (with `witness` and coupled tuple reads) + whole-input `@gen` +
  automatic choice-stream shrinking. Verification uses `@checker` (inheriting
  inputs, reading solution stdout via `read_output()` and `assert`),
  `@interactor` (paired with expression-form `ask(..., limit: K)`), and
  `@grader` (bodyless extern fn, exported entry fn, or stateful grader struct).
  I/O adds symmetric `none:` sentinels on `read`/`print`/`read_output`,
  `read_line()` (after a token `read()`, skips trailing whitespace to the next
  line; empty line is `""`; strips `\r`, Batch 22 X1), `read_until(sentinel)`,
  char-range `@tags('.', '*', '1'..='9')`, and `read()` in assignments/expressions.
  Every fenced code block in `docs/one-page-cheat-sheet.md` is compiled in CI
  as a P0 gate so docs and implementation never diverge again (Blind Critic 3). `cpl stress` is unified into one
  honest command: defaults to in-file `@brute`/`@checker`/`@interactor` (never
  emitting parameterless `@brute` into `main`), escalates sizes (`1..=20`
  default), compares line-by-line unless `@checker` (with `--close <eps>` for
  float tolerance), always builds with `-DLOCAL`, shrinks every failure and
  replays `debug(...)` side-by-side, reports max size / rejection rate (>50%
  warning) / diff mode / output diversity, warns if a `@brute`-tested fn
  mutates top-level `var`, and suppresses complexity warnings inside `@brute`/
  `@gen`/`@checker`/`@interactor`/`assume`.

- **16.11 Max-N judge and compiler-trust guardrails (6.7, 7.4, Stage 6, Batch 18 §(e), Batch 19 B19-8, Batch 20 W5).**
  Emitted C++17 never emits `using namespace std;`, always widens shift
  literals to 64-bit (`1LL` / `cp_shl`, fixing Batch 20 W5's 32-bit `1 << k`
  UB), packs `?Int` (`LLONG_MIN`) and `?F64` (quiet NaN) in 8 bytes, packs
  `Mod<P>` in 4-byte `uint32_t` for `P <= 2^31 - 1`, never uses
  `std::vector<bool>`, lowers `@memo` to flat arrays for every algebraic
  `Index` tuple up to `2^26` slots, backs `HashMap`/`HashSet` with a
  per-process salted open-addressed flat hash table, builds `Rooted` LCA/jump
  tables lazily with flat CSR `kids`, backs `Steps` with a lazy-sorted
  `vector`, and tree-shakes all unused runtime/stdlib code (plus stripping
  `CP_GEN`/`assume`/`debug`/`@brute`) so typical judge submissions are 4–10 KB
  with the `.cp` source in a top comment header.

## 17. Stdlib and local-lib aligned with the v2 lock

`stdlib/` and `examples/local-lib/` now follow §16. The calls made while
writing the library against the spec, where the spec left room:

- **17.1 `(id, op)` is spelled `Monoid(id, op)`.** A structure takes a bare
  operator (`SegTree<I, Int>(+, a)`) or a `Monoid`; `Monoid<T>.of` turns the
  first into the second. A bare tuple would need overloading on the
  argument's shape, which the checker does not have, and the name costs six
  letters. The cut wrappers (`Sum`, `Max`, `Min`, `Prod`, `Xor`, `Order`,
  `Less`, `Greater`, `Semigroup`, `Group`, `flip`, `both`, `opt`, `MaxOpt`)
  are deleted, not deprecated.
- **17.2 `Fenwick` takes no operation.** It is sums over any `T` with `+`
  and `-` (`Fenwick<D, T>()`, `f.sum(range)`); an invertible operation of
  your own goes in a `SegTree`.
- **17.3 `Rooted` replaces `Hld`.** `fold_path(a, b, ds)` and
  `apply_path(a, b, mut ds, tag)` take any structure over `Rooted<V>.Pre`,
  so one `SegTree` or `LazySegTree` serves paths and subtrees. A climbing
  stretch is folded top-down and turned with `rev()`; `Reversible` defaults
  to the identity, so only non-commutative values implement it.
- **17.4 Grid helpers are extension fns on `(R, C)`**, imported by name
  (`use std.grid.{adj4}`); `Dir4` / `Dir8` are small structs, since an enum
  cannot carry methods yet.
- **17.5 `ModR` is gone.** A run-time modulus is `let P = p` then
  `Mod<P>`, the same type as a constant one.
- **17.6 Moved out of `std`, into `examples/local-lib/`:** the structures
  listed in T13 (`Rope`, `RangeKth`, `LiChao`, `PotentialDsu`, `Cover`,
  `Hom`, `TwoStackFold`, `mo`, NTT, `XorBasis`, `Frac`, `Powers`,
  `dnc_opt`, `knuth`, `Hash2D`, `Trie`, `SuffixAutomaton`, `fold_others`,
  `fold_balanced`, `overlaps`, `borders`, `sos_sub` / `sos_sup`), plus
  `BipartiteDsu`. Moved the other way, into `std`: `TwoSat` (now in
  `std.graph`) and rollback (now `Dsu.mark` / `undo_to`).
- **17.7 Not done yet:** `Fixed<D>`, `Tri<D>`, `Functional`, `Congruence`
  and `LiChaoArena` from the T13 list; the payload-enum `FlowGraph` node
  and a real-valued `bsearch`, which wait for enums with payloads and
  `F64` ranges in the reference interpreter.
