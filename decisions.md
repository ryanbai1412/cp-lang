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

<<<<<<< HEAD
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

||||||| parent of c9932ae (docs: document fresh domains, existential returns and decisions)
=======
<<<<<<< Updated upstream
>>>>>>> c9932ae (docs: document fresh domains, existential returns and decisions)
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

## 29. Editor and LSP (2026-11-21)

29.1. **Hover shows the declaration's `///` doc, wherever it is declared.** `Hover` gains `doc`, read
from the lines above the definition `definition_location` finds (decorators skipped), so a stdlib name
shows the doc that states its cost. A member access records its owner type (`HoverEntry::owner`), so
hover and go-to-definition on `bit.prefix` reach the method in `stdlib/fenwick.cp`. The doc is the one
place a cost is written today; a computed cost line can join it later. Reverse: drop `Hover::doc` and
`HoverEntry::owner` in `crates/cpl-check/src/{hover,check}.rs`.
<<<<<<< HEAD

29.2. **Neovim, Emacs and Helix get thin setups over `cpl lsp`, smoke-tested in the gate.** Each
setup is the editor's own idiom (a Neovim runtime folder, an Emacs major mode registering with eglot
and lsp-mode, a Helix `languages.toml`) and holds no logic beyond highlighting and starting the
server, so a feature lands once, in the server. Their keyword lists are checked against
`TokenKind::keyword` by `editors/smoke.sh`, which `scripts/check.sh --full` runs, skipping (and
saying so) an editor that is not installed. Zed waits on a tree-sitter grammar, which it requires for
any language. Reverse: delete `editors/{neovim,emacs,helix,smoke.sh}` and the `--full` step.

## 30. Front-end recovery (2026-11-21)

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
