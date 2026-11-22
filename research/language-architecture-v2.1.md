# CP Lang v2.1: Wave 3 Unification, Concept-Budget Guard & Prototype Sprint Roadmap

This document locks **CP Lang v2.1** — a strictly budget-preserving refinement of CP Lang v2 (`research/language-architecture-v2.md`, `decisions.md` §16–§17) synthesized from all **21 scout friction backlogs landed across Batches 21–41** (`research/friction-backlog-batch-21.md` through `batch-41-oly-icpc.md`, covering 200+ newly ported hard problems across geometry/`F64`, strings/automata, combinatorics/games/FPS, advanced DS, greedy/bitwise, hard trees, hard graphs, hard DP, number theory/linalg, multi-topic classics, grids/simulation, binary search/sweeps, CCC/CCO/DMOPC/COCI/COI/IOI/NOI/NOIP/DMOJ/Olympiad/ICPC classics), reconciled against the max-N prototype optimization pass (`c8e381f`) and the v2 `stdlib/` + `examples/local-lib/` alignment (`72383ec`, `3d67b55`, `decisions.md` §17).

Across Batches 21–41, the locked v2 core proved overwhelmingly effective: v2 solutions cut raw C++ LOC by **61%–85%** and library-stripped C++ LOC by **39%–69%**, while eliminating real undefined behavior found in accepted corpus C++ submissions (`cco22p3` `--begin()` heap underflow, `apio09p1` `ans[n]` out-of-bounds read, `noi05p2` empty-insert segfault, `ioi08p3` shift UB, and 12–43 MB stack-overflowing VLAs in `aac4p6`, `coci21c1p2`, and `noimps22p4`).

What those 200+ ports exposed is **not** a need for new syntax, but eleven sharp cross-batch seams where two or three locked v2 features collided — plus clear evidence of which prototype gaps account for 80%+ of the `today/` boilerplate delta. Wave 3 resolves every collision by **unifying existing mechanisms, placing niche patterns in userland, and explicitly rejecting bloat**. `docs/one-page-cheat-sheet.md` and `decisions.md` §18 are updated to match this document 100%.

---

## 1. Concept-Budget Guard (Zero Expansion in v2.1)

Every hard surface budget locked in Wave 2 holds without a single addition in v2.1:

- **40 reserved keywords total** (32 contestant including `true`, `false`, `none`, plus 8 library-author). Zero keywords added: `static`, `undo`, `loop`, `step`, `into`, `pool`, `by`, `with` remain permanently excluded.
- **4 numeric types:** `Int`, `U64`, `I128`, `F64`. Zero numeric types added (`I32`/`U32` stay rejected; range types pack to 1/2/4 bytes in structs and arrays automatically).
- **8 + 3 language traits:** `Index`, `Seq`, `Copy`, `Eq`, `Ord`, `Hash`, `Show`, `Read`, plus library-author `Indexed`/`IndexedMut`, `Iterator`, `Gen`. Zero traits added.
- **7 + 2 `@` decorators:** `@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`/`@interactor`, `@grader`, plus library-author `@get`, `@test`. Zero decorators added (no `@once`).
- **One canonical spelling per construct** and **`docs/one-page-cheat-sheet.md` stays <= 1 page** (~245 lines).

---

## 2. Eleven Cross-Batch Clusters & Multi-Feature Collisions Resolved (U1–U11)

### U1. Tropical Optional (`?T`) Arithmetic, `min=`/`max=`, Equality, & Unparenthesized-`??` Guard

*Unifies Batches 27 (`HG9.5`), 28 (`D3`, `D9`), 32 (`X1`), 33 (`C10`, `C11`, `C17`), 34 (`UD11`), 35 (`K2`), 36 (`F4`), 38 (`38.4`), and 39 (`F39.2`, `F39.7`).*

Nine separate batches hit the exact same ergonomic cliff: v2 tells contestants to replace `INF` sentinels with `?Int`/`?F64` and makes `min`/`max`/`min=`/`max=` skip `none`, yet leaves arithmetic (`+`, `-`, `+=`, `-=`) unlifted. The instant a DP or shortest-path transition adds an edge or cell cost (`dp[u] + w`, `from_low.fold(..=x) - x * down`), every port degenerated into twelve nested `if let` guards (`yac6p6`), hand-written `fn plus(a: ?Int, b: ?Int) -> ?Int` helpers (`coci21c1p3`), or `fn shift(v: ?Int, by: Int) -> ?Int` wrappers (`ceoi22p5`, `rpeng3`, `ioi04p2hard`) — making `?Int` strictly longer than `INF = 1e18`!

At the same time, blindly lifting `+` over `?T` would revive the Blind Critic 2 precedence bug (`best[i] = a[i] + max(...) ?? 0` silently parsing as `(a[i] + none) ?? 0 == 0` instead of `a[i]`) unless `??` precedence is guarded.

**Locked v2.1 rule (the Tropical Optional Rule — zero new keywords, zero new operators):**

1. **`none` is the identity of `min`/`max`/`min=`/`max=`:** skips `none` on either side, including `min(gen)` / `max(gen)` over iterables of `?T` (returning `?T`) and `ans min= cand` whether `ans` is `?T` or plain `T`.
2. **`none` is the absorber of `+`, `-`, `+=`, `-=`, unary `-`, and `sum(gen)` on numeric optionals `?Int` and `?F64`:** if either operand is `none`, the result is `none` (`none + w == none`, `none - k == none`, `none += dx` stays `none`, and `sum(depth[p] for p in P)` over `?Int` returns `?Int`, yielding `none` if any term is `none`). Multiplication, division, modulo, bitwise ops, and domain stepping (`?V + k`) do **not** lift (avoiding `0 * inf` and `(v + 1) - 1` boundary traps).
3. **Unparenthesized mixing of `??` with binary arithmetic (`+`, `-`, `*`, `/`, `%`) is a compile error:** writing `a[i] + max(...) ?? 0` is rejected with the exact parenthesized fix `a[i] + (max(...) ?? 0)` or `(a[i] + max(...)) ?? 0`.
4. **Equality `==`/`!=` auto-lifts `T` to `?T`, while `<`/`<=`/`>`/`>=` stay compile errors:** `if dq.last != m - i - 1:` and `if opt == 0:` work directly (`none != x` is `true`, `none == x` is `false`), while `?T` still does **not** implement `Ord` so `MinHeap<(?Int, V)>` remains impossible.
5. **Chained `if let` / `while let` and tuple `match (a, b):`:** Document `if let a = x and let b = y:` and `while let ci = i and let cj = j:` on the cheat sheet (`apio09p1`, `dpf`, Batch 38 `38.4`); combining custom optional structs (`?Entry` in `ccc02s4hard`, Batch 36 `F4`) uses 3-arm `match (a, b):`, and "sum of all children except `v`" over `?Int` (`coci21c1p3`, Batch 33 `C11`) uses `fold_others` from `examples/local-lib/seq/fold_others.cp` over `Monoid<?Int>(0, +)` with zero subtraction.

Before (Batch 32 `rpeng3`, Batch 33 & 35 `coci21c1p3`, Batch 39 `yac6p6`, Batch 33 `btoi16p1`):
```cp
fn plus(x: ?Int, y: ?Int) -> ?Int:
    if x == none or y == none: return none
    x! + y!

if let nxt = dp[nr, c, k]: dp[r, c, k] min= nxt + a[r, c]
if let p1 = dp[x1, y1, k] and let p2 = dp2[dr, dc, kv - k]:
    qans[id] min= p1 + p2

var best: ?Int = none
if let v = from_low.fold(..=x): best max= v - x * down
if let v = from_high.fold(x..): best max= v + x * up
```

After (CP Lang v2.1 — one algebraic rule, zero helper functions):
```cp
dp[r, c, k] min= dp[nr, c, k] + a[r, c]
qans[id] min= dp[x1, y1, k] + dp2[dr, dc, kv - k]
let arrive = max(from_low.fold(..=x) - x * down, from_high.fold(x..) + x * up)
on_x[v] += dx                                       // none stays none
if let total = sum(depth[p] for p in P): best min= total
```

---

### U2. Lexically Scoped `@memo`, Runtime Flat-vs-Hash Dispatch, & Uniform Parameter Narrowing

*Unifies Batches 23 (§5), 28 (`D1`, `D2`, `D9`), 34 (`UD5`), 35 (`K1`), and 40 (`C3`).*

Three `@memo` collisions surfaced simultaneously across hard DP batches:

1. **Outer-parameter bloat (`coci21c1p3`, Batch 35 `K1`; `ioi20p4`, Batch 28 `D2`):** Restricting `@memo` to file top level forced `coci21c1p3` to pass outer loop guesses `(a_on, b_on)` as memo parameters, bloating the key from `4n` to `16n`, tripping hash fallback, and blowing runtime from 0.03s / 30 MB to 0.75s / 185 MB. Similarly, `ioi20p4` could not place `@memo` inside a `for _ in 0..q:` scenario loop.
2. **Static-only flat-vs-hash choice (`noip21p3`, Batch 28 `D1`):** Sizing `@memo` from worst-case static type bounds (`10^4 * 6e6 > 2^26`) forced hash mode even when runtime `n <= 400` fit comfortably in a flat array (1.4s / 257 MB hash vs. 0.03s flat).
3. **Parameter range narrowing ambiguity (`cheerio1s5`, Batch 40 `C3`):** Passing `now: Int` to `@memo fn best(i: 0..=n, used: 0..=k)` after `if now <= k:` left contestants unsure whether `@memo` narrows ranges like normal `fn`s, traps on out-of-range, or silently returns a sentinel.

**Locked v2.1 rule:**

- **Lexical scope for `@memo`:** `@memo` is valid on **any** `fn` — top-level or nested inside a function or loop block. It may capture immutable outer `let` bindings and enclosing loop variables (capturing an outer `var` that is mutated during the memo's lifetime is a compile error, preventing stale-cache bugs). Its cache lives for the enclosing block: the compiler hoists one reusable buffer outside loops and clears/stamps it on each entry to the block with zero per-iteration heap churn.
- **Runtime flat-vs-hash dispatch & compact `Bool` packing:** Default `@memo` over all-`Index` parameters computes `prod(K_i)` **at runtime** when the block is entered. When `prod(K_i) <= 2^24` (or `2^26` for 1-byte `?Bool` return types, packed as tri-state `uint8_t`, Batch 23 §5), it uses a flat dense array; otherwise it automatically dispatches to the per-process salted open-addressed flat `CpHashMap` (Batch 34 `UD5`, already implemented in `c8e381f`). `@memo(flat)` and `@memo(hash)` remain explicit overrides, and `cpl check --verbose` / LSP hover reports which mode each `@memo` uses (`flat`, `auto <= 2^24`, or `hash`).
- **Uniform parameter range checks & `..` precedence:** Range-typed parameters on `@memo` follow the **exact same rule as ordinary functions** — values are checked against `0..=k` on entry (trapping in local runs, elided when flow-narrowed by `if now <= k:` or range-proved, never silently returning a default). Range operators `..` and `..=` bind **looser than arithmetic (`+ - * / %`) and tighter than comparisons and `in`**, so `i + 1..=n` is always `(i + 1)..=n`.

Before (`coci21c1p3` top-level 5-arg memo -> 185 MB hash TLE risk):
```cp
@memo
fn blues(u: V, on: Bool, par_on: Bool, a_on: Bool, b_on: Bool) -> ?Int: ...
for a_on in Bool:
    for b_on in Bool: best min= blues(a, a_on, false, a_on, b_on)
```

After (CP Lang v2.1 — nested block-scoped `@memo`, 4n flat table reused per iteration):
```cp
for a_on in Bool:
    for b_on in Bool:
        @memo
        fn blues(u: V, on: Bool, par_on: Bool) -> ?Int: ...   // captures a_on, b_on; 4n flat table
        best min= blues(a, a_on, false)
```

---

### U3. Complete Algebraic `Index` Surface (`Bool`, `Char` Ranges, Row-Major Products, `Subset` Peeling) & Lossless Scalar Casts

*Unifies Batches 22 (`X5`), 28 (`D4`, `D8`), 30 (§1.2), 33 (`C8`, `C13`), 35 (`K9`), 37 (`1.4`), 38 (`38.5`), 39 (`F39.5`), and 41 (`1.7`, `1.8`).*

Several ports dropped out of the type system into manual bit-packing (`domain St = 0..4`), ASCII math (`c - 'a'`), or division/modulo decoding (`idx / m`, `idx % m`) because details of v2's 5 algebraic `Index` shapes were left implicit on the cheat sheet.

**Locked v2.1 clarifications (zero new concepts):**

- **`Bool` is a full `Index`:** `Bool.K == 2` (`false < true`), `for b in Bool:` yields `false` then `true`, and `[V, Bool, Bool]?Int` or `type St = (covered: Bool, picked: Bool)` works directly (`coci21c1p3`, Batch 33 `C8`, Batch 35 `K9`).
- **Bounded `Char` ranges are structural `Index`es:** `lo..=hi` includes `Char` bounds (`var cnt: ['a'..='z']Int = 0; for c in s: cnt[c] += 1`, Batch 41 `1.7`), obeying the exact same structural equality and debug bounds check as `[0..n]Int`.
- **Row-major product indexing:** On `(R, C)` (and any `k`-tuple of `Index`es), `.to_idx` and `.from_idx(i) -> ?(R, C)` are guaranteed **row-major** (`r.to_idx * C.K + c.to_idx`), eliminating manual `/ m` and `% m` decoding in grid shuffles (`noi14p5`, Batch 37 `1.4`).
- **`Subset<D>` iteration & O(1) bit peeling:** Because `Subset<D>` (`D.K <= 25`) is an `Index`, `for s in Subset<D>:` iterates all `2^(D.K)` subsets in ascending integer order (`nccc10s1`, Batch 41 `1.8`), `Subset<D>.empty` and `Subset<D>.full` name the empty and full masks (`dwite10c3p3`, Batch 28 `D4`), and **`s.first -> ?D`** returns the lowest set element via `ctz` (matching `Set.first`!), making O(2^n) bitmask DP peeling a one-liner (`btoi08p6`, Batch 38 `38.5`):
  ```cp
  var sa: [Subset<C>]Int = 0
  for s in Subset<C>:
      if let c = s.first: sa[s] = sa[s.without(c)] + a[c]
  ```
- **Lossless scalar casts:** `Int(ch)` returns the ASCII byte (`0..=127`), `Char(i)` constructs an ASCII `Char` (trapping outside `0..=127`), and `Int(b)` maps `Bool` to `0`/`1` (`false`/`true`), eliminating repeated `(if flag: 1 else: 0)` noise (`noip05p2`, `ioi99p2`, Batch 33 `C8`) while keeping `Int(f)` on `F64` a compile error (requiring `floor`/`ceil`/`round`/`trunc`).

---

### U4. Lexical Borrowing Without Fighting Arenas or Small-to-Large: `for mut`, `Copy` Release, RHS-Before-LHS, `.swap`, `.take()`, & `.absorb()`

*Unifies Batches 24 (§2.1), 25 (`GB8`), 26 (`TR11`), 31 (`31.1`), 34 (`UD8`), 36 (`F1`, `F7`), 38 (`38.8`, `38.11`, `38.12`), and 41 (`1.1`).*

Six batches hit predictable borrow-checker friction on three classic Gold/Platinum patterns: (a) iterating arrays of self-adjusting structs (`Block`, `Dsu`, `MonoDeque`), (b) recursive arena updates `nodes[u].l = merge(nodes[u].l, b)` and by-value `Copy` calls `spawn(teams[i])`, and (c) small-to-large merging of `List`/`Set`/`Heap` buckets out of an array (`mem[u]` into `mem[v]`). All three collapse cleanly with zero borrow-checker hacks and zero new keywords:

1. **`for mut x in xs:` (Batch 41 `1.1`):** Reuses the existing `mut` keyword to lend each element mutably during iteration (`for mut b in blocks: best max= b.query()`), while keeping generator expressions pure.
2. **Immediate `Copy` read release & RHS-before-LHS-place evaluation (Batches 24 §2.1, 25 `GB8`, 38 `38.12`):**
   - Reading a `Copy` element (`hrc[u]`, `teams[i]`, `nodes[u].l`) produces a by-value temporary that releases its borrow of the container **immediately**, before any enclosing call starts (`merge_heap(hrc[u], v)` and `spawn(teams[i])` compile cleanly with no manual temporary).
   - Every indexed assignment `container[idx].field = rhs` (or `container[idx] = rhs`) evaluates all index expressions and `rhs` **completely before** resolving the mutable target slot in `container`. Recursive arena code `nodes[u].l = merge(nodes[u].l, b)` both typechecks cleanly and is 100% immune to C++ `std::vector` reallocation UB!
3. **`.swap(i, j)` & guaranteed O(1) container `swap` (Batches 34 `UD8`, 38 `38.11`):** Every `[D]T` array and `List<T>` exposes `a.swap(i, j)` (safely a no-op when `i == j`, never needing a `law disjoint` proof), and free-fn `swap(mut a, mut b)` is guaranteed O(1) pointer swap on every heap-backed container (`List`, `Deque`, `Set`, `SortedBag`, `HashSet`, `Map`, `HashMap`, `MinHeap`, `MaxHeap`, `Str`, `[D]T`), preventing the quadratic complement-DFS trap in `egoi23p7`.
4. **`.take()` & `.absorb(mut other)` for small-to-large merging (Batches 26 `TR11`, 36 `F1`, 38 `38.8`):** Every heap container (`List`, `Set`, `HashSet`, `Map`, `HashMap`, `MinHeap`, `MaxHeap`, `Deque`, `Str`) provides `.take()` (O(1) move-out leaving an empty container behind, compiling to `std::exchange(x, {})`), and sets/maps/heaps/lists provide `a.absorb(mut b)` (O(1) swaps if `a.len < b.len`, then drains `b` into `a`, making inverted O(n²) small-to-large bugs impossible!).
5. **Independent array fills & top-of-stack mutation (Batches 23 §7.1, 29 `O6`, 33 `C9`, 36 `F7`, 41 `1.6`):** `var a: [D]T = expr` accepts any expression of type `T` and constructs `D.K` independent values (`var adj: [V]List<V> = []` never aliases lists; `[Small]Int = [2, 3, 5]` is a type error, never silently filling with lists). `xs.last` returns `?T` by value; mutating the top element uses `xs[xs.len - 1].cnt -= take` cleanly with no hidden mutable references.

Before (`cco22p5` borrow error on `for x in mem[u]`, `btoi04p3` manual small-to-large, `oly20practice139` raw index loop):
```cp
for idx in 0..mem[u].len:
    let x = mem[u][idx]
    rep[x] = v; mem[v].push(x)
mem[u].clear()

if l.heap.len < r.heap.len: swap(mut l.heap, mut r.heap)
while let v = r.heap.pop(): l.heap.push(v)

for i in 0..blocks.len: best max= blocks[i].query()
```

After (CP Lang v2.1 — zero index plumbing, zero aliasing fights):
```cp
for x in mem[u].take():             // O(1) move out, leaves mem[u] empty; zero alias on mem[v]
    rep[x] = v; mem[v].push(x)

l.heap.absorb(mut r.heap)           // impossible to invert small-into-large

for mut b in blocks: best max= b.query()
```

---

### U5. Ranges, Chunks, Boundaries & Grids: `Range.l`/`.r`, `D.chunks(k)`, `D[..k]`, `r1 & r2`, `Compressed` Slicing, `SparseTable.fold -> ?T`, & `flipped_rows`/`flipped_cols`

*Unifies Batches 24 (§3.1), 26 (§6), 32 (`X4`, `X5`, `X6`), 34 (`UD1`), 37 (`1.3`), 38 (`38.1`, `38.2`, `38.3`), and 41 (`1.2`, `1.3`).*

Every range/chunk/grid friction point across seven batches reduces to small, completely consistent methods on existing types — keeping **one `Range<I>` type**:

- **`r.l` and `r.r` on `Range<D>` (Batches 38 `38.2`, 41 `1.3`):** Every `Range<D>` (including empty ranges, where `r.l == r.r`) directly exposes its left and right boundary cuts `r.l: D.bounds` and `r.r: D.bounds`. This eliminates `rw.first!.l` / `rw.last!.r` noise inside `for rw in R.windows(k)` (`apio09p1`) and gives Rollback Mo's boundary cursors directly!
- **`D.chunks(k)` & `Compressed<T>` slicing (Batch 41 `1.2`, `1.3`):** `D.chunks(k)` (sibling to `D.windows(k)`) yields non-overlapping `Range<D>` chunks of size `k` (last chunk clamped to `D.last`). `Compressed<T>` implements `Indexed<xs.D>`, so `ys[blk]` slices its sorted values directly with zero `.from_idx` round-trips (`oly20practice139`, `nccc9s4`).
- **Domain prefix `D[..k]` and range intersection `r1 & r2` (Batches 32 `X5`, 37 `1.3`):** `D[..k]` returns `Range<D>` of the first `min(k, D.K)` elements (empty when `k <= 0`, avoiding `for c in Colour if c.to_idx < k:` scanning all `n` in `ioi12p5`); `r1 & r2` returns the intersection `Range<I>` (matching `Set`/`Bitset` `&`, eliminating `max(a, c)..min(b, d)` in `aac5p3`).
- **`p.mirror` / `r.mirror` (Batch 32 `X6`):** Reflects a `Seq` position or range across its domain (`D.from_idx(D.K - 1 - p.to_idx)!`, swapping range ends), letting 1D reflection problems (`noimps23p1`) stay on nominal domains instead of dropping to `n + 1 - x`.
- **Unambiguous 2D grid flips (`g.flipped_rows` / `g.flipped_cols`, Batch 38 `38.1`):** Replace ambiguous 2D `g.flipped` with explicit **`g.flipped_rows`** and **`g.flipped_cols`** (and guarantee `g.transposed` yields `[C, R]T` so domain-generic `fn solve<R, C>(g: [R, C]Int)` covers all 4/8 orientations cleanly in `apio09p1`).
- **`SparseTable.fold(r) -> ?T` (Batch 38 `38.3`):** Because `SparseTable` only requires an idempotent binary op (`min`, `max`, `gcd`, `&`, `|`) and NOT a monoid identity, `st.fold(range)` returns **`?T`** (`none` on empty ranges), eliminating sentinel pairs `(INF, -1)` in `apio09p2` and matching `min(gen) -> ?T` uniformly. (Boundary ranges like `V.first.r..` handle "every element after the first" cleanly when `V.K == 1`, Batch 26 §6.)

Before (`apio09p1` `!` noise + ambiguous `flipped`; `oly20practice139` manual chunk clamping):
```cp
for rw in R.windows(k).rev:
    var best: ?Int = down[rw.first!.r]
    ...
    down[rw.first!.l] = best

for ql in (0..ys.D.K).step(B):
    let qr = min(ys.D.K, ql + B)
    var b = Block(ys: [ys[ys.D.from_idx(i)!] for i in ql..qr], ...)
```

After (CP Lang v2.1):
```cp
for rw in R.windows(k).rev:
    var best: ?Int = down[rw.r]
    ...
    down[rw.l] = best

for blk in ys.D.chunks(B):
    var b = Block(ys: ys[blk], v: List.filled(blk.len, 0))
```

---

### U6. Block-Scoped `defer` (Per Loop Iteration) & Top-Level `return` Early Exit

*Unifies Batches 24 (§3.2), 31 (`31.3`), 37 (`1.6`), 38 (`38.9`), and 41 (`1.4`).*

Two small scope clarifications close recurring boilerplate across Rollback Mo's, backtracking DFS, and impossible-case early exits:

1. **`defer` is strictly block-scoped (Batches 37 `1.6`, 38 `38.9`, 41 `1.4`):** `defer stmt` executes in LIFO order at the exit of its **immediately enclosing lexical block** — including the end of each `for`/`while` iteration, an `if` branch, or a `fn` body (matching C++ RAII / Zig `defer`, never Go function-scoped `defer`). This makes both Rollback Mo's per-query undo (`nccc9s4`, `dmpg15g6`) and backtracking DFS state cleanup (`noip04p4`, `btoi17p4`) leak-proof on every `continue`/`return` without adding `undo` blocks!
2. **Bare top-level `return` for early exit (Batch 31 `31.3`):** Under CP Lang's file-as-body model, bare `return` (with no value) is legal at file top level: it exits the program cleanly with status 0, or ends the current test case inside a whole-file `@multitest`. No `exit` or `done(x)` builtin needed.

```cp
// Top-level early exit (dmopc21c6p4):
if any(c in [e.u, e.v] and d in [e.u, e.v] for e in edges):
    print(-1)
    return

// Block-scoped defer inside Rollback Mo's & backtracking (nccc9s4, btoi17p4):
for (l, r, qi) in buckets[bi]:
    let snap = state.mark()
    defer state.undo_to(snap)       // runs at end of THIS query iteration, not fn exit!
    ...
```

---

### U7. Unified `Mod<P>`, `Int` Promotion, `T.zero`/`T.one`-Seeded `sum`/`prod`, & Zero-Keyword Cross-`@multitest` Precomputation

*Unifies Batches 23 (§1, §2, §3), 29 (`O4`), 34, 37 (§3), and 41 (`1.7`).*

Batch 23 exposed a three-way collision (`ModR` × `Monoid` × `SegTree`), noisy `Z(0)`/`Z(1)` casts, broken `sum([Z(1), Z(2)])`, and cross-`@multitest` precomputation (`noi18p2`). `decisions.md` §17.5 already retired `ModR` in `stdlib/mod.cp`; v2.1 completes the unification:

1. **One `Mod<P>` value type for compile-time `const` AND runtime-frozen `let` moduli (`ioi08p3`, `dmopc21c2p5`):** Freezing `let M: 1..=1e9 = read(); type Z = Mod<M>` reuses the exact same runtime-frozen `let` binder as `domain`, giving `SegTree<Gem, Z>(*, [Gem] _ => 1)` zero-capture C++17 lowering (4-byte `uint32_t` for `P <= 2^31 - 1`, `I128` widening multiply above `2^31 - 1` so manual `mul_mod` loops vanish). `+`, `-`, `*`, `.pow(e)` work for **any** `P >= 1` (including composite `P = 998_244_352` and `P = 1`, where `Z(1) == Z(0)` automatically!); `/` and `.inv()` require `gcd(x, P) == 1` (plus `.try_inv() -> ?Z` for composite moduli). Constant divisors (`x / 2`) lower to compile-time inverse multiplications in C++17 (fixing Batch 23 §1's `O(log P)` hot-loop trap).
2. **One-way `Int` reduction + `T.zero`/`T.one`-seeded `sum`/`prod`:** `+`, `-`, `*`, `/`, `+=`, `-=`, `*=`, `/=`, `==`, `!=` accept `Int` alongside `Mod<P>` (`b = d1 - 3 * a`, `ans *= x + 1`, `if cur == 0: continue`), and `[D]Z = 0` fills with `Z(0)`. `sum(it)` seeds from the element type's zero (`0`, `0.0`, `Z(0)`, `I128(0)`, `U64(0)`, `?Int(0)`), and **`prod(it)`** joins `sum(it)` in the sequence prelude (seeded from `1`, `1.0`, `Z(1)`, etc., Batch 41 `1.7`). Operator type errors always print **type names** (`Int` and `Mod<998244353>`), never formatted values (`0` and `1`).
3. **Cross-`@multitest` precomputation without `static` or `@once` (`noi18p2`, Batch 23 §1):**
   - `Comb<P>()` **auto-grows** its factorial/inverse-factorial table lazily on demand (`O(max_n)` amortized across all test cases, eliminating `MAX_N` guesses and `O(T * MAX_N)` TLEs).
   - `@multitest` works both at line 1 (shorthand wrapping the whole file) AND on a `fn` (`@multitest fn solve():`), so any top-level precomputation written above `@multitest fn solve():` (`sieve`, `Comb`, DP tables) runs **once** before the test loop with zero new keywords!

Before (`ioi08p3` `ModR` × `Monoid` × `SegTree` + `noi18p2` manual test loop + `noi19p5` `Z(...)` noise):
```cp
let mod = ModR(m_val)
let mul_m = Monoid<Int>(1 % m_val, (a, b) => mod.mul(a, b))
var st = SegTree<Gem, Int>.of(mul_m, _ => 1 % m_val)
st.set(Gem(g), (cnt[g] + 1) % m_val)
b = d1 - Z(3) * a
```

After (CP Lang v2.1):
```cp
let M: 1..=30_000 = read()
type Z = Mod<M>
var st = SegTree<Gem, Z>(*, [Gem] _ => 1)       // zero closures; M == 1 handled automatically
st.set(g, cnt[g] + 1)
b = d1 - 3 * a
let ways = prod(cnt[c] + 1 for c in 'a'..='z')  // seeded from Z(1) when terms are Z

// Cross-@multitest precomputation (zero new keywords):
let sv = Sieve(1_000_000)
var comb = Comb<998_244_353>()                  // auto-grows across test cases
@multitest
fn solve(): ...
```

---

### U8. Value-Owned `t.Pre` on `Rooted<V>`, `t.pos(v)` / `from_parents` / `t.tour` / `lift_while`, Vertexless Graph Inference, & `Dsu.groups()`

*Unifies Batches 24 (§5.1), 26 (`TR3`, `TR5`, `TR7`, `TR14`, §6), 27 (`HG2`, `HG5`, `HG9.3`), 34 (`UD4`), 39 (`F39.1`, `F39.4`), and 41 (`1.5`).*

Four batches caught a genuine domain-safety hole in `Rooted<V>`: because `Rooted<V>.Pre` was a type-level domain shared by all trees over `V`, two trees on the same vertices (`human` and `wolf` in `ioi18p3`, `ceoi22p3`) could silently index each other's Fenwick/segment trees!

**Locked v2.1 rule:**

- **Value-owned `t.Pre` (same mechanism as `xs.D` on `compress`):** Each `Rooted<V>` instance `t` owns a distinct nominal preorder domain **`t.Pre`**, so `human.Pre` and `wolf.Pre` cannot mix. Expose **`t.pos(v) -> t.Pre`** (exact inverse of `t.preorder: [t.Pre]V`, closing Batch 39 `F39.1`), **`Rooted.from_parents(par: [V]?V)`** (Batch 26 `TR3`), **`t.tour`** yielding `(v: V, entering: Bool)` iteratively with zero recursion stack risk (Batch 26 `TR7`), and pin **`t.lift_while(u, pred) -> ?V`** to: test `pred` on `u` and its ancestors, returning the highest ancestor `w` such that `pred` holds for every node from `u` through `w`, or `none` if `pred(u)` is false (matching `stdlib/tree.cp`).
- **Vertexless graph algorithms (`topo`, `scc`, `components`, `two_color`, `articulation_points`, `biconnected_components`, `two_edge_components`):** Take either `g: Graph<V, E>` (inferring `V`) OR a neighbor closure with explicit turbofish **`topo::<Side>(u => ...)`** (Batches 27 `HG2`, 41 `1.5`), avoiding throwaway edge lists on adjacency matrices and implicit graphs.
- **`Dsu<V>.groups() -> List<List<V>>` (Batch 39 `F39.4`):** 6-line `std.dsu` method returning all connected components directly (matching ACL `dsu::groups()`).

---

### U9. Stamp Tables (`Marks<D>`), Generalized `RollingHash` (`HashVal`), `Map` 4-Lookup `?(K, V)`, `min_by`/`max_by` Ties, & Container Polish

*Unifies Batches 25 (`GB3`, `GB6`), 29 (`O11`), 30 (§3.2), 31 (`F1`), 32 (`X2`, `X8`), 33 (`C6`), 36 (`F5`), 38 (`38.7`), 39 (`F39.3`, `F39.6`, `F39.7`), 40 (`C4`), and 41 (`1.6`, `1.8`).*

- **O(1)-clear stamp table `Marks<D>()` (Batch 31 `F1`):** Keep `a.fill(x)` for real whole-array resets, and add a tiny userland `Marks<D>()` in `std` (generation-stamped visited set) reusing the exact `Set` verbs: `.add(v) -> Bool` (true iff newly marked this round), `v in m`, and **O(1) `.clear()`**. Prevents `O(T * N)` TLE cliffs like `ucrpc21g`'s `10^9` `seen.fill(false)` writes without teaching manual stamp counters.
- **`RollingHash` generalized over `[D]T` + `HashVal` (Batch 39 `F39.3`):** `RollingHash(seq)` accepts `Str`, `List<T>`, or any `[D]T` (`Char`, `Int`, `U64`, `HashVal`), returning a `Copy + Eq + Ord + Hash` 64-bit double-mod **`HashVal`** from `.get(l..r)`, with an independent salted base per hash constructor (or `RollingHash.with_base`) so 2D row/column hashing (`sac22cc5p5`) never collides on transposes.
- **Ordered `Map` parity, iterable constructors, `by:` lookups, & unused-result rule (Batches 30 §3.2, 32 `X2`/`X8`, 33 `C6`, 38 `38.7`, 39 `F39.7`):**
  - `Map<K, V>` has all four ordered lookups matching `Set`/`SortedBag`: `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`, each returning **`?(K, V)`** (key and value together; `Set`/`SortedBag` return `?K`).
  - `Set`, `SortedBag`, `HashSet`, `List`, `Deque`, `MinHeap`, `MaxHeap` all accept an optional iterable in their constructor (`Set<Int>(1..=n)`, `SortedBag<Int>(vals)`).
  - `xs.first_ge(x, by:)` / `xs.last_le(x, by:)` accept an optional `by:` key projection on sorted `List`, `[D]T`, and slices `a[l..r]` (and slices support `.sorted()`, `.rev`, `.first_ge`, `.last_le`, Batch 36 `F5`; `sorted(it, by:)` accepts any iterable including ranges, Batch 41 `1.8`).
  - Mutating `.pop()`, `.pop_front()`, `.pop_first()`, `.pop_last()`, `.add()`, `.remove()`, `.unite()` never emit unused-result warnings when called as statements.
  - Add `rotate_right(mut a, k)` beside `rotate_left` (1-line alias via Euclidean `(-k) % a.len`).
- **Deterministic tie-breaking & canonical names (Batches 25 `GB3`/`GB6`, 29 `O11`, 39 `F39.6`, 40 `C4`):**
  - `min_by(it, key) -> ?T` and `max_by(it, key) -> ?T` always return `?T` (`none` on empty), with **first extremum winning ties**.
  - `sort` / `sorted` is guaranteed **stable**.
  - Lock **`bit_len(x)`** (retiring `bit_width` drift in `proto/cpi`), confirm hex (`0xFF`) and binary (`0b1010`) literals, and pin number-theory signatures on the cheat sheet: `Sieve(n)` (`.is_prime(x)`, `.spf(x)`, `.primes`, `.factor(x)`), standalone `factor(x) -> List<(Int, Int)>` (retiring the 1-letter-apart `factorize` alias), and `divisors(x) -> List<Int>`.

---

### U10. `@grader` Constants & Shared Domains, Sibling-Sized Array Params, `Int`-Indexed `Str`, & Judge `print` Rules

*Unifies Batches 23 (§5), 25 (`GB9`), 26 (§6), 27 (`HG4`), 30 (§1.2), 33 (`C3`), 36 (`F5`), 37 (`1.1`, `1.2`), 40 (`C1`, `C2`), and 41 (`1.6`).*

- **Complete `@grader` ABI without new keywords (Batches 37 `1.1`–`1.2`, 40 `C1`):**
  1. **Extern grader constants:** `@grader const N: 0..=2000` (bodyless = extern constant defined by the judge header, `tle18p5`).
  2. **File-scope shared grader domains:** A `@grader` file may declare size-less `domain V` / `domain Colour` at file scope, bound by the first `@grader fn` parameter that fixes them, so extern callbacks (`@grader fn build(b: [V, V]Bool)`) and entry functions (`@grader fn construct(p: [V, V]0..=3) -> Bool:`) share one nominal domain (`ioi20p2`, `ioi12p5`).
  3. **Sibling-parameter array bounds:** Array parameter bounds in any `fn` signature may reference any `Int` parameter in the same parameter list left or right (`fn Assist(needed: [0..r]Bool, n: Int, k: Int, r: Int):`, `ioi15p1`, `ioi12p5`).
- **`Int`-indexed `Str` & everyday spellings (Batches 30 §1.2, 36 `F5`, 40 `C2`, 41 `1.6`):** State unmistakably on the cheat sheet that default `Str` is `Int`-indexed (`0..s.len`, zero `.raw` needed), supporting `s[i]`, `s[l..r]`, `s[..k]`, `s[k..]`, `for c in s`, `for (i, c) in s.items`, `s + ch` / `ch + s` (`Str + Char`), and `match s:` on string literals, while `[expr for x in it if cond]` builds `List<T>` and `List.filled(len, val)` fills dynamic lists.
- **Judge-native `print` rules (Batches 23 §5, 26 §6, 27 `HG4`, 33 `C3`, matching `plan/16-io.md`):** `print(v)` on a domain value prints its **declared label** (`1..=n` prints `1`..`n`, removing `.to_idx + 1` everywhere), `print(xs)` prints a `List` / 1D array **space-separated** (no brackets/commas), and `print(g)` prints `[R, C]Char` row-by-row — while `debug(...)` prints structured `[1, 2, 3]` developer syntax to stderr.

---

### U11. Compiler-Trust & Stress Guardrails: `namespace cp_user`, `LazySegTree.set` Pushdown, `std.gen`, & `--max-vals`

*Unifies Batches 21–30, 34 (`UD3`), and 36 (`F2`, `F6`).*

1. **`namespace cp_user` C++17 isolation (Batches 25 `GB9`, 36 `F2`):** Removing `using namespace std;` is necessary but **not sufficient** — POSIX headers pulled in by `<bits/stdc++.h>` (`<ctime>`, `<unistd.h>`, `<cmath>`) pollute the C++ global namespace with `tm`, `dup`, `arg`, `y0`, `y1`, `j0`, `j1`, `link`, `pipe`, `time`. Emit all user globals, structs, and functions inside `namespace cp_user { ... }` (called from `int main()` via `cp_user::cp_main()`), and prefix C++ keywords/POSIX macros (`errno`, `major`, `minor`). Zero host-header name collisions ever again.
2. **`LazySegTree.set(pos, val)` pushdown invariant (Batch 36 `F6`, `ccc02s4hard`):** Point updates interleaved with range updates MUST push all pending ancestor tags root-to-leaf before overwriting leaf `pos` (pin `ccc02s4hard` in the conformance suite).
3. **Streaming stdin for `@interactor` (Batches 26 §7, 34 `UD3`):** Both `proto/cpi` and `cpl run` must read stdin incrementally on demand (never slurping `stdin.read().split()` upfront) so interactive solvers execute identically under pipes and `@interactor`.
4. **Constructive `std.gen` library & `cpl stress --max-vals` (Batches 21–30):** Every hard batch confirmed that structural input preconditions break rejection sampling (`perm`, `surjection`, `bounded_product`, `tree`, `connected`, `dag`, `functional`, `simple_graph`, `str_of`, `rand_points`, `rand_disjoint_rects`, `rand_convex`). Provide these as composable functions in userland `std.gen` used inside `read(gen:)` / `@gen`. Add `cpl stress --max-vals` (small `n`, maximum declared scalar values) to catch 64-bit overflow cliffs (`sdoi17r1p6`, `ioi09p2`, Batch 29 `O2`) that small-value fuzzing misses!

---

## 3. Concept-Budget Guard: Userland Placements & Explicit Rejections

### Kept in Userland `examples/local-lib/` (Zero Core or Core-`std` Bloat)

Already landed in `3d67b55` and `1613d30` or queued strictly in `examples/local-lib/`:
- Already shipped: `Poly`/NTT (extend with 3-prime/split-base MTT fallback for arbitrary primes like `10^9 + 7` + `.interpolate(pts)`, Batch 23 §4 `egoi22p2`, `noi19p5`), `LiChao<T>` & `LiChaoArena` (extend to `F64` + record values, Batches 21 `E8`, 28 `D5`), `Functional<V>` (rings, peel order, and O(log n) `jump`/`dist` — already covering Batches 27 `HG9.4`, 38 `38.6`, 39 `F39.4`!), `Fixed<D>`, `Tri<D>`, `Congruence`, `Cover<Y>` (Batches 21 `E7`, 24 §1.4), `fold_others` (Batch 33 `C11`), `sos_sub`/`sos_sup`, `XorBasis`, `Line`/`Hom`, `Frac`, `Trie`, `SuffixAutomaton`, `Hash2D`, `Rope`, `RangeKth`, `mo`.
- New `examples/local-lib/` additions from Batches 26–39 (kept OUT of core `std` and off the one-page cheat sheet): `Centroids<V>` & `reroot` DP (Batch 26 `TR6`), `kruskal_tree` / `merge_tree` (Batches 26 `TR12`, 27 `HG5`), `Matrix.det`/`rank`/`solve` linear algebra (Batch 29 `O7`–`O8`), `Interner<K, D>` (Batch 27 `HG4`), `spanning(V, edges)` (Batch 35 `K3`), `ranks(xs)` (Batch 32 `X4`), `lis` (Batch 32 `X3`), `count_in_rects` (Batch 27 `HG5`), `MinCount` semiring / `floyd_warshall` (Batch 23 §5), and `RollbackArray` (Batch 24 §3.2).

### Explicitly Rejected Proposals (Do Not Re-Propose)

1. **`Pool<Node>` / `Node.Id` / `SegPool` (Batches 24 §2, 26 `TR10`):** Rejected; `type Id = 0..=MAX` (4-byte handle + 4-byte niche `?Id`) + `List<Node>` + `.push_id(x)` + guaranteed RHS-before-LHS-place evaluation (U4) + a 4-line free stack replaces it with zero knot-tying syntax.
2. **`I32` / `U32` numeric types (Batch 24 §2.2, §5.6):** Rejected; stays at 4 numeric types because range types (`-1000..=1000`, `0..=2_000_000`) pack to 1/2/4 bytes automatically.
3. **`undo` blocks / `Journaled<T>` (Batch 24 §3.2):** Rejected; stays at 1 cleanup primitive — block-scoped `defer` (U6) covers nested Rollback Mo's and backtracking.
4. **`static let` / `@once` decorator (Batch 23 §1):** Rejected; top-level code above `@multitest fn solve():` runs once and `Comb<P>()` auto-grows (U7).
5. **`ModR` (Batch 23 §2):** Permanently retired; `let M = ...; type Z = Mod<M>` unifies static and runtime moduli (U7).
6. **`domain Node = Pos | Round` sum-domain syntax (Batch 33 `C13`):** Rejected; payload `enum Node: Pos(p: Pos) | Round(r: Round)` already derives `Index` (one spelling per construct).
7. **`min=` / `max=` returning `Bool` (Batch 27 `HG9.5`):** Rejected; assignments stay statements so `if x = 0:` stays a compile error; use tuple updates `best min= (cost, parent)` or explicit `if` when counting ties.
8. **Dependent `@memo` parameter bounds (`s: 0..=(i + 1) * h[i]`, Batch 28 `D1`):** Rejected; runtime flat-vs-hash dispatch (U2) solves `noip21p3` across all `n` without jagged-stride dependent types.
9. **Rolling-row `step ... into ...:` syntax (Batch 28 `D3`):** Rejected; structural `[0..n]T` + O(1) `swap(mut cur, mut nxt)` + `nxt.fill(0)` is 2 lines with zero new syntax.
10. **Variable-length `@tags` command parser DSL (Batch 24 §5.4):** Rejected; fixed-arity `@tags` covers ~90% of query formats; plain `if op == "INSERT":` is clearer for dynamic-length commands.
11. **Single-problem helpers in core `std` (`Pos.window()`, `g.lattice(2, 1)`, `d.ccw_from.take(6)`, `at.cyc_range(k)`, `to_vis.take_except`, `dag_dp`, `common_prefix`):** Rejected from core `std`; plain loops, `Range.l`/`.r`, `Dir4`/`Dir8`, `swap`, and `topo` already express them in 3–5 lines.
12. **Bitwise `|` / `^` on `Bool` (Batches 25 `GB4`, 31 `31.2`):** Rejected; `!=` is boolean XOR (emit a helpful diagnostic) and explicit bindings avoid silent C-style short-circuit confusion.
13. **Extra syntax sugar violating concept budget (`loop:` keyword, Batch 33 `C15`; `let` clause inside generators, Batch 37 `1.7`; keyed array literals `[R: 1, G: 2]`, Batch 39 `F39.5`; `Rev<T>` duplicating `Desc<T>`, Batch 38 `38.6`; `Heap<T>(by:)` adding a 3rd heap spelling, Batch 27 `HG3`; user operators on structural `type` aliases, Batch 35 `K4`):** All rejected (`while true:`, 3-line loops, `[Color] c => match c:`, `Desc<T>`/`desc(x)`, `MinHeap`/`MaxHeap`, and `impl` on nominal `struct`s already cover every case cleanly).
14. **Runtime-sized `SegTree.from(list)` & in-place `combine_into` in core `std` (Batches 24 §1.3, 30 §1.3):** Rejected from core `std`; flat-packing layers into one `domain Flat = 0..n` is faster, and heavy-node trees stay hand-written or in `examples/local-lib/`.
15. **SMT-inverting arbitrary `where` predicates or feeding `where` into the overflow prover (Batch 28 `D7`):** Rejected; `assume` stays a local/stress check paired with constructive `std.gen` helpers (U11).
16. **Tag-free `CoverTree` forced into `Action<T, F>` (Batch 24 §1.4):** Rejected; `Cover<Y>` already lives cleanly in `examples/local-lib/ds/cover.cp`.

---

## 4. Prioritized v2.1 Prototype & Stdlib Sprint Checklist

**Already completed since Wave 2 (do NOT redo):**
- Max-N C++17 cliffs in `proto/cpc/` (`c8e381f`): salted `CpHash`/`CpArrHash`, open-addressed `CpHashMap`, dense `@memo` for `Bool`/unit-enum/bounded ranges, lazy `RootedTree::sp` + flat CSR `kids` + stack `ChainList`, 4-byte `uint32_t` `Mod<M>`, lazy-sorted `CpSteps`, `uint8_t` `Bool` arrays, fused `CpPrefix`, positive-divisor Euclidean `/` `%` fast path.
- Core v2 alignment of `stdlib/` (`72383ec`, `decisions.md` §17) and 25 niche structures in `examples/local-lib/` (`3d67b55`, `1613d30`, including `Functional<V>`, `Fixed<D>`, `Tri<D>`, `Congruence`, `LiChaoArena`).

Everything below is the **remaining delta** ordered strictly by bug severity and measured `today/`-vs-v2 LOC leverage across Batches 21–41. Keep `crates/` untouched until P0–P1 pass on the prototype.

### Priority 0 — Correctness, Silent-UB Fixes, & Cheat-Sheet Parity Gate (`proto/cpc/`, `proto/cpi/`, `proto/cp.py`)

- [ ] **P0.1 Eliminate silent UB and C++ host-header collisions (`proto/cpc/compiler.py`, `runtime.hpp`):**
  - [ ] Fix 32-bit `1 << k` shift codegen (`1LL << k` / `cp_shl` even when range-proved, fixing `GB1` / `D6` / `W5`).
  - [ ] Remove `using namespace std;` AND wrap emitted user code in `namespace cp_user { ... }` with C++/POSIX macro mangling (fixing `tm`, `dup`, `arg`, `y0`, `y1` collisions — `F2`, `GB9`).
  - [ ] Fix `<=` / `>=` codegen on structs/tuples/records (Batch 28 `D5` `g++` failure), and emit `operator<` ONLY when every field is `Ord` (never `?T`, never `Mod<P>`, `16.9`).
  - [ ] Fix `proto/cpi` value-semantics aliasing on `var t = s` for `Str`, `List`, and `[D]T` (`X4`, `O5`), and fix `[D]T = expr` (any `T` expression, independent clones per slot, reject `[Small]Int = [2, 3, 5]`, `O6`, `C9`, Batch 23 §7.1).
  - [ ] Fix `proto/cpi` `Int` read/parse overflow check (`18446744073709551615` silently accepted as `Int`, `X9`) and `List.pop() -> ?T` (currently traps on empty, `HG8`, `O11`).
- [ ] **P0.2 First-page cheat-sheet parity & CI gate (`proto/cpc/`, `crates/cpl-cli/tests/cases/`):**
  - [ ] Fix tuple/record `read()`, string interpolation `"{a} {b}"` and `"{x:.6f}"` (in both `proto/cpc` and `proto/cpi`, fixing `E1`, `K8`, Batch 23 §5), structural `[0..n]T` / `[0..=n]T` unification across functions (`D3`, Batch 30 §1.1), `a.fill(x)`, `print(xs)` space-separated + `print(v)` domain labels, and `for ... if` / `while let ... and` / chained `if let` expressions (`C2`, `38.4`, Batch 37 §2, Batch 41 §2).
  - [ ] Fix lexer crash on labeled loops `'outer: for ...` (`proto/cpc/lexer.py`, Batch 39 §3), `Set.add` / `MinHeap.len` / `MinHeap.is_empty` / `Map.items` naming drift, and `Map<(Int, Int), V>()` generic tuple parsing (`UD12`).
  - [ ] Extract every fenced snippet in `docs/one-page-cheat-sheet.md` into `crates/cpl-cli/tests/cases/` and gate CI on compiling them.
- [ ] **P0.3 Lock v2.1 core semantics (`proto/cpc/`, `proto/cpi/`):**
  - [ ] Implement **Tropical Optional Rule (U1)**: `none`-absorbing `+`/`-`/`+=`/`-=`/unary `-`/`sum` on `?Int`/`?F64`, compile error on unparenthesized `??` mixed with arithmetic, `min`/`max`/`min=`/`max=` skipping `none` on `?T` and `T` targets (incl. `min(gen)`/`max(gen)` over `?T`), and `?T == T` / `?T != T` lifting.
  - [ ] Implement **borrow & evaluation rules (U4, U6)**: `for mut x in xs:`, immediate `Copy` element release before calls (`merge_heap(hrc[u], v)`), RHS-before-LHS-place evaluation (`nodes[u].l = merge(nodes[u].l, b)`), parallel tuple assignment `(a, b) = (x, y)`, block-scoped `defer` (per loop iteration), and bare top-level `return`.
  - [ ] Implement **lexically scoped `@memo` + runtime flat-vs-hash dispatch (U2)**: nested `@memo` inside loops/fns (hoisted reusable buffer cleared per entry, rejecting mutated captured `var`), runtime `prod(K_i) <= 2^24` flat-vs-hash dispatch (`2^26` for tri-state `uint8_t` `?Bool`), and uniform parameter range narrowing + `..` precedence.
  - [ ] Implement **complete 5-shape algebraic `Index` (U3)** across both `proto/cpc` and `proto/cpi`: `Bool`, unit/payload `enum`, `?T`, structural `Int`/`Char` ranges (`['a'..='z']Int`), row-major tuples/records `(Row, Col)`, and `Subset<D>` (`for s in Subset<D>`, `.empty`, `.full`, `.first`), plus `Int(ch)`, `Char(i)`, `Int(b)`.

### Priority 1 — High-Leverage Prototype Gaps Driving 80%+ of the `today/` LOC Delta (`proto/cpc/`, `proto/cpi/`)

- [ ] **P1.1 Flat `0..len` `Int`-indexed ASCII `Str` (`proto/cpc/`):** `s[i]`, `s[l..r]`, `s[..k]`, `s[k..]`, `for c in s`, `s.items`, `s.len`, `s.is_empty`, `s.push(ch)`, `s + t`, `s += t`, `s + ch`, `s * k`, `sep.join`, `s.starts_with(x, at:)`, `s.split`, `s.split_once`, `s.trim_start`, `s.parse_int()`, `Str(chars)`, `match s:` on string literals, and `read_line()` (skip trailing ws, empty line `""`, strip `\r`).
- [ ] **P1.2 Missing core types & containers in `proto/cpc/`:**
  - [ ] `F64` (quiet-NaN niche, local NaN/Inf traps, `floor`/`ceil`/`round`/`trunc`, math intrinsics, `"{x:.6f}"`, `close`/`sign`), `U64` (wrapping, logical `>>`), `I128` (with `print`), hex/binary literals (`0xFF`, `0b1010`), `~`, `<<=`/`>>=`, `popcount`/`ctz`/`clz`/`bit_len`.
  - [ ] Unified `Mod<P>` (static + runtime-frozen `let`, one-way `Int` reduction, constant-divisor inverse lowering, `T.zero`/`T.one`-seeded `sum`/`prod`) and auto-growing `Comb<P>()`.
  - [ ] `Deque<T>`, `MonoDeque<T>`, `MaxHeap<T>` (with `desc(x)`), ordered `Set<T>` / `SortedBag<T>` / `Map<K, V>` (`.first_ge`/`.first_gt`/`.last_le`/`.last_lt`, `.rank`, `.nth`, iterable constructors), `Bitset<D>` (signed `.shifted(k)`), `SparseTable<D, T>` (`.fold -> ?T`), `LazySegTree<D, T>` (6 stock actions + `.max_right`/`.min_left` + root-to-leaf pushdown on `.set`), `Fenwick<D, Int>`, `RollingHash` (`[D]T` -> `HashVal`), and `Rooted<V>` with value-owned `t.Pre`, `t.pos(v)`, `fold_path`, `apply_path`, `lift_while`, `postorder`, `tour`, `from_parents`.
  - [ ] `struct` field defaults + mutable `struct` methods (`cco20p5`, `oly20practice139`), `Compressed<T>` (`.D`, `xs[v]`, `xs[range]`, `.at`, `.first_ge`), `p.adj4`/`p.adj8`/`Dir4`/`Dir8`/`p.step`/`g.transposed`/`g.flipped_rows`/`g.flipped_cols`, `bsearch_first`/`bsearch_last`/`bsearch_first_some`, `first_ge`/`last_le` (with `by:`) & `.sorted()` on `List` and slices, `min_by`/`max_by` (returning `?T`, first-wins ties), `dedup`, `rotate_left`/`rotate_right`, `runs`, `.step(k)`, `D.chunks(k)`, `Range.l`/`.r`, `D[..k]`, `r1 & r2`, `p.mirror`, `xs.swap(i, j)`, O(1) `swap`, `.take()`, `.absorb(mut other)`, `List.filled`, and `any`/`all`/`count`/`first`.
- [ ] **P1.3 `@grader` & `@multitest` completions (`proto/cpc/`, `proto/cpi/`):** `@grader const`, file-scope shared grader domains, sibling-parameter array bounds, `@multitest fn solve():` with top-level once-only precomputation, and streaming stdin for `@interactor`.

### Priority 2 — Stdlib v2.1 Refinements, `examples/local-lib/`, & Honest `cpl stress` (`stdlib/`, `examples/local-lib/`, `proto/cp.py`)

- [ ] **P2.1 Small `stdlib/` v2.1 refinements (do in dedicated `stdlib/` session):**
  - [ ] `stdlib/tree.cp`: make `Pre` value-owned (`t.Pre`), add `t.pos(v) -> t.Pre`, `Rooted.from_parents(par)`, and iterative `t.tour` yielding `(v, entering)`.
  - [ ] `stdlib/dsu.cp`: add `dsu.groups() -> List<List<V>>`.
  - [ ] `stdlib/grid.cp`: rename 2D `flipped` -> `flipped_rows` / `flipped_cols`; verify `g.transposed -> [C, R]T`.
  - [ ] `stdlib/sparse.cp`: return `?T` from `SparseTable.fold(range)` (`none` on empty ranges; no monoid identity required).
  - [ ] `stdlib/mod.cp`: make `Comb<P>()` lazily auto-grow on demand; add `.try_inv() -> ?Mod<P>`; add one-way `Int` ops + `T.zero`/`T.one` support.
  - [ ] `stdlib/seq.cp` & `stdlib/math.cp`: add `D.chunks(k)`, `Compressed` range slicing, `Range.l`/`.r`, `r1 & r2`, `D[..k]`, `p.mirror`, `prod(it)`, `rotate_right`, `by:` on `first_ge`/`last_le`, `Marks<D>` (O(1)-clear stamp set), `.take()`/`.absorb()`, rename `factorize` -> `factor`, and confirm `min_by`/`max_by` first-wins tie semantics.
  - [ ] `stdlib/strings.cp`: generalize `RollingHash` over `[D]T` returning `HashVal` with independent per-instance salts; accept `Str` directly without `List` copies.
  - [ ] `stdlib/lazy.cp`: add conformance test pinning `ccc02s4hard` (dual-field cross-overwrite + interleaved `.set` root-to-leaf pushdown).
- [ ] **P2.2 `examples/local-lib/` v2.1 additions:** Extend `Poly` with arbitrary-prime split-base MTT + `.interpolate(pts)`, generalize `LiChao<T>` to `F64` + record values, add `jump_while` to `Functional<V>`, and add `Centroids<V>`, `reroot`, `kruskal_tree`, `linalg` (`det`/`rank`/`solve`), `Interner`, `spanning`, `ranks`, `lis`, `count_in_rects`, `MinCount`, and `RollbackArray`.
- [ ] **P2.3 Unified honest `cpl stress` (`proto/cp.py` + `stdlib/gen.cp`):** Default to in-file `@brute`/`@checker`/`@interactor` (stripping them in `cpl build`), escalate sizes `1..=20`, diff line-by-line (plus `--close <eps>`), always build `-DLOCAL`, shrink via choice streams with side-by-side `debug(...)` replay, add `--max-vals` overflow-hunting mode (`O2`), print coverage + rejection-rate + answer-diversity summary, and ship constructive `std.gen` helpers (`perm`, `surjection`, `bounded_product`, `tree`, `connected`, `dag`, `functional`, `simple_graph`, `str_of`, `rand_points`, `rand_disjoint_rects`, `rand_convex`).

### Priority 3 — Rust Graduation Gate (`crates/`, `editors/vscode/`)

- [ ] **P3.1 Mirror locked v2.1 into `crates/` once P0–P2 pass on the prototype:** After the cheat-sheet CI gate, the 25-problem bakeoff suite, Blind Critics 1–3, Stage 6 max-N benchmarks, and differential fuzzing are green on the v2.1 prototype, implement the unified spec in `crates/` (`cpl-syntax`, `cpl-check`, `cpl-ir`, `cpl-interp`, `cpl-codegen` with `namespace cp_user`) and sync `editors/vscode/` (`languageData.ts`, `cp.tmLanguage.json`).
