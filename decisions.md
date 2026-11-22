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
| 1.3 | Optionals and niches | Revised 2026-11-20 (v2.2) | `?T`, niche when the type has a spare value, else a flag; **`?T` does NOT implement `Ord`** (`==`/`!=` auto-lift `T` to `?T`). **Tropical Optional Rule:** `min`/`max`/`min=`/`max=` skip `none` (mixed `(T, ?T) -> T`); binary `+`/`-` and unary `-` on `?Int`/`?F64` absorb `none` (`*`/`/`/`%`/`sum`/`+=`/`-=` never lift); diverging RHS (`return`/`break`/`continue`) supported on `??`; unparenthesized `??` mixed with arithmetic is an error; explicit poison-failure check required when `none` means failure inside `min=` | 1.1, 1.5 | §16.9, §18.1, §19.5, §19.10 |
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
| 2.3 | `Range<I>` | Revised 2026-11-20 (v2.2) | **One `Range<I>` type everywhere** (`Range1<I>` deleted). Stored as two bounds (`l..r`, `l..=r`, `..i`, `..=i`, `i..`, `i.r..`, `l.r..r.l`); `.l`/`.r` (boundary cuts even if empty), `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len`, `.is_empty`, `.rev`, `.step(k)`, `(lo..=hi).clamp(D)`. `D.chunks`, `D[..k]`, `r1 & r2`, and `.mirror` deleted. `..`/`..=` bind looser than arithmetic, tighter than comparisons/`in`. Empty/inverted ranges are valid no-ops except `SparseTable.fold -> T` (asserts non-empty) | | §16, §18.5, §19.7 |
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
| 3.5 | Core & library traits (7 + 2) | Revised 2026-11-20 (v2.2) | **7 core auto-derived traits** (`Index`, `Seq`, `Eq`, `Ord`, `Hash`, `Show`, `Read`; `Copy` internalized into compiler value semantics) + **2 library-author traits** (`Indexed` unified with `IndexedMut`, and `Iterable`; `Gen` dropped). Internal stdlib hooks (`Reversible`, `Neighbours`, `Target`) remain normal library traits outside the core | | §16, §19.2, research/language-architecture-v2.2.md |
| 3.6 | Implicit representation choices | Settled | only when no worse on any op or memory, and visible (G9) | | plan/00-vision.md G9 |
| 3.7 | Structural field bounds | Leaning | `A has to: V` as a general feature, not graph-specific | | plan/13-abstraction.md G9 |
| 3.8 | Associated types, consts and frozen lets | Settled 2026-11-20 | yes. Associated types (`Indexed.Elem`, `Seq.bounds`). An associated size is `const K = 10` when known at compile time and **`let K = n`** when frozen from a run-time value where the impl is declared; **`const K = n` with a run-time `n` is an error**, so `const` stays strictly compile-time (7.10). Sizing by a `var` is an error | 7.10 | plan/19-index-and-lends.md §3.1 |
| 3.9 | Coherence (who may `impl` what) | Settled 2026-11-19 | orphan rule relaxed inside one program, strict for libraries | 6.4 | §8 |
| 3.10 | `Mod<P>` (unified moduli) | Revised 2026-11-20 (v2.2) | **One `Mod<P>` type** accepting both `const` and runtime-frozen `let` `Int` moduli via 3.8 (`ModR` retired). Works for any `P >= 1` (incl. composite; `.try_inv() -> ?Z`); 4-byte `uint32_t` when `P <= 2^31 - 1`, `I128` widen above; one-way `Int` reduction; constant-divisor inverse lowering; `T.zero`-seeded `sum`; immutable `let comb = Comb<P>()` with interior-mutability lazy growth | 3.8 | §16, §17.5, §18.7, §19.10 |
| 3.11 | Decorators (`@`) | Revised 2026-11-20 (v2.2) | User-defined `decorator` macros deleted. **8 contestant decorators** (`@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`, `@interactor`, `@grader`) + **3 library-author** (`@get`, `@test`, `@law` replacing hard keyword `law`). Parameter grammars `@memo(flat\|hash)`, `@multitest(count:)`, and `@grader const` deleted. **0.2.2:** the test-case divider is the file-level directive `@!multitest` (§20), not a decorator; `@` always decorates the one item below it | | §16, §19.1, §19.3, §20 |

## 4. Control and iteration

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 4.1 | Blocks as expressions, `match`, `?` | Settled | indentation blocks, `=>` arms, `?` on optionals | | plan/17-syntax.md |
| 4.2 | Generators | Settled, revised 2026-11-19 | inlined; the compiler picks push or pull per use: push for a plain `for`, pull for `zip` or when stored as `Iterator<T>` (so a non-recursive generator can be stored); a recursive generator where pull is needed is an error suggesting `List(g)`; no hidden explicit stacks | | research/RECOMMENDATIONS.md §2, plan/12-domains-and-sequences.md §5 |
| 4.3 | Sequence views, windows & search | Revised 2026-11-20 (v2.2) | Imperative `Deque<T>` + explicit `while` for two-pointers (`slide_until`, `sliding_min`/`sliding_max`, `bsearch_first_some`, `rotate_right`, `prod`, `floor_div` deleted; `MonoDeque` demoted to `examples/local-lib/`). **6 canonical search names:** `bsearch_first`/`bsearch_last`, sorted `.first_ge`/`.last_le` (sequences -> index `?Int`/`?D`, `Set`/`SortedBag` -> `?K`, `Map` -> `?(K, V)`), linear `first`/`find`/`min_by`/`max_by`. Signed `rotate_left`, `next_permutation`/`prev_permutation`, `xs.adjacent`, `a.items`, `sort(..., by: desc)`, `Rng`/`rand`/`shuffle` | 2.3, 4.2 | §16, §19.7, §19.9, §19.10 |
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
| 5.3 | Precise borrow rules | Settled 2026-11-19, revised 2026-11-20 (v2.2) | lexical exclusivity: no stored and no returned borrows (so no lifetimes); fields borrow separately; two lends of one projection are separate when `@law disjoint` is proven from flow facts (5.13; auto-derived for trivial single-index forwarding); copyable element reads release borrows immediately before enclosing calls; **`a[idx].f = rhs`, `??=`, and compound assignments evaluate all index expressions and `rhs` before borrowing the mutable place in `a`**; `for mut` and `.absorb()` deleted; `.swap(i, j)`, O(1) `swap`, `.take()`, and mutable subslices handle same-container/small-to-large cleanly | 5.1 | §18.4, §19.4, §19.8 |
| 5.4 | Inline closures reusing a caller's `mut` borrow | Revised 2026-11-20 (v2.2) | yes: closure args and `lends`/`yield` carry inline-desugaring semantics automatically; `inline` keyword deleted | 5.3 | §19.1, research/RECOMMENDATIONS.md §1 |
| 5.5 | Moves out of arrays | Settled 2026-11-20 | `swap`, `take` (leaves default) and `pair_mut` are library fns over `at`, allowed by its `disjoint` law; no partial moves | 5.13 | plan/19-index-and-lends.md §3.6 |
| 5.6 | Pools and handles | Deleted 2026-11-20 (v2) | Replaced by `type Id = 0..=MAX` + `List<Node>` + `.push_id(x)` + `.freeze()` (see 2.7) | 2.7 | §16, research/subtraction-audit-1.md Cut 6 |
| 5.7 | `defer` | Deleted 2026-11-20 (v2.2) | Removed (1/642 ports, active bug in search-and-keep backtracking); use explicit 1-line `dsu.undo_to(snap)` or `cur.pop()` | | §19.1, research/subtraction-audit-2-v21.md §2.1 |
| 5.8 | `Rollback`, `undo`, `with` | Revised 2026-11-20 (v2.2) | `undo`, `with`, and `defer` deleted; rollback is a plain 2-method struct API (`let snap = dsu.mark(); ...; dsu.undo_to(snap)`) | 5.7 | §16, §19.1 |
| 5.9 | `@memo` control | Revised 2026-11-20 (v2.2) | Bare `@memo` only (`@memo(flat)`/`@memo(hash)` deleted): auto-dispatches flat niche-packed `[Params]?R` for any value return type up to `2^24` (`2^26` for `?Bool`/`Bool`), else salted flat hash; nested `@memo` scoped to enclosing fn/loop iteration; permits `ask(...)` and deterministic extern `@grader fn` oracles (exact 1 call per arg tuple); forbids mutated captured `var`s and `read()`/`print()` | 2.16 | §16, §19.3, §19.9 |
| 5.10 | Nested `fn` mutation (`mut fn` deleted) | Revised 2026-11-20 (v2) | **`mut fn` deleted.** Any nested `fn` declared after a `var` reads/writes it directly; the compiler tracks captured borrows automatically and forbids escaping | 5.3 | §16, research/subtraction-audit-1.md Cut 3 |
| 5.11 | Keeping the borrow rules understandable | Settled 2026-11-19 (the method; the rules stay open to change) | fresh agents repeatedly search for the smallest programs where a borrow rule surprises; each one changes the rule or its error message and is kept as a test | 5.3 | plan/agent-context.md |
| 5.12 | Projections: `lends` (tail expression) | Revised 2026-11-20 (v2.2) | `fn row(self, r: A) lends mut [B]T` lends via its tail expression or `return mut place` (`lend` keyword deleted). Post-lend write-back lets `SegTree`/`LazySegTree` repair invariants after mutations | 5.3, 5.4 | §16, §19.1 |
| 5.13 | `@law disjoint` | Revised 2026-11-20 (v2.2) | `law` demoted from hard keyword to `@law` (`@law disjoint(i, j) = i != j`); trivial single-index forwarding (`mut self.data[i.to_idx]`) auto-derives `disjoint`. Two live lends of one projection on one receiver, one `mut`, need the law proven from flow facts (`if`, `while`, `assert`, value ranges) | 3.4, 5.12 | §19.1, plan/19-index-and-lends.md §3.6 |
| 5.14 | Lent values are second-class | Settled 2026-11-20 | a value built from lent parts is lent and lives only in the block that made it: it may be passed, bound by `let`, iterated and lent onward; not put in a `var`, a field or a container, captured by a non-`inline` lambda, or returned. Borrowed-ness is a mode of the binding, not a type, so there is no `view` keyword | 5.3 | plan/19-index-and-lends.md §3.5 |

## 6. Program structure

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 6.1 | File as function body | Settled | top-level fns see the body's `let`s; calling before the `let`s are set is an error | | plan/15-program-structure.md §2 |
| 6.2 | Program vs library mode | Settled | decided by contents; `use` accepts only libraries | | plan/15-program-structure.md §2.4 |
| 6.3 | Multitest | Revised 2026-11-20 (0.2.2) | `@!multitest` is a standalone top-level **divider**: statements above it run once (precomputation), statements below it run once per test case (the count is read automatically; `@!multitest(until_eof)` reads to end of input), each case in a fresh scope with `@memo` caches reset. Single-test and multitest files are both flat top-level statements; no `fn solve()` wrapper. `@!` marks a file-level directive in force from its line down; `@` always decorates the one item below it. Top-level `return` ends the program, or the current test case below the divider. `@spec` / `@checker` read the top-level input `let`s the same way in both modes | 6.1 | §16, §19.3, §20 |
| 6.4 | Modules beyond `pub` | Open | re-exports, private-to-folder, name clashes | | plan/40-tooling.md |
| 6.5 | Parts and purity | Settled | any fn is a part; tools record I/O; no `pure` keyword | | plan/15-program-structure.md §4 |
| 6.6 | Brutes, checkers, interactors | Revised 2026-11-20 (v2.2) | `@brute`, `@checker` (inherits input bindings, uses `read_output()` + `assert`), `@interactor fn judge(sol: Interactor):` (`read()` = hidden test, `sol.recv()` = sol stdout, `sol.send(...)` = sol stdin; paired with `ask(..., limit: K)`, and `print()` auto-flushes whenever `ask()` is used) | 3.11 | §16, §19.8 |
| 6.7 | Staying under 64 KB | Locked v2.2 (2026-11-20) | Tree-shake all runtime/stdlib helpers and strip `CP_GEN`/`debug`/`@brute` in judge builds (target 4–10 KB typical, always < 64 KB) | 3.2 | §16, §19.1, research/max-n-judge-audit-1.md |
| 6.8 | Grader-style (IOI) problems | Revised 2026-11-20 (v2.2) | `@grader` without new keywords: bodyless `@grader fn` = extern oracle (callable inside `@memo`), `@grader fn` with body = exported entry, top-level `var` or `@grader struct` = stateful grader; array params may size off sibling `Int` params left or right. `@grader const` and uninitialized file-scope grader domains deleted | 3.11, 6.1 | §16, §18.10, §19.3, §19.9 |
| 6.9 | Top-level `var` | Locked v2 (2026-11-20) | allowed; any nested `fn` declared after a `var` reads/writes it directly (5.10) | 5.10 | §16 |

## 7. Cross-cutting

| # | Decision | Status | Current lean | Depends on | Where |
|---|---|---|---|---|---|
| 7.1 | Checks & predicates (`assert`) | Revised 2026-11-20 (v2.2) | **One predicate keyword (`assert`):** always checked (local + judge; lints on `mut`; feeds proven flow facts). `assume` deleted (use typed `read()` bounds, `read(gen:)`, `@checker`, and `debug(...)`). Domain indexing never checked; `List`/`Str`/`a.raw` checked unless proven or in `unchecked:` | 7.9 | §16, §19.1, research/syntax-bakeoff-2-v21.md |
| 7.2 | Overflow in release builds | Settled 2026-11-19 | proven safe: no check; provably reachable: a warning with the bound; unknown: trapped in local builds, unchecked in the judge build | 1.1 | research/RECOMMENDATIONS.md §4, plan/30-compiler.md |
| 7.3 | Trace data contract | Open | spans, typed values, writes old → new, call tree, test and part boundaries | 6.5 | plan/50-debugging.md |
| 7.4 | Zero-cost & max-N judge guarantees | Locked v2.2 (2026-11-20) | No `using namespace std;` + wrap user code in `namespace cp_user { ... }`; 8-B `?Int`/`?F64`, 4-B `Mod<P>` (`P <= 2^31-1`), 4-B `I32` stamps in opt-in `std.marks.Marks<D>`, `vector<uint8_t>` never `vector<bool>`, lexically scoped `@memo` with runtime flat (`<= 2^24`, `2^26` for tri-state `?Bool`) vs salted flat hash dispatch, lazy `Rooted` LCA + CSR `kids`, index-before-place-borrow lowering preventing `vector` UAF | 1.3, 5.9 | §16, §18.2, §19.6, §19.8 |
| 7.5 | C++ interop | Open | `@cpp` blocks with checks at the boundary | 1.1 | |
| 7.6 | Remaining syntax | Locked v2.2 (2026-11-20) | **34 reserved keywords** (30 contestant + 4 lib-author); single lambda `x => expr`; single inline-`if` `if c: a else: b`; `else if` -> `elif` diagnostic; `"{val:.6f}"` format specifiers | | §16, §19.1, docs/one-page-cheat-sheet.md |
| 7.7 | TLE / MLE / overflow estimates | Revised 2026-11-20 (v2.2) | from typed input bounds and domain sizes; **warnings only**, never errors; suppressed inside `@brute`, `@gen`, `@checker`, `@interactor` | 6.3, 7.2 | §16, §19.1 |
| 7.8 | Typed input drives generators and shrinkers | Revised 2026-11-20 (v2) | Dependent bounds (incl. record fields) + `Perm<V>` / `Tree<V>` + `read(gen:)` (with `witness` & coupled tuple reads) + `@gen` + automatic choice-stream shrinking. **`stress ... by ...` DSL and `Distinct`/`Sorted` wrappers deleted** | 6.6 | §16, research/syntax-bakeoff-1.md Fork 6 |
| 7.9 | Contracts | Revised 2026-11-20 (v2.2) | Unified into `assert` (always checked), typed `read()` bounds, `read(gen:)`, and `@checker`; `assume`, `requires`, `ensures`, `invariant`, and `read() where` deleted | 7.1 | §16, §19.1 |
| 7.10 | Compile-time `const` | Settled 2026-11-19 | adopted, with a size warning for large baked tables | 6.7 | plan/18-key-features.md §3 |
| 7.11 | REPL on a loaded test | Deferred 2026-11-19 | not now; cheap to add once the debug interpreter exists | | plan/18-key-features.md §4 |
| 7.12 | Command name and file extension | Settled 2026-11-19 (name is a placeholder) | the binary is `cpl` until Ryan picks a final name, because `cp` clashes with Unix `cp`; source files are `.cp` | | plan/b0-distribution.md |
| 7.13 | Editor support in the core | Settled 2026-11-19 | LSP with partial-parse recovery, VS Code extension and syntax highlighting are P0 (G13) | | plan/60-editor.md |
| 7.14 | Trace viewer design | Settled 2026-11-19 | designed fresh with its own design system; the old `mock/trace-mock.html` is rejected and not in the repository. What the trace must contain (7.3) is unaffected | 7.3 | plan/50-debugging.md |
| 7.15 | Sample local library | Settled 2026-11-19 | a fresh one, inspired by popular contest libraries, not a port of `ryanbai1412/cp-library`; it also proves G8 and tests library pinning (G11) | 6.4 | plan/40-tooling.md |
| 7.16 | `test` and `doc` for libraries | Settled 2026-11-19 | `cpl test` and `cpl doc` work the same on the stdlib and on a user's own library | 6.4 | plan/45-dev-tools.md |
| 7.17 | How conciseness is measured | Settled 2026-11-19 | two numbers per port: % fewer lines than the original, and % fewer than the original with library-able code stripped | | plan/80-validation.md §8.1 |
| 7.18 | Honest `cpl stress` & `debug(...)` | Locked v2 (2026-11-20) | One `cpl stress` command defaulting to in-file `@brute`/`@checker`/`@interactor`, escalating sizes (default up to 20), comparing line-by-line unless `@checker` (plus `--close <eps>` for floats), always compiling with `-DLOCAL`, shrinking every failure via choice streams, replaying `debug(...)` side-by-side, and printing max size, rejection rate (>50% warning), diff mode, and output diversity | 6.6, 7.8 | §16, research/blind-critic-2-debug-pivot.md |
| 7.19 | Grids, trees & collection verbs | Revised 2026-11-20 (v2.2) | `type Cell = (Row, Col)` + `std.grid` (`p.adj4`, `p.adj8`, `Dir4`, `p.step`, `p.ray`, `g.transposed`; `.flipped_*` deleted, `Dir8` demoted); `Rooted<V>` with value-owned `t.Pre`, `t.pos(v)`, `from_parents`, `t.tour`, `.fold_path`/`.apply_path` + `Reversible`, `.ancestor`, `.postorder`, `.lift_while -> ?V`, `.virtual_tree`; unified container verbs (`.len`/`.is_empty`, `.take()`, `List.insert`/`remove`, `Map.remove -> ?V`, `m[k] f= x` auto-insert, `SegTree`/`LazySegTree` `.all` & `Action(id, (new_f, old_f), (f, val, span), over:)`; `.absorb()` deleted; `Marks<D>` demoted to opt-in 32-bit `std.marks`; 11 near-zero structures moved to `examples/local-lib/`) | 2.16, 3.1 | §16, §18.5, §19.4, §19.6, §19.7, §19.9 |

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
  - **7 + 2 language traits** (the count as revised in 3.5): 7 auto-derived core traits (`Index`, `Seq`,
    `Eq`, `Ord` only when all fields are `Ord`, `Hash`, `Show`, `Read`)
    plus 2 library-author traits (`Indexed`, `Iterable`).
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
- **17.7 Not done yet:** the payload-enum `FlowGraph` node and a
  real-valued `bsearch`, which wait for enums with payloads and `F64`
  ranges in the reference interpreter. The last five T13 structures are
  now in `examples/local-lib/`: `Fixed<D>` (`math/fixed.cp`), `Tri<D>`
  (`ds/tri.cp`), `Functional<V>` (`graph/functional.cp`), `Congruence`
  (`math/congruence.cp`) and `LiChaoArena` (`ds/li_chao_arena.cp`).
  `Fixed<D>` has `parse` and `Show`; a `read()` of one waits for struct
  `Read` in the reference interpreter.

## 18. CP Lang v2.1 Unification & Concept-Budget Guard (2026-11-20 — Language Architect Wave 3)

Wave 3 synthesizes all 21 scout friction backlogs across Batches 21–41
(`research/friction-backlog-batch-21.md` through `batch-41-oly-icpc.md`,
covering 200+ hard ported problems) against locked v2 (§16), the max-N
prototype pass (`c8e381f`), and the aligned `stdlib/` + `examples/local-lib/`
(§17). Full before/after examples and the next-sprint checklist live in
`research/language-architecture-v2.1.md`; `docs/one-page-cheat-sheet.md`
is synchronized to match. **Zero keywords (40), zero numeric types (4),
zero traits (7 + 2, as revised in 3.5), and zero `@` decorators (7 + 2) are added.**

- **18.1 Tropical Optional Rule (`?T`), unparenthesized-`??` guard, and `?T == T` (1.3; Batches 27, 28, 32, 33, 34, 35, 36, 38, 39).**
  Across nine batches, unlifted `+`/`-` on `?Int`/`?F64` forced repeated
  `if let` guards and hand-written `plus`/`shift` helpers that made `?Int`
  longer than `INF` sentinels. CP Lang v2.1 locks one algebraic rule:
  `none` is the **identity** of `min`/`max`/`min=`/`max=` (including
  `min(gen)`/`max(gen)` over `?T` and `ans min= opt` on plain `T`), and
  `none` is the **absorber** of `+`, `-`, `+=`, `-=`, unary `-`, and
  `sum(gen)` on numeric optionals `?Int` and `?F64` (never `*`, `/`, `%`,
  bitwise, or `?V + k`). To prevent the `a[i] + max(...) ?? 0` precedence
  trap permanently, mixing `??` with binary arithmetic (`+ - * / %`) without
  parentheses is a **compile error**. Equality `==`/`!=` auto-lifts `T` to
  `?T` (`if dq.last != x:`), while `<`/`<=`/`>`/`>=` remain compile errors
  (`?T` still does not implement `Ord`). Chained `if let a = x and let b = y:`
  and `while let` are surfaced on the cheat sheet; custom `?Struct` merges
  use `match (a, b):`.
- **18.2 Lexically scoped `@memo`, runtime flat-vs-hash dispatch, and uniform range narrowing (5.9; Batches 23, 28, 34, 35, 40).**
  `@memo` is legal on any `fn` — top-level or nested inside a function or
  loop — capturing immutable outer `let`/loop variables (mutated captured
  `var` is a compile error). Its cache is scoped to the enclosing block and
  hoisted into one reusable buffer cleared/stamped on block entry (eliminating
  the 185 MB 5-arg hash cliff in `coci21c1p3` and supporting scenario loops in
  `ioi20p4`). Default `@memo` over all-`Index` parameters chooses flat vs.
  salted open-addressed `CpHashMap` **at runtime** from `prod(K_i)` (`<= 2^24`
  slots, or `2^26` for tri-state `uint8_t` `?Bool`, fixing `noip21p3`), and
  reports the chosen mode in `cpl check --verbose` / LSP hover. Range-typed
  parameters on `@memo` follow the exact same entry-check and flow-narrowing
  rules as ordinary `fn`s (`cheerio1s5`), and `..`/`..=` bind looser than
  arithmetic and tighter than comparisons/`in` (`i + 1..=n` is `(i + 1)..=n`).
- **18.3 Complete algebraic `Index` surface & lossless scalar casts (2.16; Batches 22, 28, 30, 33, 35, 37, 38, 39, 41).**
  Clarifies all five existing `Index` shapes: `Bool` (`Bool.K == 2`,
  `for b in Bool`, `[V, Bool]T`), structural `Char` ranges (`['a'..='z']Int`),
  guaranteed row-major `.to_idx`/`.from_idx` on tuple/record products `(R, C)`,
  and `Subset<D>` (`for s in Subset<D>` in ascending integer order, `.empty`,
  `.full`, and O(1) `s.first -> ?D` via `ctz` for bitmask DP peeling).
  Lossless scalar casts `Int(ch)` (`0..=127`), `Char(i)`, and `Int(b)`
  (`0 | 1`) are locked; `Int(f)` on `F64` remains a compile error.
- **18.4 Lexical borrow & evaluation rules: `for mut`, `Copy` release, RHS-before-LHS, `.swap`, `.take()`, `.absorb()` (5.3; Batches 24, 25, 26, 31, 34, 36, 38, 41).**
  `for mut x in xs:` lends elements mutably for arrays/lists of self-adjusting
  structs (`Block`, `Dsu`, `MonoDeque`) while keeping generators pure. Reading
  a `Copy` element releases its borrow immediately before enclosing calls
  (`merge_heap(hrc[u], v)`). Indexed assignment `a[i].f = rhs` evaluates
  indices and `rhs` before borrowing `a[i]` (making recursive arena updates
  `nodes[u].l = merge(nodes[u].l, b)` both borrow-safe and reallocation-safe).
  `a.swap(i, j)` on `[D]T`/`List<T>` handles `i == j` as a no-op; free-fn
  `swap(mut a, mut b)` is guaranteed O(1) on every heap container; `.take()`
  moves a heap container out in O(1) leaving an empty one behind
  (`for x in mem[u].take(): mem[v].push(x)`); and `.absorb(mut other)` drains
  small-into-large so it cannot be inverted into O(n^2). `[D]T = expr` clones
  any `T` expression independently per slot (`[D]List<T> = []` never aliases),
  and `xs.last` stays by-value (`xs[xs.len - 1]` mutates stack tops).
- **18.5 Ranges, chunks, boundaries, `SparseTable.fold -> ?T`, and 2D grid flips (2.3, 7.19; Batches 24, 26, 32, 34, 37, 38, 41).**
  Every `Range<D>` exposes `.l` and `.r` boundary cuts directly (even when
  empty, eliminating `rw.first!.l` noise in `windows(k)`). Add `D.chunks(k)`
  (clamped non-overlapping `Range<D>`), `Compressed<T>` range slicing `ys[blk]`,
  domain prefix `D[..k]`, range intersection `r1 & r2`, and `p.mirror`/`r.mirror`.
  Replace ambiguous 2D `g.flipped` with `g.flipped_rows` and `g.flipped_cols`
  (`g.transposed` yields `[C, R]T`). `SparseTable.fold(r)` returns `?T`
  (`none` on empty ranges; no monoid identity needed).
- **18.6 Block-scoped `defer` and bare top-level `return` (5.7; Batches 24, 31, 37, 38, 41).**
  `defer` is explicitly locked as **block-scoped** LIFO (runs at the exit of
  the immediately enclosing loop iteration, branch, or `fn`, never Go
  function-scoped), covering Rollback Mo's and backtracking DFS cleanly. Bare
  top-level `return` exits the program cleanly (or ends the current case in
  whole-file `@multitest`).
- **18.7 Unified `Mod<P>`, `Int` promotion, `T.zero`/`T.one`-seeded `sum`/`prod`, and cross-`@multitest` precomputation (3.10; Batches 23, 29, 34, 37, 41).**
  Confirms `ModR` retirement (§17.5): `let M = ...; type Z = Mod<M>` unifies
  static and runtime moduli (including composite `P >= 1`, `.try_inv() -> ?Z`,
  `I128` multiply past `2^31 - 1`, compile-time inverses for constant
  divisors, and one-way `Int` ops). `sum(it)` seeds from the element type's
  zero, and `prod(it)` joins the sequence prelude seeded from the element
  type's one; binary-op diagnostics always print type names. Cross-`@multitest`
  precomputation needs no `static` or `@once`: `Comb<P>()` auto-grows lazily
  on demand, and top-level code above `@multitest fn solve():` runs once.
- **18.8 Value-owned `t.Pre` on `Rooted<V>`, vertexless graph turbofish, and `Dsu.groups()` (7.19; Batches 24, 26, 27, 34, 39, 41).**
  `Pre` on `Rooted<V>` becomes a value-owned fresh domain `t.Pre` (like
  `xs.D` on `compress`), preventing cross-tree preorder mix-ups (`ioi18p3`,
  superseding the type-level `Rooted<V>.Pre` in §17.3). Add `t.pos(v) -> t.Pre`
  (inverse of `t.preorder: [t.Pre]V`), `Rooted.from_parents(par)`, and
  iterative `t.tour` yielding `(v, entering)`; pin `t.lift_while(u, pred) -> ?V`
  to highest ancestor where `pred` holds from `u` up (or `none` if `pred(u)`
  is false). Document `topo::<V>(next)` for vertexless closure graphs and add
  `Dsu<V>.groups() -> List<List<V>>`.
- **18.9 Stamp table `Marks<D>`, generalized `RollingHash`, `Map` 4-lookup `?(K, V)`, `min_by`/`max_by` ties, and container polish (Batches 25, 29, 30, 31, 32, 33, 36, 38, 39, 40, 41).**
  Add userland `Marks<D>()` (generation-stamped visited set with `.add(v) -> Bool`,
  `v in m`, and O(1) `.clear()`) alongside `a.fill(x)`. Generalize `RollingHash`
  over `Str` and `[D]T` returning `HashVal` with independent per-instance
  salted bases. Give ordered `Map<K, V>` all four lookups (`.first_ge`,
  `.first_gt`, `.last_le`, `.last_lt`) returning `?(K, V)` (`Set`/`SortedBag`
  return `?K`); accept optional iterables in container constructors; support
  `by:` on sequence `first_ge`/`last_le` and `.sorted()`/`.rev` on slices;
  exempt mutating `.pop*()`/`.add()`/`.unite()` from unused-result warnings;
  add `rotate_right`. Pin `min_by`/`max_by -> ?T` with first-extremum tie
  breaking, guaranteed stable `sort`/`sorted`, `bit_len(x)` (retiring
  `bit_width`), hex/binary literals, and `Sieve(n)` / `factor(x)` /
  `divisors(x)` (retiring `factorize`).
- **18.10 `@grader` constants & shared domains, sibling-sized array params, `Int`-indexed `Str`, and judge `print` (1.6, 6.8; Batches 23, 25–27, 30, 33, 36, 37, 40, 41).**
  Allow `@grader const N: T` (extern judge constant), file-scope size-less
  `domain V` bound by grader parameters for shared callbacks, and sibling-sized
  array parameters left or right (`fn Assist(needed: [0..r]Bool, n: Int, k: Int, r: Int)`).
  Clarify `Str` is `Int`-indexed (`0..s.len`, zero `.raw`) with `s + ch`,
  `s.items`, and `match s:` on string literals, alongside `[expr for x in it if c]`
  and `List.filled(n, v)`. Lock judge `print`: domain values print their
  declared label, lists/1D arrays print space-separated, and `[R, C]Char` grids
  print row-by-row (`debug(...)` prints structured brackets to stderr).
- **18.11 Compiler-trust & stress guardrails (7.4, 7.18; Batches 21–30, 34, 36).**
  Wrap all emitted C++17 user definitions in `namespace cp_user { ... }` with
  C++/POSIX macro mangling (eliminating `tm`, `dup`, `arg`, `y0` collisions);
  pin `LazySegTree.set` root-to-leaf pushdown (`ccc02s4hard`); stream stdin
  incrementally for `@interactor`; add `cpl stress --max-vals` for overflow
  hunting (`O2`); and place constructive generators (`perm`, `surjection`,
  `bounded_product`, `tree`, `connected`, `dag`, `functional`, `simple_graph`,
  `str_of`, `rand_points`, `rand_disjoint_rects`, `rand_convex`) in `std.gen`.
- **18.12 Concept-budget rejections (permanent; see `research/language-architecture-v2.1.md` §3).**
  Explicitly rejected to protect the 40-keyword / 4-numeric-type / 1-page
  budget: `Pool<Node>`/`Node.Id`/`SegPool`, `I32`/`U32`, `undo`/`Journaled`,
  `static let`/`@once`, `ModR`, `domain A | B` sum-domain syntax, `min=`/`max=`
  returning `Bool`, dependent `@memo` parameter bounds, rolling-row `step ... into`,
  variable-length `@tags` DSL, bitwise `|`/`^` on `Bool`, `loop:` keyword,
  `let` inside generator comprehensions, keyed array literals `[R: 1, G: 2]`,
  `Rev<T>` (use `Desc<T>`/`desc(x)`), `Heap<T>(by:)`, user operators on
  structural `type` aliases, runtime-sized `SegTree.from(list)`/`combine_into`
  in core `std`, and single-problem core helpers (`Pos.window`, `g.lattice`,
  `cyc_range`, `dag_dp`, `take_except`). Niche structures (`Centroids`,
  `reroot`, `kruskal_tree`, `linalg` `det`/`rank`/`solve`, `Interner`,
  `spanning`, `ranks`, `lis`, `count_in_rects`, `MinCount`, `RollbackArray`,
  `Functional` (already landed in `examples/local-lib/`), arbitrary-prime
  `Poly` MTT + `.interpolate`, `F64`/record `LiChao`) live in
  `examples/local-lib/`.

## 19. Locked CP Lang v2.2 Architecture (Wave 4 — Ultra-Lean 34-Keyword Spec, 2026-11-20)

Wave 4 synthesizes **Subtraction Audit #2** (`research/subtraction-audit-2-v21.md`,
`d6b3c4c`, 642 canonical `.cp` ports), **Syntax Bakeoff #2**
(`research/syntax-bakeoff-2-v21.md`, `a9ca4f8`, 20 hard multi-topic ports +
611-port incidence census), **Blind Critic #4**
(`research/blind-critic-4-v21-audit.md`, `02f6137`, 8 blind 2 AM Olympiad/ICPC
drafts + 3 pivot/WA drills), and **Batches 47–51** (50 hard CCC/CCO, IOI,
NOI/NOIP, APIO, and EGOI ports) into **CP Lang v2.2**
(`research/language-architecture-v2.2.md`, `docs/one-page-cheat-sheet.md`).
(This edit also removes a stray merge-conflict trailer left at the end of §18.)

- **19.1 Six keywords cut: 40 -> 34 reserved keywords (3.11, 5.4, 5.7, 5.12, 5.13, 7.1, 7.6, 7.9; Subtraction Audit #2 §§2.1–2.2, Syntax Bakeoff #2 Tension 5).**
  The locked keyword budget drops to **30 contestant keywords** (including 3
  literals `true`, `false`, `none`) plus **4 library-author keywords** (`trait`,
  `impl`, `lends`, `yield`):
  - **`defer` deleted:** 1/642 ports (`noi22p4.cp`, where pure `mask.with(e)`
    shrinks 8 lines to 4), active WA trap in search-and-keep backtracking
    (`noip04p4.cp`), and Batch 51's `ccc96s3` only reached for `defer cur.pop()`
    because `prev_permutation` was missing. Explicit `dsu.undo_to(snap)` and
    `cur.pop()` are 1 unmistakable line.
  - **`assume` deleted:** 6/611 ports (1.0%), saves 0 LOC and 0 keystrokes over
    `assert` while causing real misuse (`cco24p4.cp` wrote `assume` on a
    postcondition self-check). `assert` is the sole predicate keyword (always
    checked; feeds flow proofs); input constraints live in typed `read()` bounds,
    `read(gen:)`, and `@checker`; local-only tracing uses `debug(...)`.
  - **`where`, `inline`, and `lend` deleted:** 0/642 ports for `where` and
    `inline` (inline generic bounds `<T: Ord>` and implicit inline semantics on
    `lends`/`yield`/closures handle 100% of cases); `lend` appeared in 1 library
    file (`matrix.cp`) and is replaced by the tail expression (or `return mut
    place`) inside `lends` / `lends mut` bodies.
  - **`law` demoted from hard keyword to `@law`:** 0/642 contestant ports and 3
    `stdlib/` files. `@law disjoint(i, j) = i != j` keeps full SMT/property-test
    support, frees `law` as an identifier, and auto-derives disjointness for
    trivial single-index forwarding (`mut self.data[i.to_idx]`).
- **19.2 Core traits 8 -> 7 & library traits 4 -> 2 (3.5; Subtraction Audit #2 §2.12).**
  `Copy` is internalized into compiler value semantics (0 uses as a user trait
  bound across 642 ports and `stdlib/`; small `<= 16 B` non-heap values copy
  implicitly, heap values move on last use and clone via `.copy()`). The 7
  auto-derived core traits are `Index`, `Seq`, `Eq`, `Ord`, `Hash`, `Show`, and
  `Read`; the 2 library-author traits are `Indexed` (unified with `IndexedMut`)
  and `Iterable` (`Gen` dropped since choice-stream shrinking works on any `@gen`
  function).
- **19.3 Decorator parameter grammars & grader edge cases cut (3.11, 5.9, 6.3, 6.8; Subtraction Audit #2 §2.5, Syntax Bakeoff #2 Tension 2).**
  Delete `@memo(flat)` (0/611) and `@memo(hash)` (3/611, redundant with runtime
  auto-dispatch at `<= 2^24` / `2^26` for tri-state `?Bool`); bare `@memo` stays
  and explicit custom hash caches use plain `HashMap`. Delete `@multitest(count:)`
  (1/611; lost to `for _ in 0..tests:` + block-scoped `@memo` by +1 LOC / +23
  keystrokes on `noi22p4`); keep bare `@multitest`, `@multitest(until_eof)`, and
  bare `@multitest fn solve():`. Delete `@grader const` (1/642) and uninitialized
  file-scope grader domains (1/642); keep bodyless `@grader fn`, exported `@grader
  fn`, sibling-parameter array bounds, and stateful top-level `var` / `@grader
  struct`.
- **19.4 `List.absorb` and `for mut x in xs:` deleted (5.3, 7.19; Subtraction Audit #2 §§2.4 & 2.9, Syntax Bakeoff #2 Tension 3).**
  Both hit 0/642 canonical ports. `.absorb()` fails same-container aliasing
  (`cco22p5`, `noi22p1`), hides an O(`|other|`) vs O(`min`) complexity trap across
  containers, and rarely applies when small-to-large updates companion arrays;
  `.take()` + `swap(mut dst, mut src)` stays locked as the uniform 2-line idiom.
  `for mut` removes an entire second iterator borrow mode and lured illegal
  `[mut a, mut b]` arrays (`noip22p2`); indexed `for i in 0..xs.len: xs[i].f()`
  is uniform and pairs index with mutation directly.
- **19.5 Tropical Optional trims & `min=`-skips-`none` guardrail (1.3; Subtraction Audit #2 §2.3, Syntax Bakeoff #2 Tension 1, Blind Critic #4 §2).**
  Keep Tropical Optionals (`min`/`max`/`min=`/`max=` skip `none`; binary `+`/`-`
  and unary `-` on `?Int`/`?F64` absorb `none`; removing them costs +945
  keystrokes and +221 punctuation chars across 7/20 hard problems). Delete
  `sum(?T)` absorption (0/642; conflicted with `min` skipping `none`) and
  `?T +=`/`-=` lifting (0/642; silent no-ops hide uninitialized DP). Pin mixed
  `min(T, ?T) -> T` (3+ args via `min([a, b, c])`). Document and guard against the
  `etsr23r1p3` poison-failure pitfall: when `none` means impossible/failure inside
  a `min=` loop, check and break explicitly instead of bare `best min= cand`.
- **19.6 `Marks<D>` demoted from core & shrunk to 32-bit (`I32`) stamps (7.4, 7.19; Subtraction Audit #2 §2.6, Syntax Bakeoff #2 Tension 4).**
  0/642 ports, and 8-byte `[D]Int` backing caused an 80 MB vs 10 MB RAM cliff on
  `ucrpc21g`'s `10^7`-state table. Demote `Marks<D>` off the cheat sheet and core
  prelude; teach `[D]Bool` + `.fill(false)` / touched-list reset as primary. Keep
  `Marks<D>` opt-in under `std.marks` backed by `[D]0..=2_000_000_000` (lowering
  to 4-byte `int32_t` / `I32` storage in C++17, with wraparound reset).
- **19.7 Low-usage domain, range, grid, search, sequence & stdlib trims (2.3, 4.3, 7.19; Subtraction Audit #2 §§2.7–2.11, Batch 49 §4).**
  Delete `D.chunks(k)`, `D[..k]`, `r1 & r2`, `.mirror`, `.flipped_rows`/`.flipped_cols`/`.flipped`
  (keep `.transposed` and `Range.l`/`Range.r`), `bsearch_first_some`, `slide_until`,
  `rotate_right` (unified into signed Euclidean `rotate_left`), `prod(gen)`,
  `sliding_min`/`sliding_max`, `topo_prefix`, `topo::<V>` turbofish docs (handled
  by bidirectional inference), `Dsu.count` (use `.components`), and `floor_div`
  (redundant with Euclidean `/`; keep `ceil_div`). Revert `SparseTable.fold(r)`
  from `?T` back to `T` (asserting non-empty; fixes `ioi99p6`). Demote 11
  0–1-use specialized structures (`MonoDeque`, `Steps`, `TwoSat`, `Matrix`,
  `MinCostFlow`, `Dir8`, `palindromes`, `shortest`, `walk_back`, `mst`,
  `convex_hull`) from core `std` and the cheat sheet to `examples/local-lib/`.
- **19.8 Index-before-place-borrow evaluation & interactive I/O hardening (5.3, 6.6; Syntax Bakeoff #2 Tensions 3 & 5, Batch 47 §4, Batch 50 `NP1`).**
  Extend RHS-before-LHS so `a[idx].f = rhs`, `a[idx].f ??= rhs`, and compound
  assignments evaluate **all index expressions and `rhs` before borrowing the
  mutable place in `a`** — making `trie[step(o)].ends = true` and
  `trie[o].next[s] ??= trie.push_id(Node())` borrow-safe and reallocation-safe in
  C++17. In interactive programs, any use of `ask(...)` automatically flushes all
  ordinary `print(...)` calls, and `@interactor fn judge(sol: Interactor):`
  replaces overloaded `read_output()`/`print()` with explicit `sol.recv()` /
  `sol.send(...)` (`read()` reads the hidden judge test case).
- **19.9 All five Blind Critic #4 contest-pressure ambiguities pinned on the cheat sheet (4.3, 5.9, 6.8, 7.19; Blind Critic #4 §§3 & 7, Batches 47–49).**
  1. `Action(id, (new_f, old_f) => ..., (f, val, span) => ..., over: op)` argument
     order pinned (`compose` takes new tag first, old tag second; `LazySegTree<D, T>(act, a)`
     infers tag type `F`; `.all` is O(1) whole-domain fold; `.update(i, f)` does
     single-leaf push-modify-pull; `max_right`/`min_left` take/return `D.bounds`;
     empty/inverted ranges are valid no-ops).
  2. `first_ge`/`last_le` return types stated side by side: sequences return
     index (`?Int`/`?D`), `Set`/`SortedBag` return key (`?K`), `Map` returns pair
     (`?(K, V)`).
  3. `Map` entry semantics pinned: `m[k]` traps if missing, `m[k] = v` inserts or
     overwrites, `m[k] f= x` auto-inserts from op identity (`0` for `+=`/`-=`,
     `none` for `min=`/`max=`), `m[k] ??= init` lazily inserts + lends mutably,
     `m.remove(k) -> ?V` returns the removed value.
  4. `Subset<D>` constructors and all three iterations pinned (`Subset<D>.empty`,
     `.full`, `for mask in Subset<D>`, `for v in mask`, `for sub in mask.subsets`,
     plus uniform `x not in s`).
  5. Stateful `@grader` uses top-level `var` (e.g. `var state: ?State = none`) or
     `@grader struct` without uninitialized global domains; `@memo` explicitly
     permits `ask(...)` and deterministic extern `@grader fn` oracles (guaranteeing
     at most one call per distinct argument tuple) while still rejecting mutated
     captured local `var`s and `read()`/`print()`.
- **19.10 Batch 51 zero-keyword wins & contest-ergonomics polish (1.3, 3.10, 4.3, 7.6; Batch 51 `F1`/`F3`/`F5`, Blind Critic #4 §§3.4–3.7, Batch 48 §48.2, Batch 50 `NP6`).**
  Allow diverging RHS (`return ...`, `break`, `continue`) on `??` (`let u = o ??
  return (none, none)`); add `prev_permutation(mut a)` beside `next_permutation(mut a)`
  (fixing `ccc96s3` from 19 -> 8 lines); give `Comb<P>()` interior mutability so
  `let comb = Comb<MOD>()` stays immutable and freely capturable; add
  `List.insert(i, x)` and `List.remove(i) -> T`; surface `Rng(seed)`,
  `rand(lo..hi)`, `shuffle(mut xs)`, and `sort(mut a, by: desc)` on the cheat
  sheet; lock declaration-order `Ord` on derived `enum`s and the `else if` ->
  `elif` parser diagnostic.

## 20. Program structure: the `@!multitest` divider and top-level `return` (0.2.2, approved 2026-11-20)

Revises 6.3. A single-test program and a multitest program have the same shape: flat, unindented top-level
statements, run top to bottom. Neither needs a `fn solve()` wrapper.

- **20.1 `@!multitest` is a divider.** `@!` marks a file-level directive: it stands on a line of its own and
  is in force from that line down, where `@` always decorates the single item below it (the same split as
  Rust's `#![...]` and `#[...]`). `@!multitest` is the one directive, and appears once per file. Everything above it
  runs once, before the test count is read: sieves, `Comb` tables, anything shared. Everything below it runs
  once per test case. The count `t` is read automatically at the divider (`@!multitest(until_eof)` reads
  cases until the input ends instead).

  ```cp
  let sv = Sieve(1e6)                   // above: built once

  @!multitest
  let n: 2..=1e6 = read()               // below: once per test case
  print(sv.is_prime(n))
  ```

  Delete the divider and move the sieve down and what is left is the single-test program for the same problem. That
  symmetry is the point: turning a solution into a multitest one is adding a line and lifting the shared setup above it, never re-indenting.

- **20.2 Each test case is a fresh scope.** Every `let`, `var`, `domain` and `fn` below the divider is
  created anew for each case and gone at its end, so nothing leaks from one case to the next. A `@memo fn`
  below the divider belongs to its case: its cache starts empty every time. State that must outlive a case
  is declared above the divider, where it is visible to (and, for a `var`, writable from) every case.

  ```cp
  @!multitest
  let n: 1..=2e5 = read()
  domain V = 1..=n
  let a: [V]Int = read()

  @memo
  fn best(i: V) -> Int:                 // cache keyed by this case's V, emptied for the next case
      if let j = i.next:
          return max(a[i], a[i] + best(j))
      a[i]

  print(best(V.first))
  ```

- **20.3 Top-level `return`.** A `return` outside every `fn` takes no value and ends the current unit of
  work, from any depth of loops: the whole program in a single-test file (and above the divider), the
  current test case below the divider. Output printed so far is kept and flushed. This is the early exit
  that `fn solve()` wrappers existed to provide.

  ```cp
  let n: 0..=2e5 = read()               // single-test: `return` ends the program
  if n == 0:
      print("empty")
      return
  ```

  ```cp
  @!multitest
  let n: 0..=2e5 = read()               // multitest: `return` ends this case, the next one starts
  if n == 0:
      print("empty")
      return
  ```

  The two bodies are identical. `return x` at top level is an error (there is no caller to receive `x`).

- **20.4 `@spec` and `@checker` see the same inputs in both modes.** They read the top-level input `let`s
  by name. In a single-test file those are the program's inputs; below the divider they are the current
  test case's inputs, and the `@spec` / `@checker` runs once per case. The declaration is written the same
  way in both files.

  ```cp
  @!multitest
  let n: 1..=5 = read()
  print(n * n)

  @checker
  fn check():
      let x: Int = read_output()
      assert x == n * n                 // `n` is this case's `n`
  ```

- **20.5 What is not kept.** `@multitest fn solve():` is no longer part of the language as taught: the
  divider covers precomputation (the one reason the wrapper existed) and top-level `return` covers early
  exit. A second `@!multitest` in one file is an error, as is any other `@!name` and a
  `@!multitest` that is not at top level. A bare `@multitest` as a divider is the deprecated spelling of
  `@!multitest`. Problems whose case loop has another shape (a count
  per group, a sentinel) still write a plain `for _ in 0..tests:` or `read_until` loop.

- **20.6 Prototype status.** `proto/cpc` and `proto/cpi` implement 20.1 to 20.3 and 20.5's errors;
  `proto/tests/program/` holds the cases, run by both. The prototype still compiles the older
  `@multitest fn solve():` wrapper and the bare `@multitest` divider, so existing ports keep building until
  they are migrated; both should then become errors that point to `@!multitest`. `@spec` and
  `@checker` are not implemented in the prototype, so 20.4 is specified here and not yet tested.

## 21. Batch 7 calls: graph vertices, getters, randomness, the prelude budget and always-on traps (0.2.2, 2026-11-20)

These came out of the cold-writer benchmark and Friction Hunter waves 5 to 10, where the same few fights kept
recurring. Each one is implemented in the Rust compiler and covered by a program under
`crates/cpl-cli/tests/programs/`.

- **21.1 A tuple of domains is a domain for graph search.** `[V]T` where `V` stands for `(R, C)` is the same
  flat table as `[R, C]T`, so `bfs` and `dijkstra` take grid cells as vertices directly instead of a cell
  number packed by hand:

  ```cp
  use std.grid.{adj4}
  domain R = 0..3
  domain C = 0..3
  let wall: [R, C]Bool = [R, C] (r, c) => r.ord == 1 and c.ord < 2
  let d = bfs(p => [q for q in p.adj4 if not wall[q]], [(R(0), C(0))])
  print(d[(R(2), C(0))])
  ```

- **21.2 The vertex domain comes from the neighbour function.** When `peel`, `topo` or `scc` is given a
  lambda, the vertex domain is the domain the lambda indexes with its parameter, so `peel(v => adj[v])`
  needs no `::<V>`. A named `Graph` works the same way: `peel(g)` and `topo(g)` take the domain from `g`.

- **21.3 A getter reads the same with or without parentheses.** A `@get` method such as `Mod.inv` can be
  written `x.inv` or `x.inv()`. Writers coming from other languages reach for the parentheses first, and
  there is no second meaning they could have, so both spellings are accepted rather than one being an error.

- **21.4 Seeded randomness is a library type.** `std.rand` holds `Rng` (`Rng.new(seed)`, `.int(lo, hi)`,
  `.next()`), imported explicitly with `use std.rand.{Rng}`. The prelude keeps only `rand(lo, hi)` and
  `shuffle(mut xs)`, which cover generators and most randomized solutions.

- **21.5 The prelude has a budget of 80 names.** `stdlib/prelude.cp` imports exactly 80 names. Batch 7
  brought in `Dir8`, `step`, `find_all`, `windows`, `zip` and `digit`, which the benchmarks showed people
  write without a `use`, and moved `Neighbours`, `Hashable`, `Prefix1` and `HashVal` out, since they are
  named only by library authors. A name joins the prelude only when another one leaves.

- **21.6 Arithmetic traps run in every build, not only locally.** Division or modulo by zero, a shift
  outside `0..64`, a negative power and `U64` division by zero stop the program with exit status 3 in the
  submitted release binary too. A modulus above `3_037_000_499` multiplies in 128 bits, so
  `Mod<4_000_000_007>` no longer overflows. The release binary is what the judge runs, and a silent wrong
  answer there is worse than a crash; the perf tail profile (`research/perf-tail-profile-2.md`) measured
  no cost from the always-on division checks. Unproven `Int` overflow stays a local-only trap (7.x).

- **21.7 Two silent-empty cases are errors.** A domain named inside a tuple `read()`, such as
  `let (s, k): ([domain Pos]Char, Int) = read()`, used to read no values; it is now a compile error. A
  `match` on an enum without fields needs an arm for every variant.

## 22. Core syntax to 0.3 (2026-11-21)

- **22.1 `where` is the 0.3 guard keyword alongside `if` for loops and `let` conditions.**
  `for x in s where c:`, `[e for x in s where c]`, `if let x = e where c:`, `while let x = e where c:`,
  and multi-binding `if let a = x, b = y where c:` / `while let a = x, b = y where c:` all parse into
  the existing `Cond::Let` / `Cond::Guard` / filtered-`For` AST nodes. `for x in s if c:` remains
  accepted so the existing 235 corpus ports stay green without churn.
  *Reversal:* remove the `at_ident("where")` branches in `crates/cpl-syntax/src/parser/{stmt,expr}.rs`
  (or deprecate `for ... if` once ports migrate).

- **22.2 Product array syntax `[(A, B)]T` and table syntax `[(A, B)] (a, b) => ...` desugar to `[A, B]T`.**
  Both `[(A, B)]T` (0.3) and `[A, B]T` (0.2.2) produce the same flat multi-dimensional `TypeKind::Array`,
  and both `g[(a, b)]` and `g[a, b]` index it, matching 21.1 without breaking existing single-bracket grid
  diagnostics or ports. Works for domain products (`[(R, C)]Int`) and counted dimensions (`[(n, m)]Int`).
  *Reversal:* remove the tuple-flattening branch in `crates/cpl-syntax/src/parser/{types,expr}.rs`.

- **22.3 Optional arithmetic and bitwise lifting covers `+ - * / % ** & | ^ << >>` and unary `-`, `~` (`?T`).**
  Previously only `+`, `-`, and unary `-` lifted over `?Int`/`?F64`. Per `23-optionals.md`, `+ - * /` lift
  over `?Int`/`?F64`, `% ** & | ^ << >>` and unary `~` lift over `?Int`, and ordering (`< <= > >=`) still
  does not lift. Implemented via the existing `tropical` lowering in `cpl-ir` and matching rules in `cpl-check`.
  *Reversal:* narrow the `BinOp`/`UnOp` match arms in `crates/cpl-check/src/check/expr.rs` and `crates/cpl-ir/src/lower/expr.rs`.

- **22.4 `pairs()` and `with_prev()` are ordinary `yields` library functions in `stdlib/seq.cp`.**
  Per `22-sequences.md`, `xs.pairs()` yields adjacent `(prev, x)` and `xs.with_prev()` yields `(?T, T)`
  starting with `(none, first)`, with no special loop syntax or compiler privilege. Written as `yields`
  generators over `List<T>` (which also works on `[D]T` arrays via existing coercion) so they fuse into
  loops in C++ and keep `stdlib/prelude.cp` at its 80-name cap (imported via `use std.seq.{pairs, with_prev}`).
  *Reversal:* remove `pairs` and `with_prev` from `stdlib/seq.cp` (or add them to `stdlib/prelude.cp` if prelude slots open).

- **22.5 One-line colon structs (`struct Edge: to: Int, w: Int`) and dotted range-bound types (`Self.min..=Self.max`).**
  `struct_decl` now accepts `struct Name: f1: T1, f2: T2` (including shared-type `x, y: Int` and defaults),
  producing the same `StructDecl` AST node as indented/paren structs. `range_follows` in `cpl-syntax` now
  skips `.ident` member chains so `Self.min..=Self.max` and `D.first..=D.last` parse as `TypeKind::Range`.
  *Reversal:* remove the one-line colon branch in `crates/cpl-syntax/src/parser/item.rs` and restore 1-token
  `range_follows` in `crates/cpl-syntax/src/parser/types.rs`.

- **22.6 Range-checked `read::<lo..=hi>()` in expression position and homogeneous multi-binding (`var a, b, c: Z = (0, 0, 0)`, `let n, m: 1..=20 = read()`).**
  `read::<lo..=hi>()` in `cpl-ir` now lowers a `TypeKind::Range` type argument to `Spec::Int { lo, hi, text }`,
  enforcing range bounds identically to `read(lo..=hi)` and `let x: 1..=3 = read()` while keeping `::<>`
  turbofish for generic functions. Homogeneous multi-binding (`var a, b, c: T = ...` / `let n, m: T = ...`)
  desugars in `let_stmt` to a tuple pattern with repeated tuple type `(T, T, ...)`, so `var ones, one_twos, ans: Z = (0, 0, 0)`
  and `let n, m: 1..=20 = read()` both work while scalar broadcast (`var a, b: Int = 0`) remains rejected as a
  tuple/scalar mismatch.
  *Reversal:* remove the `TypeKind::Range` branch in `crates/cpl-ir/src/lower/call.rs` and remove `shared_count`
  in `crates/cpl-syntax/src/parser/stmt.rs`.

- **22.7 Parenthesized supertraits (`trait Ord(Eq)`), `&mut T` parameter syntax, opaque structs (`struct List<T>`), and multi-trait `impl A + B for T`.**
  `trait_decl` accepts `(Super1, Super2)` alongside `: Super1 + Super2`; `param` desugars `x: &mut T` to
  `Param { mutable: true, ty: Some(T) }` with no new AST node; `struct_decl` allows bodyless opaque declarations
  (`struct List<T>`, `struct Str`); `impl_decl` desugars `impl A + B for T:` into one `StmtKind::Impl` per trait.
  *Reversal:* revert the four branches in `crates/cpl-syntax/src/parser/item.rs`.

- **22.8 One-sided range types (`0..`, `2..`, `..=8`, `..n`) and associated types in `trait`/`impl` scope.**
  `TypeKind::Range` stores `lo: Option<Box<Expr>>` and `hi: Option<Box<Expr>>`, matching the optional
  bounds already used by `cpl-check::Ty::Range` and `cpl-ir::Spec::Int`. Both lower-bounded (`0..`, `2..`)
  and upper-bounded (`..n`, `..=8`) ranges now parse, type-check, and emit runtime range checks on `read()`
  and `read::<2..>()`. `StmtKind::Trait` and `StmtKind::Impl` also bring associated `type Name` declarations
  into their body generic scope so `trait Unwrap: type Out; fn unwrap(self) -> Out` checks cleanly.
  *Reversal:* make `lo`/`hi` non-optional again in `TypeKind::Range` and remove the `TypeAlias` loops in
  `crates/cpl-check/src/check/stmt.rs`.

- **22.9 Struct type-parameter defaults (`struct Pair<A, B = Int>`, `struct Graph<V: Index, W = ()>`).**
  `StructDecl` stores `generic_defaults: Vec<Option<Type>>` and `Def::Struct` stores resolved `defaults: Vec<Option<Ty>>`.
  Omitting trailing type arguments fills them from `defaults` (substituting earlier parameters) in both `cpl-check::def_type`
  and `cpl-ir::instantiate_struct` before memoization, so `Pair<Int>` and `Pair<Int, Int>` are identical types and share
  one monomorphized struct. Removed the `planned::TYPE_PARAM_DEFAULT` stub.
  *Reversal:* restore `planned::TYPE_PARAM_DEFAULT` in `crates/cpl-syntax/src/planned.rs` and remove `generic_defaults`/`defaults`.

- **22.10 Dream ergonomics batch 1: additive const type args (`Mod<1e9 + 7>`), multi-clause `for`, `guard let`, and top-level `return <unit>`.**
  Auditing all 29 dream files (`proto/ports/*/dream/*.cp`) showed these four forms recurring across 13 dreams:
  1) `type_args` parses additive value expressions (`1e9 + 7`, `n - 1`) as `TypeKind::Value` when the token after an int/ident/minus is `,`, `>`, `>>`, `+`, `-`, `*`, or `**`.
  2) `for_stmt` parses comma-separated `pat in iter [where cond]` clauses and folds them right-to-left into nested `StmtKind::For`.
  3) `guard let ... else:` delegates directly to `let_stmt()`.
  4) `StmtKind::Return(Some(e))` is accepted at top level (and in `Unit`-returning fns) when `e` has type `Unit` (e.g. `return print(ans)`), lowering to `Stmt::Expr(e); Stmt::Return(None)` while still rejecting non-unit top-level return values.
  *Reversal:* revert the four parser/check/IR hunks in `types.rs`, `stmt.rs`, `check/stmt.rs`, and `lower/stmt.rs`.

- **22.11 Dream ergonomics batch 2: typed streaming reads `read<[n]T>()`, `x with field: v` record update, scalar-fill multi-binding `var a, b: T = 0`, and typed lambda params.**
  1) `names_a_type` in `cpl-syntax` now accepts `TypeKind::Range`, `index_ty` treats arithmetic index expressions (`[V.count - 1]`) as counted arrays `0..expr`, `cpl-fmt` recognizes lowercase generic calls `f<...>(...)` via `closes_before_lparen`, and `cpl-ir` lowers zero-arg `read<T>()` / `read::<T>()` via `self.resolve(ty, true)` + `self.read(spec, span)` — one general path for ranges, counted arrays, grids, tuples, and structs.
  2) `ExprKind::With { base, updates }` is now lowered in `cpl-ir` for structs and records via a `Do` block binding `base` once and reconstructing the struct/record tuple with updated fields, removing the E0501 `with` stub.
  3) `var ones, open, ans: Z = 0` and `var ways, next: [1..=m]Z = 0` desugar to N separate `StmtKind::Let` declarations when the initializer is not a tuple or `read()`, while keeping tuple/read destructuring intact.
  4) Lambda parameter lists accept optional `: Type` annotations (`(d: Int, l: Low) => ...`).
  This brings `proto/ports/ecnerwala/dream/cf2069c.cp` (exemplar #1 in `design/EXEMPLARS-PORTS.md`) to 0 errors in `cpl check`, and `proto/ports/ecnerwala/cf2069c.cp` now matches its dream exemplar.
  *Reversal:* revert the parser/IR/fmt hunks in `expr.rs`, `types.rs`, `stmt.rs`, `call.rs`, `lower/expr.rs`, and `spacing.rs`.

## 23. Validation scoreboard and friction summary (`scripts/scoreboard.py`, 2026-11-21)

- **23.1 One unified scoreboard over auto-discovered ports.** `scripts/scoreboard.py` discovers every
  `*.cp` port with a sibling `*_orig.cpp` under `proto/ports/ecnerwala/` (and any future Rust-era port
  directory under `proto/ports/` that follows the same layout, excluding retired `scout_batch_*`,
  `blind_critic_*`, `hillclimb_*`, `sim3`, `before` directories). Concurrent port waves are picked up
  automatically with no manifest to edit. *How to reverse:* pass `--dirs` or change `discover_ports` in
  `scripts/scoreboard.py`.

- **23.2 Contestant-written token counting.** Both `.cp` and `_orig.cpp` are tokenized by language-aware
  lexers that strip comments and whitespace; `_orig.cpp` also reports a library-stripped token count using
  `bench/corpus_registry.py`'s `auto_strip` (or a sibling `.strip` file when present) so template headers
  and hand-rolled standard structures do not inflate the C++ baseline. Both raw and stripped ratios are
  recorded. *How to reverse:* swap `tokenize_cp` / `tokenize_cpp` or point to `cpl` lexer output.

- **23.3 Adaptive max-size input probing for runtime and peak RSS.** When a port has a judge-limit
  generator in `bench/corpus_generators.py` or `bench/rust_max_n_audit.py`, the scoreboard uses it.
  Otherwise it probes `gen/NAME.py 1 <SIZE>` upward (`8 -> 40 -> 200 -> 1000 -> 5000 -> 25000`), stopping
  when the original exceeds the probe budget (~0.25 s) or the input saturates/caps, falling back to the
  largest `samples/NAME.*.in`. This gives a real max-size measurement for every newly pushed port without
  hanging on super-linear problems. *How to reverse:* change `MAX_SIZE_STEPS` or add explicit specs to
  `bench/corpus_generators.py`.

- **23.4 Baseline diff and `--fast` mode.** `research/scoreboard.json` is the committed baseline.
  `--fast` runs `cpl check`, `cpl fmt --check`, `cpl lint`, `cpl build` (emitted bytes <= 64 KB), token
  counts, `friction_lint`, and the `research/frictions/*.md` summary in a few seconds so `scripts/check.sh --fast`
  can gate every push. `--check` fails on gate regressions (any passing or new port failing), emitted
  files over 64 KB or growing > 5% and > 1,024 B, median token ratio regressions (> +0.02), `wa_trap`
  regressions, open `blocker` friction increases, and (in full mode) p50/p95 runtime or RSS regressions
  (> 1.10x baseline). *How to reverse:* adjust threshold constants at the top of `scripts/scoreboard.py`.

- **23.5 Custom checker hook `checkers/NAME.py INPUT OUTPUT ANSWER` for constructive ports.**
  Matches the `cpl stress` (`crates/cpl-stress/src/judge.rs`) and testlib argument order: `INPUT` test
  file, `OUTPUT` candidate stdout, `ANSWER` original C++ stdout; exit 0 accepts, non-zero rejects with
  reason on stderr. Both `proto/ports/ecnerwala/gate.sh` and `scripts/scoreboard.py` sanity-check
  `orig` against itself before judging the C++ build and interpreter, and `scripts/scoreboard.py` tracks
  `checker_ports` in totals. *How to reverse:* remove the `CHECKER` branch in `gate.sh` and
  `run_checker`/`judge_output` in `scripts/scoreboard.py`.

- **23.6 `gen/NAME.py --max`, per-port >10% runtime ratchet, dream-gap metrics, and `PORTS.md` runtime column.**
  `generate_max_input` prefers `gen/NAME.py --max` (when `--max` appears in the generator) over explicit
  specs and adaptive probing so judge-limit worst-case inputs live beside each port. Full-mode `--check`
  ratchets any port with `max(cpp_ms, orig_ms) >= 10 ms` whose `time_ratio` regresses by > 10% against
  `research/scoreboard.json`, reports `p50 <= 1.1x` / `max <= 1.5x` goal status, and prints the worst 10
  runtime ratios. `measure_tokens` and `compute_totals` track `dream/NAME.cp` (`dream_tokens`, `dream_ratio`,
  `dream_gap_tokens`, aggregate median/p90/max). `scripts/ports-page.py` reads cached `time_ratio` values
  from `research/scoreboard.json` into the `Runtime` column of `research/PORTS.md` without re-running.
  *How to reverse:* adjust `TIME_P50_GOAL`, `TIME_MAX_GOAL`, `PERF_FLOOR_MS`, or `PERF_REGRESSION_RATIO` in
  `scripts/scoreboard.py` or remove the `Runtime` column in `scripts/ports-page.py`.
- **23.7 Interpreter-vs-C++ ratio ratchet, strict `cp/dream` ratchet, and `research/reviews/` taste-score integration.**
  In full mode, `scripts/scoreboard.py` times `cpl run` and the compiled `-O2` C++ binary on the *same* input
  (using `max_inp` when `cpp_ms <= 250 ms`, else the largest diff case), reporting `ratio_p50`/`ratio_max`
  against `INTERP_RATIO_BAR = 10.0` and `INTERP_RATIO_NORTH_STAR = 5.0` (with a `20 ms` floor to avoid startup
  noise) and failing `--full --check` on `> 10%` regressions in `p50` or any port `>= 20 ms`. In both `--fast`
  and `--full`, any increase in a port's `cp/dream` token ratio (`dream_ratio`) fails `--check`.
  `parse_review_score` reads 1–5 scores from `research/reviews/<name>.md` into `totals.taste` and the `Taste`
  column of `research/PORTS.md`.
  *How to reverse:* adjust `INTERP_RATIO_BAR`, `INTERP_RATIO_NORTH_STAR`, `INTERP_FLOOR_MS`, or the
  `dream_ratio` check in `scripts/scoreboard.py`.
- **23.8 Friction metadata gate + 10-area stdlib routing, P0 bug-replay & revision goal tests, and worst-20 dream-gap leaderboard.**
  Normalized all `research/frictions/*.md` reports to canonical `Severity` (`blocker|high|medium|low`) and
  `Goal it hurts` values (0 schema warnings), gated warning growth in `scripts/scoreboard.py --check`,
  and added `route_friction_area` mapping every friction to one of the 10 stdlib areas (`graphs`, `trees`,
  `ranges`, `strings`, `math`, `flows`, `containers`, `geometry`, `dp`, `seq`) or `core` (printed as per-area
  owner queues in `--frictions` and as an `Areas` column on `research/PORTS.md`). Added two live goal-test
  suites under `bench/goal_tests/`: (a) `bug_replay/` (12 P0 contest bug classes classified empirically via
  `cpl check` + `cpl run` as `prevented`, `caught_on_run`, or `left_to_diagnose`, ratcheted at `>= 90%` handled
  and no per-case downgrade); (b) `revision/` (2 unseen problems + 1 changed requirement each, verified via
  `cpl run`, ratcheting `lines_touched` against baseline). Added a worst-20 `cp/dream` token-ratio leaderboard
  to `scripts/ports-page.py` -> `research/PORTS.md` so the polish lane works worst-first.
  *How to reverse:* adjust `_AREA_KEYWORDS`, `BUG_REPLAY_BAR_PCT`, or the suites in `bench/goal_tests/`.
- **23.9 63-case P0 bug-replay suite (per-class rates), 10-problem revision suite, and ratcheted spelling-drift metric.**
  Expanded `bench/goal_tests/bug_replay/` from 12 to 63 cases (>= 5 per P0 class across all 11 AGENTS.md
  bug classes), evaluating `cpl check` + `cpl lint` (`prevented`) and `cpl run` (`caught_on_run` vs
  `left_to_diagnose`) in parallel with per-class rates (`59/63 = 93.7%`, 4 honest `left_to_diagnose` gaps).
  Expanded `bench/goal_tests/revision/` from 2 to 10 unseen problems + 1 requirement change each (`10/10 ok`,
  `95` lines touched). Added `measure_spelling_drift` over `proto/ports/**/*.cp` (tracking both all files
  and non-retired active+dream files) for constructs with multiple spellings (`multitest`, `sum`, `first`,
  `sort`), ratcheted strictly downward against `research/scoreboard.json` so codemods visibly drive drift
  to 0. Canonical forms live in `SPELLING_DRIFT_SPECS` (`@!multitest` per §20; method forms `xs.sum()`,
  `xs.first(...)`, `xs.sort(...)` per §49).
  *How to reverse:* flip any construct's `"canonical"` value in `SPELLING_DRIFT_SPECS` (1-line edit) or
  adjust cases under `bench/goal_tests/`.
- **23.10 Active spelling-drift zero gate, dream-score ratchet, and `check.sh --fast` <= 180s wall-time budget.**
  Aligned `measure_spelling_drift` with §53.1 by exempting canonical free-form generator calls
  (`sum(f(x) for x in xs)`, `first(i for i in ... if ...)`), added `scripts/fix-spelling-drift.py` to
  rewrite non-canonical multitest dividers (`@multitest`, `--- multitest ---`, `--- with multitest() ---`
  -> `@!multitest`) across active and dream files (driving active/dream drift to 0), and gated
  `active_non_canonical == 0` in `--fast --check`. Added `totals.dream_scores` (parsing `Dream: N/5`
  from `research/reviews/*.md`) with a baseline ratchet failing on median drop or under-4 growth.
  Added `opt-level = 1` in `Cargo.toml` `[profile.dev.package]` for `cpl-syntax`, `cpl-check`, `cpl-ir`,
  and `cpl-codegen` (matching `cpl-interp`) and overlapped `recovery_scoreboard` with the post-nextest
  checks in `scripts/check.sh`, cutting `scripts/check.sh --fast` wall time from 249s to 84s, recorded
  in `target/gate-fast-time.json` / `totals.gate_fast` and gated at `<= 180s`.
  *How to reverse:* remove the `[profile.dev.package]` `opt-level = 1` entries in `Cargo.toml`, adjust
  `GATE_FAST_BUDGET_S`, or relax the active-drift / dream-score checks in `diff_against_baseline`.

- **23.11 Attributable gate ratchets, merge-safe `.ports.jsonl` baseline (`--update PORT...`), and expanded ratchets.**
  `diff_against_baseline` now takes a git `scope` (`detect_change_scope`) so a commit fails only for
  regressions in ports/files it changed, or in ports whose emitted C++/perf/dreams changed because
  of a `crates/` or `stdlib/` diff (naming the port and cause); unrelated drift is reported as a
  warning instead of blocking other lanes. Replaced the global `new_under_4_max` quota in
  `scripts/review-check.py` with per-change attribution: new or changed ports require a review file
  (`Score: N/5` or `Score: pending`), while scores `< 4` are tracked and non-blocking.
  Split `research/scoreboard.json` into compact summary JSON plus sorted 1-line-per-port
  `research/scoreboard.ports.jsonl` and added `--update PORT...` to update only named port lines
  without touching summary JSON, so concurrent lanes rebase cleanly.
  Added strict ratchets for interpreter ratio `p50` and `>10x` count (bar `<= 10x`, north star
  `<= 5x`), dream-compile count (never fewer), and construct-split spelling drift (`getter_parens`,
  `where_filter`, `dot_pow`, `multitest`, `turbofish`, plus `sum`/`first`/`sort`).
  *How to reverse:* pass `scope=None` to `diff_against_baseline`, restore `new_under_4_max` in
  `scripts/review-check.py`, or inline `ports` back into `research/scoreboard.json`.

- **23.12 Input-hash caching + parallel post-nextest tracks in `scripts/check.sh --fast`, plus `.githooks/pre-push`.**
  Added `scripts/check_cache.py` (deterministic SHA-256 over sorted input paths, stored under
  `target/check-cache/<step>.sha256`, disabled via `--no-cache` or `CHECK_NO_CACHE=1` or `--full`)
  to cache unchanged `clippy`, `cpl_build`, `nextest_doc`, `stdlib_test`, `recovery`,
  `scoreboard_fast`, `site_bundle`, and `docs_site` steps without dropping coverage, and ran
  `validation scoreboard` in parallel with `site/bundle.py` -> `docs gate + site tests` and
  `recovery_scoreboard`. Cold `--fast` wall time dropped from 188s to **163s** (`<= 180s` budget),
  and unchanged/rebased reruns complete in **~1.3s**. Added `.githooks/pre-push` to reject merge
  conflict markers, invalid changed JSON/JSONL, and broken `cargo check --workspace --all-targets`
  after rebases (which skip `pre-commit`).
  *How to reverse:* pass `--no-cache` / remove `scripts/check_cache.py` calls in `scripts/check.sh`,
  or delete `.githooks/pre-push`.

## 24. Documentation gate in `scripts/check.sh --fast` (2026-11-21)

- **24.1 Fast docs verification runs on every commit.** `site/check_docs.py` runs inside
  `scripts/check.sh --fast` right after `site/bundle.py`. It checks every `cp` fence in
  `site/content/*.md` and `docs/one-page-cheat-sheet.md` with one batched `cpl check`, executes every
  `site/examples/*.cp` fixture and write-code course reference through `cpl run`, enforces the 250-line
  cheat-sheet budget, and validates every local HTML link and heading fragment in `site/_build/`.
  *Reason:* `site/test_site.py` compiles every snippet with `g++`, taking ~55 s, so it sat outside the
  fast gate and docs drifted (the cheat sheet exceeded 250 lines). Running `cpl check` + `cpl run` keeps
  the fast gate under a few seconds while catching broken examples immediately; `site/test_site.py`
  remains the full C++17 verification suite.
  *Reversible:* remove the `site/check_docs.py` step from `scripts/check.sh` or replace with a faster
  Rust subcommand if the docs corpus grows past a few seconds.

- **24.2 Generated Error & Lint Index (`site/errors.py`) with 100% compiler-code coverage.**
  `site/build.py` generates `errors.html` plus one flat `error-<CODE>.html` page per diagnostic,
  warning, runtime trap, and lint code (currently 61 codes: 52 from `cpl-diag` + 9 from `cpl-lint`),
  indexed in `search.json` and packed into `dist/cp-lang.html`. `site/check_docs.py` parses
  `crates/cpl-diag/src/lib.rs` and `crates/cpl-lint/src/lib.rs` directly, failing the fast gate if any
  Rust code is missing or any trigger/fix pair stops reproducing its code or compiling cleanly.
  *Reason:* flat filenames integrate automatically with `site/bundle.py` (`*.html` glob) and relative
  asset paths, adding ~40 KiB gzipped; diffing against the Rust sources prevents the error index from
  ever drifting behind new diagnostics or lints.
  *Reversible:* move `site/errors.py` entries into Rust docstrings or a `cpl explain --dump` subcommand
  once `cpl explain` exists, keeping the same `error-<CODE>.html` URLs.

- **24.3 Cookbook generated directly from validated ports (`site/cookbook.py`).**
  `site/build.py` renders `cookbook.html` by reading live `.cp` sources from `proto/ports/ecnerwala/`
  across 6 competitive programming categories (22 recipes covering trees/graphs, segment/Fenwick trees,
  DP, math/combinatorics, binary search/greedy/interactive, strings/geometry). `site/check_docs.py`
  verifies every referenced port exists, passes `cpl check`, and matches its sample cases via `cpl run`.
  *Reason:* reading directly from `proto/ports/ecnerwala/` guarantees zero code duplication and keeps
  cookbook recipes automatically tested under the corpus gate and fast docs gate.
  *Reversible:* delete `site/cookbook.py` and its hook in `site/build.py` / `site/check_docs.py`, or
  expand into per-category pages if the recipe count outgrows one page.

- **24.4 Stdlib module coverage enforced in `site/check_docs.py`.**
  `site/check_docs.py` scans `stdlib/*.cp` (excluding `prelude`) and requires every module to be
  documented as `std.<module>` in `site/content/stdlib.md` (currently 24/24 modules).
  *Reason:* previously 12 of 24 non-prelude stdlib modules were absent from `stdlib.md`; checking
  against `stdlib/*.cp` prevents new modules from landing undocumented.
  *Reversible:* remove `check_stdlib_module_coverage()` from `site/check_docs.py` or replace with
  generated `cpl doc stdlib` HTML once docstrings cover every stdlib item.

- **24.5 Writable-from-docs audit harness (`scripts/writable-from-docs.py`).**
  Added `scripts/writable-from-docs.py` (with offline unit tests in `scripts/test_writable_from_docs.py`)
  to evaluate the `AGENTS.md` "Writable from the docs" rule: sends a fresh model ONLY the 8-file
  docs bundle plus an unseen statement from `corpus/statements/` (filtering out already-ported IDs),
  then checks `cpl check`, `cpl build`, and `cpl run` on every sample case. Initial 10-problem audit
  (`abc380b`–`abc387c`, recorded in `research/writable-from-docs.md`) improved compile pass from 7/10
  to 8/10 after fixing 5 docs gaps (`"{n}"` vs `.to_str()`, `let k: Int = read()` vs `read(Int)`,
  free `abs(x)`, 1-based array bounds, single-`for` comprehensions, `Deque.push`).
  *Reason:* turns the writable-from-docs principle into a repeatable, quantitative measurement that
  surfaces concrete docs gaps on unseen contest problems.
  *Reversible:* delete `scripts/writable-from-docs.py`, `scripts/test_writable_from_docs.py`, and
  `research/writable-from-docs.md`.

- **24.6 Ports gallery page generated from `research/PORTS.md` and `dream/*.cp` (`site/ports_gallery.py`).**
  `site/build.py` renders `ports.html` directly from `research/PORTS.md` plus live `.cp`,
  `proto/ports/<group>/dream/<name>.cp`, and accepted C++ originals, showing up to 16 featured
  side-by-side cards (prioritizing all dream-bearing ports, with client-side All / With dream /
  search filtering) plus a 60-row index table. `site/check_docs.py` verifies `research/PORTS.md`
  parsing, file existence, and rendered CP / dream / C++ panes.
  *Reason:* lets cold readers compare real validated CP Lang solutions, their unconstrained dream
  form, and accepted C++ side by side while keeping `dist/cp-lang.html` under the 2.6 MB cap
  (1,988 KiB).
  *Reversible:* delete `site/ports_gallery.py` and its hooks in `site/build.py`, `site/check_docs.py`,
  and `site/assets/style.css`.

- **24.7 Playground shareable links, sample runner, and verbatim CLI error parity.**
  Added URL-hash shareable links (`#share=<base64url>` encoding `{source, stdin, samples}` with
  no server dependency, working online and offline), a multi-case sample runner pre-populated with
  verified sample cases for all 23 playground presets, and verbatim CLI `result.stderr` rendering
  in `[data-cli-stderr]` alongside clickable jump-to-line diagnostics. Fixed `Playground::check` in
  `crates/cpl-wasm/src/lib.rs` to return `exitCode = 0` on warning-only diagnostics (matching
  `cpl check` on the CLI). Added Rust parity test `error_and_trap_stderr_matches_cli_verbatim` in
  `crates/cpl-wasm/tests/parity.rs` and end-to-end Node/WASM + share-link + preset-sample parity
  tests in `site/test_playground_parity.py`, wired into `site/check_docs.py`.
  *Reason:* keeps browser playground errors byte-for-byte identical to the CLI and lets readers
  share and sample-test solutions directly in the browser without any backend.
  *Reversible:* revert the UI controls in `site/compiler.py` / `site/assets/playground.js` and
  remove `site/test_playground_parity.py`.

- **24.8 Diagnostics registry (`crates/cpl-diag/src/registry.rs`) + `cpl explain` as single source of truth for error-index pages.**
  Moved every diagnostic code's title, summary, explanation, trigger, and fix into
  `crates/cpl-diag/src/registry.rs` (`REGISTRY: &[CodeInfo]`) and `crates/cpl-lint/src/lib.rs`
  (`LINTS`), added `cpl explain <CODE>` and `cpl explain --json` in `crates/cpl-cli`, and
  refactored `site/errors.py` to generate all 69 error/lint pages directly from `cpl explain --json`.
  *Reason:* eliminates the 700-line Python catalog duplication so site error pages and CLI
  `cpl explain` always share one verified registry (and immediately picked up the 6 new taste lints
  `L0501`–`L0506`).
  *Reversible:* restore the static `ERROR_CATALOG` list in `site/errors.py` and remove
  `crates/cpl-diag/src/registry.rs` and `explain_code` in `crates/cpl-cli/src/lib.rs`.

- **24.9 55-problem mixed-difficulty writable-from-docs benchmark, ratchet, and weekly workflow.**
  Pinned 55 unseen mixed-difficulty statements (`research/writable-from-docs-set.json`: 35 CF Div2
  B–E, 12 ABC D–E, 8 ARC A–B, verified unported), upgraded `scripts/writable-from-docs.py` to
  default to `gpt-5.4` with parallel workers and ratchet JSON (`research/writable-from-docs.json`),
  wired offline ratchet enforcement into `scripts/scoreboard.py --fast --check`, and added weekly
  CI workflow `.github/workflows/writable-from-docs.yml`. Set the ratchet noise margin to 5%
  (`max(prev, observed - 0.05)`) because `gpt-5.4` does not accept `temperature=0` (~±2 problems
  variance on `n = 55`).
  *Reason:* turns writable-from-docs from a one-off small-set audit into an ongoing regression
  gate over realistic contest difficulty.
  *Reversible:* delete `.github/workflows/writable-from-docs.yml`, `research/writable-from-docs.json`,
  `research/writable-from-docs-set.json`, and the `writable_from_docs` block in `scripts/scoreboard.py`.

- **24.10 Gold-standard `From C++` guide and `Idioms` page (`site/content/cpp-guide.md`, `site/content/idioms.md`).**
  Upgraded `site/content/cpp-guide.md` with an editorial-first side-by-side C++17 vs CP Lang
  comparison (`cf2069c`), named-invariant structs, and zero-sentinel optionals, and added
  `site/content/idioms.md` covering domains, reductions, optionals, grids, sorting/heaps/maps,
  binary search, and stdlib structures — every `cp` fence verified by `site/check_docs.py`.
  *Reason:* gives C++ contestants and cold readers exemplar-grade patterns matching
  `design/EXEMPLARS-PORTS.md` while keeping every example mechanically checked against the compiler.
  *Reversible:* remove the `idioms` entry in `site/build.py`, delete `site/content/idioms.md`, and
  revert `site/content/cpp-guide.md`.

- **24.11 Deterministic 55-problem writable-from-docs v2 set + float tolerance (`1e-6`).**
  Audited the v1 55-problem manifest and replaced 15 un-evaluable problems (2 interactive, 5 CF
  redirect duplicates, 8 multi-answer constructive problems where valid answers failed exact string
  match) with 15 clean unique deterministic-output unseen problems, and added token-wise `1e-6`
  relative/absolute float tolerance to `scripts/writable-from-docs.py::outputs_match`.
  *Reason:* pass rate must reflect real correctness, not judge-harness false negatives on valid
  constructive outputs, redirected duplicates, or `3e-16` float rounding.
  *Reversible:* revert `research/writable-from-docs-set.json` and `outputs_match` in
  `scripts/writable-from-docs.py`.

- **24.12 Generated `Reading exemplars` gallery (`site/exemplars.py`) and `Try it` buttons on every docs `cp` fence.**
  Added `site/exemplars.py` generating `exemplars.html` directly from `proto/ports/**/dream/*.cp`
  paired with validated ports plus `design/sketches/ioi2025/*.cp`, and wired `Try it` links
  (`playground.html#share=<base64url>`) into `site/build.py::code_block_html` for every rendered
  `cp` code block, both gated in `site/check_docs.py`.
  *Reason:* lets cold readers compare dream vs port without hand-maintained drift and run, trace,
  or share any docs snippet in one click (online or in the offline `dist/cp-lang.html` bundle).
  *Reversible:* remove `exemplars` from `site/build.py`, delete `site/exemplars.py`, and remove
  `try_it` from `code_block_html`.

- **24.13 Generated Stdlib API reference (`site/stdlib_api.py`) + zero-undocumented-`pub fn` gate.**
  Wired `site/stdlib_api.py` onto the existing `cpl doc --format json stdlib` output (avoiding
  duplicating the dev-tools doc extractor) to generate `stdlib-api.html`, documented all 55
  remaining undocumented `pub fn` / `pub @get fn` declarations across `stdlib/*.cp` with O(...)
  costs, and added `check_stdlib_api_docs()` in `site/check_docs.py` failing on any undocumented
  public function.
  *Reason:* keeps the stdlib API reference generated from one source of truth (`///` comments) and
  prevents undocumented public stdlib functions from landing.
  *Reversible:* remove `stdlib-api` from `site/build.py`, delete `site/stdlib_api.py`, and remove
  `check_stdlib_api_docs()` from `site/check_docs.py`.

## 25. Stress testing and shrinking (0.2.2, 2026-11-21)

- **25.1 `Perm<V>` and `Tree<V>` are stdlib types.** `std.seq.Perm<V> = [V]V` and
  `std.graph.Tree<V> = [0..V.len - 1](u: V, v: V)`. Before, only `cpl stress` knew the names: the
  compiler rejected `let t: Tree<V> = read()` with E0204, so a port had to declare a local alias
  (and `proto/ports/nccc10s2.cp`, which uses `Perm<L>` bare, does not compile). As plain generic
  aliases they read, print and index the same in `cpl run` and the C++, and stress still makes and
  shrinks them as a permutation and a tree. They stay out of the prelude (21.5's budget), so a file
  names them with a `use`. Stress still recognises the alias by its name; keying it to the stdlib
  declaration instead is open. Reverse: delete the two aliases.
- **25.2 The seeded-bug benchmark compares exact minimal inputs.** Each `bench/stress/<bug>.cp` is
  a port with one classic bug planted and the correct fn as its `@brute`; `<bug>.min` is the
  smallest input that shows it, and `tests/seeded_bugs.rs` fails unless `cpl stress` prints exactly
  that input, with a median under 10 s (`GOALS` 4). Byte equality makes a shrinker regression
  (a case that is still failing but larger) fail the gate, not just a missed bug; the cost is that
  a legitimate improvement that finds a different minimum of equal size updates the `.min`.
  Reverse: compare weights (`Case::weight`) instead of bytes.
- **25.3 Letter domains are made as code points.** `domain L = 'A'..'A' + k` is read by `cpl run`
  and the C++ as numbers (`66`, not `B`) today, though `plan/19` says its values are chars; stress
  makes what the reader accepts. When the reader takes letters, make `Shape::Int` with a char bound
  produce `Value::Char`. Reverse: that same change.
- **25.4 Simple, connected and acyclic graphs are three stdlib aliases.** `std.graph` gains
  `SimpleGraph<V, E>`, `ConnectedGraph<V, E>` and `Dag<V, E>`, each `[E](u: V, v: V)`: one edge per
  value of `E`, read and printed like any edge list by `cpl run` and the C++. Stress makes them with
  no loop and no repeated pair (an unordered pair for the first two, an arc for `Dag`), the second
  connected (a random spanning tree first) and the third acyclic (arcs follow a random vertex
  order), and shrinks them by deleting a vertex with its edges, deleting edges, and renaming
  vertices in the order the edges name them, keeping each property. A deleted or renamed vertex is
  renamed in every other value read over the same vertex range (`let (s, t): (V, V)`), so the case
  keeps its meaning. The spec's `gen<Graph<V>>(simple: true)` needs named arguments on a type the
  core does not have; three names over one shape need nothing new, and a weighted graph is a local
  alias with more fields (`type SimpleGraph = [E](u: V, v: V, w: W)`), as for `Tree`. The names
  are recognised by name, like 25.1. Reverse: delete the three aliases and `Shape::Graph`.
- **25.5 Equal values are lowered together, sizes apart.** After lowering each number alone, the
  shrinker lowers every number holding one value as one (in one input, and across the test cases
  of a multitest), so a failure that needs two values equal (a count left over from an earlier
  test, a repeated element) shrinks to `1 1`, not to wherever the first value stood. Sizes stay
  out of the group: they are halved and stepped on their own, and a size that equals a value by
  chance (`n = 2`, `a = [2, 2]`) would otherwise drag the array shorter and stop the group. The
  cost is one more bisection per distinct value per round. Reverse: drop step 3b and the joint
  step in `shrink_tests`.
- **25.6 Trees are made in four shapes; graphs are renamed toward their plainest form.** Hanging
  each vertex under a uniformly earlier one alone made 27 paths and 2 stars in 400 seven-vertex
  trees (`random_trees_include_paths_and_stars`); deep and wide trees are what break recursion
  and adjacency loops. `Pick::tree` now chooses random, long (under one of the three before),
  path or star, for a `Tree` and the tree a `ConnectedGraph` starts from (68 paths, 100 stars),
  and `cpl bench` overrides it, so its path and star cases now exist for `Tree<V>` (they were
  skipped). Shrinking a graph also tries swapping two vertex names (among the first 16) and
  writing every undirected edge low end first, so a star shrinks to `1 2, 1 3, 1 4` from any
  centre. The seeded `noip14p5-unsafe-step` now reaches a lighter minimum (weight 15 for 16), so
  its `.min` changed (25.2). Reverse: make `Pick::tree` return `Random` and drop the swap and flip
  steps.
- **25.7 `std.graph.Digraph` is a directed simple graph.** `SimpleGraph` is undirected, so the
  seeded `noip14p5-unsafe-step`, which reads arcs, was made with every pair at most once (never
  `u v` and `v u`) and shrunk by flipping arcs, which changes a directed graph. `Digraph<V, E>`
  joins the other three aliases (25.4): no loop, no arc twice, `u v` and `v u` distinct, room
  `n(n-1)`, arcs never flipped while shrinking (`shrinking_a_digraph_keeps_each_arc_the_way_it_points`).
  The port now reads a `Digraph`; its `.min` is the same graph with its arcs in order. Reverse:
  drop the alias and `GraphKind::Digraph` and point the port back at `SimpleGraph`.
- **25.8 Reads in a loop are a field; reads stress can't make are errors.** 342 of 1,332 ports
  read inside `for _ in 0..q:`, and stress refused all of them. A loop over a range or a domain
  whose passes read only typed `let x: T = read()` lines (or loops like it), with types that use
  nothing bound inside the loop, is now an array field of what one pass reads: the same input,
  so no new concept, and every shrink step already works on it. A `break` or `return`, a
  `continue` before a read, or a type that uses a value read in the pass is refused, since a
  fixed row cannot describe it. Reads inside a plain (or `@memo`) fn and `read(gen:)` /
  `read(none:)` were silently left out of every case (a too-short input, not a bug found); they
  are errors now. Shrinking gains one step, run only once the others stall: lower a value by one
  while dropping one element. Reverse: drop `Builder::pass` and the `For` arm (loops are refused
  again), and the `Fn` and `reads_with_options` arms (those reads are left out again).
- **25.9 A tuple's parts can bind names for the parts after them.** A loop pass that reads `s`,
  then `[0..s]City`, has a row whose later bounds use an earlier value, which no fixed shape
  says. `Shape::Tuple` gains `binds` (the names each part binds, as a field's `names` do); make,
  fit and the list of numbers bind them in a local scope, and a bound size inside a row is a size
  for shrinking (lowered, or lowered with one element dropped). One mechanism serves rows at any
  depth, so no `Row` shape beside `Tuple`. Names worked out in the pass (`let m = k + 1`) are
  still refused. Reverse: drop `binds` and refuse a pass type that uses a name read in the pass.
- **25.10 `domain V = read(lo..=hi)` is a size read where it is declared.** The design
  declares an index space read from the input this way (`design/spec/21-index-spaces.md`); the
  generator read its bounds as a fixed domain and left the count out. The count is now a field (or
  a part of a loop row) at the place of the declaration, arrays over `V` take that many values,
  and `V` is `0..count`. Reverse: drop `read_size` and the generator refuses such domains again.
- **25.11 A row whose reads depend on a tag read first is a `Choice`.** `if op == 1: read A
  elif op == 2: read B else: ...` and `match kind: 'B' => ... _ => ...` read different values per
  tag; refusing them left out most query problems. `Shape::Choice { tag, name, arms }` holds one
  arm per branch with the literal tags that take it (`None` takes the rest). Generation picks an
  arm evenly, then a tag that takes it; fitting rebuilds an arm's values at their low ends when
  shrinking moves the tag to another arm; shrinking tries each arm's tag before lowering values,
  so a query that reads nothing wins. An `Int` tag read with no range is narrowed to the tags the
  arms test (and one more for an `else`), since a far-off tag is noise, not a query. Branches
  with guards, non-literal tests or a test of anything but the tag are still refused. Reverse:
  remove `Shape::Choice` and `Builder::choice`; such loops are refused with their old message.
- **25.12 A struct or an `@tags` enum read from the input is generated field by field.**
  `read()` reads a struct's fields in order and an `@tags("R", "C")` enum as its variant's tag,
  then that variant's fields, so the generator does the same: a struct is a `Tuple` of its field
  types, and a tagged enum is a `Choice` on a letter whose arms are its variants. Generic records
  and tags longer than one letter are still refused with the old message. Reverse: drop
  `Builder::records` and `Record`.
- **25.13 `read(none: x)` makes `?T` either `x` or a value of `T`, and the shrinker tries
  a number's low end first.** `read(none: 0)` writes an absent value as `0`, so the generator
  writes the sentinel about a quarter of the time and an integer of `T` otherwise; a present
  value shrinks toward the lower of `T`'s low end and the sentinel. Only integer `T` with a
  number (or numeric string) sentinel is accepted; anything else keeps the old refusal. Lowering
  a number now tries its low end before the binary search, because a fit that clamps
  `?(1..=n)` up to `1` hides `none: 0` below the gap. Reverse: drop `Shape::Maybe`,
  `read_none` and the low-end attempt in `shrink`'s `lower`.
- **25.14 `read(gen: f)` is drawn by running `f` on the input above it, and shrinks by
  seed.** The type next to a `read(gen: f)` holds less than `f` does (a tree written as
  `[E](u: V, v: V)`, a strictly rising `[I]1..=1e6`), so the field's value is whatever `f()`
  makes: `cpl-ir`'s `lower_gen_read` keeps the program's top-level statements above that line,
  then prints `f()` as `read()` reads it, and the interpreter runs it with `rand` seeded
  (`Io::seeded`). The value is kept as its seed and its text (`Value::Drawn`). Shrinking
  never edits inside it: every time the values above it change it is drawn again from its
  seed, a rejected smaller case is retried with up to 16 other seeds, and a pass tries those
  seeds alone. Only top-level reads outside a test case are drawn; the rest keep their
  refusal. The generator runs in the interpreter only: it makes inputs, and the solution
  under test still runs on both sides. Reverse: drop `Shape::Gen`, `Value::Drawn`,
  `lower_gen_read`, `Io::seeded` and the reseeding in `shrink`.
- **25.15 A jump counts against a loop pass only when it leaves that pass.** A `break` or
  `continue` inside a nested `for`/`while` ends only that loop, and a `return` inside a nested
  `fn` or closure ends only that call, so the pass still makes every read of its row. A
  labelled `break`/`continue` is taken as leaving, since telling which loop its label names
  is not worth it for the few ports that use one. Reverse: make `Jumps` count every jump in
  the pass again.
- **25.16 A typed read inside an expression is a field, where it runs a known number of
  times.** `read<T>()` and `read::<T>()` give their type, so `Graph.undirected(read<[m](V,
  V)>())`, `[0..n] _ => read::<Int>() - 1`, `for a in read<[n]T>():` and `match
  read::<Op>():` make fields in the order the reads run: operands left to right, a table or
  a range comprehension as one array. A read in a branch, a closure, a filtered
  comprehension or a short-circuit operand, an untyped `read()` in an expression, and `a[read()]
  = read()` (no sure order) are refused. Before this, generic reads were not seen at all, so
  their input was silently left out. Reverse: drop `Builder::inline` and refuse every read
  that is not a whole `let`.
- **25.17 The ecnerwala interactors also talk over their own stdio.** With no `-- cmd`,
  `interactors/_lib.py` drives the solution on its stdin and stdout, the way `cpl stress
  --interactor` and `cpl judge` connect one, so the 16 interactors `gate.sh` already trusts
  are the ones stress runs under, not copies. `stress_judges` stresses all 16 and the 5
  checker ports on their `gen/` cases, and checks that a solution printing `1` is caught
  under each judge. Reverse: delete the stdio branch and the test.
- **25.18 `cpl stress --explain` names the first value the two runs give a variable
  differently.** On a failure against the file's own `@brute(main)`, both run on the shrunk
  case in the tree-walking interpreter with `TraceConfig::assignments`, which logs every value
  a `let`, an assignment, a mutation or a loop gives a named variable, in fns too. For each
  name both runs use, the values it takes (repeats collapsed) are compared, and the one whose
  first difference comes earliest in the solution's run is reported with its ordinal and the
  line on each side. Comparing histories by name, not steps, lines up two programs that share
  names but not shape; a history that is only a reordering ending on the same value is a loop
  in another order and is not a divergence. Variables only one side names are skipped, so
  when no shared name differs it says so and the `debug(...)` stages remain the guide. Off by
  default: logging every assignment slows a run. Reverse: drop the flag, `assignments` and
  `first_divergence`.
- **25.19 A fn that reads is read where it is called, when every call always runs.** A fn with
  no parameters, called only as a whole top-level statement or a `let`'s whole value
  (`read_moves()`, `let a = read_sym()`), reads its body's input at each call, in order, so its
  body is walked there as if written inline, nested calls included. A parameterless fn whose
  only reads are fixed `let name: T = read()` lines reads the same values in any expression, so
  such a call is a field wherever it is. A `return` after the body's last read ends the fn as
  its end would and is allowed; one inside a nested fn or lambda is not the reader's. Any other
  call (in a branch, a loop, an argument), a parameter its reads could depend on, or a `return`
  before a read reads only some of the time, and the fn stays refused. Reverse: drop `readers`,
  `values` and `expand` in `spec.rs`.
- **25.20 Types with a fixed text form are generated from that form.** `U64` is `0..=2^64-1`,
  `Mod<P>` is `0..P`, `F64` and `I128` are whole numbers in the `Int` default range (any
  whole number is a valid `F64` token, and the shrinker lowers them like integers), and a
  stdlib struct (`Pt`, `Pt3`) is read field by field like a user struct, looked up in its
  stdlib module, unless the file declares its own type of that name. Fractions and values past
  `i64` are left to `--gen`. Reverse: drop those arms of `Builder::shape`.
- **25.21 A tag read first in a tuple or above the reads it chooses still chooses.**
  `let (kind, l): (0..=2, Cell) = read()` then `match kind` is a `Choice` whose shared parts
  are the tuple's other parts and any reads between the tag and the branch; a tag rebound
  before the branch is refused, not trusted. Reverse: drop the tuple case of `Builder::choice`.
- **25.22 A loop read's type may use the loop's own name.** `for r in 1..=n: let row:
  [0..r]Int = read()` is an array whose element `at` is made, fitted and shrunk with `r` bound
  to `lo + at`, so a triangle stays a triangle as `n` shrinks. Only a plain name pattern is
  bound; a value worked out in the pass is still refused. Reverse: drop `index` from
  `Shape::Array` and refuse the loop name again.
- **25.23 A `Str` tag compared with words is one of those words.** `let op: Str = read()` then
  `if op == "ADD" ... elif op == "SHOW" ...` is a `Choice` over `Shape::Words`, the literals in
  the order the branches name them, plus one word none of them is when an arm takes the rest.
  A word is drawn and shrunk as its position, so the shrinker moves to the first-named arm.
  Words with spaces, or an interpolated string, are refused. Reverse: drop `Shape::Words` and
  `Value::Word`.
- **25.24 The seeded-bug median bar is 1 s, over at least 21 bugs.** With ten more bugs
  across multitest leakage, overflow at bounds and off-by-one at a range end, the gate's
  median time to a minimal counterexample is 0.77 s, so the bar drops from 10 s to 1 s and
  the benchmark refuses fewer than 21 bugs. The time is wall clock in the debug build the
  gate runs, so a loaded machine is part of what it measures. Reverse: restore
  `MEDIAN_LIMIT` to 10 s and the bug floor to 4 in `tests/seeded_bugs.rs`.
- **25.25 The seeded-bug test decides by input, not by clock.** Every bug runs with
  `--seed 1 --cases 1000` and a 60 s run limit, four at a time, so whether a bug is found
  and shrunk to its `.min` is the same on any machine; the median is printed against the
  1 s bar and never asserted. A wall-clock assertion failed a full gate under load (40 s)
  that passed alone, blocking every lane. Reverse: assert `median <= 1 s` again.
- **25.26 `a[i] = read()` reads the element type `a` is declared with.** The type is
  taken from the one `let`/`var` in the file that names `a` with an array type (or
  `List<T>`, whose element is `T`); a name
  declared twice is refused, since which declaration a write means depends on scope. This
  is the read the program makes, so the generated row matches it without asking the port
  to read into a typed `let` first. Reverse: drop `elems` and the `Assign` arm in `rows`.
- **25.27 A bare `read()` takes the type of the call it is passed to.** When every
  argument is a bare `read()`, `Graph<V, E>.undirected`/`.directed` reads `[E](V, V)` and a
  struct constructor reads the struct field by field, as the interpreter does. The edges
  are any `[E](V, V)`: a port that needs a DAG or a connected map should say `Dag` or
  `ConnectedGraph`, as a typed `let` would have to. Reverse: drop `Builder::read_as`.
- **25.28 A pass that skips its first or last index reads from the rest.** A loop
  whose body starts `if v == V.first: continue` (or `== lo`, `== V.last`, `== hi`) is
  read as a loop over the other indices, so `for v in V` reading each vertex's parent
  but the root's is a row per non-root vertex. A skip in the middle stays refused. A
  `domain` a pass computes from its own values is unknown like those values: only a
  read whose type names it is refused. The parents read this way are any `V`s, not a
  tree; a port that needs a tree should read `Tree`. Reverse: drop `skipped_end` and
  return to refusing such domains.
- **25.29 A table that skips its first or last cell reads the rest.** `[D] i => if
  i == D.first: x else: y`, with `x` reading nothing, reads `y` once per cell but one,
  the same rule 25.28 gives a loop. A skip of any other cell stays refused. Reverse:
  drop `skipped_cell`.
- **25.30 `let a: List<T> = [read() for _ in lo..hi]` reads `[lo..hi]T`.** The bare
  read takes the element type the `let` declares, as `let a: [D]T = [D] _ => read()`
  already does; a comprehension with a filter or a named loop variable stays refused.
  Reverse: drop the `Comprehension` arm of `typed_read`.
- **25.31 A top-level `if`/`match` on a value read above it is a choice.** The fields
  from the tag on become one choice field, as a loop pass's row already does, so
  `let k: 1..=2 = read()` then `if k == 2:` reading in each arm generates each arm.
  Only when the branch is the last top-level line that reads, no reader fn is
  inlined and no name is bound twice: a later read could need a name the arms now
  hold. Reverse: drop `Builder::top_choice` and its call.

## 26. The stdlib surface (`plan/STDLIB.md`, 2026-11-21)

26.1. **Sorted lookup keeps `first_ge`/`first_gt`/`last_le`/`last_lt`.**
The 0.3 spec spells them `first(from:)`, `first(after:)`, `last(upto:)`,
`last(before:)`. The four names read the same in lists, `[D]T`, `Map`,
`SortedBag` and `Compressed`, are greppable, and need no overloading on
argument labels, which the core does not have. Reverse: add label
overloading to the core, then a codemod renames the family.

26.2. **`Mod<P>` and `Comb<P>` keep their names** over the spec's
`Mint(mod: p)` and `Z.factorials`: 135 and 37 port files use them and
the ideas are the same. Reverse: add aliases, codemod the ports, drop
the old names.

26.3. **`Bitset<D>` stays `Bitset`.** The spec's candidate `Bits<D>`
collides with the `Bits` trait for integer widths. Reverse: rename the
trait first, then the type.

26.4. **`Monoid` and `Action` stay values, not traits,** until user
traits can carry associated constants; values also let one type have
several monoids (min and max over `Int`) without newtypes. Reverse: make
them traits with instance values as a bridge.

26.5. **Promotion order: the local library is a staging area, and a
module moves into the stdlib when it meets the stdlib bar and ports
reach for it** (two ports outside its own batch, a friction report, or
an import such as the nine `lib.math.poly` ones). The north star is
everything ever asked at the hardest contests, so nothing is excluded;
this only orders the work. NTT products, matrices, primes,
Hopcroft-Karp, min-cost flow, Manacher, Aho-Corasick and convex hull
come first. Reverse: move a module back and leave a re-export.

26.6. **The design names every public name, by test**
(`the_stdlib_design_names_every_public_name`), so the doc cannot drift
silently. Reverse: delete the test.

26.7. **One number-theory home: `std.math` owns `pow_mod`, `is_prime`,
`factor_big`, `phi`, `primitive_root` and `floor_sum`,** promoted from
`examples/local-lib/math/primes.cp` and `floor_sum.cp`, which keep only
what builds on them. `is_prime` and `factor_big` are exact for every
`Int`, not only below 4e18, because `mul_mod` now goes through `I128`
in O(1) instead of doubling in O(log b); `factor` (trial division)
stays because it is simpler and fast enough to 1e12. Reverse: move the
functions back into the local library and import them from there.

26.8. **Rollback marks are a `Mark`; values and parities roll back
with the sets.** `Dsu.mark()` returned an `Int`, so `cf2222f` kept a
second `Int` for its own payload history and the two could be swapped.
`Mark` is one nominal type shared by `Dsu`, `DsuOf<V, T>` (a value per
set, `join(value of a's set, value of b's set)` on merge, whichever root
is kept) and `ParityDsu<V>` (promoted from the local library's
`BipartiteDsu`, renamed after what it stores, with `unite`/`diff`
spelled like the local `PotentialDsu` so a later group-valued potential
DSU can absorb it). `DsuOf` is a separate type, not a payload parameter
on `Dsu`, because a plain `Dsu` should not pay for a value array, and the
core has no default type parameters. A closure `rollback(body)` helper
is left out: its body must change the lent structure, which lexical
borrowing rejects; scoped rollback waits for the core's `defer`/`undo`
block. Reverse: make `mark()` return `Int` again and inline `DsuOf`'s
log into callers. `Dsu.size` and `DsuOf.get` bind `find(v)` before indexing: the
C++ backend copies the whole field when an index changes `self`
(`sz[self.find(v)]`), which made `cf2237h` 10x slower (1608 ms against
167 ms); the copy is the codegen lane's to remove.

26.9. **A test after the sequence is the walk's body.** `count(xs, p)`,
`all(xs, p)`, `any(xs, p)`, `first(xs, p)` and the methods `xs.count(p)`,
`xs.all(p)`, `xs.any(p)` lower to `count(p(x) for x in xs)` and
`first(x for x in xs if p(x))`, with an inline lambda's body substituted,
so they cost what the walk costs, stop where it stops, and run on
every sequence a `for` takes (`Str`, ranges, sets, `alive.each()`).
Four ports asked for it (`cf2207e2`, `cf2069e`, `cf2147h`, `cf2066e`).
Only the folds of a test take one: `sum(xs, f)` would be a second
spelling of `sum(f(x) for x in xs)`, which spec 22.2 forbids; `prod`
joins `sum` as the product fold (a struct's `one` and `*`, else 1).
`xs.count(v)` with a value still counts equal elements; a function
argument is a test. `find` stays the position search on lists and the
substring search on `Str`; the first element matching is `first`.
Reverse: drop `tested_walk` from the `count`/`all`/`any`/`first` arms
and the method hook; ports go back to the walk form.

26.10. **Stdlib areas own section 4; a cross-area item goes to the area
of the type it returns.** Ten owners each take a slice of
`plan/STDLIB.md` section 4 (table "Who owns what"); the coordinator
keeps sections 1 to 3, the prelude, `std.spec` and section 5. Items no
area named are placed by what they are: RNG and hashing with number
theory, bitsets and `List` methods with containers, `std.grid` with graphs, offline time divide and
conquer with containers (its owner designed `offline_over` beside the
rollback DSU), link-cut trees with
trees, the searches on answers with DP/search, the walk folds'
compiler side in section 3 and their library side with sequences.
Reason: one owner per name keeps one vocabulary reviewable. Reverse:
edit the table; the routing rule in `design/stdlib/README.md` follows it.

## 27. Interpreter, debugger and traces

27.1. **Runtime traps carry call stacks, multitest case numbers, and concrete values.**
`cpl_interp::Trap` records `call_stack: Vec<Frame>` (summarized when deeper than 32 frames to avoid
allocating on runaway recursion) and `case: Option<u64>`, populated on the error path from a
zero-allocation `(&str, Span)` frame stack in `Interp`. Division/remainder by zero and integer
overflow include the concrete operands. `Trap::to_diagnostic` is shared by `cpl-cli` and `cpl-wasm`
so native and WASM diagnostics stay identical. Reason: `plan/50-debugging.md` §5 requires runtime
errors to explain what happened in one screen with values, call path, and test case without slowing
normal execution. Reverse: remove `call_stack`/`case` from `Trap`, `call_frames`/`current_case` from
`Interp`, and `Trap::to_diagnostic`.

27.2. **Unboxed `OptVal::Int` / `TupleVal::OneInt` and zero-allocation interpreter hot paths.**
`Value::Opt` stores `OptVal::{None, Int(i64), Boxed(Rc<Value>)}` and `Value::Tuple` stores
`TupleVal::{OneInt(i64), Boxed(Rc<Vec<Value>>)}` (promoting to `Boxed` only on in-place field/option
mutation), keeping `size_of::<Value>()` unchanged while eliminating heap `Rc`/`Vec` allocations for
domain steps (`?Pos`) and 1-field `Int` structs (`Mod<P>`). Untraced `assign`, 1D/2D `index`,
0–3-arg `call_feeding`, step-1 `Iter::Range`, and primitive `Cmp` avoid temporary heap `Vec`s and
`i128` loop arithmetic. Measured speedup on ports: `dph` 884ms -> 693ms (-22%), `dpm` 1288ms ->
1028ms (-20%), `dpl` 542ms -> 483ms (-11%), `dpk` 770ms -> 728ms (-5%). Remaining gap to <= 30x
is tree-walking `eval`/`stmt` dispatch overhead (future bytecode VM). Reverse: revert `OptVal`,
`TupleVal`, and `assign_untraced` in `crates/cpl-interp`.

27.3. **Trace filtering (`--trace` globs/exclusions, `--test`, `--watch`, `--last`, `--depth`) and `Bound(d)` cut labels.**
`TraceConfig` gains `test_cases`, `watch`, `last_steps` (O(1) `VecDeque` ring buffer), and
`max_depth`, while `only_fn` parses comma-separated globs (`*`, `?`) and `!pat` exclusions tracked
via `pos_inside`/`neg_inside` counters in `Tracer`. `Bound(d)` labels and values now shift by
`view.domains[d].0` to match `value::show` and 1-based domain semantics. Reason: `plan/50-debugging.md`
§1 requires surgical trace capture on large/multitest runs and consistent domain-base cut labels.
Reverse: remove the extra fields from `TraceConfig`, restore single-fn `only_fn` matching, and
remove `--test`/`--watch`/`--last`/`--depth` from `RunArgs` in `crates/cpl-cli/src/lib.rs`.

## 27.4. Time-travel DAP debugger (`cpl dap`) backed by interpreter snapshots

- **Decision:** Add `cpl dap` implementing the Debug Adapter Protocol over stdio (`Content-Length`-framed JSON) with zero new dependencies, wired into `editors/vscode` (`type: "cp"`, `cp.debug` command, `DebugAdapterDescriptorFactory`). When `TraceConfig::record_frames` is enabled, each step records the full call stack (`(fn_name, line)`) alongside locals and domain-labeled globals, allowing both forward stepping (`next`, `stepIn`, `stepOut`, `continue`) and backward time-travel (`stepBack`, `reverseContinue`), hover/watch `evaluate`, and exception stops on runtime traps.
- **Why:** Reusing the deterministic snapshot recorder guarantees that backward stepping, replay of failing judge inputs (`input` / `inputText`), and domain-labeled variable inspection work out of the box with zero overhead on untraced runs or normal `--trace` runs (`record_frames: false`).
- **How to reverse:** Remove `crates/cpl-cli/src/dap.rs`, `crates/cpl-cli/tests/dap.rs`, the `dap` command dispatch in `crates/cpl-cli/src/{lib,main}.rs`, `record_frames`/`frames` in `crates/cpl-interp/src/trace.rs`, and the `debuggers`/`breakpoints`/`cp.debug` contributions in `editors/vscode/`.

27.5. **Typed register bytecode VM and flat typed `ArrayData` in `cpl-interp`.**
Plain (untraced, unprofiled) runs compile `Program` to a typed register bytecode (`crates/cpl-interp/src/vm.rs`) with unboxed `i64` register slots for `Int`/`Bool`/`Char`/`Pos`/`Bound`/1-field-`Int` structs, unboxed `(val, some)` register pairs for `?Int`/`?Pos`/`?Bool`/`?Char`, flat typed `ArrayData::{Int, OneIntStruct, OptInt, Bool, Char, Float, Values}` in `value.rs`, inlining of pure non-trapping leaf functions (e.g., `Mod<P>` arithmetic), direct unboxed VM calls sharing a flat `vm_regs_i` stack, superinstructions for 1D/2D array/list/loop hot paths, and exact fallback to `Interp::stmt`/`eval` for remaining IR constructs (syncing register slots with `Interp::stack`). Trace, profile, and DAP runs keep using the tree-walker with zero parity risk. Reason: the interpreter target is <=10x C++ `-O2` (north star <=5x); the initial VM cuts AtCoder DP ports by 4.5x–11x (`dph` 1498 -> 214 ms, `dpk` 2277 -> 428 ms, `dpl` 7882 -> 1754 ms, `dpm` 11876 -> 1084 ms). Reverse: remove `crates/cpl-interp/src/vm.rs`, the `vm` dispatch in `crates/cpl-interp/src/lib.rs`, and restore `Vec<Value>` in `Array` (`crates/cpl-interp/src/value.rs`).

27.6. **Flat packed small-tuple arrays (`ArrayData::SmallIntTuple`), modular-arithmetic superinstructions, and last-use move semantics in `cpl-interp`.**
Added flat stride-packed `ArrayData::SmallIntTuple` (2–4 scalar fields of `Int`/`OneIntStruct`/`Bool`/`Char`) with `SlotLoc::SmallTuple` and 1D/2D load/store opcodes, direct `FromLabel`/`FromOrd` and reverse-sequence `Iter::Seq(..., true)` lowering, structural `ModAddI`/`ModSubI`/`ModMulI`/`ModNegI`/`Mod::new` superinstructions (plus inlining of 2D-indexed leaf helpers that call them), and block-local last-use move opcodes (`StoreGlobalVMove`, `MoveVConsume`) so rolling-DP `ways = next` assignments keep `Rc::strong_count == 1` instead of triggering copy-on-write clones. Reason: wave-12C heavy DP ports (`cf2180f1`, `cf2183f`) hit 53–161 s and 3.1 GB RAM from boxed tuple-array cells, uncompiled `.rev`/`P(x)` loops, and 6–12-op inlined `Mod<P>` arithmetic; these general VM mechanisms cut `cf2180f1` (`n=5000, m=5000`) to 11.62 s / 211 MB and `cf2183f` (`n=5000`) to 5.73 s / 223 MB. Reverse: remove `ArrayData::SmallIntTuple` in `value.rs` and `SmallTuple`/`Mod*I`/`MoveVConsume`/`StoreGlobalVMove` in `vm.rs`.

27.7. **VM hot-loop fallback instrumentation (`CPL_VM_STATS`), frame pooling, fast scalar closures, and general bytecode lowering.**
Added loop-depth-aware fallback accounting via `CPL_VM_STATS=1|json` (`total_fallbacks`, `loop_fallbacks`, `hot_loop_fallbacks`, `top`, `loop_top`) and a zero-fallback regression ratchet in `crates/cpl-cli/tests/interp_ratio.rs`, plus general VM lowering for sequence loops, general indexing, `Do`/`Give` blocks, `List` literals/`filled`/`to_list`/`copy`/slices, `Str` slices/`StrView` indexing, `Fill`, 1D indexed mutable-arg calls (`Take*` + `WriteBack`), 2D `Value` array load/store, `F64` arithmetic/min/max, `Value`-typed `If`, structural `CmpV`, bit builtins, pooled `Value` register frames (`vm_pool_v`), and uncaptured scalar `FastClosure` dispatch. Reason: systematically drives hot-loop tree-walker fallbacks toward zero without per-port special cases; cut full-scoreboard p95 from 56.83x -> 39.37x, max from 281.6x -> 85.32x, and ports with hot-loop fallbacks from 109 -> 91 across 249 sampled ports. Reverse: remove `CPL_VM_STATS`, `vm_pool_v`, `FastClosure`, and the new opcodes in `crates/cpl-interp/src/{lib,vm}.rs` plus the ratchet test in `crates/cpl-cli/tests/interp_ratio.rs`.

## 28. Core mechanisms for a library-only stdlib (0.3, §1)

- **28.1 Reflection is a loop: `for f in fields(x)`.** The body runs once per field of a struct, record
  or tuple, in declaration order, with `f` bound to that field at its own type; `fields(a, b)` walks
  values of one type in step, binding a tuple. It is unrolled in lowering, so the interpreter and the
  C++ see ordinary code and no new IR, and a generic body (`fn show_all<T>(x: T)`) is checked per
  instantiation like any other generic body. It is the base the derives, variadic packs and
  `print`/`read` move onto (`plan/DOMAINS.md` §1). Why a loop and not a macro or a `comptime` block:
  it is the one construct a contestant already reads as "for each", it needs no new keyword, and it
  keeps generated code visible to the trace and the formatter. `break`/`continue` are refused (`E0501`)
  because an unrolled body has nowhere to jump to; `fields` is not reserved, so a user `fn fields`
  wins. To reverse: delete the `fields_loop` paths in `cpl-check` and `cpl-ir` lowering; nothing else
  depends on them yet.

- **28.2 Enums order by variant, then payload.** `<`, `sort`, `max` and the rest work on an enum whose
  payloads are all ordered: variants compare in declaration order, then field by field, the order the
  interpreter's values and the C++ `cpl_tie()` already had. Lowering refused it (`E0309`) only because
  `ordered()` had no enum case, which made the sweep-line idiom (`enum Event: Open, Query, Close`,
  then sort) need a hand-written key. Declaration order is the order a contestant writes events in,
  the same rule as struct fields and as the 0.3 spec's derived `Ord`. To reverse: drop the `Ty::Enum`
  arm of `ordered()` in `cpl-ir/src/lower/expr.rs`.

- **28.3 A variadic parameter is a tuple pack.** `..xs: Ts` takes the remaining positional arguments as
  one tuple; `Ts` must be a type parameter, bound per call to `(A, B, ..)` (the empty tuple for none),
  and a bound on it (`Ts: Show`) is a bound on the tuple, which holds when every element meets it. The
  body walks the pack with `for x in fields(xs)` (28.1), so variadics add no new loop, type form or IR:
  lowering packs the arguments into a tuple and instantiates the generic as usual. Why not the spec's
  `<Ts...>(xs: Ts...)`: the `..` on the parameter already marks it, and a second marker on the type
  parameter says nothing the parameter does not. Why not homogeneous `..xs: Int` as a list: a
  `List<Int>` parameter with `[a, b, c]` already says that, at one more pair of brackets, and two
  meanings for `..` would be a rule to remember; it is refused with that fix. To reverse: restore the
  `E0501` in `cpl-ir` `lower/mod.rs` signature lowering and drop `pack_args` and the pack branch of
  `args` in `cpl-check`.

- **28.4 `f<T>()` on a lowercase fn.** `read<(Int, Int)>()` and `read<Int>()` parse as type arguments,
  as the spec writes them, when every argument is spelled as only a type can be (a capitalized name,
  or a tuple, array, optional or record of them) and `(` follows. `a < b > (c)` stays a comparison
  because `b` is lowercase; `i < N > (k)` with a capitalized `N` would now parse as a call, a shape no
  program in the corpus has. `f::<T>()` still works. This reverses synthesis #2's deletion of
  `read<Int>()` (its `s2_read_angle` case is now `ok_s2_read_angle`), because `design/spec` writes
  `read<T>()` throughout and a pack type `read<(A, B)>()` needs a spelling. To reverse: restore the `type_like &&` gate in
  `cpl-syntax` `parser/expr.rs` and drop `names_a_type`.

- **28.5 Lazy parameters.** `fn or_else<T>(a: ?T, lazy d: () -> T) -> T` takes, at the call site,
  an expression `e` of type `T`, and passes `() => e`; the body calls `d()` when it wants the value.
  So `??`-, `assert`- and `debug`-shaped fns can be library code that evaluates its argument zero or
  one time, and the reader of the body sees each evaluation as a call. The declared type stays the
  function type (one meaning per spelling); `lazy` only changes what the caller writes, and a lambda
  is still accepted as is. `lazy` is a contextual word, so `lazy` stays a usable name. Rejected:
  Scala's by-name `d: => T` (each read re-evaluates, invisibly) and an implicit conversion from `T`
  to `() -> T` for every fn-typed parameter (the call site of a non-lazy fn would stop saying when
  its argument runs). Free fns and generic fns only so far; methods next. To reverse: drop
  `Param::lazy` in `cpl-syntax`, the `slot` match in `cpl-check` `check/call.rs`, and
  `thunk_lazy_args` in `cpl-ir` `lower/mono.rs`.

- **28.6 `fields` over an enum.** `for f in fields(e)` on an enum value runs the body over the
  payload of the variant `e` is: lowering emits one `if` per variant, each unrolled over that
  variant's fields, and the checker checks the body against every variant's field types.
  `fields(a, b)` runs only when both are the same variant. One mechanism serves structs and enums;
  what derives still need is the variant's index and name (`Ord`, `Hash`, `Show`), next. To reverse:
  drop `variant_payloads` in `cpl-check` `check/stmt.rs` and the `Ty::Enum` branch of `fields_loop`
  in `cpl-ir` `lower/stmt.rs`.

- **28.7 Derives as library code: `std.derive`.** `eq_fields`, `lt_fields` and `show_fields`
  are generic fns on `fields` (28.1) in `stdlib/derive.cp` (imported, not in the prelude, which
  is capped at 80 names): the field-by-field
  `Eq`, `Ord` and `Show`, which an `impl` can call (`fn lt(a, b) = lt_fields(b, a)` is the reverse
  order). The compiler's own structural derive stays the default for now: wiring every struct
  without an impl to these fns (lowering filled `eq_fn`/`lt_fn`/`show_fn` from them) gave the same
  output but grew emitted C++ past the scoreboard's 5% gate on many ports (ucup5s02h
  28210 -> 44772 B, cf2268c 15903 -> 28221 B), because each hook is emitted whether or not it is
  used, where `cpl_tie` costs one line. The default moves onto `std.derive` once codegen emits a
  hook only when it is reached. To reverse: delete `stdlib/derive.cp`.

## 29. Editor and LSP (2026-11-21)

29.1. **Hover shows the declaration's `///` doc, wherever it is declared.** `Hover` gains `doc`, read
from the lines above the definition `definition_location` finds (decorators skipped), so a stdlib name
shows the doc that states its cost. A member access records its owner type (`HoverEntry::owner`), so
hover and go-to-definition on `bit.prefix` reach the method in `stdlib/fenwick.cp`. The doc is the one
place a cost is written today; a computed cost line can join it later. Reverse: drop `Hover::doc` and
`HoverEntry::owner` in `crates/cpl-check/src/{hover,check}.rs`.

29.2. **Neovim, Emacs and Helix get thin setups over `cpl lsp`, smoke-tested in the gate.** Each
setup is the editor's own idiom (a Neovim runtime folder, an Emacs major mode registering with eglot
and lsp-mode, a Helix `languages.toml`) and holds no logic beyond highlighting and starting the
server, so a feature lands once, in the server. Their keyword lists are checked against
`TokenKind::keyword` by `editors/smoke.sh`, which `scripts/check.sh --full` runs, skipping (and
saying so) an editor that is not installed. Zed waits on a tree-sitter grammar, which it requires for
any language. Reverse: delete `editors/{neovim,emacs,helix,smoke.sh}` and the `--full` step.

29.3. **The LSP latency budgets hold at the median, with the 90th percentile within twice the
budget.** They were asserted on the slowest sample, and one descheduled request under the fast gate's
fully parallel load turned `main` red with no slow path in the server. The median of many warm samples
is what a user feels on each keystroke; the 90th-percentile bound still fails a slow path hit on more
than one keystroke in ten. Opening a file is now nine samples, not one. Reverse: assert `max < budget`
in `report` in `crates/cpl-lsp/tests/latency.rs`.

29.4. **Semantic tokens come from the checker's resolved names, not from a second lexer.** Each
name the checker recorded is a token, typed by what it resolved to (a domain is a `type`), so a client
colours what syntax can't tell: types against values, `var` against `let`. Keywords, literals and
comments stay with each editor's grammar, which already has them, and a name the checker could not
resolve gets no token, so the grammar's colour shows. Reverse: drop `semantic_tokens` and the
`semanticTokensProvider` capability in `crates/cpl-lsp/src/server.rs`.

29.5. **Workspace symbols search the open files and the modules they use, not the disk.** The
server has no project root of its own (a contest folder is a pile of single files), and what a
contestant looks for is in what they have open or import; a library module offers only its `pub`
names, so its tests and helpers don't crowd the picker. Matching is an in-order subsequence, as
editors' pickers expect. Reverse: walk the workspace folders from `initialize` instead of `docs`
in the `workspace/symbol` arm of `crates/cpl-lsp/src/server.rs`.

29.6. **The server's run lenses are `cpl` argument lists, opted into by the client.** A lens's command
is `cpl.exec` with the arguments to pass to `cpl` (`run <file> <input> --trace`), so a client needs one
generic handler (Neovim: a terminal split) rather than a command per action, and the server stays
the one place that finds samples and decides which lenses a file gets. It answers only when
`initializationOptions.codeLens` is true, because VS Code draws richer lenses of its own (results
inline, the trace panel) and would show both. Reverse: drop `code_lenses`, the `codeLens` arm and the
capability in `crates/cpl-lsp/src/server.rs`, and the handler in `editors/neovim/plugin/cplang.lua`.

29.7. **Range inlay hints reuse the lint's bounds, top level only, for now.** `cpl_lint::proven_ranges`
exposes what `rules/bounds.rs` already proves from declared input bounds (`let n: 1..=100` makes
`let m = n * n + 1` lie in `2..=10001`), so the hint and the lint that warns about overflow can't
disagree; a single-value range (a literal) gets no hint. The general version is a range analysis in
the checker that hover, hints, widths and overflow proofs all read, which this should move onto.
Reverse: drop the `proven` lookup in the `inlayHint` arm of `crates/cpl-lsp/src/server.rs` and
`proven_ranges` in `crates/cpl-lint`.

29.8. **Completion ranks by the expected type as text.** The expected type comes from where the
cursor is (the parameter an argument is for, by name or position; a `let x: T =`; the variable an
`=` or `op=` assigns), and an item fits when its type, or the return type of a fn, is the same
text. Fits sort first; the rest keep their order. Comparing printed types misses fits through
generics and range-to-`Int` widening, but needs no inference at the cursor, which is usually
mid-edit and unparsed. The general version asks the checker for the expected type of a hole and
for assignability. Reverse: drop `expected_type` and `rank` in `crates/cpl-lsp/src/complete.rs`.

29.9. **Signature help writes parameters as the declaration does.** A default shows as `= 1` and
named-only parameters follow a `;`, so the label reads like the `fn` line. The active parameter is
the one an argument names (`back: |`), else the positional one at its place; named-only ones are
never positional, so past the last positional one none is active. Reverse: drop `default` and
`named_only` from `SigParam` in `crates/cpl-lsp/src/complete.rs`.

29.10. **Rename edits every open file and the user library that declares the name.** The files are
the open ones plus the declaring file, read from disk when it is not open; a contest folder has no
project root to scan, as with workspace symbols (29.5). A use is renamed when go-to-definition
takes it to the declaration, and a `use m.{name}` when it imports the name from that file. The
standard library's names are refused, and so is a new name already declared or imported at the top
level of a file the rename edits. Shadowing inside a fn is not checked yet. Reverse: put back the
`target.path != uri` refusal in the `textDocument/rename` arm of `crates/cpl-lsp/src/server.rs`.

29.11. **Quick fixes go to the errors the broken ports hit first most often.** The broken-ports
benchmark (`crates/cpl-cli/tests/recovery.rs`) now prints each first error's code and how often it
carries a fix: E0101 unclosed delimiter (1579, all fixed), E0102 expected a value (1413, none),
E0103 expected a token (790, 771) and E0004 unknown character (398, none). E0102 after an operator
and before a value is a doubled operator, so its fix removes the second one (584 of the 1413); E0004
replaces `#` with `//`, a Unicode `≤ ≥ ≠ × −` with its ASCII operator and removes anything else. The
fix is on the diagnostic, so `cpl check`, the LSP and every editor get it, and the LSP test applies
each to a real port broken that way. Reverse: drop `doubled_operator` in
`crates/cpl-syntax/src/parser/expr.rs` and the `with_fix` after `UNKNOWN_CHAR` in `lexer.rs`.

29.12. **An unclosed bracket's fix inserts the closer where it was most likely dropped.** With one
bracket open, the fix goes before the first `=` or assignment, the first `:` that ends the line or
sits in a `[`, or a `for` inside an index (`if a[i] == b[i:` becomes `b[i]:`); otherwise, and in a
type (`[0..nInt`), it stays where recovery closes. Recovery itself is unchanged: the token stream
still closes at the end, because closing earlier left the rest of the line outside any bracket and
raised the benchmark's cascaded errors from 194 to 259. `recovery.rs` now also counts whether a
first fix restores the port's line and leaves no error: E0101 restores 451 → 582 of 1612 and checks
clean 814 → 950; cascades stay 194. Reverse: use `close_at` for the fix in `close_unclosed`.

29.13. **The trace viewer talks to the editor through two messages.** In a webview the viewer posts
`{type: "step", line}` when the shown step moves to another line and `{type: "reveal", line}` when
a source line is clicked (`TraceCore.editorLink` in `tools/trace/index.html`); in a browser it posts
nothing. VS Code keeps one trace panel per file, reused by the next trace, highlights the step's
line in the file and moves the cursor on a click. Any editor with a webview (JetBrains' JCEF) can
speak the same two messages. A unit test feeds what the real viewer posts to the extension's
parser, so the two sides can't drift. Reverse: drop `editorLink` and `src/traceView.ts` and create
a fresh panel per trace in `Runner.trace`.

29.14. **A checker error whose span takes in poisoned code is dropped.** `f(a[i, j)` closes the `(`
and reports the `[` as unclosed, but the call then sees two arguments in `a[i, j` and one in the
call, and reported `E0303` or `E0305` before the real cause on the same line. Sinks already dropped
errors inside a poisoned span; `Checker::error` now also drops those whose span contains one
(`Sink::encloses_poison`), since an error about all of an expression that holds unreadable code is
an echo of the syntax error. Broken ports: cascaded errors 187 → 145 of 4319, first error on the
right line 99.7% before and after, `E0303` 17 → 0 and `E0305` 13 → 0 as first errors (now the
`E0101` with its fix). Reverse: drop the `encloses_poison` check in `Checker::error`.

29.15. **The JetBrains plugin uses the IntelliJ Platform's own LSP API.** It starts `cpl lsp` through
`platform.lsp.serverSupportProvider`, so every server feature reaches it with no client code, and
it adds only what the server can't: a highlighting lexer (keywords checked against the compiler by
`editors/smoke.sh`), run configurations, a run marker and the trace viewer in a JCEF editor tab.
That API ships only in the commercial IDEs from 2024.3, so Community and Android Studio get
nothing; LSP4IJ would reach them but is a third-party plugin and a second client to keep working.
`--full` runs `buildPlugin` when `java` is installed; the platform tests join it once they have run.
Reverse: delete `editors/jetbrains/` and its block in `editors/smoke.sh`.

29.16. **The server publishes the lints, and `cpl fix` is its fix-all.** After the checker's
diagnostics it publishes what `cpl lint` reports under the file's `cp.toml`, each lint's fix a
quick fix, and a `source.fixAll.cpl` action applies `cpl fix` to the buffer (the same
`apply_fixes` loop to a fixed point), so an editor's fix-on-save and the command line rewrite
alike. Lints run on the whole file on every edit: 492 lines open in 10 ms and a keystroke
reaches diagnostics in 5 ms (debug build). The playground's build leaves them out: the lint
rules added 85,600 B to `dist/cp-lang.html` and took it over its 2,600,000 B cap. Reverse: drop `lints` from `diagnostics` and
`codeAction` in `crates/cpl-lsp/src/server.rs`.
The fix-all is `cpl_lint::fix`, the same function `cpl fix` runs: first the removed-syntax
rewrites the parser offers (`where` as a filter, `--- multitest ---`), since a file that
does not parse gets no lints, then the lints; so a file with `where` and `xs.len()` comes out
with `if` and `xs.len` in one action. The replaced spellings (`.pow` as L0205, getter `()` as
L0206, the removed syntax as E0503) reach the editor as quick fixes with no editor code of
their own. Reverse: inline `fix_removed_syntax` back into `crates/cpl-lint/src/cli.rs`.

29.17. **Every member read hovers.** Hover entries were recorded only on the struct path of
`Checker::field`; a built-in member (`xs.len`, a position's `next`) returned earlier and had no
hover. `field` now wraps the lookup and records the entry when the lookup did not, so a getter
read without parentheses hovers its type whatever owns it. Reverse: inline `field_ty` back
into `field` in `crates/cpl-check/src/check/expr.rs`.

29.18. **Hover states the documented cost and the proven range.** A hover on a fn or member
whose `///` doc (stdlib) or catalog line (built-in members) states an `O(...)` adds a
`cost:` line with the first one, so the cost is read without parsing prose; a built-in
method whose call is typed later (`xs.sorted()`) hovers its catalog signature. A top-level
`let` hovers the range `cpl_lint::proven_ranges` proves, the same one its inlay hint shows.
The cost comes from the docs because the docs already state every stdlib cost (AGENTS.md); a
computed cost would need a cost analysis the checker does not have. Reverse: drop `cost_of`
in `crates/cpl-check/src/hover.rs` and the range branch of `textDocument/hover`.

29.19. **Proven ranges reach into fns and loops.** `cpl_lint::proven_ranges` walks fn bodies,
`for`, `while`, `unchecked` and plain `if` blocks with a copy of the top-level bounds: a typed
range parameter (`k: 1..=n`) and a loop over `lo..hi` bind their range, a `let` binds what its
value proves, and any other binding of a name forgets its bound, so a shadowing local never
inherits one. Intervals stay non-relational (`5 * k - 4 * k` is not `k`), the same arithmetic the
lint already proves overflow with. `var`s get no range: a later assignment would make it a lie.
Hover adds the range only to a binding typed `Int`; a loop variable already hovers `i: 0..k`.
Reverse: restore the top-level loop in `rules::proven_ranges`.

29.20. **A misspelled option is a quick fix; signature help points at the next option.** The
checker's "did you mean" for an unknown argument name (E0310) and for a trait fn now carries
the same `change to` fix an unknown field already had, so `xs.reach(cylic: true)` is one click
from `cyclic:`. Past the positional parameters signature help marks the first named-only
parameter not given yet, instead of none: for `reach(while:, cyclic:)`, where every parameter
is named-only, nothing was ever marked after the first. Reverse: drop the two `fixes.push` in
`check/call.rs` and `check/stmt.rs`, and the fallback in `complete.rs::active_param`.

29.21. **A token-level tree-sitter grammar, for highlighting only.** `editors/tree-sitter-cp`
gives Helix, Neovim and Zed highlighting from one grammar: a flat sequence of tokens with
`fn` names, calls, members, types and constants picked out. It has no blocks, so it needs no
indentation scanner, and a half-typed line cannot turn the rest of the file into an error;
every port and library parses without one. Folding and semantics come from `cpl lsp`, which
already folds and sends semantic tokens. The parser is generated at ABI 14, which Neovim 0.10
and Helix 25.01 load (15 they do not). Each editor's capture names differ, so each has its own
copy of the queries, and the smoke test checks the patterns stay the same. Reverse: a full
grammar with an external scanner for indentation would add folds, indents and text objects.

29.22. **Each editor's smoke test drives the server its setup starts.** Neovim and Emacs run
headless, so their tests are the editor itself (Emacs over jsonrpc.el, what eglot is built on,
since Emacs 27 has no eglot). Helix has no headless mode: its test checks `hx --health` and
the grammar build, then `editors/lsp-smoke.py` starts the command its `languages.toml` names
and checks `initialize` and the published error. Reverse: drop `lsp-smoke.py` and keep
`--health`.

29.23. **Zed gets an extension that pins the grammar's commit.** A Zed language comes only from
an extension, and Zed builds an extension's grammar from a git repository at a fixed commit, so
`editors/zed/extension.toml` names the commit that last changed `editors/tree-sitter-cp`, and
`editors/zed/smoke.sh` fails when the tree's grammar differs from that commit's (move `rev`
forward in the same push). The extension starts `cpl lsp` from the PATH unless Zed's
`lsp.cpl.binary` setting names another. `zed_extension_api` pulls `smallvec`, pinned in its
`Cargo.lock` to a release old enough for the package proxy. Reverse: delete `editors/zed`.

29.24. **A lambda passed to a built-in method gets its parameter types from the catalog.** In
`xs.map(x => ...)` the checker types the call from the receiver, and inferred the lambda with
untyped parameters, so `x`, and every `let` in a block body after `=>`, hovered as `_` and got
no inlay hint. Now the catalog row's parameter type (`f: T -> U`, `while: (T, T) -> Bool`)
gives each unwritten parameter its type, with `T` the receiver's element, before the lambda is
inferred: one rule for every built-in method that takes a lambda, and the body is checked with
real types. Reverse: drop `catalog_lambda_params` and the `lambda_params` field.

29.25. **The editor's overflow warning names inputs the prover found, and is a note.** An
interval hull that leaves `Int` is only a suspicion (`a - b` with `a >= b` known, a `var`'s
hull), so the editor asks `cpl-prover` for inputs in their declared ranges that overflow the
operation and shows a soft note only when it finds some: `area * area overflows Int when n =
55109`. Only names it can follow take part (range-typed inputs and params, immutable `let`s of
arithmetic over them); a narrowed name, a loop variable or a `var` makes it say nothing, so a
note is never a false alarm. It is a note in the editor, not a lint, until the checker owns
ranges; it fires on none of the 305 ports and examples. Reverse: drop `Bounds::overflow` and
`overflows` in the server.

29.26. **`var`s that read each other get one joint hull.** A binary search's `lo = mid + 1`
reads `mid`, a `let` in the loop, and `mid` reads `hi`, a later `var`, so solving one `var` at a
time gave none of them a range. Now each `var`'s hull is solved with every `var` and `let` in
its scope that one place in the file binds (so each use reads that binding), iterating to the
least fixed point (up to 72 rounds, enough to halve a gap of `10^18`). A `var` that never settles
is dropped and the rest solved again. Top-level statements now keep the ranges of names bound
above them; fn and impl bodies still see only declared ranges, since they may run anywhere.
Reverse: restore `var_range`'s one-name solve and the per-statement `Bounds` clone.

29.27. **A loop with no `break` leaves its condition false, and a fn's writes block narrowing.**
After `while w > 9:` with no `break` and no `else`, the statements that follow see `w <= 9`,
unless they assign `w`. A name that any fn or lambda body assigns is never narrowed, by a loop or
an `if`, since a call can change it between the test and the use. Reverse: drop the post-loop
`narrow` call, `breaks` and `Bounds::fn_writes`.

29.28. **An assignment in a `while` body reads the names its condition bounds.** `c += 1` under
`while c < n:` sees `c < n`, so `c` keeps `0..=n` instead of having no range. Only assignments
directly in the body, and only names no earlier statement there assigns. Hulls now grow until a
round of assignments stays inside them, jumping to the edge of `Int` after 64 rounds, then shrink
while each round stays inside: a guarded counter's first hull is wider than it needs. Reverse:
drop `guarded` and its lookup in `var_range`'s round.

29.29. **Completion offers stdlib names that need a `use`, and adds it.** Before, a name outside
the prelude completed only after its `use` was written, so the contestant had to know the module.
Now names of two or more letters complete from every stdlib module, ranked after what is in
scope, with the module shown and the `use` as an extra edit. The unresolved-name quick fix had put
its `use` at the start of the item holding the name, possibly mid-file; both now share
`cpl_check::import_edit`, which joins the module's existing `use` line or goes below the last one.
Reverse: drop the `STD_NAMES` loop in `scope_completion` and the fix rewrite in `Database`.

29.30. **Rename reaches files on disk that import the name.** A contest folder holds one solution
per problem, most of them closed, all importing the same `lib`. Renaming a library fn from one file
left the closed ones broken. Rename now also edits each `.cp` file of the project (the folder of
the nearest `cp.toml`, else the file's own) with a `use` line naming the old name, checked by the
same go-to-definition test as open files. The scan reads at most 5000 entries and skips hidden
folders, `target`, `dist` and `node_modules`. Reverse: drop `rename::project_files`.

29.31. **A comparison of two names bounds their difference.** Intervals alone give `n - a` in
`-8..=100` inside `if a < n:`, though it is at least 1. The bounds now keep, per pair of names, an
interval for `a - b` that a condition proved, and `a - b` is cut to it. A pair is forgotten when
either name is bound again, and never recorded for a name the branch or any fn assigns, so the
relation holds wherever it is read. Only a difference of two plain names reads it, which keeps the
analysis linear (no closure over chains like `a < b < c`). Reverse: drop `Bounds::diffs`.

29.32. **A `for` loop's head hovers how often it runs, with no warning.** `plan/60` asks for the
estimate and for a soft warning on likely TLE. The count is the size of the range or domain the
loop walks, times the counts of the `for`s around it; a `while` or a walk over a list makes the
total unknown, and a fn body counts from 1, since its callers are not followed. On the ports and
examples 12 of 1307 counted loops pass 1e10 in all, in solutions that pass their tests, so the
bound overstates the work there and a warning on it would be a false alarm; the warning waits for
a cost the checker derives. Reverse: drop `proven_loops` and `loop_hover`.

29.33. **A relation holds until a statement assigns one of its names.** 29.31 recorded no relation
for a name the block assigns anywhere, so a binary search's body, which always assigns `lo` or
`hi`, got none. A loop's condition holds at the top of each pass, so the relation is now recorded
regardless and dropped before the first statement that assigns either name (a compound statement
counts as a whole, so a nested loop that writes the name drops it before it is entered). `let m =
(x + y) / 2` relates `m` to both ends: `m - x` is half of `y - x` rounded down, `y - m` half rounded
up. Reverse: pass the block back to `relate` and drop `relate_midpoint`.

29.34. **Relations follow assignments, `if`s and `while` loops.** After `x = y + c` (also `x += c`),
`x - z` is `y - z` shifted by `c` for each relation of `y`. The branches of an `if` start from what
held before it, and after it each relation every branch (and the fall-through) proves holds, as their
hull; names a branch declares are left out. A `while` that compares two names its body assigns
starts from their difference on entry (a `var` holds its initial value until assigned), runs its
body again until that stops widening, at most three times, and keeps the result as an invariant
inside the loop and, with no `break`, after it, where the failed condition then narrows it: a
binary search ends with `lo - hi = 0`. A body with `continue` gets no invariant, since `continue`
returns to the condition without the rest of the body. A test that narrows a name now keeps its
relations. A difference that is a constant only because of a relation hovers as `= 0`. The extra
body passes cost at most four times a loop's analysis, more for nested searches; the latency gate
still passes. Reverse: drop `carried`, `join` and `loop_invariant`.

29.35. **Rename is refused when it would change what a name means.** Before answering, the server
applies the edits to every file it renames in, re-checks them, and asks go-to-definition again at
each use of the old and the new name; if any resolves to a different declaration than before (two
locals merged, `x` captured by a nearer `let y` once `x` becomes `y`, a global shadowed), the rename
is refused with the line of the first such use. With that check, the old blanket refusal of a name
already declared at the top level applies only when the renamed declaration is itself top-level, so
a local may take a global's name it doesn't use. Uses in files the rename doesn't edit are not
re-checked. Reverse by dropping `rename::changed_meaning` and the `top_level` condition.

29.36. **Hover shows why an index is in bounds.** `xs.len` becomes a name the relations track
(`xs.len`) when `xs` is bound once and never assigned, lent, passed `mut` or the receiver of a
method call anywhere in the file, so its length never changes; it is in `0..=i64::MAX`. Then
`var hi = xs.len` relates `hi` to it, `var (lo, hi) = (0, xs.len)` binds each name as its own `var`,
`let mid = (lo + hi) / 2` relates `mid` to whatever `lo` and `hi` are related to, and `for i in
0..xs.len` keeps `i < xs.len`. An index `xs[i]` with `i` at least 0 and `i - xs.len` below 0 hovers
the chain that proves it (`0 <= lo <= mid < hi <= ys.len`). Ranges a `.len` bounds only by that
type limit are not shown. Two changes came with it: a `while` invariant that still moves after two
rounds widens to what the names' values allow instead of being dropped, and a write in a fn to a
name that fn declared earlier no longer counts as a write some call could make, so a fn's own locals
get relations; an `if` test is read before the branches' assignments clear relations. Reverse by
dropping `Bounds::fixed`/`atom` (back to plain names), `in_bounds` and `index_hover`.

29.37. **A list's length is fixed once nothing changes it any more.** A sequence bound once whose
length some statement changes (`push`, assignment, `mut` argument, any method call on it) has its
`.len` tracked from the end of its last change, or of the outermost loop around that change which
its declaration is outside of, since the loop's next turn could change it again. A change inside a
fn or lambda body declared outside the sequence's own scope could happen at any call, so that
sequence gets no fixed length. A `.len` read before that point is not a name the relations know,
so no fact about it is made where it could still move: `xs[i]` followed by `xs.push` gets no proof,
even though the push comes after the use. Ports and examples with an index proof: 64 to 121, none
lost. Reverse by dropping `Bounds::stable` (back to sequences no statement ever changes).

## 30. Front-end recovery (2026-11-21)

- **28.8 `impl Trait for` a built-in type; `Hash` on it.** `impl Hash for Int:
  fn hash(self) -> U64 = ..` gives `Int` a method `x.hash()`, found by the
  value's type after `fields` unrolls or a generic is instantiated; before,
  an `impl` attached only to structs. `std.derive` uses it for a `Hash`
  trait over `Int`, `U64`, `Char`, `Bool` and `Str` (splitmix64) and
  `hash_fields`, so `impl Hash for P: fn hash(self) = hash_fields(self)` is
  a struct's whole derive. Reason: the derive must be library code, and a
  field-wise fold needs one method name that dispatches on each field's
  type; free fns cannot overload. Not yet: the compiler's own `HashMap` key
  hash does not call `Hash`. Reverse: delete `attach_impls_for_builtins`
  and `prim_methods` in `cpl-ir/src/lower` and the `Hash` part of
  `stdlib/derive.cp`.
- **28.9 A struct built field by field: `P(e for f in fields(..))`.** A
  struct constructor whose one argument is a generator over `fields` runs
  `e` once per field, in order, with that field's type expected, and
  builds the struct from the values: `P(read() for _ in fields(P))` reads
  one, `P(x + y for (x, y) in fields(a, b))` adds two. `fields(P)` names
  the type being built and binds nothing (`_`); values must be of that
  type. It is the construction half of reflection (28.1 walks fields,
  this makes them), so `Read` and field-wise maps are library code:
  `std.derive.read_fields<T>() = T(read() for _ in fields(T))`. A struct
  with a `new` still goes through `new`, so its invariants hold. Rejected:
  `T(..tuple)` spread (`..t` already means a range) and `fields(mut x)`
  (needs a starting value, and a struct has none). Reverse: delete
  `fields_ctor` in `cpl-ir/src/lower/call.rs` and `cpl-check/src/check/call.rs`.
- **30.1 A broken-ports benchmark scores recovery.** `crates/cpl-cli/tests/recovery.rs` applies six
  single-line edits (drop a closer, drop a block colon, cut a value, double an operator, garble a
  declared name) to every port and counts errors, cascades (every error after the first, since each copy
  has one root cause) and whether the first error is on the broken line. It gates no silent copies,
  first-error accuracy >= 99.5% and cascades per broken copy under a ratchet. The ratchet is a rate, not a
  count, because port lanes keep adding ports. *Reason:* "no cascades" needs a number.
  `RECOVERY_SHOW=<edit name>` prints every copy that cascades under that edit. *Reversible:* delete the
  test and its `scripts/check.sh` step.
- **30.2 Lexer diagnostics are pushed before the lexer's poison.** *Reason:* the poison exists to keep
  other errors off a reported span; applied first it also hid the report itself, so `let $x = 1` checked
  clean. *Reversible:* revert the sink order in `cpl-check/src/db.rs`.
- **30.3 The parser reads past an unreadable character inside a statement.** `bump` skips `T::Error`
  tokens that do not start a line and marks the statement recovering, so it is poisoned. *Reason:* the
  character is already reported; dropping the whole line also dropped the names it declares. A
  line-initial error token still drops the line, since nothing says what statement it was.
  *Reversible:* remove the loop in `Parser::bump`.
- **30.4 A function whose header has a syntax error stays bound with an unknown type.** `FileCtx::def`
  maps the defs of a statement poisoned on its first line through `Def::recovered`, which gives
  functions `Ty::Error`. *Reason:* a header missing its `)` parsed as a different signature
  (`fn ask(d: Char, dist: Int -> Int:` takes a `fn(Int) -> Int`), and every call reported a mismatch.
  Values keep their inferred type: a broken initializer already infers `Ty::Error` where it is broken,
  and `var total = 0 +` should still hover as `Int`. Struct and enum headers keep their parsed shape
  (open: a struct missing its `)` still reports every later field use). *Reversible:* drop the
  `head_is_poisoned` branch in `FileCtx::def`.
- **30.5 Planned syntax is one table, parsed and reported as `E0504`.** `cpl-syntax/src/planned.rs`
  lists each 0.3 form this version does not implement: its feature name, the design section that
  specifies it, what to write today, and an example whose one error is that one. The parser reads the
  form (so the statement and its names survive), reports "not implemented yet: <feature> (see
  <section>)" with the help line, and poisons the form's span. First entries: a type-parameter default
  (ignored) and `Str(len: ...)` (read as `Str`); `domain` in a binding pattern was an entry until
  the core lane implemented it the same day, and the entry went with it. *Reason:* one mechanism for
  every planned form instead of per-site messages; the generated golden test and the section-exists
  test keep each entry honest, and an entry is deleted when its feature lands. Forms that already
  parse (`repeat`, `loop`, one-line `enum`, unary plus, `var a = 0, b = 1`) need no entry.
  *Reversible:* delete `planned.rs` and the three `Parser::planned` call sites.
- **30.6 Golden diagnostics are rendered `cpl check` output.** `cpl-cli/tests/diagnostics/*.cp` with
  the exact stderr beside it in `.err`; planned-syntax examples are written there from the table.
  `BLESS=1` rewrites them. *Reason:* the rendered text is what the contestant reads, including help
  lines and spans, so that is what is pinned. *Reversible:* delete the directory and `diagnostics.rs`.

- **28.10 A struct's `?` borrows a field's niche; no `Niche` trait.** `?S`
  for a struct with an `Int` field (not narrowed below 64 bits) or a
  position field stores `none` as that field at `INT64_MIN` or -1, the
  value no `S` holds (decision 49.1), so `?Edge` is as wide as `Edge`: 16
  bytes for `Edge(to: Int, w: Int)`, not 24. The first such field is used,
  fixed per type (spec 22: niches are fixed per type, never per use). It
  majorizes `std::optional<S>` (narrower, same operations), so it is
  implicit. A user `Niche` trait naming a `none` value was rejected: every
  struct that has a free value already has one in a field, and a trait would
  let a library claim a value its own code can produce, turning a wrong
  `none` into silent corruption. Interpreter unchanged. Measured (`scoreboard.py
  --full`, 7 runs, same machine): `cf2147f` 604 -> 420 ms (2.84x -> 1.80x
  matched C++), RSS 24.2 -> 19.9 MB; `cf2057d` RSS 9.9 -> 8.7 MB. Emitted C++
  grows about 1.1 KB on the six ports that use it (the `OptField` runtime
  piece), so the baseline was rewritten. Reverse: drop the
  `Ty::Struct` arm of `Ty::Opt` in `cpl-codegen/src/names.rs` and the
  `niche_field` runtime piece.

- **28.11 A trait writes its own unimplemented message.** A call
  `f(x)` whose `T: Tr` bound `x`'s type has no `impl Tr for` now fails at
  the call, `` `Bool` does not implement `Tr` ``, instead of later inside
  `f`'s body as "no method"; a trait declared with
  `@on_unimplemented("...{T}...")` adds its text as the help, `{T}` the
  type shown (spec 14 writes `#on_unimplemented`; `@` is the one
  decorator sigil, CORE-ALIGN). Any library can declare it, so the stdlib
  has no privilege (`std.derive.Hash` uses it to point at `hash_fields`).
  Only traits declared in a module are checked; an `impl` over a generic
  type counts as met, so no valid program is newly rejected (291/291 ports
  pass `scoreboard.py --fast`). Test: `diagnostics/trait_names_its_own_fix`.
  Reverse: return `true` from `implements` in `cpl-ir/src/lower/mono.rs`.
- **28.12 `Gen` is a trait of static fns, and a generic call takes its
  return type from the type wanted.** `trait Gen: fn gen(mut r: Rng) ->
  Self` has no `self`; `T.gen(mut r)` inside `fn gen<T: Gen>` calls the
  `impl` of whatever fills `T`, built-in types included. A generic fn whose
  type parameter appears only in its return type (`fn gen<T: Gen>(mut r:
  Rng) -> T`) takes it from the expected type, so `let x: Int = gen(mut r)`
  and `P(gen(mut r) for _ in fields(P))` draw each field by its own type;
  before, such a parameter silently defaulted to `Int`. `gen_fields<T>`
  is then one line of `std.derive`, and `impl Gen for S: fn gen(mut r: Rng)
  -> S = gen_fields(mut r)` derives a generator for a stress test. Same
  seed, same values in `cpl run` and the C++ (`programs/derive_gen`).
  *Reason:* generators derived from types are the stress goal, and the
  missing pieces were general (static trait calls, return-type inference),
  not generator-specific. *Reverse:* drop `call_hint` in
  `cpl-ir/src/lower/mono.rs` and the built-in arm before `match
  self.sym(base)` in `Lowerer::method`.
- **28.13 `read()` of a struct calls its `impl Read`.** A struct whose
  `impl Read` declares `fn read() -> S` is read through it wherever `read()`
  meets it: alone, as an array element, as a field of another struct. The
  read plan (`Spec`) gains one node, `Spec::Call`, holding the lowered call,
  so the interpreter evaluates it and the C++ emits it like any expression.
  Before, the impl was accepted and silently ignored, and the input was read
  field by field (silent wrongness). Test: `programs/read_impl` (1-based
  input read as 0-based cells). *Reason:* spec 00 makes `Read` a library
  trait with return-type dispatch; this is the user half of that. `cpl
  stress` still generates such a struct's input field by field. *Reverse:*
  drop the `read` lookup at the top of `struct_spec` in
  `cpl-ir/src/lower/mod.rs`.
- **28.14 `??` stays core.** The §1 queue listed `??` for the stdlib on lazy
  parameters; it stays a core operator. Its right side may leave the
  enclosing fn or loop (`read_opt() ?? return`, `?? continue`), and a lazy
  argument is a closure, where `return` would leave only the closure. Spec 00
  places `??` in core C4 for exactly this ("laziness and `Never`"). A library
  `fn or<T>(x: ?T, lazy d: () -> T) -> T` would be a second spelling of the
  same idea with less power. *Reverse:* give lazy arguments non-local
  `return`/`break` (a core change), then define `??` as that fn.

- **30.7 Dreams are a diagnostics corpus.** `scripts/dream-gaps.py` checks every
  `proto/ports/*/dream/*.cp` and files each error under the gap it names: `syntax:` (an `E0504`
  feature), `library:` (a name, module or method "not in the stdlib (yet)"), `import:` (a stdlib name
  used without its `use`), `rule:` (a decided rule the dream breaks), `backend:` (an `E0501` the
  interpreter or codegen can't run yet), or `unnamed:`. The scoreboard
  prints the ranked summary. The bar is that every dream's first error names its gap. *Reason:* the
  factory needs to see which missing construct blocks the most dreams, and a dream whose first error
  is "expected X" teaches nothing. *Reversible:* drop the script and the scoreboard's `dreams` block.
- **30.8 A planned form outside the design still gets `E0504`, marked "(not in the design)".**
  Dreams reach for forms the design never specified (`fn (a: T) + (b: T)`, `by: <`, `Str of "01?"`,
  `enum E: a = -1`, `e is x`, `var a, b: T = 0`). Each is read, reported once by name with today's
  spelling as help, and recovered as today's nearest form. *Reason:* naming the feature is the useful
  message, and the marker keeps the table from passing these off as designed; a design section
  replaces the marker when the design takes the form up. *Reversible:* make `see` mandatory and turn
  those entries back into parse errors.
- **30.9 "Not in the stdlib (yet)" goes on names unknown everywhere, except one-letter names.** An
  unresolved name with no close in-scope or prelude name, no stdlib module and no removed keyword gets
  the suffix, plus a did-you-mean against stdlib names (with the `use` as a fix) when one is close; a
  missing `std.` module or a name a `std.` module lacks gets it too. No stdlib export is one letter
  and contest code is full of one-letter locals, so `no `w` in scope` stays bare. *Reversible:* the
  `unknown` condition in `Checker::unresolved` and the two branches in `db.rs`.
- **30.10 `const A = 1, B = 2` declares several constants; a field used in a method without `self`
  says so.** The first is how dreams write related limits, costs nothing to read (each becomes its own
  `const`, so every tool sees ordinary declarations); the second turns a bare "not in scope" into the fix (add `self`).
  Fields already resolve bare inside methods that take `self`. *Reversible:* the `while` after the
  first `const` in `parser/stmt.rs`; the `field_of_self` branch in `Checker::unresolved`.

- **30.11 C spellings (`&&`, `||`, `!x`, `..<`) are errors with the cp spelling as the fix, and parse
  as that spelling.** *Reason:* they are the commonest slips from C++ and Python, the fix is exact,
  and recovering as `and`/`or`/`not`/`..` keeps the rest of the file checked. A single `&` or `|`
  stays bitwise. *Reversible:* `Parser::c_logic` and `Parser::range_op` in `cpl-syntax`.
- **30.12 `cpl fix` applies the fixes errors carry first, then lint fixes.** It is the editor's
  quick fixes run in a loop (at most 8 rounds); once no error is left it also makes the fixes
  `cpl lint --fix` makes, which the lint codemods (L0204, L0415, L0510, L0511) document as `cpl fix`.
  Assigning to a `let` fixes the declaration (`let` becomes `var`), and only the first assignment to
  one `let` is reported. A plain number where a domain is needed gets `V(...)` as its fix. *Reason:*
  one command for every machine-applicable fix; lint fixes wait for a file without errors so a
  codemod never rewrites a program that does not check. The fix-it benchmark measures error fixes
  alone (`apply_error_fixes`). *Reversible:* `cpl_cli::apply_error_fixes` and `apply_lint_fixes`;
  drop the `ASSIGN_TO_LET` de-duplication in `db.rs`.
- **30.13 Fix-its are measured on a second benchmark of common mistakes.** The broken-ports edits
  (cut a value, drop a closer) have no single right fix, so `fix_it_scoreboard` plants one mistake
  per copy instead (C spellings, `let` for a `var`, a misspelled member) and counts errors with a
  fix and copies a `cpl fix` restores byte for byte. Ratchet: fix-its on at least 88% of errors and
  restoring at least 95% of the copies that report one. Only this benchmark applies every fix: the
  broken-ports scoreboard counts fix-its but skips the rewrite, which kept the recovery test inside
  the fast gate's time. *Reversible:* delete `fix_it_scoreboard`.
- **30.14 A domain or range mismatch labels where each side's index space comes from.** The
  message already names both spaces (`a` is indexed by V, but `n` is a 1..=9); a label on each plain
  name's declaration says where each got it (`a` is declared over V here, `n` is declared as a
  1..=9 here). A top-level name declared in another item is labelled as a point at its use, and the
  database moves the label onto the declaration. *Reason:* the fix is usually at the declaration,
  not the use. *Reversible:* `Checker::origin` and the `WRONG_DOMAIN` branch in `db.rs`.
- **30.15 After a lowering error, `cpl run` does not report names it does not know.** The checker
  has accepted every name by the time lowering runs, so an unknown name there means an earlier
  lowering error dropped its binding (`for (i, c) in s.item:` loses `i` and `c`). *Reason:* one error
  per root cause; this cut the member-typo benchmark from 656 to 522 errors. *Reversible:*
  `Lower::reported_an_error` in `cpl-ir/src/lower/mod.rs`.

## 31. Generated C++: stack, layout and niches (0.3)

Settled by Ryan for 0.3: the judge gives a big stack, the generated C++ runs
the program on a thread with the profile's stack, no recursion-depth
tracking in any build, large arrays never on the stack, `?Int` 8 bytes
everywhere. The entries below are the calls made implementing that.

31.1. **Calls are limited by the stack they fill, never by how deep they
nest.** The interpreter's 400,000-call limit and its depth counter are
gone; `cpl run` stops a program only when its calls would overflow the
interpreter's own stack, with `the stack ran out: `f` never stops
calling itself?`. The browser build's 1,000-call limit became a 3 MB
budget of the module's stack, halved on each retry exactly as the depth
was. Reason: a depth limit rejects correct programs the judge accepts (a
DFS down a path of 10^6 vertices), so `cpl run` and the judge disagreed
on valid input. Reverse: put `depth`/`max_depth` back in `Interp` and
`Limits`, and the check in `Interp::stack_left`.

31.2. **The interpreter reserves 8 GB of stack, falling back to 3 GB, 256
MB and 8 MB, and lets calls fill at most half the machine's memory.** A
call takes about 4 kB of interpreter stack (a 10^6-deep DFS: 4.05 GB RSS,
2.65 s), so 8 GB holds two million. The reservation is address space; only
the pages a recursion reaches are committed. A recursion without end fills
all it may use (16.7 GB in 9 s with a 16 GB budget, which got the test
killed on a loaded machine), hence the cap at half of `MemTotal`. Reverse:
`STACK_BYTES` and `memory_for_stack` in `cpl-interp`. The better fix is a
smaller interpreter frame; that is the interpreter lane's.

31.3. **`cpl-interp` is built with `opt-level = 1` in the dev profile.**
Unoptimized, a call took about seven times the stack, so a debug-built
`cpl` and the tests rejected recursion the release build runs. Cost: a
few seconds more on a clean debug build. Reverse: drop
`[profile.dev.package.cpl-interp]` from `Cargo.toml`.

31.4. **The C++ runs the program on a thread with the judge profile's
stack, whatever the program.** A judge profile's `thread_stack_mb`
(`cp.toml`) says how much; every built-in profile asks for 1 GB. `main`
asks for that, halving
down to 8 MB until a thread starts, and runs the program on its own stack
if none does (a judge that counts the reservation against its memory
limit, or one whose old glibc is linked without `-pthread`, where
`pthread_create` is a weak null). A profile without threads
(`thread_stack_mb = 0`, as on an olympiad's CMS) writes a plain `main` and
`cpl build` warns. This replaces the 256 MB inline-assembly stack switch
chosen by a recursion analysis, and with it 34.5: even a judge with a deep
stack of its own gets the thread, so a profile with a wrong or unknown
stack size cannot crash a deep program. Reason: the
analysis cannot see every deep path (a deep loop of non-recursive frames,
library recursion), the asm worked only on x86-64 Linux without ASan, and
the judge is assumed to give a big stack, so every program gets it. Cost:
about 700 B of source, only the judge's OS's threads (median +717 B over
246 ports, largest file 59,071 B; a hello world is 1,830 B) and one thread
start (0.06 ms a run); the scoreboard baseline takes the growth. Reverse:
`Target::thread_stack` in `cpl-codegen/src/lib.rs`, `THREAD_MAIN` in
`emit.rs` and `Profile::thread_stack` in `cpl-judge`.

## 32. Profiling and coverage (§8 tooling, 2026-11-21)

- **32.1 A profile counts statements, keyed by their span in the user's file.** `cpl profile` and
  `cpl cover` read one tally: how many times each statement began and the time until the next one began
  (a sampler's attribution). The interpreter keeps it in `cpl_interp::profile`; `cpl_codegen::emit_profiled`
  writes the same books into the C++ as a counter call before each statement, so the two agree on every
  count (`native_profile_counts_what_the_interpreter_counts_*`). Reason: one model serves both engines and
  coverage, and parity is testable. Reverse: drop `emit_profiled` and sample the native binary with `perf`.
- **32.2 Library code is the caller's line.** Imported modules' fns (`Program.library_fns`) and the
  closures they make are not counted; their time lands on the user's statement that called them, which
  is the line the contestant can change. Reverse: count them under their own module's file.
- **32.3 A statement inside an expression, or an `if` that is the next arm of a chain, is not its own
  site.** Statements of `ExprKind::Do` blocks are how an expression is lowered and the C++ may write it
  another way; an `elif` or `match` arm may become one `switch`. Counting them made the two engines
  disagree on 126 of 183 programs. Reverse: give lowered blocks their own spans and mark them in codegen.
- **32.4 Branch coverage comes from the same counts.** An `if` that ran and whose first statement ran
  every time, or never, went one way. No separate instrumentation. Reverse: count arms explicitly.

## 33. Fresh domains and existential returns (2026-11-21)

33.1. **Existential return syntax `-> (domain P, [P]Int)` and `let (domain P, part) = f(..)`**
uses `TypeKind::DomainBinder(Name)` in parenthesized tuple types and `PatternKind::Domain(Name)` in
patterns, alongside existing record-based `(new domain C, ...)` syntax. Reason: matches the 0.3
spec while keeping existing stdlib signatures (`components`, `scc`) working without breaking ports.
Reverse: unify record and tuple syntax under one AST form once all stdlib signatures migrate.

33.2. **One hidden runtime slot per fresh declaration site, saved and restored on function re-entry.**
Each `domain D = <expr>` declaration owns one `(lo, len)` slot in both `cpl-interp` and `cpl-codegen`
(generic functions specialize per domain and read the slot directly with no hidden witness parameter).
Shared `Program::fn_guarded_domains` collects local domains defined in a function body that do not
escape through its return type; `cpl-interp` saves/restores them on function entry/exit and
`cpl-codegen` emits an RAII `cpl::DomGuard` at the top of the function body. Reason: zero overhead for
generic code, correct recursion, escaping value-owned domains (`Compressed<D, T>`, `Rooted<V, P>`)
survive return, and identical semantics across interpreter and C++. Reverse: pass explicit `(lo, len)`
witness structs to generic functions if re-entrant closures ever need independent live instances of the
same declaration site.

33.3. **Fresh-domain escape checking (`E0313`) allows explicitly existential returns and
domain-carrying structs.** Returning a fresh domain value, assigning it to an outer variable,
storing it into an outer container (`.push`/`.insert`/`.set`/`.add`), or capturing it in a returned
closure is rejected with `E0313`, naming the domain and pointing to `-> (domain P, ...)` syntax.
User-defined structs parameterized by a fresh domain (e.g. `Compressed<D, T>`, `Rooted<V, P>`) pack
the domain existentially as a value-owned domain (`xs.D`, `t.Pre`). Reason: prevents dangling
index-space escapes while keeping value-owned stdlib domains (`compress`, `Rooted`) ergonomic.
Reverse: require explicit `domain D` field annotations on domain-carrying structs instead of
inferring from struct type arguments.

33.4. **Runtime-bounded fields in `read()` records (`w: 0..=m`) keep `Int` arithmetic and validate
against the runtime bound.** `let edges: [0..m](u: V, v: V, w: 0..=m) = read()` validates `0 <= w <= m`
in both interpreter (`R0001`) and C++ (`cpl::read_in(0, m)`), preserving `Int` arithmetic on `e.w`
without duplicating the maximum constant. Reason: resolves `research/frictions/cf2222f.md`.
Reverse: none needed; this is general range-type lowering.

## 34. Judge profiles (§9, 0.2.2, 2026-11-21)

- **34.1 A judge is a profile, in its own crate.** `crates/cpl-judge` holds the profile type, the
  built-ins (`generic`, `codeforces`, `atcoder`, `cses`, `icpc`, `kattis`, `dmoj`) and their selection.
  It depends on nothing, so the CLI, the wasm playground and the stress tools can share it; codegen takes
  only a `cpl_codegen::Target` (a header stamp and whether the judge's stack is deep), so it knows no
  judge by name. Reverse: fold the crate into `cpl-cli` and pass the limit alone.
- **34.2 The flag beats the file, which beats the project, which beats `generic`.** `--judge`, then a
  `// judge:` line (like `// limits:`), then `judge = "<name>"` under `[judge]` in the nearest `cp.toml`.
  `[judge.<name>]` tables change a built-in's fields or add one with `extends`, so a course's judge needs
  no code. Unknown names and keys are errors listing what is allowed. Reverse: drop a source from
  `cpl_judge::resolve`.
- **34.3 The default stays `generic`, the old behaviour.** `g++ -std=c++17 -O2`, an 8 MB stack and
  64 KB: the floor most judges clear, so a file built with no profile is what 0.2.1 wrote plus one header
  line. Reverse: change the `None` arm of `resolve`.
- **34.4 A profile records the judge's real command and says where each fact comes from.** Flags are
  copied from the judge's published command (AtCoder's library and link flags left out); a fact that could
  not be read from a public source is marked unconfirmed in `evidence`, shown by `cpl judges` (the
  Codeforces page sits behind a browser check). Reverse: none needed; correct a fact when a source says so.
- **34.5 (replaced by 31.4: every program runs on a thread with the profile's stack) Deep recursion skips its own stack only when the judge's stack is known to be deep.** At least
  256 MB or unlimited (`codeforces` links with a 256 MB stack, DOMjudge's runguard sets it unlimited); an
  undocumented stack counts as 8 MB. The switch is harmless where it is not needed, so the error is on the
  safe side. Reverse: `Profile::deep_stack` returning `false`.
- **34.6 The source limit is the profile's, with a warning above 75%.** The 48 KB warning of goal 5 for a
  64 KB judge, scaled. The size line names the judge and the share used. Reverse: `Profile::warn_at`.
- **34.7 Locally, a judge's flags are used as far as the compiler here accepts them.** Each profile's
  command is probed once (an empty program built with `-Werror`); a rejected flag is dropped or given the
  older spelling of the same standard, and reported. A flag and its value (`-x c++`) and a
  `--whole-archive` span are probed as one. A Windows profile builds with a MinGW cross compiler when one
  is installed. This tests what the judge's flags mean for the generated code without the judge's exact
  compiler version; containers with those versions remain open. Reverse: probe nothing and require every
  flag.
- **34.8 Every port under every profile's flags is a `--full` check.** 1,799 compiles take about two
  and a half minutes on eight cores, too slow for `--fast`; `--fast` builds every port for every profile (size, stamp,
  stack rule) and compiles and runs the deep-recursion program under each. Reverse: move the ignored test
  into `--fast`.
- **34.9 Local runs build a `.cp` file for its judge and run it on the judge's stack.** `run --release`,
  `test`, `stress` and `hunt` resolve the file's profile, emit for it, build with its flags as the host g++
  accepts them (the host compiler for a Windows judge) and wrap the binary in `sh -c 'ulimit -s N'` when N
  differs from the current limit; a stack the shell can't grant fails with exit 125 and names the judge.
  Goal 5 says local runs mimic the profile, and the stack is where they differed most (a recursion that
  fits 8 MB here could overflow a judge, and the reverse). The `g++ -std=c++17 -O2` build keeps its
  precompiled header; other flag sets build without it. Reverse: `build_judged` back to
  `compile_cpp(&emit_cpp(..))`.
- **34.10 The size report is measured on the file, by the code generator's own parts.** The emitter keeps
  the text of each part it writes (the header, each runtime piece, the types, the globals, each fn, the
  top-level code), the report measures it as the file has it (narrowed when the file is) and calls the rest
  glue, so the parts add up to the file by construction. A fn is named as its source names it (`Rooted.path`),
  its instances summed. Shown on `--size` and on every build over the limit, the one time it is needed.
  Reverse: drop `Emitter::chunks` and `cpl_codegen::sizes`.
- **34.11 `cpl profile --native` builds for the file's judge.** It resolves the profile like `cpl build`,
  writes the counted C++ for that target (its thread stack, its OS's threads), builds with
  the judge's flags this compiler accepts and runs on the judge's stack, the same `Profile::on_stack` command
  the stress runner uses; a profile timed on another build can rank the lines differently. `CXX` still
  overrides the compiler. Reverse: `emit_profiled` without a target and `g++ -std=c++17 -O2`.
- **34.12 `cpl test` gives MLE past the problem's memory limit.** A case whose sampled peak memory
  (`VmHWM`, as `bench` reads it) is over the limit is `MLE` whatever it printed, as a judge stops it there;
  a time limit outranks it. Sampling can miss a peak in the last 0.3 ms of a run, so an MLE is certain and an
  `ok` is not proof. Reverse: drop `Verdict::MemoryLimit` and the check in `run_cases`.
- **34.13 The comments on the C++ give way first, before the source in the header.** A file over its
  judge's limit loses, in order, its comment lines (all but the header at the top; a comment after code
  stays), then the source in the header, then half its indentation. The contestant's own source says more
  on the judge's page than the generator's notes on its runtime, and both cost the same to the judge.
  `Fitted::note` says what was left out, for the CLI and the playground alike. Reverse: drop the
  `uncommented` rung in `emit_within_for`.
- **34.14 `cpl submit` is `cpl build` then `cpl test`, and sends nothing.** It takes no `--judge`: the file
  or project names its judge, so the build and the cases cannot disagree on one. No cases is a warning,
  not a failure, since many problems are tried before their samples are saved. It sends nothing because
  no judge we target offers a submission API it allows scripts to use, and a scraped login is a credential
  in a tool that must not hold one. Reverse: drop `submit` from `cpl-cli`.

## 35. Taste lints (§8 tooling, 2026-11-21)

1. **The `plan/PORTING.md` taste checklist is machine-checked as lints
   L0501–L0506** (`search_loop`, `discarded_result`, `symmetric_write`,
   `loop_reduction`, `index_only_loop`, `repeated_literal`), read off the
   syntax tree like the other lints. Reason: ports passed every gate and still
   read like translated C++; a lint names the construct at the place.
   Reverse: drop `rules/taste.rs` and the six `Lint`s.
2. **A fix-it only for the exact shape, and never when it could change
   meaning:** not when the loop holds a `?` (it would leave the
   comprehension, not the fn), when the file declares a fn of the
   reduction's name, when the accumulator starts from a non-number (`sum`
   of strings), or for `max`/`min` (empty has no value). The rewrite keeps
   the declaration's type and edits only its initializer. Applying every fix
   to the corpus changes 136 files and breaks none that checked before.
   Reverse: widen the shapes in `exact_search` and `reduce`.
3. **`repeated_literal` counts lines, not occurrences** (`-1e9..=1e9` is one
   fact), and fires for a value of at least 1e6 on two lines or at least
   1000 on three. Reason: bounds and moduli are big; small numbers repeat
   honestly. Reverse: change the thresholds in `repeated_literals`.
4. **Two friction detectors become lints:** `push_after_literal` as L0507
   `push_loop` (with a fix to a comprehension, `head + [..]` after a
   non-empty start) and `none_check_unwrap` as L0508 (no fix: the right
   rewrite, `if let` or comparing the option, depends on the use). They
   are taste lints too: the checklist's hand-rolled structures and
   sentinels. Reverse: drop them from `LINTS`.
5. **`repeated_literal` skips the types of `let x: T = read()`:** equal
   statement bounds of different inputs are separate facts. The first
   30-hit sample had 6 false positives, all such bounds; after the
   change a second sample of 30 had 1 (a loop with side effects read as
   a search). Reverse: drop `input_types` in `repeated_literals`.
6. **`cpl lint --taste` reports only L05xx, a line per file and a total;
   the scoreboard counts it per port apart from `lint_warnings`,** so a
   taste finding is a review signal and never fails the gate. Reverse:
   fold `taste` into `lint_warnings` in `scripts/scoreboard.py`.
7. **The friction detector `negative_slice` gets no lint:** `a[..-k]` is
   already the check error E0309 with the same fix, so the friction script's
   one `wa_trap` is stale. Reverse: none needed.

## 36. Symmetric storage, derived domains, and domain iteration helpers (0.3, 2026-11-21)

- **36.1 Symmetric 2D tables are a storage dimension `Sym<V>`, not a new container type.**
  `[Sym<V>]T` packs an unordered pair `(u, v) : (V, V)` into a flat triangular array of size
  `n * (n + 1) / 2` via `sym_idx(a, b) = max(a, b) * (max(a, b) + 1) / 2 + min(a, b)`.
  Both `adj[u, v]` and `adj[(u, v)]` index it, with `adj[u, v]` and `adj[v, u]` aliasing the
  same cell. Diagonal cells `(u, u)` are included so `Sym<V>` works uniformly for both graphs
  with self-loops and reflexive relations (costs `n` extra cells over strict `n*(n-1)/2`, avoiding
  a trap on `adj[u, u]`). Reason: eliminates duplicated symmetric writes (`cf2150f`) and halves
  memory while keeping the exact `[Dim]T` array model, formatter, linter, and trace display.
  Reverse: remove `Dim::Sym`/`DimSpec::Sym` and `Builtin::SymIdx` (`cpl-check`, `cpl-ir`,
  `cpl-interp`, `cpl-codegen`).
- **36.2 Domain-typed iteration helpers `V.after(i)`, `V.before(i)`, and `V.pairs`.**
  `V.after(i)` and `V.before(i)` return `Range<V>` covering `(i.to_idx + 1)..V.count` and
  `0..i.to_idx` respectively, and lower directly to `Iter::Range` in `for` loops and comprehensions
  so they emit zero `cpl::Range` runtime overhead. `V.pairs` yields unordered distinct pairs
  `(i, j) : (V, V)` with `i < j`: in `for (i, j) in V.pairs` (without `break`) it lowers directly
  to two nested `Stmt::For` loops with zero allocation; as a value or with `break` it materializes
  `List<(V, V)>`; and `domain P = V.pairs` declares a derived domain of size `n * (n - 1) / 2`.
  Reason: mined from `cf2150f`, `cf2147h`, `cf2129b`, and `proto/ports/ecnerwala/dream/` where
  ports repeatedly wrote `for j in V if j > i:` or `0..i` / `i + 1..n`.
  Reverse: remove the domain member arms in `cpl-check/src/check/expr.rs` and `cpl-ir/src/lower/{call,expr,stmt}.rs`.
- **36.3 Derived product domains (`domain S = A * B`, `domain E = edges(V)`) remain staged behind `Sym<V>` and `[A, B]T`.**
  Multi-dimensional tables `[A, B]T` and `[(A, B)]T` already cover dense product indexing, and
  `domain P = V.pairs` covers unordered pair domains. General algebraic product domains
  (`domain State = (V, K)`, `research/frictions/cf2178h.md`, `dream/cf2178h.cp`) and graph-edge
  domains (`domain E = edges(V)`) need bijective packing/unpacking (`Pos<A * B> <-> (A, B)`)
  and are queued as the next step in `plan/DOMAINS.md`. Reason: keep commits small and avoid
  shipping half-baked product destructuring. Reverse: implement product-domain packing in `cpl-ir`
  and `cpl-check`.
## 37. Taste review gate (2026-11-21)

37.1. **Ports newer than the dream-first rule need a dream and a review; old ports are a
baseline.** `research/reviews/baseline.json` lists the 261 ports that predate `plan/PORTING.md`;
`scripts/review-check.py` (in `scripts/check.sh --fast`) fails on any later port without
`proto/ports/<group>/dream/NAME.cp` or `research/reviews/NAME.md`, and reports the median score,
the count under 4 and the port/dream token ratio. The count of later ports scoring under 4 is
ratcheted (`new_under_4_max`), and the ratchet only blocks commits that add a port, so a review
that lowers a score never breaks main. Reason: taste was self-reviewed and nothing checked it.
Reverse: delete the step from `scripts/check.sh`.
## 38. Geometry standard library (`design/stdlib/geometry.md`, 2026-11-21)

- **38.1 Two distinct point types: `Pt` (exact `Int`) and `Vec` (careful `F64`).**
  Integer and floating-point geometry have different safe operations: `Pt` has exact equality,
  lexicographic `Ord`, `Hash`, `Show` (`"x y"`), `unit_dir`, and epsilon-free predicates; `Vec` has
  `unit`, `scale`, `div`, `project`, `reflect`, epsilon-parameterized predicates, and omits `Show`
  to respect decision 1.5 (no bare float printing). Separating them keeps `Pt` non-generic in
  `std.prelude` (no default type parameters needed) and makes the exact-vs-approximate boundary
  visible in the type (`p.to_vec`). Reverse: unify into `Pt<T = Int>` if default type parameters
  and conditional impls land.
- **38.2 Exact orientation predicates compute cross and dot signs in `I128`.**
  `dot` and `cross` stay `Int` for arithmetic ergonomics, with `dot128`/`cross128` alongside;
  `turn`, `ByAngle`, `ByAngleFrom`, `angle_less`, `same_dir`, `on_segment`, `segments_meet`,
  `locate`, `locate_convex`, hulls, and Minkowski sums compute orientation signs in `I128`,
  eliminating silent 64-bit overflow up to `1e18` coordinates at negligible cost (`imul` 64x64->128).
  Reverse: switch internal helpers back to `Int` cross/dot.
- **38.3 Options as named defaults and O(n) chain builders.**
  `segments_meet(..., strict: Bool = false)` and `convex_hull(..., collinear: Bool = false)` avoid
  suffixed variants. `lower_hull` and `upper_hull` take already `(x, y)`-sorted input in O(n) so
  divide-and-conquer hulls (`ucup4s25e`) stay O(n log n) alongside `minkowski_upper`. `locate` and
  `locate_convex` return `Location` (`Inside | Boundary | Outside`) rather than `1 / 0 / -1`, and
  `Vec.rot(by: Vec)` uses a named option instead of a `_by` twin (coordinator round, §2).
  Reverse: return `Int` or add a separate method.

## 39. Strings stdlib (`std.strings`, 2026-11-21)

- **39.1 Alphabet-domain automata (`Trie<A>`, `AhoCorasick<A>`, `SuffixAutomaton<A>`, `PalindromicTree<A>`).**
  String automata parameterize over an alphabet domain `A` (such as `'a'..='z'` or `0..2`) rather
  than hardcoding 26 lowercase letters. Reason: binary/digit/general alphabets avoid wasted memory
  and `c - 'a'` arithmetic while keeping `[A]?Int` / `[A]Int` inline and bounds-checked at domain
  entry (`design/stdlib/strings.md` §2). Reverse: add a concrete `Str`-specialized wrapper if ports
  show repeated `[Letter(c) for c in s]` boilerplate.
- **39.2 Layered suffix arrays: O(n) free functions plus O(n log n) `SuffixArray<T>`.**
  `suffix_array(s)` and `lcp_array(s, sa)` stay O(n) SA-IS + Kasai with no hidden RMQ build;
  `SuffixArray<T>(s)` builds `sa`, `rank`, `lcp`, and a sparse table for O(1) `lcp_between` and
  `compare`, O(|pat| log n) `matches`/`count`, and O(log n) `lcp_range`. Reason: visible costs
  (no secret O(n log n) work behind `suffix_array`) while eliminating repeated binary search and
  RMQ boilerplate (`research/friction-backlog-batch-10.md` T4). Reverse: drop `SuffixArray<T>`.
- **39.3 `manacher(s)` returns `Palindromes(odd, even)` with O(1) `is_palindrome(l, r)`.**
  Keeping `odd` and `even` as public fields preserves direct radius access while adding
  `is_palindrome(l, r)`, `count`, and `longest` so callers never re-derive the center formula.
  Reason: eliminates a classic off-by-one bug class. Reverse: return a plain tuple `(odd, even)`.
- **39.4 `Hash2D<R, C>` returns double-mod `HashVal`, not a single-mod `Int`.**
  Promoted from `examples/local-lib/str/hash2d.cp`, upgraded to the same double-mod `HashVal` as
  `RollingHash` so 2D hashes are anti-hack safe on Codeforces. Reason: single 31-bit hashes collide
  easily at contest grid sizes. Reverse: revert to a single `Mod<2147483647>`.

## 40. Number theory, combinatorics, NTT, and linear algebra (`design/stdlib/math.md`, 2026-11-21)

- **40.1 `Mod<P>.comb(limit = 0)` ties `Comb<P>` to `Mod<P>` without repeating `P`.**
  After `type Z = Mod<998_244_353>`, `let comb = Z.comb(n)` constructs `Comb<998_244_353>(n)`
  directly, keeping the modulus in one declaration (`research/frictions/cf2084e.md`,
  `research/frictions/cf2150g.md`). `Comb<P>(limit)` stays valid. Reverse: remove `Mod.comb`.
- **40.2 `cyclic: Bool = false` is a named option on `convolve`, `convolve_mod`, and `convolve_int`.**
  Avoids separate `cyclic_convolve*` names and follows `plan/STDLIB.md` §2; bitwise convolutions
  (`convolve_or`, `convolve_and`, `convolve_xor`) and `sos_sub`/`sos_sup` live in `std.ntt`
  alongside NTT so all convolution transforms share one module (`research/frictions/cf2201e.md`).
  Reverse: split bitwise transforms into `std.sos` or add `cyclic_convolve` wrappers.
- **40.3 `sieve_multiplicative` takes `(n, one:, mul:, prime_pow:)` explicitly.**
  Taking `mul: (T, T) -> T` works uniformly for `Int`, `Mod<P>`, and user structs without needing
  new core trait constraints, and runs in strict `O(n)` by tracking each number's largest
  prime-power divisor (`research/frictions/cf2135e2.md`). Reverse: specialize on `T: Mul`.
- **40.4 Exact field elimination lives on `Mat<T>`; pivoted `F64` elimination is `*_real(..., eps: 1e-9)`.**
  Floating-point Gaussian elimination requires partial pivoting by `abs` and a tolerance `eps`,
  whereas exact elimination over `Mod<P>` checks `== T.zero`. Keeping them separate avoids hidden
  `F64` specialization and respects CP Lang's default rejection of float equality. `XorBasis`
  lives in `std.matrix` with a `Monoid` combinator for segment trees. Reverse: unify behind a
  `Field` trait once traits support associated pivoting policies.

## 41. `std.seq` complete surface (`design/stdlib/seq.md`, 2026-11-21)

- **41.1 Sequence functions take `self: [D]T` whenever they read by index**, so one
  definition covers `List<T>`, `[V]T`, and `Str` with zero runtime dispatch
  and preserves `D` in return types (`?D`, `Range<D>`, `[D]?D`, `[D.bounds]T`).
  Reason: `runs` and `pairs` typed on `List<T>` forced manual loops on `Str`
  (`cf2084h`, `cf2069e`) and dropped domains on `[V]T`. Reverse: narrow
  signatures back to `List<T>`.
- **41.2 `chunk_by` and `run_ranges` yield `Range<D>` instead of copied `List<T>`
  slices.** Reason: `runs_by` allocated a `List<T>` per run and ran 50x slower
  than a manual loop on `cf2135f`; yielding `Range<D>` is O(1) extra space and
  zero allocations while giving `r.len`, `a[r.first!]`, and `for i in r:`.
  Reverse: return materialized slices once borrowed slice views exist.
- **41.3 `prefix_fold` and `suffix_fold` return `[D.bounds]T`** (cut-indexed),
  while `scan` and `suffix_scan` return `[D]T` (element-indexed, no identity).
  Reason: matches `std.prefix.Prefix1` and eliminates `.ord` / `+ 1`
  bookkeeping for outside-range queries `op(pref[r.lo], suff[r.hi])`
  (`cf2062e2`). Reverse: drop `prefix_fold`/`suffix_fold` or return `[D]T`.
- **41.4 General two-pointer loops stay user code; `std.seq` provides only
  fixed-window `sliding_min`/`sliding_max` and monotone-stack
  `prev_less`/`next_less`/`prev_greater`/`next_greater` (returning `[D]?D`).**
  Reason: arbitrary two-pointer state is clearer as a 5-line loop than a
  3-callback combinator, while sliding extrema and nearest-smaller/larger are
  universal, sentinel-prone patterns. Reverse: add or remove individual helpers
  in `stdlib/seq.cp`.

## 42. Range structures stdlib (`design/stdlib/ranges.md`, §4.10, 2026-11-21)

- **42.1 `DualSegTree<D, T, F>` is its own type, not a mode on `LazySegTree`.**
  Range-apply + point-get needs no fold monoid, no `pull`, and no `span`
  array (`cf2255e2` friction). Putting it on `LazySegTree` forces a fake
  monoid and wastes half the work. It pushes top-down by default so
  non-commutative tags are always sound. Reverse: fold into `LazySegTree`.
- **42.2 `DualFenwick<D, T>` and `RangeFenwick<D>` are separate types from
  `Fenwick<D, T>`.** Adding `range_add` onto `Fenwick` would let a user
  call `sum(r)` on a difference-array Fenwick and get nonsense. Separate
  types make the invariant unbreakable. Reverse: drop the types and add
  a helper method.
- **42.3 `fill: ?T = none` is a named constructor argument on `SegTree.new`,
  `LazySegTree.new`, and `DualSegTree.new`.** Follows the named-option
  rule instead of adding `.filled(...)` variants, and avoids allocating a
  throwaway `[D]T` array for constant-initialized DP trees (`cf2207f`
  friction). Reverse: remove the parameter.
- **42.4 `prefix_fold` and `suffix_fold` return `[D.bounds]T`, not `List<T>`.**
  Querying everything outside `r: Range<D>` is `m.op(pref[r.lo],
  suff[r.hi])` with zero `.ord` / `.ord + 1` arithmetic (`cf2062e2`
  friction). Coordinates with 41.3 so the pair lives in one module
  (`std.prefix` or `std.seq`, not duplicated). Reverse: return a wrapper struct.
- **42.5 `MaxSub` lives in `std.algebra` and implements `Reversible`.**
  Kadane's 4-field summary is the most repeated custom monoid in the
  corpus (`cf2255e2`), and `Reversible` makes tree-path max-subarray
  queries work with `Rooted.fold_path`. Reverse: move to a separate
  module or local-lib.
- **42.6 Algebra stays value-based (`Monoid`, `Action`); static dispatch is
  left to the core lane.** Moving `Monoid`/`Action` to traits waits on
  associated constants (decision 26.4), and removing `std::function`
  overhead is a general closure-monomorphization pass in `cpl-codegen`,
  not a stdlib-specific hack. Reverse: switch to traits once associated
  constants land.

## 43. Flows and matching (`std.flow`, 2026-11-21)

The area's design is `design/stdlib/flows.md`; these are its calls.

- **43.1 Two networks, one arc vocabulary.** `FlowGraph<V>` (Dinic) and `CostFlow<V>`
  (successive shortest paths with potentials) stay separate types sharing `Arc` and the
  accessors `flow`, `cap`, `source`, `target`. The cost solver is `min_cost_flow`, not
  `flow(s, t)`, because `flow(a)` already means an arc's flow. Reason: one name, one meaning;
  their solvers have different cost contracts. Reverse: merge into one type with an optional
  cost per arc.
- **43.2 Objectives are named, never negated.** `min_weight_matching` exists beside
  `max_weight_matching`; bounded flows are `max_bounded_flow`/`min_bounded_flow`. Reason:
  `research/frictions/cf2062g.md`, where a minimum was written as `-max(-w)`. Reverse: keep one
  of each pair.
- **43.3 Reductions are functions over the user's domains.** `max_closure`, the matchings,
  bounded flows and `gomory_hu` take domains and edge tables (`[E](u: V, v: V, ...)`) and build
  their network, source and sink inside a fresh domain. Reason: the corpus flattens
  `Item + Pair + Src + Snk` into one numbered domain (`noi06p4` K11, `dmopc20c7p5`,
  `noi09p4`); the general fix is sum domains (spec §21.11), a core feature, and until it lands
  the library keeps the extra vertices out of user code. Reverse: once sum domains land these
  stay; nothing to undo.
- **43.4 `cap: none` is 2^60.** An unbounded arc is a capacity no finite cut picks, with room
  for eight of them in a sum before 63-bit overflow. Reason: matches the shipped `FlowGraph.add`;
  a true infinity would need a sum type in the inner loop. Reverse: a `Cap` enum.
- **43.5 Min-cost flow returns to `std`.** `CostFlow` replaces `examples/local-lib`'s
  `MinCostFlow`, reversing its 0.2.2 relocation (section 19); `max_matching` and the
  bounded flows move up with it. Reason: the area survey (design section 1) puts min-cost
  flow beside Dinic in every contest library (AtCoder Library's `mcf_graph`, KACTL), and a
  second network type outside `std` split the `Arc` vocabulary. Reverse: move `CostFlow`
  back to the local library.
- **43.6 `global_min_cut` is dense Stoer-Wagner.** O(V^3) time and O(V^2) memory over a
  `List` matrix, not the O(VE + V^2 log V) heap form. Reason: global-cut problems have
  V <= ~500, where the dense scan beats the heap and has no `Int.MAX` sentinel trick
  (merged vertices are dropped by an `alive` table). Reverse: switch the phase scan to
  `MinHeap` if a port needs sparse V in the thousands.
- **43.7 `max_weight_general_matching(edges)` is the dense O(V^3) weighted blossom.**
  It takes `[E](u, v, w)` like `gomory_hu` and returns `(total, mate: [V]?V)` like
  `general_matching`; it maximizes weight, not size, so edges with `w <= 0` are dropped.
  No `perfect:` option and no `min_` twin yet: min-weight perfect matching is the same call
  on `big - w`, and no corpus port asks for it. Reason: contest instances have V <= ~500,
  where the (2V+1)^2 edge table is a few MB and beats sparse forms. Reverse: add
  `perfect: Bool = false` (labels start at `big`), or a sparse priority-queue variant if a
  port needs V in the thousands.
- **43.8 `std.flow` hot loops run in `unchecked:`.** Dinic's BFS and path search,
  `CostFlow.slope`, Hungarian, Hopcroft-Karp, `general_matching`'s search and the blossom
  slack scans index arc and label tables by ids the library itself created, so the checks
  never fire; dropping them takes min-cost flow from 1.41x to 1.14x matched C++. Reverse: remove the `unchecked:` lines if a
  check-elision pass proves these ids in range.
  Amended: `unchecked:` drops every index-validity check in the C++, which includes the
  `pos` check of a domain position made by `V.nth(i)` or `V(label)`, as well as list and
  array bounds. It keeps the check on a store into a range type (`x: 0..=100 = i`), since
  that guards a narrowed width and skipping it would truncate silently rather than crash.
  `cpl run` keeps both checks. Hungarian's 2.25M-step scan pays a `pos` call per step
  otherwise (39 ms of 373 ms). Reverse: drop the `Raw` wrapper in `vouch_pos`.
- **43.9 `FlowGraph.min_cut(s)` traps unless a completed `max_flow(s, t)` came since the
  last `add` or `reset`.** One `?V` field records it; reading the cut after a `limit:` stop,
  an edit, or from another source was silently wrong before. No `min_cut(s, t)` returning
  `(value, side)`: no corpus port reads an s-t side (cf1404e, dmopc20c7p5, noi06p4 print the
  value), so it would only rename `max_flow`. Reverse: drop the field, or add the pair form
  when two ports read the side.

## 44. Containers area design (§4.8–4.9, 2026-11-21)

- **44.1 `SortedBag` cursors are read-only opaque handles (`BagCursor`) with O(1) amortized
  `next_cursor`/`prev_cursor`, plus `entries()`/`entries_rev()` generators.**
  `SortedBag` adds parent pointers and a recycled free list so removed nodes do not leak RSS
  (`cf2066e.md` #1, #2). Cursors stay read-only rather than supporting erase-by-cursor because
  treap split/merge invalidates structural paths and O(log n) erase by key is already fast;
  the hot path in `cf2066e` is O(1)-per-step local walking (`entries_rev` + `prev`/`next`).
  `OrderedSet<K>` is its unique-key twin with `rank`, `count_range`, and `first_where` (`cf2262b.md`).
  Reverse: drop `BagCursor` and parent pointers from `stdlib/bag.cp`.
- **44.2 `IntervalMap<D, T>.assign` returns `List<(range: Range<D>, val: T)>`, not a `yields`
  generator.**
  Returning the overwritten runs guarantees the maximal-run partition invariant always holds even
  if the caller `break`s or `return`s early, and costs zero extra asymptotic time because total
  overwritten runs across `q` calls is O(n + q). Built directly over `Range<D>` so callers stay
  in their nominal domain (`tree.Pre`, `cf2237h.md`).
  Reverse: change `assign` to `yields` once mutating generators guarantee post-break cleanup.
- **44.3 `Bitset.subset_sums<D>` uses binary-carry grouping; `shift_or` mutates in place.**
  Naive `reach.union(reach.shifted(w))` allocates per item and hits a 10x TLE on duplicate
  weights (`cf2187d.md`). `shift_or(k)` eliminates per-step allocation for custom shift DPs,
  and `subset_sums<D>(weights)` provides the O(K sqrt(S) / 64) binary-carry knapsack in stdlib.
  Reverse: remove `subset_sums` or `shift_or` from `stdlib/bitset.cp`.
- **44.4 `PotentialDsu<V>` and `RollbackArray<D, T>` share `std.dsu.Mark`.**
  All rollbackable structures in `std.dsu` use the same nominal `Mark` handle and `mark()`/
  `undo_to(snap)` protocol so multi-structure divide-and-conquer rollback is uniform.
  Reverse: move `PotentialDsu`/`RollbackArray` back to `examples/local-lib/ds/`.
- **44.5 `FlatMap<V>` in `std.hashmap` is an explicit opt-in open-addressing `Int` hash table.**
  Builtin `HashMap<K, V>` remains the default for arbitrary keys; `FlatMap<V>` uses per-instance
  salted `splitmix64` and backward-shift deletion for low RSS and cache locality on large `Int`
  key sets without silently substituting for `HashMap`.
  Reverse: remove `stdlib/hashmap.cp`.

## 45. Trees (`std.tree`, design/stdlib/trees.md, 2026-11-21)

1. **One `Rooted` value stays the hub for every static-tree technique**:
   subtree ranges, LCA, k-th ancestor, heavy-light paths, rerooting and
   small-to-large all read the same heavy-first preorder `t.Pre`. No `Hld`,
   `EulerTour` or `Lca` type. Reason: one order makes subtrees and chains
   ranges at once, so one range structure serves both. Reverse: split the
   path operations into their own struct.
2. **`up` and `reroot` share one signature `(m, lift, finish)`; `down`
   replaces the preorder-and-parent loop.** Promoting a rooted DP to all
   roots is a one-word edit, and `reroot` uses prefix and suffix sibling
   folds, so the monoid needs no inverse. The cf2062e2 dream's `top_down`
   returning `(out, state)` pairs is not added: `down` returns the state per
   vertex and the output is read off it. Reverse: add `top_down`.
3. **Small-to-large gets primitives, not a higher-order `dsu_on_tree`:**
   `heavy`, `light` and `subtree`. The keep/clear order is the algorithm and
   stays in user code. Reverse: add the wrapper over these three.
4. **`Centroids` is its own struct built from adjacency**, promoted from
   `examples/local-lib/graph/centroids.cp`, because its parent is not the
   rooted parent. `ancestors(v)` (with distances) and `piece(c)` (with the
   branch) are the two shapes every centroid problem uses. Reverse: keep it
   in the local library.
5. **No `toward`/`next`:** `step(a, b, 1)` already names the neighbour of
   `a` on the way to `b`. Reverse: add the alias.
6. **LCA stays chain jumping** (O(log n), O(n) memory) rather than an Euler
   RMQ (O(1), O(n log n) memory) until a matched benchmark shows it losing.
   Reverse: add an RMQ behind `lca` if `bench/trees` says so.
7. **Dynamic trees are a separate `LinkCut` with `Rooted`'s verbs**
   (`fold_path`, `lca`), after the static API matches C++. Reverse: drop it
   from the plan. Landed before the static API matches C++ after all, since
   `LinkCut` shares no code with `Rooted`; `lca` takes the root explicitly
   because every query re-roots, and reads take `mut self` because splaying
   is how they stay O(log n).
8. **`heavy`, `far` and `at` stay indexable fields**, not methods as the
   first draft proposed. Ten corpus ports index them (`t.at[p]`,
   `t.heavy[v]`), the read is O(1) either way, and a field and a method
   cannot share a name. Reverse: rename the fields and add the methods.
9. **`Centroids.ancestors` yields instead of returning a list**: it is read
   once per query in the hot loop, and building the list cost 0.5 s of
   4.6 s at n = 10^6 in `bench/trees`. Reverse: return `List`.

## 46. Graphs stdlib design (`design/stdlib/graphs.md`, 2026-11-21)

1. **Searches return `Paths`, not a bare distance array.** `bfs`, `bfs01`,
   `dijkstra`, `fixed_cost_paths` and `bellman_ford` return
   `Paths<V, W, A>`: `p[v]` is the distance (so existing `dist[v]` code reads
   the same), and `parent`, `via`, `order` and `path(t)` answer the
   follow-up (print the path, DP over the shortest-path DAG) without a second
   search. Reason: the local `shortest`/`walk_back` pair and every port that
   printed a path re-derived parents by hand. Cost: two more arrays of n and
   one store per settled vertex, measured in `bench/graph/`. Reverse: return
   `.dist` and drop the struct.
2. **0-1 BFS and fixed-cost searches are named algorithms** (`bfs01`,
   `fixed_cost_paths`), not a `dijkstra` option. Reason: AGENTS.md "name the
   algorithm"; the deque and the per-move FIFOs have different costs and
   different preconditions (weights in {0, 1}; fixed non-negative move
   costs), and each traps when its precondition fails. Reverse: fold into
   `dijkstra(.., via: Deque)`.
3. **Unit and 0-1 searches take `from: List<V>`; weighted ones take
   `from: List<(V, W)>`.** The starting distance is the super source without
   an extra vertex; a unit search has nothing to offset. Reverse: accept
   both shapes everywhere.
4. **`euler_walk`, returning `Walk<V, E>` (start vertex plus arcs).** The
   local `euler_path` returned bare edge indices, losing the direction an
   undirected edge was walked. `Walk` is the same arc shape a `Graph` yields,
   so `w.vertices` and `w.arcs` need no translation. Reverse: rename back
   and return `List<E>`.
5. **One spelling: `two_color`.** A scout port wished for `two_colour`; the
   stdlib keeps one name per idea. Reverse: none needed.
6. **Promoted from `examples/local-lib/graph/`:** `TwoSat`/`Lit` and
   `Functional` move unchanged in shape; `shortest`/`walk_back` are
   subsumed by `Paths`. Minimum spanning trees stay with `std.dsu`'s
   owner. Reverse: keep the local modules.
7. **`out` is the one name for a vertex's arcs; `Graph.adj` is gone.** Same
   signature and cost as `out`, which `Neighbours` already used. Reverse:
   add `adj` back as a second spelling.
8. **`Graph.directed`/`Graph.undirected` stay two constructors**, not one
   with `directed: Bool`: directedness is fixed when the graph is built,
   and the list form already picks pairs or triples from the tuple shape.
   `euler_walk` takes `directed:` because its `next` cannot say. Reverse:
   add `Graph(edges, directed:)` with the two as spellings.
9. **`std.grid` stays a thin module over `(R, C)` product domains.** A
   cell is a pair of domain values, so `[R, C]T` indexes by it and every
   graph algorithm takes `p => [q for q in p.adj4 if ...]` as `next`; a
   `Grid` type would only rename the pair. Reverse: add `struct Grid<R, C>`
   wrapping the cell pair with `adj4`/`adj8` as methods.
10. **List-form hooks (`<fn>_of_pairs`, `<fn>_of_triples`) are hidden by
    name** from completion (`cpl_check::is_list_form_hook`), the VS Code
    tables and the generated API page. Debt: the suffix stands in for a
    general hidden-from-docs marker on a `pub fn`, which does not exist yet.
    Reverse: list them again, or replace the suffix test with that marker.
11. **Every search takes `until: ?V = none`** (`bfs`, `bfs01`, `dijkstra`,
    `fixed_cost_paths`): stop once that vertex is settled, so `order` ends
    there and its distances are exact. An option, not a `_to` twin, and
    `?V` so one target costs one comparison per settled vertex. A set of
    targets or a predicate waits for a port that needs one. Reverse: drop
    the option; callers stopping at a target pay for the whole space.

## 47. DP and search in the stdlib (`design/stdlib/dp.md`, 2026-11-21)

1. **Searches stay in `std.seq`; the DP toolkit is a new `std.dp`.**
   Reason: `bsearch_real`, `ternary_*` and `parallel_bsearch` are searches
   on ranges like their neighbours; monotone queues, Li Chao and the
   optimizations are called from DP loops. Reverse: move the names.
2. **`bsearch_real` returns `?F64`** (none when `ok(hi)` fails), replacing
   the plan's "`hi` when none". Reason: same shape as `bsearch_first`, and
   "nothing works" is a different answer from `hi`. Reverse: return `hi`.
3. **One generic `LiChao<L, K>(pick, xs)`** replaces the local `LiChao`,
   `LiChaoTag` and `LiChaoF`; any `L` with `at(x) -> K` works, so Aliens'
   `(cost, used)` tie-break and max envelopes are the same tree (backlog
   14 P12). No monotone-slope CHT deque yet: Li Chao covers every
   insertion order at O(log). Reverse: add `Hull` beside it.
4. **No second window queue.** The sliding-window optimum is the
   containers area's `std.window.MonoDeque`; `std.dp` points at it. An
   earlier draft renamed it `MonoQueue` (it never pushes at the front);
   that naming call belongs to its owner. Reverse: move it into `std.dp`.
7. **`ternary_max_real` / `ternary_min_real` keep the `_real` suffix.**
   `bsearch_real` keeps it, the real forms take `lo, hi` where the integer
   ones take a range, and they always return an answer where the integer
   ones return `?I`; one name over both would need `F64` ranges. Reverse:
   one `ternary(r, f, by: max)` beside `bsearch_first` once `Range<F64>`
   exists, with the four names as deprecated aliases for one release.
8. **One SOS transform, in `std.ntt`.** `std.dp` drops its `sos_sub`,
   `sos_sup` and `sos_sub_inv`; the math area's `sos_sub` / `sos_sup`
   (`inverse: Bool = false`) are the one implementation, and the
   coordinator's unified `sos(f, over: Sub | Sup = Sub, inverse:)` is
   theirs to land. Reverse: restore the `std.dp` functions.
5. **No `digit_dp`, interval-DP or rolling-table library types.** Their
   state is the problem; `@memo` and two rows say it. A rolling table is a
   core question (frictions `cf2183e`, `cf2084g1`). Reverse: add them.
6. **`parallel_bsearch` takes `reset` / `apply` / `ok` as operations**, the
   state living in user variables written by named fns. Reason: any
   structure works, and lambdas cannot assign. Reverse: take a state
   value and `(S, I) -> S`.

## 48. Dream-port core syntax batch (2026-11-21)

1. **`--- multitest ---` / `--- with multitest() ---` wrap-lines are provisional (program shape is a deferred design question).**
   They lower to the same `Decorator { directive: true }` AST node as `@!multitest` so dream files parse and check today, while `@!multitest` remains the canonical spelling for corpus ports — do not codemod ports onto wrap-lines until program shape settles. Reverse: remove `wrap_line` from `cpl-syntax/src/parser/stmt.rs` and `is_directive_line` from `items.rs`.
2. **Typed tuple patterns `let (a: T, b: U) = ...` desugar to `let (a, b): (T, U) = ...`.**
   Reason: zero AST/IR/codegen duplication; works identically with `read()` and any tuple expression. Reverse: remove `has_typed_tuple_pattern` in `cpl-syntax/src/parser/stmt.rs`.
3. **Fieldless enum-indexed arrays `[Sign]T` lower to `DimSpec::Int(k)` and tag-checked indexing.**
   Reason: matches `[Bool]T` without new IR ops, interpreter changes, or C++ runtime types; evaluates the index expression once via a temp. Reverse: remove the `Sym::Enum` arm in `cpl-ir/src/lower/mod.rs` (`dim`) and `expr.rs` (`index_expr`).
4. **Canonical generic call spelling is `f<T>(...)` without `::`; `f::<T>(...)` is warned by `L0204` (`turbofish_type_args`) with a `cpl fix` / `cpl lint --fix` codemod.**
   Reason: eliminates two spellings for one idea (`read::<T>()` vs `read<T>()`, `topo::<V>(...)` vs `topo<V>(...)`) while preserving backwards compatibility during migration. Reverse: remove `TURBOFISH_TYPE_ARGS` from `cpl-lint` and `"fix"` from `cpl-cli`.
5. **Typed lambda parameter annotations `(x: T) => body` are retained in `ExprKind::Lambda.param_tys` and typechecked in both `cpl-check` and `cpl-ir`, with `Ty::fits_in` checking `Ty::Fn` parameter/return compatibility.**
   Reason: parsed-but-ignored type annotations are a silent-wrongness trap. Reverse: revert `param_tys` checks.
6. **`x is pat` (optionally chained with `and cond`) desugars to `if let pat = x: cond else: false` (and `Cond::Let` in `if`/`while` headers), and comparison operators (`<`, `>`, `<=`, `>=`, `==`, `!=`) are accepted as operator-section arguments (`by: <`).**
   Reason: reuses existing `Cond::Let` narrowing and lambda lowering with zero new IR ops. Reverse: remove `at_is_pattern` and comparison arms in `cpl-syntax/src/parser/expr.rs`.
7. **Fieldless enums accept explicit integer discriminants (`enum Sign: neg = -1, zero = 0, pos = 1`) with `Int(e)` and `Enum(i)` conversions lowered via `IsVariant`/equality chains.**
   Reason: supports signed/custom discriminants without changing enum tag representation (`0..n`). Reverse: remove `Variant.value` and `enum_discriminants`.
8. **Named mutable aliases (`let st = mut a[i]`) bind to a frozen `Place` whose index subexpressions are evaluated once into temp locals.**
   Reason: preserves value semantics for ordinary `let` while giving zero-copy mutable element aliases (`st.lift += 1`) that agree across interpreter and C++17 codegen. Reverse: remove `ExprKind::Mut` and `FnCx.mut_aliases`.

## 49. `Int` stops one above `-2^63` (codegen and performance, 2026-11-21)

- **49.1 `Int.MIN` is `-(2^63 - 1)` in the compiler too, and `-2^63` is no
  `Int` anywhere.** The spec settled this (1.1, `design/spec/11-types.md`);
  the implementation still had `Int.MIN = i64::MIN`. Now `read` rejects
  `-9223372036854775808` in both `cpl run` and the C++, arithmetic and
  narrowing (`Int(x)` from `I128`/`F64`, `floor`/`ceil`/`round`,
  `bit_ceil`) that would produce it stop with the usual overflow error, and
  constants folded by codegen agree. This frees the pattern as `?Int`'s
  `none` everywhere. `Int.MIN / -1` is `Int.MAX`, so `ediv` and `emod`
  lose their `-1` guards. Reverse: set `cpl_ir::INT_MIN` back to
  `i64::MIN` and drop the `!= i64::MIN` filters; `?Int` then needs 16 bytes
  outside proven tables again.
- **49.2 Float to `Int` is exact at the ends in both builds.** The
  interpreter's bound was `±9.2e18`, the C++'s `±2^63`, so `Int(9.21e18)`
  disagreed; both now accept exactly the floats strictly between `-2^63`
  and `2^63`, and `floor`/`ceil`/`round` share the check (they cast
  unchecked in C++, undefined past the range). Reverse: none needed; any
  bound must be the same in both.
- **49.3 `Bitset` words are `U64`.** A word with only bit 63 set is
  `-2^63`, which is no `Int`, and the old helpers built it from `Int.MIN`.
  With `U64` (wrapping, logical shifts) the helpers for bit 63 disappear.
  Reverse: none while `Int` stays symmetric.
- **49.4 Every `?Int` is `cpl::OptInt`, 8 bytes, with `-2^63` as `none`.**
  With 49.1 the pattern is free everywhere, so locals, fields, arguments,
  lists, maps and tables all use it; the proof that a table never stores
  `Int.MIN` (`niche::packed_tables`) and the pack/unpack plumbing in
  codegen are gone. Reason: values take the fewest bytes their types
  allow, and a layout that depended on a whole-program proof changed with
  unrelated edits. Reverse: map `Ty::Opt(Int)` back to `std::optional<ll>`
  in `names.rs` (and restore the table proof to keep tables packed).
  Cost: the `Niche` template (about 1 KB) now appears in every program
  that touches a `?Int`, including a passing `xs.first`; 65 small ports grew
  more than 5% and the baseline was rewritten. The template was trimmed
  (unused `>`, `<=`, `>=`, a duplicate constructor); over the corpus the
  median went 10,345 -> 10,899 B, p95 28,064 -> 27,734 B, max 58,744 ->
  58,821 B, none over 64 KB.

## 50. MonoDeque single constructor and Bitset.subset_sums (2026-11-21)

1. **`MonoDeque<T>.new(by: (T, T) -> T = min)` replaces the two-constructor `new(pick)` + `by(better)` split, stores `by` directly without wrapping in a lambda, and adds `best_at -> ?Int` and `drop_after(max_at: Int)`.**
   Reason: options are named arguments rather than separate constructors; storing `by` as-is lets C++ codegen inline `min`/`max` directly (~2x faster than wrapping `pick` in a lambda); `best_at` supplies DP argmin in O(1); `drop_after(max_at)` mirrors `drop_before(min_at)` for right-to-left sweeps without negating positions or adding state to `MonoDeque` (`research/frictions/ucup4s24f.md`). Reverse: restore `MonoDeque.by(better: (T, T) -> Bool)` beside `new(pick)` and drop `best_at`/`drop_after`.
2. **Bitset knapsack is `Bitset<D>.subset_sums(weights)` rather than a free `std.bitset.subset_sums`.**
   Reason: avoids colliding with `std.dp.subset_sums(xs) -> List<Int>` (meet-in-the-middle) and eliminates the turbofish at call sites (`Bitset<Sum>.subset_sums(ws)`). Reverse: expose a free `subset_sums<D>` in `std.bitset`.
## 51. `Seq` and `Indexed` trait unification (2026-11-21)

1. **Built-in `Seq` and `Indexed` traits are backed by one central `Ty::seq_elem()` / `is_seq()` / `is_indexed()` predicate in `cpl-check` and `cpl-ir`.**
   `Str`, `List<T>`, 1D `[D]T`, `Range<I>`, and O(1) slice/reverse views all satisfy `Seq` and `Indexed` without conversion, and generic functions written `<S: Seq, T>` or `<S: Indexed>` check bounds at instantiation and deduce element type `T` from `S`. Reason: eliminates per-container overloads (`runs`, `prefix_function`, `z_function`, `is_palindrome`, `is_sorted`, `prod`, `pairs`, `excl_fold`, `sum`/`min`/`max`) and gives O(1) views a single plug-in point (`Ty::StrView`, `Ty::ListView`). Reverse: revert `seq_elem`/`is_seq`/`is_indexed` and restore concrete parameter types in `stdlib/seq.cp` and `stdlib/strings.cp`.
2. **Zero-based `s.at(i)` and `s.get(i)` are the uniform `Indexed` accessors, while `a[i]` preserves domain safety on `[D]T`.**
   `Str` gains `.first`, `.last`, `.last(k)`, `.get(i)`, `.at(i)`; `Range<I>` gains `r[i]`, `.get(i)`, `.at(i)`, `.last(k)` and `sum`/`min`/`max`/`runs`; 1D `[D]T` gains `.get(i)` and `.at(i)`. Generic `<S: Indexed>` algorithms (such as `prefix_function` and `z_function`) index via `s.at(i)` so nominal-domain arrays `[V]T` work without weakening `WRONG_DOMAIN` checks on `a[i]`. Reverse: remove `at`/`get` arms in `cpl-ir/src/lower/call.rs`.
## 52. Static closure & operation-value lowering (2026-11-21)

1. **Three-tier static closure lowering in `cpl-codegen` (`devirt.rs`, `emit.rs`, `expr.rs`, `stmt.rs`), with `std::function` only for genuinely dynamic values.**
   - *Direct elision (`StaticDirect`):* when a struct fn-field or fn parameter holds a single statically known zero-capture closure and never escapes to a dynamic `std::function` home or assignment, calls emit directly as `cpl_fnN(...)` and the `std::function` field/parameter is replaced with a 1-byte `bool` placeholder (`false` at construction/call sites). Eliminates `std::function` completely from `Monoid`, `Action`, `SegTree`, `LazySegTree`, `DualSegTree`, `SparseTable`, `DisjointSparseTable`.
   - *Template functor parameters (`StaticTemplate`):* any non-recursive, non-entry/judge/hook function with a non-`mut`, non-reassigned `Ty::Fn` parameter that is not directly elided lowers to a C++17 function template (`template <class F0, ...>`, `F0&& p`), passing lambdas (including capturing/mutable ones) by type with zero `std::function` and full inlining.
   - *Local `auto` bindings (`StaticLocalAuto`):* any non-hoisted, non-reassigned local closure binding (`let f = ...` or nested `fn via(...) = ...`) emits `auto f = [...](...) { ... };` instead of `std::function<...>`.
   - *Dynamic fallback (`DynamicStdFunction`):* genuinely dynamic fn values (struct fields holding multiple or capturing closures, recursive/entry/judge fn parameters, mutable/reassigned fn slots) retain `std::function`. `#include <functional>` is emitted only when `std::function` appears in the output.
   - *Visibility (`cpl explain <file.cp>`):* `cpl explain <file.cp>` (and `--json`) runs `cpl_codegen::explain_closures` and reports every fn-typed site's lowering mode (`static: direct`, `static: template`, `static: auto`, or `dynamic: std::function` with reason), fulfilling the "nothing secret under the hood" rule.
   - *Return-tuple move:* returning a struct/tuple literal moves owned non-scalar local fields (`std::move(table)`) when unreferenced by sibling fields, preventing O(n log n) vector copies on range-structure constructors (`DisjointSparseTable` 35.4 ms / 1.98x → 18.9 ms / 1.09x vs matched C++17).
   *Reverse:* revert `elidable`, `template_params`, `local_auto`, and `explain_closures` in `crates/cpl-codegen/src/devirt.rs`, the `emit.rs`/`expr.rs`/`stmt.rs` hooks, and `explain_file` in `crates/cpl-cli/src/lib.rs`.
## 53. Folds and sorts of a sequence are methods (2026-11-21)

1. **Method form is canonical for a fold or sort of one sequence:**
   `xs.sum()`, `xs.prod()`, `xs.min()`, `xs.max()`, `xs.count(p)`,
   `xs.all(p)`, `xs.any(p)`, `xs.first(p)`, `xs.sorted(by: k)`,
   `xs.sort(by: k)`. The free form stays only where it reads as math: a
   generator (`sum(f(x) for x in xs)`, `count(c == x for c in cs)`) or two
   or more values (`min(a, b)`). Every `List`, `[D]T`, `Range`, `Set` and
   `Str` has the methods; each lowers to the same fold as the free form, so
   the interpreter, C++, `xs.max()!`/`?? d` and the costs are identical.
   Reason: ports mixed `count(xs, p)`, `sort(mut xs, by:)` and `xs.all(p)`
   in one file; a receiver reads left to right, chains (`xs[1..].max()`),
   and completes in the editor. Cost: one token per site (`a.sum()` is 5
   tokens, `sum(a)` 4), which moved two dream gaps by under 0.5%.
   Reverse: delete `free_fold` (L0509) and run the inverse rewrite; the
   free forms never stopped working.
2. **The element a test holds for is `first`, `find` is the index of a
   value.** `xs.find(x => ..)` (batch 14) and `first(xs, p)` (§26.9) said
   the same thing; `xs.first` is already the first element, so `xs.first(p)`
   is the first one a test holds for, as `xs.count(p)` beside
   `xs.count(x)`. L0509 rewrites `xs.find(lambda)` to `xs.first(lambda)`.
   Reverse: drop that arm of `free_fold`.
3. **`cpl lint --fix` (L0509) is the codemod.** It skips receivers that are
   types or domains (`sorted(V, by: k)` walks a domain) and `min([a, b, c])`,
   which waits on a variadic `min(a, b, c)` from the core lane. Dreams that
   do not parse yet were migrated with the same patterns by hand.

## 54. The offline bundle cap moves to 2,800,000 bytes (2026-11-21)

`dist/cp-lang.html` reached 2,601,595 bytes on main with ten stdlib area
owners adding modules at once; 1,792,438 of it is the compiler (the WASM,
gzip -9 and base64), so the cap tracks the compiler and the stdlib it
embeds, not the pages. The cheap cuts are taken: the pages already ship as
one gzip blob (4f74ff40), and `opt-level="s"` halves the call depth a
program can reach in the browser (scripts/build-wasm.sh). The cap stays a
ratchet against accidental growth: 2,800,000 is about 7% headroom. Reason:
every lane's gate was red on bytes no single commit owned. Reverse: set the
number back in scripts/check.sh and site/test_site.py once the bundle is
under it, e.g. by running wasm-opt -Os in the bundle build.
Reversed the same day: the search index shipped every page's prose a second
time (571 kB of text, 178 kB gzipped). The offline shell now rebuilds each
entry's text from its page on load (1013fbfe), which brought the bundle to
about 2,400,000 bytes, so the cap is back at 2,600,000.

## 55. `Comb<M>` keyed on `Mod<P>` type (2026-11-21)

1. **`Comb<M>` replaces `Comb<P>` as the single combinatorics table spelling (supersedes 40.1).**
   `Comb<M>` takes the modular type `M = Mod<P>` directly — `type Z = Mod<998_244_353>; let comb = Comb<Z>(n)` (or inline `Comb<Mod<P>>(n)`) — so the modulus is written once for both compile-time and runtime moduli, and there is one way per job instead of both `Comb<P>` and `Z.comb()`. `Comb<M>(limit)` pre-sizes `f` and `g` to exact capacity `limit + 1` (clamped at `M.MOD`), while lazy `ensure(n)` continues to double capacity for amortized O(1) growth. All ports, tests, examples, docs, and snippets are migrated in the same commit. Reverse: change `pub struct Comb<M>` back to `pub struct Comb<P>` in `stdlib/mod.cp` (replacing `M` with `Mod<P>` and `M.MOD` with `P`) and rewrite `Comb<Z>` callers back to `Comb<P>` or `Comb<Z.MOD>`.

## 56. `monge_path` returns two lists (2026-11-21)

`std.dp.monge_path(n, w)` returns `(val: List<K>, prev: List<Int>)`, with
`prev[0] = 0`, rather than a list of `(val:, from: ?Int)` pairs. Reason: an
imported generic fn returning `List<(val: K, ...)>` instantiated its second
field as `_` under `cpl run`, and two flat lists are what the caller walks.
It is the O(n log n) candidate deque; online LARSCH (O(n)) is the next step
and keeps the signature. Reverse: return the pair list.

## 57. `Fenwick.first_ge` replaces `Fenwick.lower_bound` (2026-11-21)

1. **Rename `Fenwick.lower_bound(target)` to `Fenwick.first_ge(target)`** and migrate all callers (6 ports + stdlib tests + docs).
   Reason: `lower_bound` is the C++ name; `plan/STDLIB.md` §2 standardizes sorted/monotone prefix lookup on the four-name family `first_ge`/`first_gt`/`last_le`/`last_lt` (`first_ge(target)` returns the first boundary `b` with `prefix(b) >= target`, or `none`). Keeps `kth(k)` as the 1-based position convenience over `first_ge(k)`. Reverse: rename `first_ge` back to `lower_bound` in `stdlib/fenwick.cp`, `plan/STDLIB.md`, `design/stdlib/ranges.md`, and the 6 caller ports.

## 58. One owner for syntax; destructured domains are type names (2026-11-21)

A one-hour factory trial had syntax lanes add a third multitest spelling
instead of choosing one, and a polish rewrite hand-rolled Gomory–Hu next to
`std.flow.gomory_hu`. Syntax is now owned by the coordinator: lanes propose
dream snippets, the owner accepts or rejects, and an accepted form replaces
the old spelling in the same change with a `cpl fix` codemod
(`plan/OPERATING.md`). First ruling: `let (domain C, comp) = f(...)` binds
`C` as a type name for the rest of its scope, nested `fn` signatures
included, exactly like `domain C = ...`; nested fns read the declaration's
hidden slot and the escape rule is unchanged. Reverse by returning syntax to
the core lane in `plan/OPERATING.md`.

## 59. Heterogeneous operator traits (`Add`/`Sub`/`Mul`/`Div`/`Rem` with `Rhs` and `Out`) (2026-11-21)

1. **Operator traits accept an optional `Rhs` type argument (defaulting to `Self`) and an optional `type Out = T` associated type (defaulting to the method's return type or `Self`).**
   For example, `impl Mul<F64> for Vec: fn mul(a, k) = Vec(a.x * k, a.y * k)` and `impl Mul<Vec> for Mat2: type Out = Vec; fn mul(lhs, rhs) -> Out = ...` coexist on the same struct alongside homogeneous `impl Mul for Mat2` without colliding in `cpl-check` (keyed by `trait_impl_key`) or `cpl-ir` (dispatched by `(method, rhs_ty)` in `StructInfo.op_methods`). Reason: eliminates `.scale(k)` / `.div(k)` workarounds in `std.geometry` (`research/frictions/cf2138f.md`), `std.mod`, and `std.matrix` with one general mechanism available to all user and stdlib structs, while keeping every existing `impl Add/Sub/Mul/Div for T` unchanged. Reverse: revert `trait_impl_key`/`Def::Impl.trait_args`/`out_ty` in `cpl-check` and `StructInfo.op_methods`/`struct_op_fn_for` in `cpl-ir`.

## 60. Sequence `by:` options replace `_by` twins; `.items` is the one `enumerate` (2026-11-21)

1. **Remove `_by` twins in favour of `by:` on the base operation.**
   `min_by`, `max_by`, `argmax_by`, `argmin_by`, `is_sorted_by`, `runs_by`,
   `run_ranges_by`, `first_ge_by`, `last_le_by`, `first_gt_by`, and
   `last_lt_by` are removed from `stdlib/seq.cp` and `stdlib/prelude.cp`.
   `min(xs, by: k)` / `xs.min(by: k)` and `max(xs, by: k)` / `xs.max(by: k)`
   lower in one pass with first-extremum tie-breaking (matching `min`/`max`
   and `argmax`/`argmin`); `is_sorted`, `runs`, `run_ranges`, `argmax`,
   `argmin`, `first_ge`, `last_le`, `first_gt`, and `last_lt` take an
   optional `by: T -> K = x => x` parameter. `chunk_by` stays because it
   takes a binary adjacency predicate `(T, T) -> Bool`, not a key function.
   `cpl lint --fix` (L0510 `by_twin`) rewrites both method and free `_by`
   calls to base + `by:`. Reason: one naming scheme (`AGENTS.md`,
   `plan/STDLIB.md` §1) without duplicate twin entries. Reverse: re-add the
   `_by` wrappers in `stdlib/seq.cp` and drop L0510.
2. **`.items` is the one spelling for index-element iteration; no `enumerate`.**
   Sequences (`List`, `[D]T`, `Str`, views) and maps both iterate pairs via
   `.items` (`for (i, x) in xs.items:`), preserving domain index types `D`
   on `[D]T`. Bare `enumerate(xs)` and `xs.enumerate()` are rejected with a
   diagnostic pointing directly to `.items`. Reason: one spelling across
   sequences and maps, already uniform in ports. Reverse: add an `enumerate`
   helper to `std.seq` and remove the diagnostic hint.
3. **Variadic `min(a, b, c)` / `max(a, b, c)` already works** in checker,
   IR (`ExprKind::Builtin(Min/Max, args)`), interpreter, and C++17
   (`std::min<T>({...})`), including optional skipping; no core extension
   is needed.
4. **Wide `Range<I>` binary/ternary searches keep bounds in `I`.**
   `bsearch_first`, `bsearch_last`, `ternary_max`, and `ternary_min` in
   `stdlib/seq.cp` track `lo`/`hi` directly in `I` (`lo + (hi - lo) / 2`)
   rather than `Int` offsets from `r.len`, preventing 64-bit overflow on
   `Range<I128>` wider than `2^63` while preserving domain-safe `Pos(D)`
   arithmetic. Reverse: restore `r.len - 1` offset arithmetic.

## 61. O(1) slice and reverse views (2026-11-21)

1. **`s[a..b]`, `xs[a..b]`, `s.rev`, and `xs.rev` return O(1) read-only views (`Ty::StrView`, `Ty::ListView<T>`), never implicit copies; copying is explicit via `.to_str()` / `.to_list()`.**
   - **Reason:** By the rule "anything that looks O(1) is O(1)", `s[lo..hi]` copying O(k) (and previously lowering to an O(k^2) push loop) and `s.rev` materializing O(n) caused severe hidden blowups (`cf2096e` at 310x, `ucup4s25h` at 26.1x, `cf2245a` at 4.40x).
   - **Representation:**
     - IR: `Ty::StrView` and `Ty::ListView(Box<Ty>)`, both wired into `Ty::seq_elem()` so they satisfy `Seq` and `Indexed` without conversion.
     - Interpreter: `Value::StrView(Rc<StrViewVal>)` and `Value::ListView(Rc<ListViewVal>)` carrying `(base, lo, len, rev)` with O(1) indexing, nested slicing, reverse, comparison, hashing, and iteration.
     - C++ codegen: `cpl::StrView` (`std::string_view sv; bool rev;`) and `cpl::ListView<T>` (`const T* ptr; size_t n; bool rev;`) with index-based iterators (no pointer-before-start UB), `CpHash` overloads, and explicit `to_str`/`to_list`.
   - **Lifetime & mutability safety:** Views are tracked as read-only lends in `cpl-check`. Mutating the base while a view is live is diagnosed (`E0602`), mutating a read-only view is diagnosed (`E0401`), and escaping a view from its lending scope is diagnosed (`E0603`, suggesting `.to_str()` / `.to_list()`). Direct range-place updates (`a[l..r] = 0`, `a[l..r] min= b[l..r]`) remain in-place writes to `var a`.
   - **Visibility:** `cpl explain <file>` prints each lowered slice/reverse view (`O(1), read-only`) and explicit copy/concat (`O(n), owned`) with its C++ representation.
   - **Reverse:** Remove `Ty::StrView`/`Ty::ListView` and lower slices/`.rev` back to allocating `std::string`/`std::vector` copies, dropping the read-only lend flags in `cpl-check/src/check/lends.rs`.

## 62. Containers Batch 2 and wave-12A perf lowerings (2026-11-21)

1. **`Bitset<D>` adopts `Subset<D>` parity (`empty`, `full` as `@get`, `single`, `complement`, `set`, plus `of(xs: List<D>)` for generators) while staying ordinary stdlib code.**
   Reason: matches `Subset<D>` constructors/operations and accepts `Bitset<V>.of(k for k in V if ...)`; direct `for v in b` stays a general core-lane iteration protocol (no `Bitset` compiler privilege). Reverse: revert the added methods in `stdlib/bitset.cp`.
2. **Bounded-domain `Bitset<D>` (`D.K <= 4096` statically provable) reuses `cpl::Row<U64, W>` inline storage and `cpl-codegen` fuses `(a & b & ...).len`/`.count`/`.is_empty` into one zero-allocation popcount pass.**
   Reason: eliminates O(n^3) heap allocations on `cf1146h` (`2.82x` -> `1.29x` matched C++) without adding a new runtime container type. Reverse: remove the `Bitset` branch in `cpl-codegen/src/scope.rs::rows` and `try_fused_bitset_op` in `expr.rs`.
3. **`sort(by: key)` with a cheap pure key expression lowers to in-place `std::stable_sort` with an inlined comparator instead of `cpl::sort_by`'s three auxiliary vectors.**
   Reason: preserves stability and interpreter parity while eliminating O(n) temp allocations and index indirection (`cf1284e`: `1.63x` -> `1.14x` matched C++, -709 B emitted). Reverse: remove the `is_cheap_pure_key` branch in `cpl-codegen/src/stmt.rs`.

## 63. One name per tree traversal: `t.Pre` (2026-11-21)

Dreams spelled preorder five ways (`t.Pre`, `t.preorder`, `t.postorder`,
`tree.post_order`, `t.preorder.rev`). The canonical form is the preorder index
space `t.Pre`: `for v in t.Pre` visits parents first, `for v in t.Pre.rev`
visits children first, and `t.sub(v)` is a range of `t.Pre`. `preorder` and
`postorder` go away, with a `cpl fix` codemod. A true postorder (siblings
in order) is rarely needed in contest code; it stays `t.Pre.rev` until a port
shows it matters. Reverse by restoring `postorder` as an alias.

Amended: `t.Pre` yields positions, so vertex loops iterate the stored
preorder array instead: `for v in t.at` (parents first), `for v in t.at.rev`
(children first) and `for (p, v) in t.at.items` when the position matters
too; `for p in t.Pre` stays the position loop. Each is one line built from
general mechanisms (an array is iterable, `.rev` is the decision 61 view,
`.items` is decision 60's enumerate), so there is no new rule and no core
work, and the loop reads `at` in place where `preorder` built a list. Lint
L0511 (`cpl fix`) rewrites `.preorder` to `.at` and `.postorder` /
`.post_order` to `.at.rev`. Reverse by making `t.Pre` yield vertices (a core
iteration rule) or by restoring the two getters.

## 64. `site/test_site.py` joins the fast gate (2026-11-21)

**Decision.** `scripts/check.sh --fast` runs `site/test_site.py` after the
docs gate. Its two failures are fixed: a playground `#share=` fragment is
state, not a page id, so the link test skips it as `site/check_docs.py`
already does, and the exemplars and ports pages name dreams and ports without
their repository paths.

**Why.** Nothing ran the site tests, so broken links and internal paths in
reader copy landed unnoticed. They cost about two minutes, most of it
compiling documented programs natively.

**Reverse.** Delete the `site tests` step from `scripts/check.sh` and run
`python3 site/test_site.py` from `--full` instead.

## 65. `aliens` is generic over the slope type (2026-11-21)

`std.dp.aliens<L>` searches `Range<L>` and returns `L`, so `I128` slopes work
(cf2183h's reach about 8e19). Exact ties are the caller's job: scale each
edge by (max count + 1) and add 1, which keeps Monge and makes the least
count win, instead of a half-integer penalty with interpolation in the
library. Reverse: an `Int`-only `aliens` plus a separate wide variant.

## 66. Dream domain constructs: subsets, enum arrays, sequence ops, and Item/Cut ranges (0.3, 2026-11-21)

- **66.1 Domain-wide subsets (`V.all`, `Bitset<V>`, `adjacency(edges, into: Bitset)`).**
  `Bitset<D>` adds non-mutating `.without(v)`, `.find(pred)`, `.any(pred)`, `@get fn empty`, and `impl Show`.
  Any struct providing `each(self)` (including `Bitset<D>`) is directly iterable in `for v in s:` via one general
  `Iter::Seq` lowering rule in `cpl-ir` and `element_ty` lookup in `cpl-check`. `V.all` continues to default to
  `Range<V>` (preserving existing `List<V>` / `Range<V>` callers), and lifts cleanly to `Bitset<V>.full()` wherever
  a `Bitset<V>` is expected, in `V.all - side` / `&` / `|` / `^`, and in `V.all.without(v)`.
  `adjacency(edges, into: Bitset)` is a general graph builder returning `[V]Bitset<V>` (with `into: List` or default
  returning `[V]List<V>`), inferring `V` from the edge record/tuple endpoints.
  Majorization check for `V.all` contextual lift: `V.all` stays `Range<V>` (two ints, O(1), zero allocation) in all
  non-Bitset contexts (`for v in V.all`, `V.all.len`, etc.) and lifts to `Bitset<V>.full()` only where the static
  context explicitly requires `Bitset<V>` (a `Bitset<V>` parameter/annotation, `V.all - side` / `&` / `|` / `^`, or
  `V.all.without(v)`). In those positions `Bitset<V>` is the only valid representation and the generated code is
  identical to writing `Bitset<V>.full()` explicitly (same O(n/64) time and space, never allocating a bitset outside
  Bitset contexts), so the contextual lift majorizes explicit `Bitset<V>.full()` without hidden cost.
  Reason: eliminates manual `Bitset<V>.full()` / `.each()` / odd-adjacency boilerplate (`cf2147h`, IOI sketches).
  Reverse: remove the `each`-based `element_ty`/`iter` path, `Range -> Bitset` coercion, and `adjacency` lowering.
- **66.2 Fieldless enums as first-class array dimensions (`[Sign]T`, `[Sign] x => ...`, explicit variant values).**
  Fieldless enum variants accept optional constant integer discriminants (`enum Sign: neg = -1, zero = 0, pos = 1`,
  read via `Int(s)`). Any fieldless enum `E` is a valid array dimension `Dim::Enum(E)` / `DimSpec::Enum(E)` of static
  length `E.variants.len`, supports `[E] x => expr` table comprehensions and `for x in E:` iteration, and indexes
  with `Cast::EnumOrd(E)` so distinct enums cannot index each other's tables.
  Reason: enables `[Sign]Path` DP/segment-tree tags (`design/EXEMPLARS-PORTS.md`) without manual ordinal arrays or
  losing enum type safety.
  Reverse: remove `Dim::Enum`/`DimSpec::Enum` and `Cast::{EnumOrd, EnumVal, OrdToEnum}` across IR, interp, and codegen.
- **66.3 Domain-level sequence operations (`Seq`/`Indexed` parity: `partition`, `sort(by:)`, `group(by:)`, `at`, `first(p)`, `filter`).**
  `Ty::Domain(D)` satisfies `Seq` and `Indexed` via `Ty::seq_elem() == Some(Ty::Pos(D))`, inheriting `.at(i)` and
  `.get(i)` directly from `Indexed`. Every domain `D` (and any sequence `List<T>`, `[D]T`, `Range<T>`, `Set<T>`)
  supports `.partition(keep)` returning `(List<D>, List<D>)` (matching `stdlib/seq.cp`), `.group(by: key)` returning
  `List<List<D>>` in deterministic first-seen key order (§60 `by:` option), `.first(pred)` returning `?D` (matching
  `xs.first(p)`), `.filter(pred)` returning `List<D>`, and `D.sort(by: ...)` returning `List<D>` (matching
  `xs.sort(by:)`, where `by:` accepts a lambda, function name, or `[D]K` array). `cpl fix` (`L0510` `by_twin` +
  `L0511` `domain_spelling`) rewrites `D.sorted(by:)`, `xs.first(where: p)`, and `xs.group_by(k)`.
  Reason: aligns domain sequence ops with the one-spelling rules (§53, §60) and `Seq`/`Indexed` traits.
  Reverse: remove `Ty::Domain` from `Ty::seq_elem()`, the sequence-op arms in `cpl-check`/`cpl-ir`, and `L0511`.
- **66.4 Typed `Item`/`Cut` pairs with `minmax` and `choose2`.**
  `minmax(a, b)` preserves the operand's domain/position type (`(Cut, Cut)` for `Cut` inputs), so `let (lo, hi) = minmax(p[i], s[i]); low.apply(lo..hi, d)`
  passes a typed `Range<Cut>` directly to `LazySegTree<Cut, _>.apply` without converting through `Int` or conflating
  `Item` and `Cut`. `choose2(n)` lowers directly to `n * (n - 1) / 2`.
  Reason: eliminates `min`/`max` + `Cut(lo)..Cut(hi)` boilerplate in `cf2147f` while keeping `Item` and `Cut` distinct.
  Reverse: remove `minmax`/`choose2` from `cpl-check/src/prelude.rs`, `cpl-check/src/check/call.rs`, and `cpl-ir/src/lower/call.rs`.
- **61.5 Destructured fresh domains (`let (domain C, comp) = f(...)`) are type names for the rest of their scope, nested `fn`s included (clarifies 33.1/33.2 and 58).**
  A domain bound by destructuring `let (domain C, comp) = f(...)` has the same scoping as `domain C = ...`: top-level
  bindings pre-register their `DomainId` in `collect_all` and reuse that slot at the `let` site so sibling/nested `fn`s
  can name `C` in parameter/return types and read the declaration's hidden slot directly; local bindings already resolve
  through `outer_cxs`. The escape rule (`E0313`) is unchanged.
  Reason: enables `fn force(c: C, val: Int)` after `let (domain C, comp) = two_edge_components(g)` (`cf2135c`).
  Reverse: remove `register_pat_domains` in `crates/cpl-ir/src/lower/mod.rs` and the top-level slot reuse in `crates/cpl-ir/src/lower/stmt.rs`.
- **66.6 `D.bounds` as a first-class index domain in `SegTree`, `LazySegTree`, and `Fenwick`.**
  Generic type parameters bound to `Ty::Bound(d)` (`D = V.bounds`) act as full index domains in `cpl-ir`: `D.K` /
  `D.len` / `D.count` lower to `DomLen(d) + 1`, `for i in D:` iterates `0..DomLen(d) + 1` as `Ty::Bound(d)`, `[D]T`
  lowers to `DimSpec::Bound(d)`, and `D.nth(k)` / `Range<D>.{lo,hi}` preserve `Ty::Bound(d)`. In `cpl-check` and
  `cpl-ir`, when a call argument expects `Range<D.bounds>`, boundary ranges (`..=lo.l`, `a..=b`, `a..b`, `a..`)
  keep `.bounds` and permit `..=` rather than stripping to `Range<D>` (while non-bounds contexts like `a[l.l..r.r]`
  keep stripping to `Range<D>`). Partial generic constructor inference (`LazySegTree<V.bounds, Int>(AddMax, fill: 0)`)
  binds the supplied prefix of type parameters and infers the remaining ones (`F`) from arguments.
  Reason: eliminates shadow `0..=n` domains and `Pos(lo.to_idx)` casts (`cf2207f`).
  Reverse: remove the `GenArg::Ty(Ty::Bound(_))` arms in `cpl-ir/src/lower/{mod,expr,call}.rs` and `expected_bounds_range` in `cpl-check`.

## 67. `IntervalMap.assign` zero-alloc `on_old:` callback (2026-11-21)

1. **`IntervalMap.assign(mut self, r: Range<D>, val: T, on_old: (Range<D>, T) -> () = (_, _) => ())` reports overwritten runs via an optional callback instead of returning a `List` or `-> yields`.**
   Reason: `cpl_ir::pushed::Feeds` intentionally does not push `-> yields` on `mut self` methods (an early `break` could leave the partition half-mutated), whereas a callback parameter both enforces partition completion and devirtualizes to an inlined template functor in C++ with zero heap allocation. Combined with a single-run fast path, `cf2237h` dropped from `1.66s` to `1.26s` on max binary-tree input and `IntervalMap` runs at `1.15x` matched C++ `std::map` ODT (`0.15s` vs `0.13s`). Reverse: change `assign` back to return `List<(range: Range<D>, val: T)>` in `stdlib/interval.cp` and restore `for (r, t) in paint.assign(...)` in `proto/ports/ecnerwala/cf2237h.cp`.

## 68. `LazyTreap` in `std.lazy` and flat `SparseTable` storage (2026-11-21)

1. **Add `LazyTreap<T, F>` to `std.lazy` (no new module, no compiler privilege).**
   Arena-backed randomized treap over a contiguous node arena with subtree-size-aware lazy tag pushdown (`apply(v, left_sz, f)`, `compose(new_f, old_f)`, optional `split(f, left_sz)`, optional `before(x, y)`), supporting both value-ordered insertion (`insert`, for slope-trick / priority-with-lazy-updates) and 0-based implicit sequence operations (`push`, `insert_at`, `apply_range`, `get`), plus `apply_all`, `take(k)`, and `to_list`. Stores pending tags as `?F` so `F` needs neither `id` nor `Eq`, and lowers all operation closures statically (`0 std::function`). Adopted by `proto/ports/ecnerwala/cf2084g2.cp` (895 -> 359 tokens, 1.05x of dream; 0.90x time vs C++ original on `n = 500_000`). Reverse: remove `LazyTreap` from `stdlib/lazy.cp` and restore the hand-rolled treap in `cf2084g2.cp`.
2. **Flatten `SparseTable` storage in `stdlib/sparse.cp`.**
   Store all `bit_width(n)` levels in a single flat `List<T>` with `unchecked:` indexing in build and `fold`, matching `DisjointSparseTable` and eliminating nested `std::vector` heap allocations. Reverse: restore `rows: List<List<T>>` in `stdlib/sparse.cp`.

## 69. `std.poly`: formal power series over NTT on plain `List<Mod<P>>` (2026-11-21)

1. **`std.poly` operates directly on `List<Mod<P>>` with explicit output length `n` (`inv`, `log`, `exp`, `pow`, `sqrt`, plus `deriv`, `integral`).**
   - **Reason:** Using plain `List<Mod<P>>` (lowest degree first) gives zero-overhead interop with `std.ntt.convolve<P>`, `Comb<M>`, and list comprehensions without introducing a wrapper `Poly<P>` type. Requiring explicit truncation length `n: Int` (modulo `x^n`) matches every contest FPS call site (`ucup5s06a`, `ucup5s03a`, `ucup5s05f`) and avoids sentinel defaults. `inv` uses half-size Newton doubling (5 NTTs of size `2m` per step), `integral` uses linear-time modular inverses `1..=m`, `pow` handles leading zeros and large `k` via `exp(k * log(...))`, and `sqrt` returns `?List<Mod<P>>` via Tonelli-Shanks + Newton doubling.
   - **Reverse:** Remove `stdlib/poly.cp`, remove its rows from `stdlib/README.md`, `site/content/stdlib.md`, `editors/vscode/`, `design/stdlib/math.md` §2.5, and `plan/STDLIB.md`, and switch the four migrated ports back to `lib.math.poly`.

## 70. Comprehension filters are `if`; power is `**` (2026-11-21)

The corpus spelled a comprehension filter two ways (`for x in xs if p` 543
times, `for x in xs where p` 38) and a power three ways (`a ** b`,
`a.pow(b)`, and `a ^ b` in one dream, where `^` is xor). One spelling each:
filters are `if`, familiar from Python and already the majority; power is
`**`, with `.pow` removed from the surface and `^` meaning only xor.
`cpl fix` rewrites `where` filters to `if` and `.pow(e)` to `** e`.
Reverse by restoring `where` as a parser alias.

## 71. `TimeTree.run` and `offline_over` zero-step-allocation callbacks (2026-11-21)

1. **Add `TimeTree<E>.run<M, R>(self, mark:, apply:, answer:, undo:) -> List<R>` and `offline_over<E, M, R>(q, items, by:, mark:, apply:, answer:, undo:) -> List<R>` in `stdlib/timetree.cp`, keeping `steps()` for compatibility.**
   Reason: `TimeTree.steps()` materialized an `O(q + m log q)` `List<TimeStep>` and required callers to maintain a per-event `marks: List<Mark>` stack. Callback-driven `run` / `offline_over` takes one `mark()` per non-empty segment-tree node, devirtualizes all four callbacks in C++ codegen with zero `std::function`, and returns `[answer(0), ..., answer(q - 1)]` directly. On `cf2069f` (`n = 400k, q = 400k`), median user time dropped from `1.31s` to `1.05s` (-20%) and peak RSS dropped from `356 MB` to `146 MB` (2.44x less memory). Reverse: remove `run`, `run_node`, and `offline_over` from `stdlib/timetree.cp` and restore `tt.steps()` in `proto/ports/ecnerwala/cf2069f.cp`.
## 72. The compiler bundles `stdlib/` without its `@test` fns (2026-11-21)

`crates/cpl-check/build.rs` embeds every module so `use std.x` resolves in
the browser and for a lone `cpl` binary. It embedded the files verbatim,
tests included, though a program importing a module never runs them and
`cpl test` reads the file itself. The build now blanks each `@test` fn to
empty lines, so every span into a bundled module keeps its line, and
`dist/cp-lang.html` drops from 2,490,285 to 2,448,349 bytes under the
2.6 MB cap. Doc comments stay: hover and `cpl doc` read them. Test:
`the_bundled_stdlib_drops_tests_and_keeps_every_line`. Reverse by
embedding the files with `include_str!` again.

## 73. `Flip<T>` / `flip_action` in `std.lazy`, combinator devirtualization, and `LazyTreap` fast path (2026-11-21)

1. **Add `Flip<T>` (`Flip.of`) and `flip_action<T, M>(m)` to `std.lazy`.**
   Encapsulates the paired-complement range-toggle pattern (`(now, flipped)` with `Bool` XOR tag) for any monoid `m` with zero compiler privilege. Adopted by `proto/ports/ecnerwala/cf2152g.cp` (387 -> 280 tokens, 1.27x of dream; 35,469 -> 30,720 B emitted). Reverse: remove `Flip` and `flip_action` from `stdlib/lazy.cp` and restore the hand-rolled `Action` in `cf2152g.cp`.
2. **Treat closures whose captures are only used as `StaticDirect` callees as captureless in `cpl-codegen`.**
   In `crates/cpl-codegen/src/devirt.rs`, a closure that captures a `Monoid` or `Action` solely to invoke its `StaticDirect` `op`/`apply`/`compose` field has zero live captures in emitted C++, so it lowers to a top-level `static auto cpl_fnN` (emitted in dependency order in `emit.rs`) with zero `std::function`. Gives 0 `std::function` on `flip_action` (`cf2152g`: 8 static direct, 0 dynamic) and any user-written `Monoid`/`Action` combinator. Reverse: restore `captures.is_empty()` in `devirt.rs`.
3. **Avoid sibling cache misses and tagless pushdown calls in `LazyTreap.ins` / `split_val`.**
   Increment `nodes[i].sz` directly on descent in `ins` and guard `push_tag` with `if let f = nodes[i].tag:` in `ins`/`split_val`, bringing `bench/ranges/lazy_treap` from `111.9 ms` (`1.34x`) down to `85.8 ms` vs `81.1 ms` matched C++17 (`1.06x`). Reverse: restore `down`/`pull` calls in `stdlib/lazy.cp`.
## 74. `monge_path` is online LARSCH; `aliens` stops at count `k` (2026-11-21)

`std.dp.monge_path` now runs the online LARSCH algorithm (O(n) calls of
`w`) instead of the O(n log n) candidate deque, with the same signature
(decision 56); on cf2183h it makes the same number of cost calls as the
original's template. `aliens` stops as soon as a slope's least count is
exactly `k`, which the law already makes exact. Reverse: the deque is ten
lines and needs no level stacks, if LARSCH's bookkeeping ever costs more
than its calls save.

## 75. `L0415` (`loop_invariant_mod_div`) lint with `cpl fix` (2026-11-21)

1. **Flag loop-invariant `Mod<P>` divisors inside `for`/`while` loops (`L0415`, `loop_invariant_mod_div`) and offer a `cpl fix` hoist.**
   - **Reason:** `Mod<P>` division runs an `O(log P)` modular inverse on every iteration; hoisting `let inv_d = d.inv` (or `Z(k).inv` for an integer divisor) before the outermost invariant loop replaces it with `O(1)` multiplication with zero false positives (0 false positives across 1,429 corpus + stdlib files, 2 real hits in `cf2180g` and `noip22p3`). Auto-fix attaches only when the division is unconditional in the loop or the divisor is a nonzero literal so hoisting cannot introduce a division-by-zero trap.
   - **Deferred:** A second rule for loop-varying small-integer divisors when `Comb<M>` is in scope (the `cf2245f` pattern) is deferred because it has a single corpus hit and requires range/sign/depth guards; reopen if a second instance appears.
   - **Reverse:** Remove `crates/cpl-lint/src/rules/mod_div.rs`, `LOOP_INVARIANT_MOD_DIV` from `crates/cpl-lint/src/lib.rs`, and its call in `crates/cpl-lint/src/rules/mod.rs`.

## 76. `range:` on `std.seq.{reverse, rotate_left, rotate_right}` (debt) and `scan_in_place` (2026-11-21)

1. **Keep `range: ?Range<Int> = none` on `std.seq.{reverse, rotate_left, rotate_right}` as temporary debt until core supports mutable slice views.**
   - **Reason:** The coherent spelling for in-place subrange operations (`reverse`, `rotate_left`, `rotate_right`, `sort`, `fill`) is a mutable slice receiver (`xs[l..r].reverse()`), with no per-method `range:` option. Decision 61's `ListView` is strictly read-only across all four compiler crates (`view_readonly` in `cpl-check/lends.rs`, not a `Place` in `cpl-ir`, immutable `Rc<Vec<Value>>` in `cpl-interp`, `const T*` in `cpl::View<T>`), and mutable slices also require distinguishing fixed-length slice receivers from growable `List<T>` (`push`/`pop`). Filed as `Area: core` friction in `research/frictions/cf2207h3.md`.
   - **Reverse:** Once core lands mutable slice views, rewrite `reverse(mut xs, range: r)` / `rotate_*(mut xs, k, range: r)` to `xs[r].reverse()` / `xs[r].rotate_*(k)` via a `cpl fix` lint and drop the `range:` parameter.
2. **Add `scan_in_place<T>(mut self: List<T>, f: (T, T) -> T)` and `suffix_scan_in_place<T>(mut self: List<T>, f: (T, T) -> T)` to `std.seq`; codify `_in_place` naming rule and log `[D]T` receiver debt.**
   - **Reason:** In-place prefix/suffix scans over a growing `List` had no allocation-free, lint-clean spelling (`for i in 1..a.len: a[i] += a[i - 1]` trips `L0414`, while `scan` allocates a fresh copy every call). Adopted by `proto/ports/ecnerwala/cf2187f2.cp`.
   - **Naming:** Logged in `plan/STDLIB.md` §2 ("Mutator vs copy twins"): where mutation is the default, the mutator owns the bare verb and the copy takes the participle (`sort`/`sorted`); where the pure copy already owns the established verb (`scan`, `suffix_scan`), the zero-allocation mutating twin uses `_in_place`.
   - **Debt (`[D]T`) — resolved:** All length-preserving in-place sequence mutators (`scan_in_place`, `suffix_scan_in_place`, `reverse`, `rotate_left`, `rotate_right`, `nth_element`, `next_permutation`, `prev_permutation`, and `std.rand.Rng.shuffle`) now use the built-in `IndexedMut` trait (`is_indexed_mut()` + zero-based `set_at(i, x)` / `swap_idx(i, j)`), covering `List<T>`, 1D `[D]T` (nominal, range, and cut domains), and `Str` with one signature and no per-type overloads. Mutable slice views remain tracked in `research/frictions/cf2207h3.md` #1b and will plug into `is_indexed_mut()` automatically once core adds them.
   - **Reverse:** Remove `scan_in_place` and `suffix_scan_in_place` from `stdlib/seq.cp` and `plan/STDLIB.md`, revert the §2 naming rule, and restore `ends = scan(ends, (x, y) => x + y)` in `cf2187f2.cp`.

## 77. The fast gate's 180 s budget warns instead of failing (2026-11-21)

`scripts/check.sh --fast` failed when its wall time passed 180 s, even with
every check green. Wall time depends on the machine and on how warm the
build is: the same tree took 154-165 s on one VM, 207 s after a cold
rebuild, and 197-199 s on another idle 8-core VM, so a correct commit could
not land. The overrun now prints a warning, and the scoreboard keeps the
time in `totals.gate_fast` and lists it under `warnings`, not
`regressions`. Test: `test_dream_scores_and_gate_fast_ratchets`. Reverse by
restoring the `fail` in `check.sh` and appending to `regressions` in
`diff_against_baseline`.

## 78. `Paths` keeps one witness array, `back` (2026-11-21)

1. **`Paths<V, W, A>` stores `back: [V]?(from: V, via: A)` in place of separate `parent: [V]?V` and `via: [V]?A`.**
   Reason: every search wrote both arrays on every improving relaxation, and g++ keeps those stores because the arrays escape through the returned struct, so a port that reads only `p[v]` paid two extra writes and two extra arrays of n. On `bench/graph/dijkstra` (n = 500k, m = 2M) the merge takes 0.478 s to 0.412 s against 0.350-0.368 s hand C++ (1.30x to 1.18x). One array of pairs makes that one write and one slot per vertex, with `path`/`arcs` unchanged; nothing read `.parent` or `.via` directly, so no accessor is kept. A program that never reads the witnesses at all is the codegen lane's general dead-field elimination over monomorphized structs, not a `_nopaths` twin. Reverse: restore the two fields in `stdlib/graph.cp` and the `parent[v] = u; via[v] = a` writes in each search.

## 79. `JumpTree`: trees grown leaf by leaf (2026-11-21)

1. **`std.tree.JumpTree<V>` keeps one skew-binary jump pointer per vertex: `add(v, p)` is O(1); `ancestor`, `lca`, `lift_while` and `root_of` are O(log n), with `Rooted`'s meanings.**
   - **Reason:** cf2152f builds its trees right to left, each vertex's parent already placed. Building a `Rooted` for them made the port 2.7x slower than the C++ and use 4x the memory (0.60 s / 64 MB vs 0.22 s / 15 MB). The port's own jump pointers were the right structure, but no other port could reuse them. With `JumpTree` the port runs in 0.12 s vs 0.11 s for the C++, with identical output.
   - **Also:** `Rooted.ancestor(v, k)` returns `none` for negative `k`; it used to return a descendant. The new test that checks `JumpTree` against `Rooted` found this.
   - **Reverse:** move the struct back into `cf2152f.cp`.
## 80. One `prefix(a)` spelling with `L0513` (`prefix_twin`) (2026-11-21)

- **Decision:** Standardize on `prefix(a)` as the single public spelling for static prefix sums across `List<T>`, `[D]T`, and `[R, C]T`. Keep `prefix_n`, `prefix1`, and `prefix2` in `stdlib/prefix.cp` as the underlying monomorphic constructors dispatched by `BUILT_ON`, and add taste lint `L0513` (`prefix_twin`) with `cpl lint --fix` rewriting `prefix_n(a)` / `prefix1(a)` / `prefix2(a)` to `prefix(a)`.
- **Reason:** The prelude already dispatches `prefix(a)` cleanly by argument type to `PrefixN<T>`, `Prefix1<D, T>`, and `Prefix2<R, C, T>`, so users never need to remember or import the dimension-suffixed helpers. Rewriting existing ports removes `use std.prefix.{prefix_n}` boilerplate and unifies the family under one guessable name without adding core privilege.
- **Reverse:** Remove `L0513` (`PREFIX_TWIN`) from `crates/cpl-lint`.

## 81. `std.slope.{Concave, Convex}`: the slope trick as a library type (2026-11-21)

**Decision.** A concave function on integer points is `f(lo)` plus its
falling slopes in a `LazyTreap`; `max_convolve` merges slopes,
`add_quadratic(q)` adds a rank-indexed progression read from three values of
`q` (a fourth and the sign of `q''` are checked), and `at`/`max` read it
back. `Convex` wraps a negated `Concave`, one design for both. `cf2084g2`
drops its hand-written treap lambdas for it.
**Why.** The `apply`/`compose`/`split` lambdas were the slope trick itself,
which every slope-trick port would rewrite; the operations name the
recurrence (`dp(c) = max(dp(c), dp(c - 1))` is `max_convolve(flat(0..=1))`).
**Cost.** +3.3 KB emitted in `cf2084g2` (the struct and `to_list`), runtime
within 1.1x of the bare treap and 0.8x the original.
**Reversal.** Delete `stdlib/slope.cp` and restore the `LazyTreap` port from
history; nothing else depends on it.

## 82. A global stored once with a literal is `static constexpr` (2026-11-21)

`const M = 1_000_000_007` in the main file was emitted as `ll M = 0;` and
set in `solve()`, so g++ could not see the modulus and `x % M` compiled to a
hardware divide. Codegen now emits a top-level `Int` or `Bool` global that is
stored exactly once, with a literal, as `static constexpr` and drops the
store. `(x * 31 + i) % M` over 10^8 steps: 635 -> 492 ms. The interpreter is
unchanged: the value is the same, only where C++ keeps it moves. Closures that
alias such a global by reference bind `const auto&`. Test:
`global_stored_once_with_a_literal_is_constexpr`. Reverse by deleting
`Emitter::const_global` and the `const_globals` checks in `stmt.rs` and
`expr.rs`.

## 83. Canonical-spelling docs gate and reasoning-enabled writable-from-docs harness (2026-11-21)

1. **Enforce accepted CP Lang spellings in `site/check_docs.py` (`check_canonical_spellings`).**
   - Rejects legacy `where` loop/comprehension filters, parenthesized getters (`.len()`, `.is_empty()`, `.inv()`, `.rev()`, `.first()`, `.last()`, `.bounds()`), legacy multitest dividers (`--- multitest ---`, `@multitest`), `.preorder`/`.postorder`, `.pow(...)` on numbers/`Mod<P>`, and `cpl lint` spelling rules `L0509`–`L0512` across all docs, site examples, and course JSON.
   - **Reason:** Prevents documentation examples from drifting back to deprecated spellings after Decisions 53–70 and ensures models/readers learn only canonical syntax.
   - **Reverse:** Remove `check_canonical_spellings()` from `site/check_docs.py`.
2. **Use `reasoning_effort="medium"` (overrideable via `WFD_REASONING_EFFORT`), `max_completion_tokens=16384`, and `300s` HTTP timeout for `gpt-5*`/`o3`/`o4` in `scripts/writable-from-docs.py`.**
   - **Reason:** With default `reasoning_effort="none"`, `gpt-5.4` produced ~18 algorithmic WAs on Div2 C–E / ARC B problems that masked real CP Lang documentation gaps. Enabling medium reasoning surfaced genuine doc gaps (`Mod<P>.v`, lent `Str` slices, `Int`-only ranges, parenthesized multi-line expressions and `??=`, `mut` parameter semantics, `[D]Char` `sep: " "`, `Map[k]` trap semantics) and lifted pass rate from 29/55 (52.7%) to 48/55 (87.3%).
   - **Reverse:** Remove `payload["reasoning_effort"]` and restore `max_completion_tokens = 4096` and `timeout = 180` in `scripts/writable-from-docs.py`.

## 84. Negative `sqrt` traps in `cpl run` and `cpl build --checked` (2026-11-21)

1. **`sqrt(x)` for `x < 0.0` traps with `sqrt(<x>) of a negative number` (exit 3) in `cpl-interp` and under `-DLOCAL` (`cpl build --checked`), while release builds stay raw `std::sqrt(x)`.**
   - **Reason:** Calling `sqrt` on a tiny negative float (e.g. `r*r - d*d = -1e-15` from rounding) previously produced a silent `NaN` that poisoned downstream comparisons and sorts (`flt_sqrt_neg_nan`, the last unhandled case in the P0 `floating-point` bug-replay class). Checking `x < 0.0` in `cpl-interp` and inside `#ifdef LOCAL` in `cpl::fsqrt` catches the domain error identically in debug and checked builds with zero release overhead (`-0.0 < 0.0` is false, so `sqrt(-0.0)` still succeeds). Brings `floating-point` bug-replay to 6/6 (100%) and total P0 bug-replay to 60/63 (95.2%).
   - **Test:** `sqrt_negative_traps_in_interp_and_checked_cpp_not_release`, `golden_math_assert_and_input_errors`, and `bench/goal_tests/bug_replay/flt_sqrt_neg_nan.cp`.
   - **Reverse:** Remove the `x < 0.0` check in `crates/cpl-interp/src/lib.rs`, remove the `fsqrt` piece from `crates/cpl-codegen/src/runtime.rs`, and emit `std::sqrt(...)` directly in `crates/cpl-codegen/src/expr.rs`.

## 85. Open-addressing `cpl::HashMap` and `cpl::HashSet` in codegen (2026-11-21)

`HashMap<K, V>` (and unordered `Map<K, V>` / `Set<K>`) now lowers to an
open-addressing linear-probing `cpl::HashMap` / `cpl::HashSet` with 7-bit
fingerprints, backward-shift deletion (zero tombstones), and full salted
`splitmix64` hashing instead of `std::unordered_map` / `std::unordered_set`.
**Why.** Node-based `std::unordered_map` ran 1.32-1.76x slower than C++
`__gnu_pbds::gp_hash_table` with salted `splitmix64` (`bench/containers/run.sh`).
Full salted `splitmix64` (dropping the unmixed low-6-bit passthrough in `CpHash`)
also eliminates power-of-two stride clustering on adversarial inputs (`i * 64`,
`i * 126271`).
**Evidence.** `bench/containers/run.sh` vs `gp_hash_table` (`splitmix64`):
`hash_insert` 1.56x -> 0.66x, `hash_lookup` 1.76x -> 0.75x, `hash_antihash`
1.32x -> 0.65x. Emitted size on `ucup5s05b` stays within 5% (+529 B).
**Reversal.** Switch `names.rs` back to `std::unordered_map` / `std::unordered_set`,
restore the `<unordered_map>` / `<unordered_set>` includes in `emit.rs`, and
drop the `cphashmap` / `cphashset` pieces from `runtime.rs`.

## 86. `Rooted` builds breadth-first with flat child ranges (2026-11-21)

- **Decision.** `Rooted` roots its tree by breadth-first search and keeps the
  children of `v` as the range `bfs[k0[v]..k1[v]]` of the search order instead
  of one `List` per vertex; `from_parents` counting-sorts the parents into the
  same form. On a graph with extra edges the tree is the breadth-first tree, so
  `depth` is the distance from the root and the old depth-first promise (every
  other edge joins an ancestor) is gone.
- **Why.** Building the tree was 2.4x matched C++ (CSR + BFS): one heap list per
  vertex and a stack of `(u, p, w)` triples. Breadth-first order makes the
  children contiguous for free. Back edges and low-links belong to `std.graph`,
  and no port relied on the depth-first tree.
- **Evidence.** `bench/trees/root_bench`: 0.63 s to 0.45 s at n = 10^6 from a
  `Tree<V>` edge list (the rest is `Graph.undirected` building per-vertex lists
  for a list of pairs), and 0.30 s against 0.27 s (1.1x) from a `[E]` edge array.
- **Reverse.** Restore the depth-first stack in `Rooted.build` and a `kid: [V]List<V>`
  field; `children` keeps its signature either way.

## 87. `sort(cmp:)` / `sorted(cmp:)` with checked strict-weak-order traps (2026-11-21)

- **Decision:** Add a `cmp: (a, b) => Bool` option to `sort` and `sorted` (mutually exclusive with `by:`, composing with `desc: true`), lowering through `SortBy::Fn { func, is_order: true }` to stable O(n log n) sorting (`cpl::sort_cmp` in C++, `checked_merge_sort_by` in the interpreter). In `cpl run` and `cpl build --checked` (`-DLOCAL`), non-strict-weak-order comparators trap on irreflexivity (`cmp(x, x)`), asymmetry (`cmp(a, b) && cmp(b, a)`), and post-sort order violations (intransitivity); release builds compile to zero-overhead `std::stable_sort`.
- **Reason:** Exchange-argument sorts (`a.x * b.y < b.x * a.y`), angle sorts, and comparator-over-state sorts (like `cf2135f`) cannot be expressed as a single key and previously forced a wrapper struct + `impl Ord`. Named `cmp:` fits the options scheme, avoids UB in checked builds, and keeps release performance identical to C++.
- **Reverse:** Remove the `cmp` arm from `sort_place` in `crates/cpl-ir/src/lower/call.rs` and `sort_cmp` in `crates/cpl-codegen/src/runtime.rs`.

## 88. Product domains `domain State = (V, K)` (2026-11-21)

1. **Lower product domains (`domain State = (V, K)` and named `domain State = (v: V, d: Dir, ok: Bool)`) to a flat row-major `DomainId` with `lo = 0` and `len = Π len(F_i)`.**
   - **Reason:** Packing factors into a standard nominal `DomainId` (`idx = (...((ord_0 * len_1 + ord_1) * len_2)...)`) means `[State]T` is a single contiguous flat array with zero runtime representation overhead, and every existing domain-generic structure (`SegTree<State, T>`, `LazySegTree`, `Fenwick`, `Bitset<State>`, `for s in State`, `State.len`) works automatically in both `cpl-interp` and `cpl-codegen` with zero backend special cases. In `cpl-check` and `cpl-ir`, `(v, k)` tuples/records coerce to `State` (so `dp[(v, k)]`, `dp[v, k]`, `State(v, k)`, and `fn f() -> ?State: (v, k)` work while swapped order `(k, v)` is rejected with `E0301`), and `let (v, k) = s`, `for (v, k) in State`, `s.0`/`s.1`, and named `s.v` decode via div/rem. `DomainInfo` records factor types and combined `max_size` so LSP hover reports total state count and array byte size.
   - **Reverse:** Remove `factors` from `DomainInfo`, `product_domains` and its encode/decode helpers from `crates/cpl-ir/src/lower/{mod,stmt,expr,call}.rs`, and the product-factor checks from `crates/cpl-check/src/check/{types,stmt,expr,call}.rs`.

## 89. `LazyLinkCut<V, T, F>` is a second type beside `LinkCut` (2026-11-21)

- **Decision.** Path updates on a dynamic forest are `LazyLinkCut.new(act, a)`
  with `apply_path(u, v, f)`, taking the `Action` that `LazySegTree` takes;
  `LinkCut` keeps no tags. `get` is `mut` and splays.
- **Why.** `SegTree` / `LazySegTree` already split this way, so the name is
  guessable, and the tag and per-node count would cost `LinkCut` users (1.01x
  matched today) memory and time for nothing.
- **Law.** `apply(f, x.rev(), k) == apply(f, x, k).rev()`; every stock action
  satisfies it.
- **Evidence.** `bench/trees/lazy_lct_bench`: 1.82 s and 2.05 s against C++
  2.11 s and 2.12 s at n = 2 * 10^5, 10^6 operations, identical output.
- **Reverse.** Fold the tags into `LinkCut<V, T>` with an optional `act:`.

## 90. Partial-injection `a.inverse()` on `[D]?E -> [E]?D` (2026-11-21)

- **Decision:** Type and lower `a.inverse()` uniformly on 1D arrays in `cpl-check` and `cpl-ir`: a partial injection `a: [D]?E` (where `E` is an index domain, fieldless enum, or bounded integer range) returns `[E]?D` and traps on duplicate images (`Stmt::Trap` in both interpreter and C++); a total bijection `a: [D]E` keeps returning `[E]D` and traps on duplicate or missing images. Arrays with non-index elements (e.g. `[D]?Int`) are rejected at check time with `E0302`.
- **Reason:** Partial permutations with holes (`a: [P]?(1..=n) = read(none: 0)` in `cf2084g1` and `a: [Pos]?(0..Pos.count) = read(none: -1)` in `cf2084e`) previously forced hand-written inversion loops because `std.seq.inverse` only accepted total bijections `[I]V`. Typing `.inverse()` by the input preserves one spelling across total and partial maps and works without extra imports.
- **Reverse:** Remove `array_inverse_ret` in `crates/cpl-check/src/check/expr.rs`, the `inverse` arm in `crates/cpl-check/src/check/call.rs`, and `lower_array_inverse` + `local_specs`/`global_specs` in `crates/cpl-ir/src/lower/{mod,stmt,call}.rs`.

## 91. Array parameters stored into a returned struct are taken by value and moved (2026-11-21)

A non-scalar, non-`mut` parameter that a fn returns as one field of the
tuple or struct it builds (and that nothing else in that return names) is a
C++ by-value parameter, moved into the result. A `return f(...)` passes each
local it owns, named once in the call, with `std::move`. Before, such a
parameter was `const T&` and copied into the result: `Rooted.finish` copied
six `[V]` arrays, and `bench/trees/root_bench` went from 178 MB to 139 MB RSS
with identical output. A parameter used anywhere else in the return, a global,
or a `mut` argument is never moved from. Test:
`a_param_returned_in_a_struct_is_taken_by_value_and_moved_in` and the
`sunk_params.cp` differential program. Reverse: drop `scope::sunk_params` and
the `last_use_call` flag; parameters go back to `const&` plus a copy.

## 92. Unify `factor_big` into `factor` in `std.math` (2026-11-21)

- **Decision.** Remove `factor_big(x)` from `std.math`; `factor(x: Int)` handles every 64-bit `Int >= 1` using trial division for `x <= 1_000_000` (`O(sqrt x)`) and trial division below 100 + deterministic Miller-Rabin + Pollard's rho above (`O(x^(1/4) log x)` expected). `Sieve(n).factor(x)` remains the `O(log x)` bulk-query path.
- **Why.** `factor_big` beside `factor` was a width twin violating one-spelling-per-job. A single threshold branch at `1_000_000` keeps small inputs as fast as plain trial division while preventing `O(sqrt x)` TLE on 64-bit inputs (including `phi(x)`). Zero corpus ports called `factor_big`.
- **Evidence.** `cf2201e` emitted C++ grew by only +82 B (`35,974 -> 36,056 B`); `cf2147g` is `20,364 B` (32% of 64 KB cap); all 19 tests in `stdlib/math.cp` and `examples/local-lib/math/primes.cp` pass across `1..2000`, random `1..1e12`, and 64-bit semiprimes/primes.
- **Reverse.** Restore `pub fn factor_big(x: Int)` in `stdlib/math.cp` and revert `factor(x)` to trial division only.
## 93. One-spelling syntax rulings & position/range/enum ergonomics (2026-11-21)

1. **Built-in `MemberKind::Get` properties (`len`, `is_empty`, `first`, `last`, `rev`, `items`) are read without parentheses (`xs.len`, not `xs.len()`).**
   - **Reason:** One spelling per idea. Calling a built-in getter with empty `()` now fails `cpl check` (`E0306` with fix-it) and triggers `L0206` (`getter_parens`) in `cpl fix`. Removed the temporary zero-arg compatibility arm and the special `rev()` arm in `cpl-check`.
   - **Reverse:** Restore the zero-arg `Get` compatibility in `crates/cpl-check/src/check/call.rs` and remove `L0206` from `crates/cpl-lint`.
2. **Comprehension/generator/loop filters use `if` only; power uses `**` only; multitest uses `@!multitest` only.**
   - **Reason:** One spelling per idea. Removed `where` filter guards and `--- multitest ---` / `--- with multitest() ---` wrap lines from `cpl-syntax` (`E0503` with machine-applicable fixes applied by `cpl fix`), moved `Mod<P>.pow` / `Mat<T>.pow` onto `impl Pow` with `.pow(...)` rejected in `cpl-check` and codemodded by `L0205` (`pow_method`), and kept `^` strictly as bitwise xor.
   - **Reverse:** Re-enable the removed parser branches in `crates/cpl-syntax`, restore `.pow` on `Mod<P>`/`Mat<T>`, and drop `L0205`.
3. **Position `p.back(k)` / `p.ahead(k) -> ?D`, `Range<T>.drop(k)` / `.take(k) -> Range<T>`, and enum `is` / `if let` variant patterns (`phase is Seeking`, `Split(base)`).**
   - **Reason:** Accepted from dream rewrites (`cf2150g` et al.). `ahead(k)`/`back(k)` lower via `Builtin::PosOffset` beside `prev`/`next`; `Range<T>.drop(k)`/`.take(k)` clamp `k` to `[0, max(0, hi - lo)]` without overflow; `Cond::Let` in `cpl-ir` dispatches `PatternKind::Variant` on `Ty::Enum` through `self.pattern` (`IsVariant` + `Payload`) so `phase is Seeking` and payload `match` agree in interpreter and C++17.
   - **Reverse:** Remove the `ahead`/`back`/`drop`/`take` arms in `crates/cpl-check/src/check/expr.rs` and `crates/cpl-ir/src/lower/call.rs`, and revert `cond_let` in `crates/cpl-ir/src/lower/stmt.rs`.
## 94. `xs.distinct()` and consistent empty-sequence rule for `fold(f)` / `min()` / `max()` (2026-11-21)

- **Decision.**
  1. Add `distinct<S: Seq, T>(self: S) -> List<T>` to `std.seq` and `std.prelude`: returns distinct values in first-seen order in O(n) expected time and O(n) space via `HashSet`, across every `Seq` (`List`, `ListView`, `[D]T`, `Str`, `StrView`, `Range`). Keep `dedup(mut a)` / `a.deduped()` for adjacent-only O(n) deduplication (no hashing, e.g. after `sort()`); no parallel `unique` name.
  2. Extend `xs.fold(f)` across every `Seq` to accept both monoids (`+`, `*`, `^`, `|`, `&`, `gcd`, `min`, `max`, `Monoid<T>`, which return their identity when empty) and arbitrary binary functions `(T, T) -> T`, obeying the exact same empty-sequence rule as `xs.min()` / `xs.max()`: bare `xs.fold(f)` returns `T` and traps when empty, while `xs.fold(f)!` or `xs.fold(f) ?? d` asks for `?T` (`none` when empty). Also include `ListView` and `StrView` in `expr_maybe_opt_reduce` so `view.min() ?? d` / `view.max() ?? d` / `view.fold(f) ?? d` behave identically to owned sequences.
- **Reason.** Closes `research/frictions/cf2115a.md` (`a.fold(gcd)` and `a.distinct()` on `[D]T`), eliminates the gap between monoid folds and arbitrary binary folds without inventing a second empty-sequence convention beside `min`/`max`, and keeps adjacent `dedup`/`deduped` cleanly separated from hashing `distinct`.
- **Reverse.** Remove `distinct` from `stdlib/seq.cp` and `stdlib/prelude.cp`, and remove `fold_seq`/`reduce_fold_fn` from `crates/cpl-ir/src/lower/call.rs`.

## 95. `xs.reach(while:, cyclic:)` names the two-pointer sweep (2026-11-21)

- **Decision.** Add `reach(self, while: (T, T) -> Bool, cyclic: Bool = false) -> List<Int>` to every sequence: for each `i`, the largest `k` such that `while(xs[i], xs[i + j])` holds for every `1 <= j <= k` (indices taken mod `len` and `k < len` when `cyclic`). Cost: O(n) calls of `while` when the reach drops by at most one from `i` to `i + 1`, which holds whenever `while(a, ·)` is true-then-false along a sorted order; debug builds trap when that is violated.
- **Why.** The hand-written loop `while j < i + m and p(xs[i], xs[j % m]): j += 1` with a `j = max(j, i + 1)` clamp appears in angular sweeps, circular arcs and distance windows (cf1284e and others), and a binary search per element costs a log factor. One method replaces the loop and states the quantity; it is not a cyclic-only helper, and `cyclic_span` is not added.
- **Reverse.** Remove `reach` from `std.seq`; ports go back to explicit two pointers.
## 96. A memo the declarations prove small is a flat table alone (2026-11-21)

`@memo` over positions, `Bool`s and ranged `Int`s used to choose at run
time between a flat table and a hash map, so both were emitted (the hash
map and `CpHash` are about 6.5 KB). When every argument's cell count has a
compile-time upper bound and the product of those bounds is at most
`DENSE_LIMIT`, only the flat table and its fill are emitted: no hash map
and no `flat` flag. The bound of a range end is the interval the
declarations prove for it, the same inference that gives domain index
widths: constants, top-level `let n: lo..=hi`, domains over such ends,
top-level untyped `let`s, `xs.min()!`, `xs.max()!` and `xs.sum()` of a
top-level array with a range element type (`hp: [0..n]1..=400`), and
negation, sums, differences and products of these. So cf2115c's
`k: 0..=singles, s: 0..=shines` with `shines = hp.min()! - 1` and
`singles = hp.sum() - n * hp.min()!` is at most 8000 x 400 cells. Interval
arithmetic cannot see that `singles >= 0`, so a range whose cell count may
be below 1 is clamped to no cells at fill time instead of failing the
proof. This majorizes:
the flat path was already the one taken at run time. A call outside a
declared range was cached in the hash map before and is now computed
uncached, with the same result; in-range calls are unaffected. Otherwise
the run-time choice stays. Reverse by making `proven` false in
`lower/memo.rs`.

## 97. Auto-loaded `Bitset` for domain subset ops, struct array scalar-fill defaults, range-alias dims, `[0..n]?T = read(none:)`, and `?T.map`/`.filter` (2026-11-21)

- **Decision.**
  1. Add `("all", "Bitset")` and `("without", "Bitset")` to `cpl_check::prelude::BUILT_ON` (skipping `stdlib/` paths to prevent stdlib cycles) so methods on values returned by prelude/domain members (`for v in V.all.without(c)`) work without importing `std.bitset.{Bitset}`.
  2. Allow scalar-fill broadcast (`array_fills`) on struct field defaults and function parameter defaults (`count: [Vol]Int = 0`, `delta: [0..=n + 1]Z = 0`), resolve `Sym::Alias` in `Lowerer::dim` so range aliases (`type Vol = 1..=5`) work as array dimensions, and lower typed `let a: [0..n]?T = read(none: sentinel)` through `Spec` + `reads_none_as`.
  3. Add `?T.map(f)` and `?T.filter(p)` across `cpl-check` and `cpl-ir` (accepting lambdas, functions, or arrays), lowering to `Do`/`If`/`IsSome`/`Unwrap`/`Some`/`None` with zero new backend ops.
- **Why.** Mining the 39 under-4 dreams (`Dream: [1-3]/5`) showed struct-field array defaults (`cf2066e`, `cf2124f2`), range-indexed sentinel reads (`cf2129d`), and `?T.map`/`.filter` (`cf2124f2`, `wtf26d`, `cf2084g1`, `cf2084g2`, `cf2084h`) were the top domain/type-system gaps, while `V.all.without(c)` required an explicit `Bitset` import even though the user never named `Bitset`.
- **Evidence.** `proto/ports/ecnerwala/dream/cf2129d.cp` and `cf2084g1.cp` now pass `cpl check` with 0 errors; `cf2084g1` port adopts `at[k + 1].map(...)` and passes `gate.sh cf2084g1 5`; `existential_compress.cp` and `dream_gaps.cp` pass differential interp + sanitized C++17 parity.
- **Reverse.** Remove the two `BUILT_ON` entries in `crates/cpl-check/src/prelude.rs`, revert `array_fills` on defaults / `Sym::Alias` in `Lowerer::dim` / `read_none_arg` in `crates/cpl-ir/src/lower/stmt.rs`, and remove the `Ty::Optional` `"map" | "filter"` arms in `crates/cpl-check/src/check/call.rs` and `crates/cpl-ir/src/lower/call.rs`.

## 98. Reject `++`, numeric-only unary `+`, `fmt` AST equivalence, and indented lambda block bodies (2026-11-21)

1. **`++` is a lexer error ("CP Lang concatenates with `+`") with a `+` fix-it; unary `+` is numeric-only (`E0302` otherwise); `cpl fmt` corpus test checks parsed AST equivalence.**
   - **Reason:** Previously `xs ++ [v] ++ ys` lexed as `+` followed by unary `+`, the parser silently dropped unary `+`, and `cpl fmt` rewrote it to `xs + +[v] + +ys` which ran on `List`. Now `UnOp::Pos` is preserved in the AST, checked as numeric-only (`E0302` on non-numeric operands), `++` is rejected in `cpl-syntax::lexer` with a machine-applicable `+` fix-it (applied by `cpl fix`), and `cpl-fmt::tests::formats_the_whole_corpus` compares span-stripped parsed ASTs before and after formatting across all 1000+ corpus `.cp` files.
   - **Reverse:** Remove the `(b'+', b'+', _)` arm in `crates/cpl-syntax/src/lexer.rs` and `UnOp::Pos` across `cpl-syntax`/`cpl-check`/`cpl-ir`/`cpl-interp`/`cpl-codegen`.
2. **Indented block bodies after `=>` on lambdas and table comprehensions, plus bracketed `match`/`if`/`else` blocks.**
   - **Reason:** Accepted syntax so multi-statement lambdas work cleanly in table comprehensions (`[0..=s, 0..n] (i, j) =>\n    ...`), named arguments (`by: x =>\n    ...`), and call arguments without wrapping in braces. The lexer tracks `layout_blocks` inside brackets so `Newline`/`Indent`/`Dedent` tokens are emitted for `=>` blocks (and `match`/`if`/`else` blocks ending in `:`) inside `(...)`/`[...]`, `split_items` keeps bracketed items intact, and `Parser::lambda_body` parses `Newline + Indent` into `ExprKind::Block(Block)` whose value is its last expression (matching `fn ... :`, formatter, LSP folding, and `--trace`).
   - **Reverse:** Remove `layout_blocks` in `crates/cpl-syntax/src/lexer.rs`, `brackets` in `crates/cpl-syntax/src/items.rs`, and `lambda_body` in `crates/cpl-syntax/src/parser/expr.rs`.
## 99. The offline bundle ships Try it programs as text, not base64 (2026-11-21)

Every Try it link carried its program as a base64 share hash, which gzip
cannot shrink: 248 links cost about 77 kB of `dist/cp-lang.html`, which had
reached 2,599,001 of its 2,600,000 bytes. `site/bundle.py` now swaps each
hash for `#share-ref=N` into a table of plain payloads, and
`site/offline.html` re-encodes them when it renders a page, so every link
is byte-identical to the online site's
(`test_offline_bundle_rebuilds_every_try_it_link_exactly`). The online
pages are unchanged. Reverse: drop `SHARE_HREF` from `site/bundle.py` and
the `share-ref` replace from `site/offline.html`.
## 100. Validate stores into range-typed places via `ExprKind::RangeCheck` (2026-11-21)

- **Decision.** Every store into a range-typed place (scalar `var`/`let`, struct field and constructor arg, array element including compound assignment / `max=` / `min=` / fill / comprehension, and `List` `.push`/`.resize`/`.fill`) lowers through `Lowerer::check_range`, which wraps unproved values in `ExprKind::RangeCheck { value, lo, hi, text }`. Both `cpl-interp` and `cpl-codegen` (`cpl::check_range` under `#ifdef LOCAL`) trap with exit code 3 and `{v} is outside \`{text}\``. Release builds (`-DLOCAL` unset) compile `cpl::check_range` to a zero-cost passthrough, including for narrowed 32-bit struct fields (and future narrowed arrays).
- **Why.** Narrowed `0..=1_000_000` struct fields previously truncated silently in C++ while `cpl run` printed the raw 64-bit value, breaking interpreter/C++ parity, and scalar/array ranges were unenforced on assignment. Checking in `cpl run` and `--checked` (`-DLOCAL`) catches every contract violation locally with identical diagnostics, while eliding proven-in-range stores (literals, narrower range-typed sources, `x % P` into `0..P`, `Mod<P>` branch-refined `+`/`-`/neg) keeps hot loops and `Mod<P>` codegen completely free of checks. Keeping release zero-cost matches `fsqrt`, `shl128`, and `sort` comparator validation (`#ifdef LOCAL`) so valid programs pay zero runtime overhead on judges.
- **Reverse.** Remove `ExprKind::RangeCheck` and `crates/cpl-ir/src/lower/range.rs` to revert validation; or remove the `#ifdef LOCAL` guard in `cpl::check_range` in `crates/cpl-codegen/src/runtime.rs` if release builds should also trap before narrowing truncation.
## 101. A range's ends are `start` and `end`; `end` is exclusive (2026-11-21)

`(2..=4).hi` was 5: every range is stored half-open, and `.lo`/`.hi`
expose that, but `hi` reads as "the highest element". A cold review of
agc077a flagged `print(op.span.lo + 1, op.span.hi)` as an off-by-one when
it was correct, which is the trap in reverse. The accessors are renamed
to `r.start` and `r.end`, with C++ `end()` and Python `stop` meaning: `end`
is one past the last element for `a..b` and `a..=b` alike. The last
element is `r.last` (the sequence getter, `?T`, `none` when empty) and the
count is `r.len`. `.lo`/`.hi` on ranges become errors with a type-directed
`cpl fix` codemod; user record fields named `lo`/`hi` are untouched.
- **Why.** Off-by-one is a P0 class; an accessor name must not suggest
  inclusive when it is exclusive. `start`/`end` is the familiar pair.
- **Reverse.** Restore `.lo`/`.hi` as the range accessors and run the
  codemod backwards.
- **Status.** Landed (checker, IR, interpreter, codegen, `cpl fix` type-directed
  codemod, LSP completion/hover/quick-fix, stdlib, ports, dreams, docs).

## 102. `PermTree` is a binary permutation tree over a fresh node domain (2026-11-21)

- **Decision.** Add `PermTree.of(p: [I]V)` and `PermKind` to `std.tree`: the binary interval-decomposition tree in O(n), nodes in a fresh domain of 2n - 1 members, kinds `Leaf`, `Incr`, `Decr`, `Partial`, `Full` (a `Partial` left child closed by its right one), with `left`/`right` (trap on a leaf), `span` (positions) and `vals` (value interval, exact unless `Partial`). The tree stays binary rather than ecnerwala's n-ary `FULL` nodes: a chain of `Decr` nodes is the n-ary decreasing node read in order, and both corpus folds walk exactly that chain. `cf2187f1`/`f2` drop their two copies of the construction (634 -> 200 and 750 -> 314 tokens).
- **Reverse.** Remove `PermTree`/`PermKind` from `stdlib/tree.cp` and restore the hand-built trees in the two ports from history.

## 103. `SortedSet.range(r)` and `SortedBag.range(r)` replace the 5 bound-query methods (2026-11-21)

- **Decision:** `SortedBag<K>.range(r)` and `SortedSet<K>.range(r)` (`std.bag.SortedSet`, superseding `OrderedSet`) return an O(1) `SortedRange<K>` view over any ordered range (`a..b`, `a..=b`, `..b`, `..=b`, `a..`, `..`, or `Range<D>`). `.first`, `.last`, `.len`, and `.is_empty` run in O(log n) expected time; iteration (`for x in s.range(r)`, `.rev`, comprehensions, and folds like `.sum()`) runs in O(log n + k) expected time with zero view allocation. `first_ge`, `first_gt`, `last_le`, `last_lt`, and `count_range` are removed from `SortedBag`/`SortedSet` with checker diagnostics and lint `L0515` (`cpl fix` codemod) rewriting all 5 methods to `.range(...)`.
- **Reason:** Closes the `cf2062e2` dream friction where querying keys outside a subtree (`heavier.range(..r.lo)`, `heavier.range(r.hi..)`, `heavier.range(r)`) needed five separate bound methods with hand-picked comparisons. Lowering `SortedRange<K>` to captured range bounds over the receiver place keeps view construction O(1) and matches hand-written C++ treaps (`1.12x` in `bench/containers/sorted_range`).
- **Reverse:** Re-add `first_ge`/`first_gt`/`last_le`/`last_lt`/`count_range` on `SortedBag`/`SortedSet` in `stdlib/bag.cp`, remove `SortedRange` handling from `cpl-check`/`cpl-ir`, and drop `L0515` from `cpl-lint`.
## 104. `.map` on fixed arrays `[D]T` / `[D, E]T` preserves index domains (2026-11-21)

- **Decision.** `.map(f)` on a fixed array `[D]T` returns `[D]U`, and on a multi-dimensional fixed array `[D, E]T` returns `[D, E]U` (so nested `[D][E]T` preserves both domains through `a.map(row => row.map(f))`). `List<T>`, `Str`, `Range<I>`, `Set`/`HashSet`, `Domain`, and O(1) slices/views remain unchanged and return `List<U>`. All other element-wise/shape-preserving stdlib array operations (`scan`, `suffix_scan`, `prev_less`/`next_less`/`prev_greater`/`next_greater`, `compose`, `inverse`, `perm_pow`, `copy`) already return domain-typed arrays; reordering or length-changing ops (`reversed`, `sorted`, `filter`) stay `List`.
- **Why.** Mapping over a domain-indexed array keeps a 1-to-1 correspondence with each index position, so dropping to `List<U>` threw away the domain proof and forced hand-written comprehensions (e.g., `proto/ports/ecnerwala/dream/agc077a.cp:52`, `Block(seen.map(f), real.map(f))` with `seen: [Side]Run` failing with E0302).
- **Evidence.** `array_map_preserves_index_domain` in `crates/cpl-check/src/tests.rs`, `crates/cpl-cli/tests/programs/array_map.cp` (interpreter + C++17 differential parity across 1D enum/domain/1-based-range/empty arrays, 2D `[R, C]T`, nested `[Side][C]T`, and unchanged `List`/`Str`/slices), and updated `Array.map` catalog row (`[0..3]Int`) passing `builtin_catalog`. Closes `research/frictions/agc077a.md`.
- **Reverse.** Remove the `Ty::Array` arm for `"map"` in `crates/cpl-check/src/check/call.rs`, remove `lower_array_map` in `crates/cpl-ir/src/lower/call.rs`, and change the `Array` `"map"` row in `crates/cpl-check/src/builtins.rs` back to `List<U>`.
## 105. Single-iterator `IntervalMap.assign` lowering and unchecked `SortedBag` hot loops (2026-11-21)

- **Decision:** Lower `std.interval.IntervalMap<D, T>.assign` in `cpl-codegen` to a single-iterator `cpl::interval_assign` helper (one `upper_bound(lo)` search, in-place overwrite when `sl == lo`, O(1) `erase(it)` across overwritten runs, O(1) left/right neighbor coalescing, and `emplace_hint`), keeping `stdlib/interval.cp`'s ordinary CP Lang body as the interpreter reference; simplify `assign_point` to delegate to `assign`. Wrap `SortedBag`'s internal treap node-index loops (`rank`, `rank_le`, `range_first`, `range_last`, `range_each`, `range_each_rev`, `cursor_ge/gt/le/lt`, `next_cursor`, `prev_cursor`, `sub_of`, `pull`, `split_lt`, `split_le`, `merge`) in `unchecked:`.
- **Reason:** `Map<K, V>` does not expose raw mutable `std::map::iterator` values (which would allow iterator invalidation in C++), so the pure-`Map` `assign` performed ~6 tree searches per call (`bench/containers/interval_map` `87.1 ms`, `1.17x`; `cf2237h` seed 1 `1.013 s`, seed 2 `0.694 s`). Single-iterator lowering cuts `interval_map` to `41.6 ms` (`0.56x` of 2-iter C++ `std::map`) and `cf2237h` to `0.514 s` / `0.321 s` (-4,335 B emitted). Unchecked treap indexing drops `bench/containers/sorted_range` from `1.12x` to `1.00x` (`181.2 ms` vs `180.4 ms` C++).
- **Reverse:** Remove `is_interval_map_assign` from `crates/cpl-codegen/src/emit.rs` and `interval_assign` from `runtime.rs` (`stdlib/interval.cp`'s body runs in both `cpl run` and `cpl build`), and remove `unchecked:` blocks in `stdlib/bag.cp`.
## 106. Narrow 1D range-typed array and list elements to 32-bit storage (2026-11-21)

- **Decision.** Narrow 1D range-typed arrays and lists (`[V]lo..=hi`, `List<lo..=hi>`, and domain-relative bounds `[V]0..=V.count` / `[V]1..=V.count`) to `std::vector<uint32_t>` (when `0 <= lo` and `hi <= u32::MAX`, or domain-relative `0..=V.count` / `1..=V.count`) or `std::vector<int32_t>` (when `i32::MIN <= lo` and `hi <= i32::MAX`) across globals, locals/params, returns, and struct fields (`Program::narrow_elems` + `scope::narrow_elems`). A candidate stays narrow only if every flow (assignments, calls, returns, `mut` args, struct constructors, closure captures, slices/views, non-natural `.sort`) lands in a place with the same narrowed element width; any flow into plain `[V]Int`/`List<Int>` or mismatched width widens it to `ll`. Narrowed element reads widen via `ll(...)` before arithmetic so `a[i] * a[j]` cannot overflow 32-bit. `stdlib/tree.cp`'s `Rooted` uses `[V]0..=V.count` for `dep`/`k0`/`k1` and `[V]1..=V.count` for `sz` (`far` stays `[V]Int`), and `expect_declared` propagates declared field element ranges into `[D] _ => ...` comprehensions so `Marks<D>.stamp` (`[D]0..=2_000_000_000`) also narrows to `std::vector<uint32_t>`.
- **Why.** Extends ranged scalar struct-field narrowing (Decision 55) to 1D arrays and lists, makes the stdlib docs' 4-byte-per-element storage claim (`Marks<D>`) true, and cuts memory on large trees (`bench/trees/root_bench` at `n = 10^6`: `Rooted` drops `dep`/`sz`/`k0`/`k1` from 8 B to 4 B each, peak RSS ~123.7 MB / 0.35 s vs hand-written C++ 69.8 MB / 0.23 s). Stores are already validated by `ExprKind::RangeCheck` (Decision 100), so checked builds trap on out-of-range writes while release stays zero-cost.
- **Reverse.** Return an empty map from `scope::narrow_elems` in `crates/cpl-codegen/src/scope.rs` (or clear `Program::narrow_elems` in `crates/cpl-ir/src/lower/mod.rs`) to emit all ranged arrays/lists as `std::vector<ll>`; optionally revert `Rooted`'s `dep`/`sz`/`k0`/`k1` fields in `stdlib/tree.cp` to `[V]Int`.
## 107. Collapse `std.prefix` to one `prefix(a)` / `Prefix` and one `Diff` family (2026-11-21)

- **Decision.** Collapse `prefix1`/`prefix2`/`prefix_n` and `Prefix1`/`Prefix2`/`PrefixN` into one `prefix(a)` constructor and one user-visible `Prefix` type family dispatching by shape across `[D]T` (`Prefix<D, T>`), `[R, C]T` (`Prefix<R, C, T>`), and `List<T>`/`[0..n]T` (`Prefix<T>`). Every shape exposes O(1) `.sum(range...)` and O(1) `.prefix(bound...)` (adding `.prefix(r, c)` on 2-D grids) with build/query costs documented. Collapse `Diff1<D, T>` and `Diff2<R, C, T>` into one `Diff` family (`Diff<D, T>()` and `Diff<R, C, T>()`). Internal specializations (`_Prefix2`, `_PrefixList`, `_Diff2`) stay hidden from `cpl-doc` and display as `Prefix`/`Diff`. `stdlib/prelude.cp` exports `use std.prefix.{Diff, Prefix}` (net -2 prelude names). The removed arity-suffixed names (`prefix1`, `prefix2`, `prefix_n`, `Prefix1`, `Prefix2`, `PrefixN`, `Diff1`, `Diff2`) are compile errors with machine-applicable fixes in `cpl-check` and `cpl fix` (`L0513`), and all live ports, dreams, tests, editor data, and docs are migrated.
- **Reason.** One spelling per idea: prefix sums and difference arrays are one concept parameterized by shape, not three or two separately named APIs by arity.
- **Reverse.** Restore `Prefix1`/`Prefix2`/`PrefixN`/`prefix1`/`prefix2`/`prefix_n`/`Diff1`/`Diff2` in `stdlib/prefix.cp` and `stdlib/prelude.cp`, remove `prefix_shape_struct`/`INTERNAL_BUILT_ON` dispatch in `cpl-check`/`cpl-ir`, and revert `L0513` to warning-only call rewrites.

## 108. `Rooted.virtual` returns a virtual tree over a fresh domain (2026-11-21)

- **Decision.** `t.virtual(keys)` replaces `t.virtual_tree(keys)`. It returns `VirtualTree<V, K>` whose own domain `vt.K` lists the kept vertices (keys and pairwise LCAs) in preorder, with `at: [K]V`, `up: [K]?K`, `len: [K]Int` (weighted distance to `up`) and `key: [K]Bool`. A per-query DP is then `var dp: [vt.K]Int = 0` and `for x in vt.K.rev`, children first, with no `[V]` scratch to reset and no edge list to turn back into parents. Matched C++ (`bench/trees/virtual_bench`): 1.12x and 1.18x at n = 2·10^5, q = 10^5, k = 10; 0.9x at n = 10^6, k = 1000.
- **Reverse.** Restore `virtual_tree` returning `(nodes, edges)` from history; callers rebuild parents from the edges.
## 109. A product-domain loop is nested integer loops (2026-11-21)

`for (r, c) in Square` over `domain Square = (Row, Col)`, or over an anonymous
`(Row, Col)`, walked the cell ordinals and decoded each with `ediv`/`emod`.
It now lowers to one integer loop per factor, outermost first, the way
`V.pairs` already did, binding each name to its loop variable: same order,
same values, no division, and the Euclidean helpers are no longer emitted
for it (cf1404e's dream: 15,837 -> 15,277 B). It majorizes, so `cpl explain`
has nothing to show. `break` leaves the whole nest and `continue` steps the
innermost loop, so the nest acts as the one loop written; `V.pairs` with a
`break` used to materialize the pair list and now takes the nested loops
too. A pattern that does not name each factor (`for s in Square`) keeps the
ordinal loop. Reverse by deleting `product_heads` in `lower/stmt.rs`.
## 110. CSR `Adj<V, A>` for pair/triple graphs and zero-copy array-to-list lowering (2026-11-21)

- **Decision.** Return a compressed sparse row struct `Adj<V, A>` (`start: List<0..=2_000_000_000>`, `arcs: List<A>`, with `out`/`get`/`deg`) from `Graph.directed`/`Graph.undirected` on plain pair/triple lists (`directed_of_pairs`, `undirected_of_pairs`, `directed_of_triples`, `undirected_of_triples`) instead of `[V]List<...>`, narrow `Graph<V, E>.start` to `List<0..=2_000_000_000>`, preserve element narrowing through `Builtin::Copy` and `Builtin::ListFilled`, lower 1D array arguments in `list_form` via `List(edges)` (`Builtin::ArrayToList`, zero-copy) rather than a `[each cell for each cell in edges]` copy loop, and pre-allocate `order` with `unchecked:` inner loops in `Rooted.build`/`Rooted.finish`.
- **Why.** `Graph.undirected(edges)` on `n = 10^6` vertices previously allocated `n` separate `std::vector<int>` adjacency lists plus a 16 MB temporary edge-list copy, leaving `bench/trees/root_bench` at `0.35–0.41 s / 121–124 MB` (`1.52x` time, `1.77x` memory vs hand C++ `0.29 s / 69.8 MB`). CSR `Adj`, zero-copy `ArrayToList`, narrowing `start`/`at` through `Copy`/`ListFilled`, and pre-allocated `Rooted` BFS bring `root_bench` to `0.22 s / 79.8 MB` (`0.76x` time, `1.14x` memory vs hand C++).
- **Reverse.** Change `directed_of_pairs`/`undirected_of_pairs`/`directed_of_triples`/`undirected_of_triples` in `stdlib/graph.cp` back to `[V]List<...>` and remove `Adj<V, A>`.

## 111. `if let` binds in headers; `x is pat` is a Bool expression (2026-11-21)

- **Decision.** One spelling per position. In `if`/`elif`/`while` headers an
  optional or pattern is bound with `if let p = x` / `while let p = x`, chained
  with `, cond`. `x is pat` (with `and cond`) is for value position, where a
  `Bool` is wanted (`eaters.set(c, on: tops.first_gt(c) is q and a[q] <= s)`,
  `phase is Seeking`). A header written `if x is p:` or `while x is p:` gets a
  lint with a machine-applicable `cpl fix` to the `let` form.
- **Reason.** §48.6 made `is` desugar to `if let`, so both spellings reached
  headers and dreams started mixing them. The rule keeps one spelling per
  position without losing the expression form.
- **Reverse.** Drop the lint and its codemod.
## 112. Push-relabel is a second named max-flow method, `FlowGraph.push_relabel` (2026-11-21)

`FlowGraph.push_relabel(s, t) -> Int` computes the same maximum flow as
`max_flow(s, t)` by highest-label push-relabel with the gap heuristic
(KACTL's), O(V^2 sqrt E), and leaves a valid flow, so `flow(a)` and
`min_cut(s)` work after it. It is a method named after its algorithm, like
`dijkstra` and `bellman_ford` beside each other, not an `algo:` option on
`max_flow`: with a runtime option every Dinic program carried the unused
solver into its C++ (cf1404e grew 15,430 B to 19,640 B), and an unchosen
method is not emitted. It has no `limit:`, since a preflow cannot stop at a
bound. Why now: cf1404e's original uses it and runs 12x faster than Dinic on
the all-`#` grid. Reverse: delete the method; ports go back to `max_flow`.

## 114. Append-only `Prefix.filling(order: FillOrder)` across 1D, 2D, and List (2026-11-21)

- **Decision.** Extend the unified `Prefix` family (`Prefix<D, T>`, `Prefix<R, C, T>`, `Prefix<T>`) with `FillOrder = Forward | Reverse` and `Prefix<...>.filling(order: FillOrder = FillOrder.Forward)`, returning dedicated append-only tables (`FillPrefix<D, T>`, `FillPrefix<R, C, T>`, `FillPrefix<T>`) with O(1) `.set(...)` in strict fill order (row-major in 2D) and O(1) `.sum(...)`/`.prefix(...)`. Static `prefix(a)` tables keep zero extra fields or branches (zero emitted-size or hot-loop overhead for static users like `cf2129e`). Each `FillPrefix` tracks `rev: Bool` and `next: Int`; `.set` traps on out-of-order writes and `.sum`/`.prefix` trap on any non-empty query reading unfilled cells, with identical trapping in interpreter and C++. `FillOrder`, `FillPrefix`, and `_FillPrefix*` are brought in alongside `Prefix`/`prefix` via `INTERNAL_BUILT_ON`.
- **Reason.** Corpus audit found two real DPs that fill a prefix/suffix sum table while querying the already-filled part (`cf2183f` in 2D reverse and `cf2229e` in 1D forward), meeting the two-user bar (other 2D prefix-sum ports like `ioi07p5` and `apio09p1` are static rather than append-only). Extending `Prefix` directly rather than adding a one-off `RectSums` struct keeps one type and one O(1) `.sum` vocabulary across static and incremental 1D/2D/list prefix and suffix tables.
- **Reverse.** Remove `FillOrder`, `filling`, `set`, and the `rev`/`next` fields from `stdlib/prefix.cp` and `FillOrder` from `INTERNAL_BUILT_ON` in `crates/cpl-check/src/prelude.rs`; revert `cf2183f` and `cf2229e` to hand-maintained prefix/suffix arrays.

## 119. `Tree` is in the prelude (2026-11-21)

`Tree<V>` (`std.graph`) is how a program reads an unrooted tree, and
`Graph` and `Rooted`, the two things it is passed to, were already in the
prelude, so `read<Tree<V>>()` needed a `use` that nothing else did
(dream/cf2183f). The measurements before adding it: no corpus port,
dream or example declares its own `Tree` (two test fixtures do, and a
local declaration shadows the prelude as before); the emitted C++ of a
non-tree port (cf2124a, 5,813 B) and a tree port (cf2183f, 21,023 B) is
byte-identical, and build time is unchanged within noise, because the
prelude brings in only the names a program uses. Reverse: remove `Tree`
from the `std.graph` line of `stdlib/prelude.cp`; files that spell
`Tree<` without a `use` then need `use std.graph.{Tree}`.

## 120. `std.flow.Arc` ids are `0..2^31` (2026-11-21)

`Arc(id: 0..2_147_483_648)`, not `Int`, so every arc handle in the generated
C++ is 4 bytes, as KACTL's and ACL's `int` edges are. A network that large
does not fit in memory (each arc also holds an `Int` capacity), so the bound
costs no program. An arc id past 2^31 traps with "is outside
`0..2_147_483_648`" in the interpreter and in checked C++
(`runtime_errors.rs`, `range_typed_stores_trap_in_interp_and_checked_cpp`).
`bounded_flow`'s missing back arc was the sentinel `Arc(-1)` and is now
`?Arc`. Measured on bench/flow, best of 5, against matched C++: Dinic 1.29x
(1.50x with `Int` ids), push-relabel 1.12x (1.31x), bounded flow 1.18x
(1.26x), closure 1.06x (1.14x). Reverse: `Arc(id: Int)` and the sentinel back.
## 121. In-place `std.mo.mo` driver with separate `add`/`remove` closures (2026-11-21)

- **Decision.** Add `pub fn mo<D, R>(queries: List<Range<D>>, add: D -> (), remove: D -> (), answer: Int -> R) -> List<R>` to `stdlib/mo.cp`, keeping `mo_order` public as the escape hatch (and `mo_steps` for existing callers). `mo` walks `mo_order(queries)` with in-place 4-pointer loops (`add`/`remove` O((n + q)·√n) times, `answer(qi)` `q` times with `queries[qi]` active), returning results in original query order with zero per-query step-list allocation and zero `std::function` (all callbacks lower statically).
- **Reason.** Closes both high frictions in `research/frictions/cf2006d.md` ("Mo's algorithm has no driver" and "The port runs 2.1x the original on the max input"): `cf2006d` drops from `2.1x` (`430–470 ms`) to `1.34x` (`271.8 ms` vs `202.4 ms`), and `cf2129e` drops its hand-written 4-loop Mo driver (`575.4 ms` vs `452.6 ms`, `1.27x`). Separate `add`/`remove` closures avoid encoding two distinct events behind a `±1` sentinel, and `answer: Int -> R` matches `TimeTree.run` so queries carrying extra per-query data (like `cf2129e`'s `k`) work without duplicating the window.
- **Reverse.** Remove `pub fn mo` from `stdlib/mo.cp` and revert `cf2006d.cp` / `cf2129e.cp` to `mo_order` + manual loops.
## 116. Type arguments on a receiver fill a list-form fn's generics; they never pick another fn (2026-11-21)

- **Decision.** `Graph<V>.undirected(pairs)` resolves to the same `undirected_of_pairs` / `_of_triples` fn as `Graph.undirected(pairs)`, in the checker (`list_form`) and in lowering (`list_form_call`). Type arguments on the receiver fill the fn's like-named generics (`V`), so `Graph<V>.undirected([(1, 2)])` is an error saying it wants a `List<(V, V)>`. A pattern form naming fields (`for (u, v, w: weight) in edges`) was considered and not added: names are written once where the data enters (`read<[m](u: V, v: V, w: Int)>()`), and no dream or friction report uses it (open question in `design/stdlib/graphs.md`).
- **Also (no new rule).** The checker's existing positional-to-named fit (`(V, V)` fits `(u: V, v: V)`) now reaches through optionals (`?(Int, Int)` fits `?(a: Int, b: Int)`), and lowering retypes in place a value whose type differs from the wanted one only in tuple field names (`same_cells`, through arrays, optionals and lists). Both back ends hold tuples and records the same way, as `std::tuple` in C++ and a tuple value in the interpreter, so this is free. `[n](Int, Int)` into `[n](a: Int, b: Int)` used to be rejected in lowering. Swapped field types still do not fit (`tests/diagnostics/named_tuple_cells_keep_their_order`). Reverse by dropping `same_cells` from `lower/mod.rs` and `lower/stmt.rs` and the `Optional`/`Optional` arm from `Ty::fits_in`.
- **Why.** Four dreams (cf2035f, cf2057e2, cf2150f, cf2152g) write `Graph<V>.undirected(read<[n](V, V)>())`. That skipped the pair-list fn and fell through to the declared-edge-domain form, so writing a type argument changed which fn ran.
- **Reverse.** Restore the bare-name-only `list_form` in `cpl-check/src/check/call.rs`, and inline `list_form_call` back into the unparameterized branch of `cpl-ir/src/lower/call.rs`.

## 113. Add `IntervalMap` to `stdlib/prelude.cp` (2026-11-21)

- **Decision.** Re-export `std.interval.IntervalMap` from `stdlib/prelude.cp` (allocated prelude slot under decision 118, cap 80).
- **Evidence.** 2 corpus ports (`cf2237h.cp`, `cf2057f.cp`) and 2 dreams (`dream/cf2237h.cp`, `dream/cf2057f.cp`) use `IntervalMap`; 0 name collisions across the corpus; emitted C++ is byte-identical across measured ports (`cf2237h`, `cf2057f`, `cf2022c`, `cf2262b`) and build time is within noise (`+0.05..+3.7 ms`).
- **Reverse.** Delete `use std.interval.{IntervalMap}` from `stdlib/prelude.cp` and restore explicit `use std.interval.{IntervalMap}` in callers.

## 117. Add `SortedSet` to `stdlib/prelude.cp` (80/80 cap) and ordered-container API parity (2026-11-21)

- **Decision.**
  1. Re-export `std.bag.SortedSet` from `stdlib/prelude.cp` (prelude count 79 -> 80). **The prelude is now full at 80/80, so any later addition must remove an existing prelude name.**
  2. Rename `SortedBag.first_where` / `SortedSet.first_where` to `bsearch_first(ok:)` and add `bsearch_last(ok:)`, matching `std.seq.bsearch_first`/`bsearch_last` labels and semantics, with boundary monotonicity debug assertions. Remove `first_where` outright.
  3. Add `set(x, on: Bool)` on `Set`, `HashSet`, and `SortedSet`, matching `Bitset.set(i, on:)`.
  4. Add `Set.range(r)` and `Map.range(r)` returning `SortedRange<T>` / `SortedRange<(K, V)>` views with O(log n) `.first`/`.last`/`.is_empty` and O(log n + k) forward/reverse iteration. Reject `.len` on `Set`/`Map` range views at typecheck time with help pointing to `SortedSet`/`SortedBag` (because counting a tree range costs O(log n + k), not O(1)/O(log n)). Enforce ordered-container catalog parity in `crates/cpl-cli/tests/builtin_catalog.rs`.
  5. Hold `drain(after:/before:, while:)` pending the syntax lane's general answer on keyword-labelled parameters for ordinary functions.
- **Evidence.** `SortedSet`: 2 corpus ports (`cf2066e.cp`, `cf2262b.cp`) and 3 dreams (`dream/cf2062e2.cp`, `dream/cf2066e.cp`, `dream/cf2262b.cp`) use it; 0 name collisions across the corpus; emitted C++ is byte-identical across measured ports (`cf2066e` 33,421 B, `cf2062e2` 28,364 B, `cf2262b` 17,270 B, `cf2022c` 9,560 B) and build time is within noise (`-25.6..+2.4 ms`). Rewriting `cf2262b` collapses 3 parallel structures (`Fenwick`, `Set`, `Map`) into one `SortedSet<I>` and passes `gate.sh cf2262b 20`.
- **Reverse.** Remove `SortedSet` from `stdlib/prelude.cp`, restore `first_where` in `stdlib/bag.cp`, and remove `Set`/`Map` branches from `sorted_range_call`/`SortedView` and `Set.set` lowering.

## 115. Extend element narrowing to 2D grids, nested sequences, and all-ranged tuple elements (2026-11-21)

- **Decision.** Replace `Program::narrow_elems`'s flat `IntWidth` with recursive `NarrowLayout` (`Int(IntWidth)`, `Tuple(Vec<IntWidth>)`, `Seq(Box<NarrowLayout>)`), narrowing multi-dimensional arrays (`[R, C]lo..=hi` -> `cpl::Grid<uint32_t|int32_t, 2>`), nested arrays/lists (`[D][E]lo..=hi`, `[D]List<lo..=hi>` -> `std::vector<std::vector<uint32_t|int32_t>>`), and sequences of all-ranged tuples (`[V](r1, r2)` -> `std::vector<std::tuple<uint32_t|int32_t, ...>>`). Flow analysis tracks `(ElemLoc, depth)` and widens the root container conservatively on any mismatch (plain `[R, C]Int` / `[E]Int` / `[V](Int, Int)` params, incompatible assignments/push/extend/for-binding, `mut` element escapes, or non-natural sort). Leaf `Int` and `Tuple` reads widen to `ll` / `std::tuple<ll, ...>` before arithmetic.
- **Why.** Multi-dimensional DP tables, nested adjacency/bucket tables, and ranged pair lists are among the largest memory consumers in contest solutions; generalizing `NarrowLayout` handles all three with one recursive rule instead of separate passes, while preserving 64-bit arithmetic and interpreter/C++ parity.
- **Reverse.** Restrict `RangeInfo::narrow_elem_layout` in `crates/cpl-ir/src/lower/range.rs` back to 1D `Ty::Int` sequences only.

## 118. `Compressed` and `Reversible` leave the prelude for `Tree`, `IntervalMap`, `SortedSet` and `divmod` (2026-11-21)

Four names with users who write them without a `use` were queued for a prelude that held 78 of its
80 names: `Tree` (30 ports, 3 dreams), `IntervalMap` (2 ports, 2 dreams), `SortedSet` (3 dreams) and
`divmod` (3 dreams). `IntervalMap` (113) and `SortedSet` (117) took the last two slots. Following
21.5, two names that only library authors write move out to make room for the other two: `Compressed`
is the return type of `compress` and no port or dream names it, and `Reversible` is a trait bound named
only in `stdlib/` and `examples/local-lib`. Programs that call `compress` are unchanged. Reverse by
restoring the two lines in `stdlib/prelude.cp` and dropping two of the four.

## 122. `std.lazy.LazyTreap` sizes and priorities are `0..2^31` (2026-11-21)

`TreapNode.sz` and `TreapNode.pri` are `0..2_147_483_648` (priorities are
the top 31 bits of `Rng.next()`), so a node with an `Int` value and a
two-word tag is 48 B instead of 56 B, as decision 120 did for arcs. A treap
of 2^31 nodes does not fit in memory, so the bound costs no program; a size
past it traps in the interpreter and in checked C++ (`runtime_errors.rs`).
The child links stay `?Int`: a `?(0..2^31)` field is still an 8-byte
`OptInt` in the C++. cf2084g2 at n = 5e5, best of 9: 130.0 -> 127.4 ms with
`reserve`, 146.5 -> 138.3 ms without. Reverse: `pri: U64`, `sz: Int`.

## 123. Mutable slice views (`cpl::MutView<T>`) via existing range-lending rules (2026-11-21)

- **Decision.** A range slice `xs[l..r]` (or `xs[l..=r]`, `a[D(l)..D(r)]`, `cuts[c1..c2]`) of a mutable `List<T>` or 1D `[D]T` (nominal, range, or cut domain) is an O(1) fixed-length mutable view (`Ty::MutView<T>` in IR, `cpl::MutView<T>` holding `T* ptr` + `size_t n` in C++17, `Value::MutView` holding `(target: Part, lo, len)` in the interpreter). It implements `IndexedMut` (`at`, `set_at`, `swap_idx`) so every in-place sequence mutator (`sort`, `reverse`, `rotate_left`, `rotate_right`, `scan_in_place`, `suffix_scan_in_place`, `shuffle`, `next_permutation`, `prev_permutation`, `nth_element`, `fill`, element assignment `v[i] = x`) works through one shared mechanism with zero copies and no per-method special cases. Length-changing `List` methods (`push`, `pop`, `clear`, `insert`, `remove`, `resize`, `extend`, etc.) are rejected with `E0401`.
- **Lending.** Reuses the existing lending rules (`cpl-check/src/check/lends.rs`) with no core change of character: a slice `let v = xs[l..r]` that is mutated (or passed `mut`) lends `mut` over `Step::Slice(l..r)` of `xs` for `v`'s live range, so any aliasing read or write of `xs` (or an overlapping mutable slice) while `v` is live is a compile error (`E0602`), while statically disjoint slices (`xs[0..2]` and `xs[2..4]`) both work. Read-only slices of immutable (`let`) bases or `Str` remain read-only (`E0401` on mutation). `cpl explain` reports `mutable slice view -> cpl::MutView<T> (T* + len), O(1), fixed-length mut lend`.
- **Evidence.** Interp/C++17 parity test `crates/cpl-cli/tests/programs/seq_mut_slice.cp` (covering `List<Int>`, `[D]Int`, `[D.bounds]Int`, `[0..6]Int`, nested and disjoint slices); checker compile-fail tests in `crates/cpl-check/src/tests.rs` (`mutable_slice_views_lend_and_reject_aliasing`); `cpl explain` test in `crates/cpl-cli/src/lib.rs`; matched benchmark `bench/seq/slice_sort.cp` (`xs[l..r].sort()` vs `std::sort(xs.data() + l, xs.data() + r)`): **1.05x** (213.9 ms vs 202.9 ms, 9,393 B emitted C++); port migrations: `cf2207h3.cp` (678 → 658 tokens, -20, `gate.sh` PASS) and `cf2207h1.cp` (1,007 → 813 tokens, -194, `gate.sh` PASS). Closes `research/frictions/cf2207h3.md` #1b.
- **Reverse.** Restore `note_readonly_view` for list/array slices in `cpl-check/src/check/expr.rs`, remove `Ty::MutView` / `ExprKind::MutSlice` / `cpl::MutView`, and revert `cf2207h3.cp` / `cf2207h1.cp`.

## 124. A half-open range of cuts is also the piece it bounds (2026-11-21)

- **Decision.** With cuts `j, i: D.bounds`, `j..i` is a `Range<D.bounds>`
  (iterating it yields cuts). Wherever a `Range<D>` is expected, it
  converts implicitly to the positions between cut `j` and cut `i`:
  `cost(j..i)`, `pre.sum(j..i)`, `a[j..i]`, `window.move(to: j..i)`. The
  two readings are the same set under `c ↦ c.right`. Every cut in `j..i`
  has a right neighbour because `i` is excluded, and both are stored as
  the same two integers. `j..=i` over cuts stays a range of cuts only
  (its last cut may be the end) and gets no conversion. Constant integer
  literals are accepted for cut-domain values, checked against the
  domain's range (`var mex: Potential.bounds = 0`).
- **Why.** DP transitions iterate cuts (`for j in lo..i: dp[i] min= dp[j] +
  cost(j..i)`) and pass the piece between them on the same line. A
  single rule keeps both readings without a cast or a second spelling,
  and the "ranges are over cuts" model stays exact: a range's ends are
  cuts, its elements are positions.
- **Rejected.** Context deciding what `j..i` means (one text, two
  meanings); `..` versus `..=` choosing the element type; mandatory
  `span(j, i)` (honest but noisy on the commonest DP line, and the
  fallback if the conversion ever surprises reviewers).
- **Reverse.** Drop the conversion; call sites use `span(j, i)`.
- **Status.** Accepted; not implemented. Blocked first on `D.bounds` as a
  value in `cpl run` / `cpl build` (E0501).

## 125. Loops take an `else` that runs when no `break` left them (2026-11-21)

- **Decision.** `for` and `while` accept a trailing `else:` block at the
  loop's indentation, with Python's meaning: it runs when the loop ends
  without `break` (labelled `break 'outer` from an inner loop also skips
  the inner loop's `else`). Labelled `break` / `continue` stay as settled
  in 4.6.
- **Why.** "Search, and handle not-found" is a common contest shape that
  otherwise needs a `found` flag, which is a stale-state bug site.
- **Reverse.** Delete the form; a codemod rewrites it to a flag.
- **Status.** Implemented (parser, checker, formatter, LSP, IR, interpreter, and C++17 codegen; parity test `crates/cpl-cli/tests/programs/loop_else.cp`, fmt test `formats_loop_else`, LSP test `folding_ranges_cover_loop_else`).

## 126. Library loops are generators; early exit stays in `for` (2026-11-21)

- **Decision.** Loop-shaped library facilities (traversals, enumerations,
  divisors, submasks, enter/exit walks) are generators consumed with a
  plain `for`. `break`, `continue`, `return`, labelled `break 'outer` and
  loop `else` (§125) work in that body even when the generator is
  recursive: the compiler inlines a non-recursive push generator to a
  jump, and threads a hidden stop path through the frames of a recursive
  one, only in specializations whose body can exit (`cpl explain` shows
  it). Enter/exit walks yield events (`Enter(v)` / `Exit(v)`). A body
  that steers the generator uses a method on the yielded item, read by the
  generator after the `yield` returns (`step.prune()` skips a subtree);
  this is ordinary library code. Lockstep over recursive generators is
  `List(g)` (4.2's existing error). Folds and predicates stay closures
  (`t.up(+, finish:)`, `bsearch_first`).
- **Deferred.** `return` / `break` inside a closure literal acting on the
  enclosing function (Kotlin's inline lambdas, Ruby blocks). Revisit only
  if ports keep needing early exit from closure-shaped library calls.
- **Why.** Library loops get exactly the control flow of built-in loops
  with no per-library protocol (no `ControlFlow` values, no `found`
  flags), which is the no-privileged-stdlib test; and no closure ever has
  two meanings of `return`.
- **Reverse.** Allow closure-level non-local exit for call-only closure
  parameters, or require library loops to accept a stop signal.
- **Status.** Accepted. `break` and `return` out of a `for` over a
  recursive `yields` fn already run identically in `cpl run` and the C++.
  Not yet done: item-steered `prune`, `Enter`/`Exit` events, labelled
  break and loop `else` coverage over generators, and converting the
  remaining callback-shaped stdlib traversals.

## 127. Recursive lambdas; no method overloading (2026-11-21)

- **Decision.** A lambda bound with `let` may call itself by its binding
  name (`let fact = n => if n == 0: 1 else: n * fact(n - 1)`). It is
  inferred like any lambda; when the recursion leaves the return type
  undetermined, the error asks for an annotation (`(n: Int) -> Int =>`).
  A `var` binding cannot refer to itself. Recursive lambdas and recursive
  nested `fn`s lower to a self-passing generic C++ lambda, never
  `std::function`.
- **Decision.** Methods do not overload by argument type, like free fns
  (see "free fns cannot overload" above). One name has one generic signature; variants are
  named arguments (`at:` / `over:`) or protocols.
- **Rule.** Every accepted decision states its C++ lowering and cost. A
  `std::function`, hidden allocation or hidden traversal is logged as
  debt in `plan/DOMAINS.md`.
- **Why.** A self-passing lambda is a direct monomorphized call, so
  recursive lambdas cost nothing over a nested `fn`. Overloads make
  errors, hover and docs ambiguous, and break one spelling per idea.
- **Reverse.** Forbid self-reference (nested `fn` only), or allow method
  overloads with a resolution rule.
- **Status.** Accepted; recursive lambdas not implemented (today they fall
  back to `std::function`, decisions around `DynamicStdFunction`).

## 128. Moves, overlapping `mut` places, and map defaults (2026-11-21)

- **Decision.** Returning a container is always O(1): returns move, never
  copy. `let b = a` on a heap value (`List`, `Map`, `Str`, structs holding
  them) is a move when it is `a`'s last use and otherwise an error whose
  fix is `a.clone()`. Using a moved-from variable is a compile error;
  moving out of a place that stays alive is `x.take()`, which leaves it
  empty. Functions never take ownership of an argument: parameters are
  lends, `mut` parameters may mutate (and `absorb(mut other)` may drain)
  but the caller keeps the variable.
- **Decision.** Two `mut` places in one container in the same call
  (`f(mut a[i], mut a[j])`) are a compile error unless the compiler proves
  them distinct. Changing a container's length while a place in it is
  lent (`let x = mut xs[i]` then `xs.push(...)`) is a compile error, the
  same rule as overlapping slices (E0602). Both are "for now": a runtime
  distinctness check can relax the first later.
- **Decision.** `m[k] += 1` (any compound assignment into a map) inserts
  the value type's default for a missing `k` first. A plain read of a
  missing key is `none` for `Map` and the default for `Counter`.
- **Why.** Costs stay visible (no silent deep copy), invalidation and
  aliasing bugs can't be written, and counting reads like the idea.
- **Reverse.** Implicit copies for `let b = a`; runtime overlap checks;
  `m[k] += 1` on a missing key as an error.
- **Status.** Accepted (Ryan, 2026-11-21). Audit current checker behaviour
  against each rule; implement the gaps with compile-fail tests.

## 129. Compound assignment is a statement; `min=` / `max=` report improvement (2026-11-21)

- **Decision.** `x op= y` (`+=`, `-=`, `x f= y` for any two-argument fn)
  is a statement with no value, like plain `=`. The exception is `min=`
  and `max=`: as an expression they are a `Bool`, true when `x` strictly
  improved (including from `none`), so relaxation reads
  `if dist[v] min= nd: parent[v] = u`. Used as a statement the `Bool` is
  discarded. Bump-and-read uses a named mutable alias, which evaluates
  the place once: `let k = mut count[c]; k += 1`.
- **Why.** No `if x = y` bugs and no evaluation-order puzzles; the one
  value contestants constantly test after an update ("did it improve?")
  has a direct spelling with one comparison, as in hand-written C++.
- **Rejected for now.** Every `op=` yielding the new value
  (`let k = count[c] += 1`): convenient, but it puts writes inside
  expressions; revisit with port evidence.
- **Reverse.** Make `min=` / `max=` plain statements, or give every `op=`
  the new value.
- **Status.** Accepted (Ryan, 2026-11-21); not implemented.

## 130. Chained comparisons point one way (2026-11-21)

- **Decision.** `a < b <= c` keeps Python's meaning (each operand
  evaluated once, links joined by `and`), as implemented today, but only
  for chains in one direction: all of `<`/`<=` or all of `>`/`>=`.
  Mixed directions (`a < b > c`) and chains containing `==`/`!=` become
  a compile error suggesting `and`.
- **Why.** `0 <= i < j <= n` reads like the statement and costs one
  comparison per link; mixed chains are almost always a typo and read
  ambiguously (C++ compiles `a < b < c` as `(a < b) < c`).
- **Reverse.** Accept every chain again.
- **Status.** Accepted; today mixed chains are accepted (`print(1 < x > 2)`
  runs), so the checker needs the error plus a compile-fail test.

## 131. Inference unifies siblings within one expression, never across statements (2026-11-21)

- **Decision.** Element types of list literals, `if`/`else` branches,
  `match` arms and comprehension results are unified across all siblings
  before generic parameters are fixed, with `none` joining `T` to `?T`:
  `[Pair(1, none), Pair(2, 3)]` is a `List<Pair<Int, ?Int>>`. Inference
  never looks at later statements: `var x = none` followed by `x = 3`
  stays an error asking for `?Int` where `x` is declared. Range inference
  (model A: `let` keeps its computed range, an unannotated `var` widens to
  `Int`, `var x: lo..=hi` checks each assignment) stays intraprocedural,
  goes through arithmetic on `let`s, runs no loop fixpoints and crosses no
  calls; whatever it can't prove keeps its runtime check.
- **Why.** Fixes the reproduced error on `[Pair(1, none), Pair(2, 3)]`
  while keeping inference local, predictable and linear (compile speed
  beats exotic analysis). Range inference only affects widths and elided
  checks, never meaning.
- **Reverse.** First-element inference; or flow-sensitive inference
  across statements.
- **Status.** Accepted; sibling unification not implemented.

## 132. No custom destructuring; views are methods returning enums (2026-11-21)

- **Decision.** Patterns match only the stored shape of structs, enums
  and tuples (with nesting, guards, `if let`, `let ... else`). A library
  type cannot define its own pattern (Scala `unapply`, F# active
  patterns). A view is a method returning an enum:
  `match t.node(v): Leaf => ... Inner(l, r) => ...`.
- **Why.** A custom pattern runs arbitrary code while looking like a free
  field read (costs stop being visible), defeats exhaustiveness checking,
  and needs a matcher protocol through the checker, LSP and trace, for a
  saving of one method call.
- **Reverse.** If 3 or more library families need the method-returning-
  enum workaround and it hurts readability, add declared view enums that
  lower to that method call, keeping exhaustiveness and `cpl explain`.
- **Status.** Accepted (closed by design).

## 133. Named-argument and record-field punning `f(x:)` (2026-11-21)

- **Decision.** A named call argument or parenthesized record field whose value is a variable of the same name may be written in punned shorthand: `f(moves:, cost:, step:)` means `f(moves: moves, cost: cost, step: step)`, and `(moves:, cost:)` means `(moves: moves, cost: cost)`.
- **Mechanism.** Parser-only desugaring in `Parser::arg` and `ExprKind::Record` (`crates/cpl-syntax/src/parser/expr.rs`): when a `name:` label is immediately followed by `,` or `)`, the parser builds `ExprKind::Name(name)` at `name.span`. No checker, IR, interpreter, or C++ codegen special case is needed; LSP hover resolves the local's type through the synthesized expression span. `cpl-fmt` preserves tokens strictly and formats punned calls/records without spaces before `:` or `,`. Lint `L0209` (`redundant_arg_name`) warns on `name: name` with a machine-applicable `cpl fix` to `name:`.
- **Evidence.** Corpus audit found 155 exact `name: name` sites across 54 port files and 75 across 10 stdlib files (52 reviewed port files and all 10 stdlib files migrated via `L0209`; 2 unreviewed ports left untouched for review-check). Interp/C++17 parity test `crates/cpl-cli/tests/programs/named_arg_punning.cp` (`345 12 7`), formatter test `formats_punned_named_args_and_records`, LSP test `hover_on_punned_named_arg_shows_local_type`, lint test `redundant_arg_name_shortens_to_punning`, and docs in `site/content/types-and-control.md`.
- **Reverse.** Remove the punning branch in `Parser::arg` and `ExprKind::Record`, remove `L0209`, and expand `name:` back to `name: name`.

## 134. A runtime ratio counts only when the original runs 100 ms (2026-11-21)

The scoreboard's runtime p50/max and the per-port runtime ratchet count a port
only when its original runs at least 100 ms on the port's max input
(`PERF_FLOOR_MS`, was 10 ms on either side). Below that the ratio measures
process startup and the scanner, not the algorithm: 142 ports were measured,
136 of them on inputs whose original ran 2-99 ms, and the 6 left had p50
1.54x where the summary said about 1.0x. A port below the floor is recorded
as `perf: unmeasured` with the reason; it gets counted again once its
generator (or a spec in `bench/corpus_generators.py`) reaches the
statement's max constraints. Specs at max constraints were added for
cf2261g, cf2115f1, ucup5s01a and cf2062h (n = 14). A spec with no generator
marks its port unmeasured with the spec's reason instead of falling back to
the adaptive probe: cf2157c, whose statement limits are not on file. Reverse: set `PERF_FLOOR_MS` back to 10
and apply it to `max(cpp_ms, orig_ms)`.

The ratio compares one compiler with itself: when the original only builds
with `zig c++` (C++23 deducing `this`, as in cf2261g), the port's timed
binary is built with `zig c++` too, and `perf.compiler` records which. Before,
such a port's g++ build was timed against a clang-built original, which put
cf2261g at 1.39x where the same compiler gives 0.99x. Reverse: time
`port_bin` in `evaluate_full_port` again.

## 135. A loop reaches a row it never moves once (2026-11-21)

Inside `for x in ..`, a row of a nested array reached by indices the body
never changes (`ways[mid][lo]` inside `for rest`) is found and checked the
first time the body reaches it, then reused through a pointer
(`cpl::lazy`). It is kept only when nothing in the body can move the row or
change a size checked on the way to it: no store, push or `mut` argument at
a shallower depth of the same array, no closure or call through a fn value,
and no call to a fn whose summary writes that array; a row of a parameter
is kept only in a body with no calls. Rows reached without a check are not
cached. Each check that is skipped would pass, and a loop that never reaches
the row never checks it, so traps and output are unchanged: it majorizes
(the cost is a few bytes per loop). Why now: cf2062h's subset loop repeated
`cpl::at(cpl::at(ways, mid), lo)` per element and ran 0.62 s vs the
original's 0.46 s; with this it runs 0.39 s. Corpus: +10,314 B over 29
programs (max +4.7%, cf2062h), none broken. Reverse: return no rows from
`Emitter::loop_rows` in `crates/cpl-codegen/src/rowcache.rs`.

## 136. A local read for the last time moves into its place (2026-11-21)

`x = y`, where `y` is a local its block declared and the rest of the block
never names (no read, store or `mut` argument) and the block has no closure,
writes `x = std::move(y)`. The next pass of a loop declares `y` afresh, so
nothing can see the moved-from value: output and traps are unchanged and
the store is never slower, so it majorizes. Comprehension blocks already
moved their result this way; this is the same rule for any block. Why now:
ucup5s01a's `dp = next` per round copied a 500-element row: 1.16-1.25 s
before, 1.10-1.17 s after, vs 1.08-1.23 s for the original (same input,
same g++). Corpus: +1,111 B over 87 programs, none broken. Reverse: return
false from `last_read` in `crates/cpl-codegen/src/stmt.rs`.

## 137. A recursive local closure is passed itself, not a `std::function` (2026-11-21)

Implements §127 for nested `fn`s. A local closure or nested `fn` that
calls itself, bound once and not hoisted, emits
`auto go = cpl::rec([...](auto& go_2, ll u) { ... go_2(...) })`: the
lambda takes itself as its first parameter instead of capturing its own
`std::function`, so every call is a direct, inlinable call of one
closure type. `cpl::Rec` passes `*this` first, so `go(1)`, `go` passed on
as a value and the body's own calls all keep the source's argument list.
`cpl explain` reports it `static: auto`. Why: the indirect call through
`std::function` blocked inlining on every recursive step; on a 2e6-node
subtree-size DFS run 20 times, 1.09 s -> 0.97 s against 0.83 s for the
hand-written self-passing lambda (the rest is the checked `adj[u]` and
the reading loop). Corpus: programs emitting `std::function` 61 -> 37;
the 24 programs that lost it grow +143 to +201 B (+4,366 B in total),
the price of `Rec` over `#include <functional>`. Reverse: drop the
`local_rec` arm of `Stmt::Let` in `crates/cpl-codegen/src/stmt.rs`.

## 138. A recursive fn's closure parameter is a template (2026-11-21)

Implements §127 for fn parameters of recursive fns. A fn-typed parameter of
a fn in a call cycle is a template parameter (`template <class F0> ll
walk(ll n, F0&& f)`), monomorphized per closure type like any other fn's,
when every call to that fn from inside its cycle passes a template fn
parameter of the caller straight on: `walk(n - 1, f)`, or `odd(n - 1, f)`
inside `even`. Then the recursion instantiates one closure type. A call
that passes anything else, such as `wrap(n - 1, x => f(x) + 1)`, would
instantiate without end, so that parameter keeps `std::function` and
`cpl explain` says "a call in its recursion passes a different closure".
A parameter that only ever holds one closure was already lowered direct.
Why: the indirect call through `std::function` blocked inlining on each
step; a 2e6-node tree walk with two different weight closures, run 10
times each, goes 1.04 s -> 0.98 s against 0.84 s for the hand-written
template (the rest is the checked `adj[u]` and the reading loop). Corpus:
no program had the shape, so programs emitting `std::function` stay at 37;
parity test `rec_fn_param`. Reverse: put `|| self.recursive_fns.contains(&id)`
back in `compute_template_params` and drop
`keep_recursive_templates_finite` in `crates/cpl-codegen/src/devirt.rs`.

## 139. Offline bundle splits port galleries into a lazy `gallery` part (2026-11-21)

- **Decision.** `site/bundle.py` ships `ports.html` (keeping its side-by-side
  `Accepted C++` pane) and `exemplars.html` plus their `#gallery-share-ref=`
  payloads in a separate gzipped `gallery` part (`CAPS = {"compiler": 2_100_000,
  "docs": 500_000, "gallery": 150_000}`), unpacked lazily by `site/offline.html`
  on first visit to a gallery page. `site/exemplars.py` emits `#share=` Try-it
  links only for the 12 featured side-by-side pairs (linking featured index rows
  to `#ex-<id>` instead of embedding full port source for every dream in the
  index table).
- **Why.** Every new dream in `proto/ports/` previously added ~1.2 KiB gzipped
  directly into `docs` (pushing `docs` to 611 KiB against the 620 KiB cap).
  Splitting `gallery` makes `docs` invariant to port count (436 KiB vs 500 KiB
  cap) and cuts `gallery` to 90 KiB (vs 150 KiB cap) with O(1) featured cards.
- **Reverse.** Merge `gallery_pages` and `gallery_shares` back into `docs` in
  `site/bundle.py`, remove `ensureGallery()` in `site/offline.html`, and restore
  per-row `_share_href` in `site/exemplars.py`.

## 140. A fn field nothing reads is dropped (2026-11-21)

Step one of §127 for struct fields. A field of fn or optional-fn type that
no expression reads (no `s.f`, no `mut s.f` argument) holds no behaviour, so
it is elided like a field holding one known closure: a `bool` placeholder,
constructors pass `false`, and a store `s.f = g` is left out. A value or
store is left out only when evaluating it runs nothing and cannot trap (a
variable, a captureless closure, `none`, `some` of one), so the interpreter
and the C++ still agree. `cpl explain` reports `[static: dropped]`. The
common case is `SegTree.source`/`LazySegTree.source` in a program that never
calls `refresh`: it was a `std::optional<std::function<...>>` even when every
construction left it `none`. A `?fn` field that is read keeps
`std::function` with "optional fn field that is read". Why: the
`std::function` costs size and pulls `<functional>` for nothing. Corpus:
programs emitting `std::function` 37 -> 22; the 15 that changed shrink
81-157 B each (1,480 B total); no g++ syntax failure is new. Parity test
`unread_fn_field`. Reverse: drop `drop_unread_fn_fields` in
`crates/cpl-codegen/src/devirt.rs` and the unread-store skip in
`StmtEmitter::assign`. *Amended by §142:* such a field is now left out of
the struct entirely, like any unread field, instead of a `bool` placeholder.

## 141. Dream gaps get one error that names the construct (2026-11-21)

- **Decision.** Each class of dream construct the front end does not have is reported once, at the construct, as `E0504` "not implemented yet: <feature> (see design/…)" or "(not in the design)": a domain parameter `fn f<domain D>`, several domains on one `domain` line, a value on the indented lines after `=`, a line continued by an indented operator line, statements in parentheses `(a; b)`, the range spelling `lo <.. hi`, a domain used as a value, the cuts of a domain spelled `D.Cut` (design §21.4; today `D.bounds`), an operator used as a value (`compose: +`, design `stdlib/ranges.md` §6.1; read as an error node), and a `var` binding inside a pattern (`(var a, b)`; the binding is read as a plain name and its mutability is not checked, so the one error is the `var`). Planned forms met while the parser speculates are queued and reported only if that branch is kept. `Sink::finish` keeps only the first `E0504` per message, since every later use of the feature is the same root cause. An unknown type close to a built-in or stdlib type says "not in the stdlib (yet)" beside the did-you-mean; a type used as a value is reported as a back-end limitation.
- **Why.** The dream gate counted 29/60 dreams whose first error named the missing capability; generic "expected X" and mismatch errors hid what each dream needs. Each fix is a class of construct with a golden in `crates/cpl-cli/tests/diagnostics/`.
- **Reverse.** Drop the entries from `PLANNED` in `crates/cpl-syntax/src/planned.rs` with their parser/checker call sites, the `speculative_planned` queue in `crates/cpl-syntax/src/parser.rs`, the `E0504` dedup in `Sink::finish`, and the `Dim::Enum` arm in `Lowerer::list`.
- **Addendum.** More classes are named the same way: `match` arms on the `match` line (`match x: 0 => a; _ => b`, read into real arms), `let … in`, maps `{k: v}` and sets `{x for x in xs}` in braces, `m default 0`, `lazy[D] i => …`, `domain A = V if p(v)`, a pattern in place of a parameter name (`fn f((r, c): Pt)`, design §14.1; the parameter binds and the pattern is unpacked on the body's first line, so later errors surface), `some x` patterns, `[_]T`, `a <=> b then c <=> d`, `.member` without its type, and types of literal values (`'+' | '-'`). A context `with io(...)` says "not implemented yet, and deferred" (`DEFERRED` in `planned.rs`). An indented `if` or `match` after `=` is the value (the core lane implemented it); any other indented value stays planned. `let mut x` is read as `var x` with a fix, and `mut` in a `let` condition is an error whose fix removes it. A capitalized unknown name is offered only capitalized names, since it is a type or a variant. *Why:* the owner's dream buckets (indented arms, stray tokens, `?` in a branch, contexts); the gate went from 64/84 first errors and 167/219 errors named to 68/84 and 182/219. *Reverse:* drop the entries and their call sites, `DEFERRED` with `Planned::deferred`, and the `let mut` and label arms in `stmt.rs`, `pattern.rs` and `item.rs`.

## 142. A field nothing reads is left out, with its stores (2026-11-21)

- **Decision.** §140 for every field type. A struct field that no
  expression reads is left out of the emitted struct, its constructors and
  its `cpl_tie`. A read is `s.f`, `mut s.f` as an argument, a store that is
  not a plain `=` (`s.f += 1` reads `s.f`), or any operation that sees the
  whole value: comparing, ordering, hashing, printing, reading from input,
  sorting naturally, or using the struct as a map key or set element, which
  keeps every field of that struct and of the structs inside it. A store
  `place = v` into a dropped field writes nothing; it is left out only when
  the place cannot trap (every index is a position of the domain its array
  is over), and `v` is still evaluated as `(void)(v);` unless it runs
  nothing, so its traps and effects stay (reading an array at positions of
  its domains runs nothing). A local whose only uses are such stores,
  `xs.push(v)` of a value that runs nothing, and being placed in a dropped
  field is left out with them, which is what removes the work; repeated
  until nothing more goes, since a dropped store reads nothing. A
  parameter used only that way stays in the signature as
  `[[maybe_unused]] const T&`, so it is not copied. A
  method that is never called reads nothing. `cpl explain` lists each
  dropped field as `[dropped] struct S.f`.
- **Why.** Result structs carry witnesses most callers never read:
  `Paths.back` and `Paths.order` in std.graph. g++ keeps the stores because
  the arrays escape through the returned struct. The interpreter still
  computes every field; since nothing reads a dropped field, both print the
  same and trap at the same place.
- **Reverse.** Drop `drop_unread_fields`/`drop_dead_locals` in
  `crates/cpl-codegen/src/devirt.rs` (back to fn fields only) and the
  `dropped_store` skips in `crates/cpl-codegen/src/stmt.rs`.
