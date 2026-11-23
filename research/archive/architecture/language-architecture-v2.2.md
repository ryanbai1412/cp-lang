# CP Lang v2.2 — Unified Language Architecture (Wave 4 Lock)

**Status:** Locked specification (2026-11-20). Supersedes `research/archive/architecture/language-architecture-v2.1.md`.
**Evidence base:**
- Subtraction Audit #2 (`research/archive/audits/subtraction-audit-2-v21.md`, `d6b3c4c` — 642 canonical `.cp` ports across every CP domain, plus `stdlib/` and `examples/local-lib/`)
- Syntax Bakeoff #2 (`research/archive/architecture/syntax-bakeoff-2-v21.md`, `a9ca4f8` — 20 hard multi-topic ports + 611-port incidence census across five architectural tensions)
- Blind Critic #4 (`research/archive/blind-evals/blind-critic-4-v21-audit.md`, `02f6137` — 8 blind 2 AM Olympiad/ICPC drafts + 3 mid-contest pivot and planted-WA drills from the cheat sheet alone)
- Corpus porting Batches 47–51 (`research/archive/frictions/friction-backlog-batch-47-ccc-cco-2.md` through `batch-51-v21-olympiads.md` — 50 hard CCC/CCO, IOI, NOI/NOIP, APIO, and EGOI problems)

---

## 0. Executive Summary & Concept Budget

CP Lang v2.1 proved its core ergonomic thesis: in Blind Critic #4, eight cold-start drafts written from the one-page cheat sheet alone totaled 682 lines against 1,471 lines of original C++ (-53.6% raw LOC) with zero borrow-checker deadlocks and clean differential verification. At the same time, two independent empirical audits over 642 canonical ports revealed creeping bloat in v2.1: 40 reserved keywords (up from 37), 8 visible core traits, decorator parameter mini-grammars (`@memo(flat|hash)`, `@multitest(count:)`), unused helpers (`List.absorb`, `for mut`, `D.chunks`, `r1 & r2`, `.flipped_rows`, `bsearch_first_some`, `slide_until`, `prod`, `Marks<D>`), and five concrete 2 AM cheat-sheet ambiguities that caused real first-draft stumbles.

Wave 4 locks **CP Lang v2.2** as an ultra-lean specification that cuts everything that failed to earn its keep across 500+ ports, keeps every abstraction that empirically pays for itself, and pins every semantic edge case so a contestant reading only the one-page cheat sheet at 2 AM writes compiling, bug-free code on the first try.

**Locked v2.2 Concept Budget:**
- **Reserved keywords: 34 total** (down from 40) — **30 contestant keywords** (including the 3 literals `true`, `false`, `none`) and **4 library-author keywords** (`trait`, `impl`, `lends`, `yield`). Six v2.1 keywords are removed or demoted: `defer`, `assume`, `where`, `inline`, `lend`, and hard keyword `law` (now `@law`).
- **Numeric types: 4** (unchanged) — `Int`, `U64`, `I128`, `F64`. Bounded range types pack to 1, 2, or 4 bytes automatically.
- **Core auto-derived traits: 7** (down from 8) — `Index`, `Seq`, `Eq`, `Ord`, `Hash`, `Show`, `Read`. `Copy` is internalized into compiler value semantics.
- **Library-author traits: 2** (down from 4) — `Indexed` (unified with `IndexedMut`) and `Iterable`. `Gen` is dropped (choice-stream shrinking works on any `@gen` function).
- **`@` decorators: 8 contestant + 3 library-author** — `@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`, `@interactor`, `@grader`, plus library-author `@get`, `@test`, `@law`. Decorator parameter grammars `@memo(flat|hash)`, `@multitest(count:)`, and `@grader const` are deleted.
- **One-page cheat sheet:** Trimmed from 242 lines back under ~205 lines while resolving all five Blind Critic #4 ambiguities inline.

---

## 1. High-Confidence Subtractions Executed

Every cut below requires agreement across the 642-port corpus incidence and the 20-problem multi-topic bakeoff, plus a clean, idiomatic replacement that requires zero new concepts.

### 1.1 Six Keywords Cut (40 -> 34)

**1. Delete `defer`.**
Across 642 canonical ports, `defer` appeared in exactly one file (`noi22p4.cp`), where it mutated a 5-bit `Copy` `Subset<0..5>` bitmask and was strictly longer (8 lines vs 4 pure lines) than passing `mask.with(e)` by value. In Batch 51, `ccc96s3` reached for `defer cur.pop()` only because `prev_permutation` was missing; adding `prev_permutation(mut a)` shrinks `ccc96s3` from 19 lines to 8 with zero `defer`. Worse, `defer` is a 2 AM correctness trap in search-and-keep backtracking (such as `noip04p4.cp`, where an early `return true` must keep the chosen assignment rather than undo it) and cannot express sibling undo without anonymous blocks. Explicit undo is one unmistakable line:

```cp
// Rollback DSU (explicit, works across sibling branches and early returns)
let snap = dsu.mark()
solve(left)
dsu.undo_to(snap)

// Backtracking path push/pop
cur.push(x)
dfs(step + 1)
cur.pop()
```

**2. Delete `assume`.**
`assume` appeared in only 6 of 611 ports (1.0%), saves zero lines and zero keystrokes over `assert` (both are 6 letters), and already caused a real misuse in `cco24p4.cp` where a postcondition self-check was written as `assume` instead of `assert` (silently stripping a sanity check on the judge). In v2.2 there is **one** assertion keyword:

- `assert cond` is always checked (locally and on the judge), feeds proven flow facts to the range and disjointness checker, and lints if `cond` contains a `mut` call.
- Input constraints live in typed `read()` bounds (`let (n, m): (1..=2e5, 1..=n) = read()`), constructive `read(gen: f)` / `@gen`, and `@checker`.
- Local-only diagnostics use `debug(...)`, which is stripped on judge builds.

**3. Delete `where`, `inline`, and `lend` (library-author keywords 8 -> 4).**
Across all 642 ports, `stdlib/`, and `examples/local-lib/`, `where` and `inline` had zero occurrences. Inline generic bounds (`fn f<T: Ord, D: Index>(...)`) cover 100% of generic constraints, and `lends`, `yield`, and closure arguments already carry the inline-desugaring contract without a spelling keyword. `lend` appeared in a single library file (`matrix.cp`): inside a `lends` or `lends mut` body, every exit lends a place, so a plain tail expression (or `return mut place` on an early exit) replaces `lend`:

```cp
pub fn at(mut self, i: D) lends mut T:
    mut self.data[i.to_idx]
```

**4. Demote `law` from hard keyword to `@law`.**
`law` appeared in only 3 `stdlib/` files and 0 contestant ports. Moving it to a library-author decorator (`@law disjoint(i, j) = i != j`, `@law assoc(a, b, c) = op(op(a, b), c) == op(a, op(b, c))`) preserves 100% of disjointness-proof and property-testing semantics while freeing `law` as a regular identifier. In addition, trivial single-index forwarding projections (`mut self.data[i.to_idx]`) auto-derive `disjoint(i, j) = i != j` without writing `@law` by hand.

**Final 34 Reserved Keywords:**
- **Contestant (30, including 3 literals):** `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `and`, `or`, `not`, `assert`, `mut`, `self`, `unchecked`, `true`, `false`, `none`.
- **Library-author only (4):** `trait`, `impl`, `lends`, `yield`.

### 1.2 Core Traits 8 -> 7 & Library Traits 4 -> 2

- **Internalize `Copy`.** Across 642 ports and all of `stdlib/`, `Copy` was written as a user trait bound zero times. Copyability is a compiler value-layout property: small non-heap values (`<= 16 B`, including `Int`, `U64`, `I128`, `F64`, `Bool`, `Char`, `Mod<P>`, `Domain`, `Subset<D>`, and small flat structs/enums) copy implicitly; heap-backed containers move on last use and duplicate via `.copy()`.
- **7 core auto-derived traits:** `Index`, `Seq`, `Eq`, `Ord`, `Hash`, `Show`, `Read`.
- **2 library-author traits:** `Indexed` (unified with `IndexedMut` via `lends` / `lends mut`) and `Iterable`. `Gen` is dropped because choice-stream shrinking works on any `@gen` function taking `rng: Rng`. Internal `stdlib` helper traits (`Reversible`, `Neighbours`, `Target`) remain normal library traits outside the language core.

### 1.3 Decorator Parameter Grammars & Grader Edge Cases Cut

**Delete `@memo(flat)` and `@memo(hash)`.**
`@memo(flat)` appeared in 0 of 611 ports and `@memo(hash)` in 3 of 611, where it was already redundant (`etsr23r2p4` has `prod(K_i) > 2^24`, so runtime auto-dispatch selects the salted flat `CpHashMap` automatically). v2.2 keeps **bare `@memo` only**: the compiler picks a flat niche-packed `[Params]?R` table whenever every parameter is an algebraic `Index` and `prod(K_i) <= 2^24` (or `<= 2^26` for tri-state `?Bool` / `Bool` tables), and falls back to a salted open-addressed hash table otherwise. Anyone who wants explicit custom hash-table storage writes `var dp = HashMap<Key, Val>()`, which is plain userland code.

**Delete `@multitest(count: expr)`.**
Appeared in 1 of 611 ports (`noi22p4.cp`), where it lost head-to-head to a plain `for _ in 0..tests:` loop by +1 LOC, +23 keystrokes, +6 punctuation characters, and an artificial dummy `fn solve():`. Because nested `@memo` is block-scoped in v2.2, any `@memo fn` declared inside `for _ in 0..tests:` resets its table automatically on every iteration. v2.2 keeps bare whole-file `@multitest` (and `@multitest(until_eof)` when a problem terminates at EOF), plus bare `@multitest fn solve():` when file-top precomputation must run once before reading `T`.

**Delete `@grader const` and uninitialized file-scope grader domains.**
`@grader const` served 1 novelty problem out of 642, and uninitialized file-scope `domain Colour` (bound later inside a grader function) appeared in 1 file out of 642 while introducing a half-global domain lifetime. Grader boundaries pass plain `Int` and `[0..n]T` arrays (sibling-parameter array bounds such as `fn assist(needed: [0..r]Bool, n: Int, k: Int, r: Int)` stay because they are purely local to the signature), and construct local `domain`s inside functions as needed. See §3.5 for the locked stateful `@grader` pattern.

### 1.4 `List.absorb` and `for mut x in xs:` Deleted

**Delete `List.absorb(mut other)` / `.absorb()`.**
Both Subtraction Audit #2 and Syntax Bakeoff #2 reject `.absorb()`: 0 uses across 642 canonical ports, blocked by same-container aliasing (`adj[u].absorb(mut adj[v])`), and dangerous because `List.absorb` is O(`|other|`) while `MinHeap.absorb` is O(`|other| log |self|`), masking quadratic TLE traps behind one verb. Worse, real small-to-large DSU-on-tree merges almost always update companion arrays (weights, sums, counts) for each moved element. Keep **`.take()`** and **`swap(mut a, mut b)`**; two uniform lines handle every small-to-large merge without hidden complexity:

```cp
if dst.len < src.len: swap(mut dst, mut src)
for (k, v) in src.take():
    dst[k] += v
    total += v
```

**Delete `for mut x in xs:`.**
Zero uses across 642 canonical ports (only 1 motivating file in all of `proto/ports/`, saving 14 characters), and in Syntax Bakeoff #2 it lured an illegal borrow-array pattern (`[mut empty, mut single, mut ripe]` in `noip22p2`) while Blind Critic #4 hesitated on how to pair it with indices (`for (bi, mut b)`). Deleting `for mut` removes an entire second iterator borrow mode from the compiler and cheat sheet. Indexed loops are uniform and give index plus mutation together:

```cp
for i in 0..xs.len:
    xs[i].query(l, r)

for d in D:
    buckets[d].rebuild()
```

### 1.5 Tropical Optional Trims & the `min=`-Skips-`none` Guardrail

Tropical Optionals (`?Int`, `?F64`) remain one of CP Lang's highest-payoff features — Syntax Bakeoff #2 proved that removing them costs +4 LOC, +945 typed keystrokes (+9.8% on affected files), +221 punctuation characters, and +36 `?` characters across 7 of 20 hard problems (`yac6p6` alone swells by +606 keystrokes and +26 `?`). v2.2 makes two targeted subtractions to remove hidden traps, and locks an explicit guardrail:

- **Delete `?Int`/`?F64` absorption on `sum(gen)`.** Zero uses across 642 ports, and it created a dangerous contradiction where `min(gen)` skips `none` while `sum(gen)` poisoned the whole result to `none`. `sum(gen)` sums non-optional numeric iterables only.
- **Delete `+=` and `-=` lifting on `?Int`/`?F64`.** Zero uses across 642 ports; `x += y` silently becoming a no-op when `x` is `none` masks uninitialized-DP bugs. Write `dp[u] = dp[u] + w` when you explicitly want `none`-absorbing addition, or `dp[v] min= dp[u] + w` for transitions.
- **Lock the `min=`-skips-`none` poison guardrail.** `min`, `max`, `min=`, and `max=` treat `none` as the tropical identity (`+inf` / `-inf`) and **skip** `none`, whereas binary `+`, `-`, and unary `-` **absorb** `none` (and `*`, `/`, `%`, bitwise ops, and `?V + k` never lift). Whenever `none` represents **failure/impossibility** inside a `min=` loop where every branch must succeed (like `etsr23r1p3.cp`), test and propagate failure explicitly instead of relying on bare `best min= cand`:

```cp
// Tropical relaxation: unreachable states are skipped automatically
var dist: [V]?Int = none
dist[s] = 0
dist[v] min= dist[u] + w

// Poison-failure guard: if ANY required branch fails, the whole state fails
var best: ?Int = n
for c in Alpha if c in mask:
    if let prev = dp[mask.without(c)] and let hit = prv[prev, c]:
        best min= hit
    else:
        best = none
        break
dp[mask] = best
```

- **Lock mixed-type `min`/`max` return type.** `min(a: T, b: ?T) -> T` and `min(a: ?T, b: T) -> T` return definite `T` (because skipping `none` on the optional operand always leaves at least the known `T`), while `min(a: ?T, b: ?T) -> ?T`. Three or more values use `min([a, b, c])` / `max([a, b, c])`.

### 1.6 `Marks<D>` Demoted from Core & Shrunk to 32-Bit (`I32`) Stamps

Across 642 canonical ports, `Marks<D>` had zero usage, and Syntax Bakeoff #2 exposed an 80 MB vs 10 MB memory cliff on `10^7`-state tables (`ucrpc21g.cp`) because v2.1 backed `Marks<D>` with 8-byte `[D]Int` stamps.

- **Demote `Marks<D>` out of the core prelude and off the one-page cheat sheet.** Teach plain `[D]Bool` (1 byte/element) with `.fill(false)` for small/one-shot tables and explicit touched-list reset for hot multi-query loops — the same idiom also resets value tables like `[D]?Int` and `[D]List<V>`:

```cp
var seen: [V]Bool = false
var touched = List<V>()
// inside each query:
if not seen[u]:
    seen[u] = true
    touched.push(u)
// O(|touched|) reset:
for u in touched.take(): seen[u] = false
```

- **Shrink optional `std.marks.Marks<D>` to 32-bit stamps.** For 1D multi-query graph problems (`noi21p3`, `egoi22p7`) that opt into `use std.marks.{Marks}`, `Marks<D>` stores its stamp table as `[D]0..=2_000_000_000` (lowering to 4-byte `int32_t` / `I32` storage in C++17, halving memory from 8x to 4x while keeping CP Lang's 4-numeric-type rule intact), resetting the array to `0` if `gen` ever wraps at `2_000_000_000`.

### 1.7 Low-Usage Domain, Range, Grid, Search, Sequence & Stdlib Trims

Every helper below had 0 or 1 uses across 642 canonical ports and is removed from the core language, core `std` prelude, and one-page cheat sheet:

- **Domain & range sugar deleted:** `D.chunks(k)` (use `for lo in (0..n).step(B): let hi = min(n, lo + B)`), `D[..k]` (use `for c in D if c.to_idx < k:`), range intersection `r1 & r2` (use `max(a, c)..min(b, d)`), and `.mirror` on ranges/domains. **Keep `Range.l` and `Range.r`** (heavily used for segment-tree splits `(r.l..m.r, m.r..r.r)` and DP boundaries).
- **Grid flips deleted:** `.flipped_rows`, `.flipped_cols`, and `.flipped` (0/642 ports and 2 AM horizontal-vs-vertical English ambiguity). **Keep `.transposed`** (mathematically unambiguous; used in 4 ports). Explicit comprehensions `[N, N] (r, c) => g[...]` make reflections unmistakable.
- **Search & sequence helpers deleted:** `bsearch_first_some` (plain `bsearch_first` + one final call avoids copying witness state per probe), `slide_until` (cannot hold incremental window state; explicit `while` two-pointer loop is 3 lines), `rotate_right` (unified into signed Euclidean `rotate_left(mut a, k)` where negative `k` rotates right), `prod(gen)` (overflows `Int`; use an explicit loop over `Mod<P>`), `sliding_min`/`sliding_max`, `topo_prefix`, `Dsu.count` (duplicate of `Dsu.components`), and `floor_div` (redundant with built-in Euclidean `/` for positive divisors; keep `ceil_div`). Also fold `topo::<V>(next)` into standard bidirectional inference so `topo(adj)` works directly on `[V]List<V>` and `Graph<V>`.
- **`SparseTable.fold(r)` reverted from `?T` back to `T`:** Asserts `not r.is_empty` locally. Batch 49 (`ioi99p6.cp`) and Subtraction Audit #2 proved returning `?T` breaks arithmetic like `mx.fold(l..=r) - mn.fold(l..=r) <= c` and forces `!` noise on every non-empty inclusive query.
- **Eleven 0–1-use specialized structures demoted from core `std` & cheat sheet to `examples/local-lib/`:** `MonoDeque`, `Steps`, `TwoSat`, `Matrix`, `palindromes`, `shortest`, `walk_back`, `mst`, `convex_hull`, `Dir8`, and `MinCostFlow`. (Note: `next_permutation(mut a)` stays in `std.seq` alongside the new `prev_permutation(mut a)` per Batch 51.)

---

## 2. What Pays Its Rent (Locked Keeps & Core Clarifications)

The following mechanisms demonstrated strong empirical leverage across 500+ ports and stay locked in v2.2:

### 2.1 Index-Before-Place-Borrow & RHS-Before-LHS Evaluation

Batch 50 (`noi20p5`, `noi18p3`, `NP1`) and Syntax Bakeoff #2 Tension 3 proved that RHS-before-LHS evaluation alone is not enough when an index expression itself mutates or grows the same arena (`trie[step(o, BothGoRight)].ends = true` or `trie[o].next[s] ??= trie.push_id(Node())`). v2.2 extends the evaluation-order guarantee uniformly across `=`, `??=`, and every compound assignment (`+=`, `min=`, `|=`, etc.):

1. Evaluate every index subexpression on the left-hand side (left to right) to its index value.
2. Evaluate the right-hand side expression completely (for `??=`, only after reading whether the place is `none`, and **before** re-acquiring the `&mut` place address so arena reallocation cannot invalidate a reference!).
3. Acquire the `&mut` place address in the container and write the result.

```cp
// Both compile cleanly and are reallocation-safe in emitted C++17:
trie[step(o, BothGoRight)].ends = true
trie[o].next[s] ??= trie.push_id(Node())
nodes[u].l = merge(nodes[u].l, b)
```

In emitted C++17, `a[idx_expr].f = rhs_expr` lowers to `auto __i = idx_expr; auto __v = rhs_expr; a[__i].f = std::move(__v);`, eliminating the classic C++ `vector` dangling-reference UB entirely. Immediate borrow release for `Copy` element reads (`merge(hrc[u], v)`) and mutable subslices (`sort(mut a[l..=r])`, `reverse(mut path[i + 1..=j])`) stay locked alongside this rule.

### 2.2 Interactive I/O: Auto-Flush `print()` with `ask()` & Explicit `sol: Interactor`

Syntax Bakeoff #2 Tension 5 and Batch 47 (`dmopc20c6p6`, `cco01p4io`) surfaced two interactive footguns: forgetting `flush: true` on a plain `print()` inside a mixed one-way + request-reply protocol, and confusing which side `read_output()` vs `print()` talked to inside `@interactor`.

- **Auto-flush `print()` whenever `ask()` is used:** If a program calls `ask(...)` anywhere, every `print(...)` in that program flushes stdout automatically (zero extra keywords, zero `flush: true` boilerplate). `print(..., flush: true)` remains valid for pure `print`/`read` interactive solutions that never call `ask()`.
- **Explicit `sol: Interactor` channel:** `@interactor` takes an explicit `sol: Interactor` handle so the two streams can never be confused:
  - `read()` reads the hidden judge test case (generated by `@gen` or fed on stdin).
  - `sol.recv()` reads the solution's stdout.
  - `sol.send(...)` writes to the solution's stdin and flushes automatically.

```cp
@interactor
fn judge(sol: Interactor):
    let (n, secret): (1..=1000, 1..=n) = read()
    sol.send(n)
    var qs = 0
    while true:
        let kind: Str = sol.recv()
        if kind == "!":
            let guess: Int = sol.recv()
            assert guess == secret
            return
        qs += 1
        assert qs <= 12
        let x: Int = sol.recv()
        sol.send(if x < secret: "<" else: ">=")
```

### 2.3 Interior Mutability on `Comb<P>()` & Core Stdlib Polish

- **`Comb<P>()` uses interior mutability for lazy table growth.** `let comb = Comb<MOD>()` is an immutable binding (`comb.choose(n, k)`, `comb.fact(n)`, `comb.inv_fact(n)`, `comb.arrange(n, k)` all take `self`, not `mut self`), backed by the same internal lazy-cache mechanism as `@memo`. Helpers take `comb` without `mut`, and closures or `@memo` functions capture `comb` freely.
- **`List.insert(i, x)` and `List.remove(i) -> T`:** Added to `List<T>` (Blind Critic #4 `ioi11p5` sqrt-bucket splits and Batch 48 `egoi23p5`), lowering directly to `vector::insert` and `vector::erase`.
- **`Rng(seed)`, `rand(lo..hi)`, and `shuffle(mut xs)`:** Surfaced directly on the cheat sheet (Blind Critic #4 + Batch 47 `cco07p5`, `dmopc21c5p5`), backed by deterministic `splitmix64` / `xorshift64*` so treap priorities, randomized algorithms, and anti-hack shuffling never hand-roll RNGs.
- **`sort(mut a, by: desc)`:** Because `desc<T>(x: T) -> Desc<T>` is an ordinary function, passing `by: desc` sorts descending with zero extra API.

---

## 3. Resolving All Five Blind Critic #4 Ambiguities & Batch 51 Wins

Blind Critic #4 tested v2.1 strictly from the one-page cheat sheet under 2 AM contest pressure and identified five concrete specification/cheat-sheet ambiguities, while Batch 51 found two high-leverage zero-keyword wins. All seven are locked below and written verbatim onto the v2.2 cheat sheet.

### 3.1 `Action` Parameter Order, `LazySegTree` Arity, and Segment-Tree Boundaries

Blind Critic #4 (`ioi14p2` Wall, `ioi11p5` Elephants) and Batches 47–49 all stumbled on the argument order of `Action` and whether `LazySegTree` takes 2 or 3 type arguments. v2.2 pins the exact signature:

```cp
// Action<T, F>(id, compose, apply, over: op)
// - compose(new_f, old_f): apply old_f first, then new_f
// - apply(f, val, span):   tag f, folded value val, leaf count span
// - over: bare operator (+, min, max, ...) or Monoid(id, op)
let clamp = Action(
    (0, INF),
    (new_f, old_f) => (max(new_f.0, min(new_f.1, old_f.0)),
                       max(new_f.0, min(new_f.1, old_f.1))),
    (f, val, span) => max(f.0, min(f.1, val)),
    over: min,
)
// LazySegTree<D, T>(act, a) infers tag type F from act
var st = LazySegTree<Col, Int>(clamp, [Col] 0)
st.apply(l..=r, (lo, hi))
let total = st.all
```

Additional pinned guarantees on `SegTree` and `LazySegTree`:
- **`st.all`** is an O(1) read of the whole-domain fold (`t[1]`).
- **`st.update(i, f)`** on `LazySegTree` performs push-modify-pull on leaf `i` in a single O(`log n`) traversal (in addition to `st.set(i, x)` and `st.apply(r, f)`).
- **Empty and inverted ranges** (`lo > hi` or `l..l`) are valid no-ops: `fold` returns `monoid.id`, `apply` does nothing.
- **`max_right(l, pred)` / `min_left(r, pred)`** operate on `D.bounds` boundaries: `max_right(l, pred)` takes a start bound `l: D.bounds` and returns the furthest bound `r: D.bounds` with `pred(fold(l..r))` true; `min_left(r, pred)` takes an end bound `r: D.bounds` and returns the earliest bound `l: D.bounds` with `pred(fold(l..r))` true.

### 3.2 `first_ge` / `last_le` Return Types Across Containers

Because `List<Int>` has the same type (`?Int`) for both an index and a value, Blind Critic #4 noted a dangerous 2 AM confusion between sequence binary search and ordered-set lookup. v2.2 states all three return types side by side:

- **Sequences** (`List<T>`, `[D]T`, slices; optional `by: key`): `xs.first_ge(x)` / `xs.last_le(x)` return the **optional index** (`?Int` for `List`/slice, `?D` for `[D]T`).
- **Ordered sets** (`Set<K>`, `SortedBag<K>`): `s.first_ge(k)` / `s.first_gt(k)` / `s.last_le(k)` / `s.last_lt(k)` return the **optional key** `?K`.
- **Ordered maps** (`Map<K, V>`): `m.first_ge(k)` / `m.first_gt(k)` / `m.last_le(k)` / `m.last_lt(k)` return the **optional pair** `?(K, V)`.

### 3.3 `Map` Entry, Auto-Insert Compound Assignment, and Removal Semantics

Blind Critic #4 (`ioi13p6` Game, `ioi18p1` Combo) flagged uncertainty about whether `m[k] += 1` traps on a missing key. v2.2 locks the exact `Map<K, V>` / `HashMap<K, V>` contract:

- **`m[k]` (read or plain assign read)** traps if `k` is absent.
- **`m[k] = v`** inserts or overwrites `k`.
- **`m.get(k) -> ?V`** non-trapping lookup.
- **`m[k] f= x`** auto-inserts from the operator's identity when `k` is absent (`0` for `+=`/`-=`, `none` for `min=`/`max=` on `?T`, `1` for `*=`, `false` for `|=`, `true` for `&=`), then applies `f= x`.
- **`m[k] ??= init`** evaluates `init` lazily only when `k` is absent, inserts it, and lends a mutable place (`(m[k] ??= []).push(x)`).
- **`m.remove(k) -> ?V`** deletes `k` and returns its previous value (or `none` if absent; resolves Batch 48 §48.6).

### 3.4 `.all`, Complete `Subset<D>` Iteration, and `not in`

Blind Critic #4 (`ioi20p1` Plants, `ioi14p2`) stumbled when `Subset<D>` omitted member iteration and `.all` was missing from `SegTree`/`LazySegTree`. v2.2 makes every `Subset<D>` (`D.K <= 25`, 4-byte `Copy` bitmask) verb explicit:

```cp
// Constructors and set operations
let e = Subset<D>.empty
let f = Subset<D>.full
let m2 = mask.with(v)        // mask | {v}
let m3 = mask.without(v)     // mask \ {v}
let first_elem: ?D = mask.first

// Three distinct, unambiguous iterations:
for mask in Subset<D>:       // all 2^(D.K) subsets, ascending mask order
    for v in mask:           // each member element v: D present in mask
        ...
    for sub in mask.subsets: // each submask sub: Subset<D> of mask
        ...

// Membership negation works uniformly everywhere `in` works:
if v not in mask: ...
```

### 3.5 Stateful `@grader` & `@memo` Interaction with Extern Grader Oracles

Blind Critic #4 (`ioi11p5`, `ioi20p1`, `ioi15p6` Towns) surfaced two grader gaps: how multi-function stateful IOI graders hold state between `init(...)` and `query(...)`, and whether `@memo` can wrap an extern `@grader fn` oracle.

**1. Stateful `@grader` pattern (zero magic global domains):**
Top-level `var` bindings (or a `@grader struct` when the judge instantiates an object) persist across exported `@grader fn` calls. Initialize them in `@grader fn init(...)` and construct local `domain`s inside functions as needed:

```cp
var seg: LazySegTree<0..=200_000, Int> = LazySegTree<0..=200_000, Int>(AddMin, [0..=200_000] 0)
var n_len = 0

@grader fn init(k: Int, r: [0..n]Int):
    n_len = r.len
    ...

@grader fn compare_plants(x: Int, y: Int) -> Int:
    ...
```

(When a `@grader` problem provides an explicit `init` that creates runtime-sized structures whose size isn't known at file load, declare top-level `var state: ?State = none` and initialize `state = State(...)` in `init(...)`, or use `List<T>` / `var` reassigned in `init`.)

**2. `@memo` + extern `@grader fn` oracles:**
`@memo` forbids capturing a mutated local `var` (which would silently return stale cached values) and forbids `read()`/`print()`. However, **`@memo` explicitly permits calling `ask(...)` and bodyless extern `@grader fn` oracles** (such as IOI 2015 Towns' `get_distance(a, b)`). The compiler guarantees that each distinct argument tuple invokes the extern oracle at most once, turning `@memo` into an exact query-budget cache:

```cp
@grader fn get_distance(i: Int, j: Int) -> Int

@memo
fn dist(a: V, b: V) -> Int:
    if a == b: return 0
    if a > b: return dist(b, a)
    get_distance(a.to_idx, b.to_idx)
```

**3. `@memo` scoping & flat-table eligibility:**
- A nested `@memo fn` declared inside a function or loop body owns a table scoped to that enclosing invocation/iteration and resets automatically on each entry (eliminating manual `memo.clear()` across multitests or per-query subproblems).
- Flat packing applies to any `Copy`/value return type (`Int`, `I128`, `F64`, `?Int`, `Bool`, `?Bool`, small structs/enums) when all parameters are algebraic `Index` types with `prod(K_i) <= 2^24` (or `<= 2^26` for tri-state `?Bool` / `Bool` tables); otherwise the runtime automatically uses a salted flat hash table.

### 3.6 Two Zero-Keyword Batch 51 Wins & Small Ergonomic Clarifications

**1. Diverging RHS on `??` (`return`, `break`, `continue`).**
Because `return ...`, `break`, and `continue` have bottom type (`!`), they unify with any expected type on the right-hand side of `??`. This eliminates the awkward `if o == none: return ...; let u = o!` two-line pattern across every treap, dynamic segment tree, and link-cut tree (`ioi13p6`, `egoi22p4`, `ccc05s5`, `noi05p2`):

```cp
let u = o ?? return (none, none)
let top = st.pop() ?? break
let nxt = cur + 1 ?? continue
```

**2. `prev_permutation(mut a) -> Bool`.**
Added to `std.seq` as the exact symmetric partner of `next_permutation(mut a) -> Bool` (fixing `ccc96s3`'s +46.2% library-stripped LOC regression in Batch 51 with zero new keywords).

**3. Zero-keyword micro-clarifications locked for compiler & docs:**
- **Parser diagnostic on `else if`:** Emit an immediate targeted fix-it (`use 'elif', not 'else if'`, Blind Critic #4 §3.7).
- **Derived `enum` `Ord`:** Follows variant declaration order (Batch 50 `NP6`), so `enum Side: Left; Right` satisfies `Left < Right`.
- **Canonical segment-tree range split:** `let m = r.mid!; (r.l..m.r, m.r..r.r)` splits any range `r` cleanly into two half-open subranges using `Range.l` and `Range.r` without raw `.to_idx` arithmetic (Batch 51 `F5`).

---

## 4. Implementation Migration Plan (Follow-Up Sessions)

This Wave 4 commit locks the specification (`research/archive/architecture/language-architecture-v2.2.md`, `docs/one-page-cheat-sheet.md`, and `decisions.md` §19) without touching implementation directories. Follow-up engineering sessions should migrate the codebase to match v2.2 in four crisp passes:

1. **Rust frontend & C++17 codegen (`crates/`):**
   - Remove reserved keywords `defer`, `assume`, `where`, `inline`, `lend`, and hard keyword `law`; add `@law` decorator and auto-disjointness for trivial single-index forwarding.
   - Internalize `Copy`; remove `for mut` loop mode; reject `@memo(flat|hash)`, `@multitest(count:)`, `@grader const`, and uninitialized file-scope domains with clear migration diagnostics.
   - Implement diverging RHS (`return`/`break`/`continue`) on `??`, mixed `min(T, ?T) -> T`, removal of `sum(?T)` and `?T +=`/`-=`, `else if` -> `elif` diagnostic, and index-before-place-borrow + RHS-before-LHS lowering across `=`, `??=`, and compound assignments.
   - Allow extern `@grader fn` calls inside `@memo`; auto-flush `print()` when `ask()` is present; implement `sol: Interactor` (`sol.recv()` / `sol.send(...)`).
2. **Python prototype (`proto/`):**
   - Mirror the same v2.2 syntax/semantics trims, diverging `??`, `prev_permutation`, `List.insert`/`remove`, `Map.remove -> ?V`, `Comb<P>` interior mutability, and `SparseTable.fold -> T`.
3. **Standard library & local-lib (`stdlib/`, `examples/local-lib/`):**
   - Delete `List.absorb`, `D.chunks`, `D[..k]`, `r1 & r2`, `.mirror`, `.flipped_rows`/`.flipped_cols`, `bsearch_first_some`, `slide_until`, `rotate_right`, `prod`, `sliding_min`/`sliding_max`, `topo_prefix`, `Dsu.count`, `floor_div`.
   - Revert `SparseTable.fold(r)` to `T`; add `prev_permutation`, `List.insert`/`List.remove`, `LazySegTree.update(i, f)`, `Map.remove -> ?V`, and `Comb<P>` interior mutability; shrink `stdlib/marks.cp` to `[D]0..=2_000_000_000` (4-byte `I32` storage) and move it out of the default prelude.
   - Relocate the 11 near-zero specialized structures (`MonoDeque`, `Steps`, `TwoSat`, `Matrix`, `palindromes`, `shortest`, `walk_back`, `mst`, `convex_hull`, `Dir8`, `MinCostFlow`) to `examples/local-lib/`.
4. **Docs site, editor grammar & corpus sweep (`site/`, `editors/`, `proto/ports/`):**
   - Update TextMate/LSP keyword lists (34 keywords), docs course chapters, and sweep existing canonical ports for any removed v2.1 spellings (`defer`, `assume`, `.absorb`, `for mut`, `@memo(hash)`).
