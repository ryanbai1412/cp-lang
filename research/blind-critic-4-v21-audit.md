# Blind Critic #4: v2.1 One-Page Cheat-Sheet Contest-Pressure & Pivot Audit

## 1. Protocol and Scope

This audit stress-tested the locked CP Lang v2.1 one-page cheat sheet (`docs/one-page-cheat-sheet.md`) under realistic contest pressure on eight genuinely hard, previously unported problems from `corpus/`, followed by three measured mid-contest pivot and debugging drills and differential verification against the original accepted C++ submissions.

The blind protocol was held strictly:

1. Before writing a single line of code, I read **only** `docs/one-page-cheat-sheet.md`. I did not open `proto/cpc/`, `research/`, `decisions.md`, or `stdlib/` until all eight first drafts (`proto/ports/blind_critic_4/*_draft.cp`) were written and saved.
2. All eight problems were checked for zero overlap with existing `proto/ports/`, `scout_batch_*`, `blind_critic_*`, and the families reserved for Batches 50 (`noi*`, `noip*`) and 51 (`apio*`, `egoi*`, `ccc*`, `cco*`, `btoi*`, `ceoi*`, `joi*`).
3. Every hesitation, ambiguity, syntax guess, and mental-overhead spike was logged inline (`BLIND-GUESS(H1)`..`H37`) while drafting.
4. Only after all eight first drafts were frozen did I inspect `research/language-architecture-v2.1.md`, `decisions.md`, `stdlib/`, and `proto/cpc/`, port all eight into runnable `proto/cpc` forms (`proto/ports/blind_critic_4/<problem>.cp`), execute the three mid-contest drills, and differentially verify all eight against the original C++ submissions.

### The Eight Selected Problems

Six are full IOI classics covering sqrt-decomposition, sparse column DP, cyclic lazy segment trees with binary lifting, messy permutation cycle-hull intervals, 2D dynamic GCD segment trees, interactive tree-hub majority voting, and two-phase combinatorial encoding/decoding; the eighth is a non-typeable constructive/heuristic DMOPC optimization problem:

- **`ioi11p5` (Dancing Elephants, orig `3469626.cpp`)**: online move queries over 1D intervals using sqrt-bucket decomposition and periodic rebuilds.
- **`ioi22p1` (Fish, orig `4785724.cpp`)**: sparse 2-state (`up`/`down`) column DP over sorted per-column fish prefix sums.
- **`ioi20p1` (Plants, orig `5725518.cpp`)**: cyclic range-add/max topological extraction, ordered active/ready sets, and circular binary lifting.
- **`ioi17p6` (Books, orig `5659304.cpp`)** *(messy ad-hoc #1)*: permutation cycle decomposition with nested cycle-hull interval expansion.
- **`ioi13p6` (Game, orig `4688844.cpp`)**: 2D dynamic segment tree over a `10^9 x 10^9` grid supporting point updates and rectangle GCD queries.
- **`ioi15p6` (Towns, orig `5723344.cpp`)**: interactive weighted-tree hub radius and balanced-centroid check via cached `get_distance` and Boyer-Moore branch-majority voting.
- **`ioi11p6` (Parrots, orig `3735613.cpp`)**: two-phase (`encode`/`decode`) order-invariant combinatorial ranking of 8-byte blocks into `<= 5n` symbols in `0..255`.
- **`dmopc21c5p5` (Prime Difference Permutation, orig `4223999.cpp`)** *(messy ad-hoc / non-typeable constructive-heuristic #2)*: maximize adjacent prime differences in a permutation of `1..n` using exact `Subset<V>` Held-Karp DP for `n <= 12` and multi-start 2-opt local search for larger `n`.

---

## 2. Verdict on the Six Headline v2.1 Features

Reading the cheat sheet alone made the v2.1 design feel remarkably cohesive: the eight blind first drafts total **682 non-comment LOC** versus **1,471 raw C++ LOC** (53.6% raw reduction) and **~1,089–1,224 library-stripped C++ LOC** (37.4%–44.3% reduction after removing headers, debug macros, local graders, and hand-rolled segment trees). Still, writing eight hard problems from the cheat sheet exposed sharp documentation gaps and a few genuine language-design collisions.

### Tropical Optionals (`?Int`, `?F64`)

Tropical optionals were the single biggest clarity win in DP and interval code. Replacing sentinel `INF` / `-INF` arithmetic with `var cover: ?Int = none` (`ioi11p5`) and `var best: ?Int = none` (`ioi22p1`) eliminated sentinel-overflow bugs and made unreachable states self-documenting.

Two small ambiguities still caused hesitation under contest pressure:

- **Mixed-type `max`/`min` and multi-argument arity**: In `ioi22p1_draft.cp`, I hesitated over whether `max(0, prev_opt)` accepts `(Int, ?Int)` (promoting `Int` to `?Int` or returning `Int` since `0` is non-none) and whether `max(a, b, c)` accepts 3+ arguments or requires `max([a, b, c])!`.
- **In-place optional relaxation `best max= cand`**: When `best: ?Int` and `cand: ?Int`, does `best max= cand` skip `none` on the RHS as expected, and does `best = best + delta` stay `none` when `best` is `none`? `research/language-architecture-v2.1.md` §U1 confirms yes for both, but the cheat sheet only illustrates `a + b` and `min(xs)` and omits an explicit one-liner showing `dp[j] min= dp[i] + w` with `dp: [V]?Int = none`.

### Lexically Scoped `@memo`

Lexically scoped `@memo` worked naturally in `ioi11p6_draft.cp` (exact multiset-counting DP `ways(rem: 0..=40, lo: 0..=32) -> I128`) and felt great to write without hoisting global arrays.

Two real questions surfaced immediately:

- **Purity and extern calls (`ioi15p6_draft.cp`)**: In interactive problems like Towns, the most natural use of `@memo` is caching a deterministic grader oracle:

```cp
@memo
fn cached_dist(a: V, b: V) -> Int:
    get_distance(a.to_idx, b.to_idx)
```

Nothing on the cheat sheet states whether `@memo` permits calling an extern `@grader fn` (which has hidden I/O / query-counting side effects in the judge) or enforces purity. Under contest pressure, I had to guess whether the compiler would reject this or silently allow it.
- **Return types and dense packing**: In `ioi11p6_draft.cp`, `@memo` returned `I128` over two bounded ranges `0..=40` and `0..=32`. The cheat sheet doesn't clarify whether non-`Int` return types (`I128`, structs, `?Int`) still get flat O(1) tables or fall back to hashing.

### `Marks<D>`

`Marks<D>` (`var seen = Marks<V>()`) had almost zero mental overhead in `ioi17p6_draft.cp` and `dmopc21c5p5_draft.cp`. Reusing the exact `Set` verbs (`.add(v) -> Bool`, `v in seen`, `.clear()`) meant I remembered it immediately without flipping back to the cheat sheet.

The only two tiny snags were:
- Checking membership negation: I hesitated between `not (v in seen)` and `v not in seen` (the cheat sheet mentions `in` in §9/§10 without stating whether `not in` is a valid operator).
- Constructor spelling: §10 writes `Marks<D>()` (with parentheses) while `Set<T>` is written `Set<T>` without showing an empty-constructor call (`Set<V>()` vs `{}`).

### `for mut`

`for mut b in buckets:` (`ioi11p5_draft.cp`) and `for mut col in cols:` (`ioi22p1_draft.cp`) immediately cleaned up one of the most annoying C++ / Rust patterns (re-indexing `buckets[bi]` everywhere just to mutate elements).

However, two extensions came up on the very first problem (`ioi11p5_draft.cp`) and were undocumented on the cheat sheet:

- **Indexed mutable iteration**: When I needed both the bucket index and a mutable reference to the bucket, I guessed `for (bi, mut b) in buckets.items:` without knowing if `mut` is legal inside a destructured pattern or only directly after `for`.
- **Mutating a subslice in place**: In `dmopc21c5p5_draft.cp`, 2-opt reverses a subsegment of `path`: `reverse(mut path[i + 1..=j])`. The cheat sheet documents `xs[l..r]` slices and `mut` arguments, but never confirms whether a slice expression can be passed to a `mut` parameter (`reverse(mut path[i + 1..=j])` or `sort(mut a[l..r])`).

### `Bool` / `Char` / `Subset` `Index`

Using `Subset<V>` as an array index (`var dp: [Subset<V>, V]?Int = none` in `dmopc21c5p5_draft.cp`) read like textbook pseudocode and completely eliminated `1 << v`, `(mask >> v) & 1`, and `mask | (1 << v)` bit-twiddling bugs.

Two gaps showed up when drafting `dmopc21c5p5_draft.cp` and `ioi22p1_draft.cp`:

- **Two different iterations on `Subset<D>`**: The cheat sheet lists `Subset<D>` under `Index` shapes and mentions `.with`/`.without`/`.first`/`s.subsets`, leaving out `.empty`, `.full`, and the distinction between iterating **all subsets** (`for mask in Subset<V>:`) versus iterating **elements inside one subset** (`for u in mask:`). I had to guess all three spellings.
- **`Bool` as a DP index vs small `enum`**: In `ioi22p1_draft.cp`, `[Col, Bool]List<Int>` works, but `dp[col, false]` / `dp[col, true]` is much harder to read under contest pressure than `enum Dir: Up | Down` with `[Col, Dir]List<Int>`. Keeping `Bool: Index` is still great for digit DP (`tight: Bool`, `started: Bool`), where booleans already have self-evident names at the call site.

### `Comb<P>()`

`Comb<P>()` surfaced **two** important findings — one design collision and one scope gap:

1. **Auto-growth vs `let`/`var` and lexical borrowing (design collision)**: The cheat sheet §11 lists `Comb<P>()` with zero arguments (matching v2.1 §U7's lazy auto-growing design, where `research/language-architecture-v2.1.md` writes `var comb = Comb<998_244_353>()`), whereas `stdlib/mod.cp` still declares `Comb<P>.new(limit: Int)` with immutable `fn choose(self, n, k)`. If auto-growing `Comb<P>()` mutates internal `f`/`g` lists via `mut self`, then every helper function taking `comb` must pass `mut comb`, and no closure or nested `@memo` can capture `comb`! Either `Comb<P>()` needs **interior mutability** (so `.choose(n, k)` takes `self`, works with `let comb = Comb<P>()`, and can be freely captured inside closures/`@memo`), or the cheat sheet must warn contestants to write `var comb = Comb<P>()` and pass `mut comb`. Interior mutability (like `@memo`'s hidden cache) is vastly cleaner under contest pressure.
2. **Exact (non-modular) combinatorics (`ioi11p6`)**: In Parrots, we needed exact `I128` binomial/multiset counts, not modular counts. `Comb<P>()` is strictly modular, so I fell back to a 6-line `@memo` DP. That is totally fine (keeping `Comb<P>()` focused on modular counting avoids feature bloat), as long as the cheat sheet explicitly notes `Comb<P>()` is modular.

---

## 3. High-Impact Blind Findings Beyond the Six Features

Inspecting `stdlib/`, `decisions.md`, and `proto/cpc/` after freezing the eight blind drafts revealed **seven high-severity cheat-sheet traps** where a contestant reading only `docs/one-page-cheat-sheet.md` will write code that either fails to compile or silently does the wrong thing.

### 1. `Action(id, compose, apply, over:)` Argument Order & Arity Trap (Critical)

In `ioi20p1_draft.cp`, I needed a custom `LazySegTree` maintaining `(mx, pos)` under range addition. The cheat sheet §11 says only:

> `LazySegTree<D, T>(AddSum, a)` (or `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`, or `Action(id, compose, apply, over:)`)

Reading that line blindly, I guessed:

```cp
let act = Action(
    0,
    (a, b) => a + b,
    (node, tag) => MaxPos<V>(mx: node.mx + tag, pos: node.pos),
    over: Monoid(MaxPos<V>(), merge_max),
)
```

Checking `stdlib/lazy.cp` lines 10 and 36–37 afterward showed I got **three things wrong at once**:

```cp
// stdlib/lazy.cp:
pub struct LazySegTree<D, T, F>(act: Action<T, F>):
    ...
    t[p] = act.apply(f, t[p], span[p])        // 3 args, tag `f` FIRST, then value, then span!
    if p < size: lz[p] = act.compose(f, lz[p]) // new tag `f` FIRST, old tag `lz[p]` SECOND!
```

For any non-commutative lazy action (affine `ax + b`, range assign, matrix multiplication), swapping `compose(new_f, old_f)` silently corrupts answers without a compiler error, and missing `span` breaks compilation immediately. The cheat sheet must show the exact signature `Action(id, (new_f, old_f) => ..., (f, val, span) => ..., over: op)` (and clarify whether `LazySegTree<D, T>` infers `F` or needs 3 type args).

### 2. `first_ge` / `last_le` Silent Return-Type Split (High)

Cheat sheet §10 groups all sorted lookups into one line:

> Sorted lookup (uniform on `List`, `[D]T`, slices, `Set`, `Map`, `SortedBag`; `by:` on sequences): `xs.first_ge(x)`, `xs.last_le(x)`

In `ioi11p5_draft.cp` and `ioi22p1_draft.cp`, I had to stop and guess whether `xs.first_ge(x)` on a `List<Int>` returns the **element** `?Int` (matching `Set<Int>`) or the **index** `?Int` (matching `lower_bound - begin`). In fact, `stdlib/seq.cp` returns **index** `?Int`/`?D` on sequences, `Set`/`SortedBag` return **key** `?K`, and `Map` returns **pair** `?(K, V)`. Calling these "uniform" without naming the three return types is a direct bug magnet because on `List<Int>`, both an index and an element have type `?Int`!

### 3. `Map` `m[k]` Trap vs `m[k] += 1` Auto-Insert Contradiction (High)

Cheat sheet §9 says:

> `Map<K, V>` / `HashMap<K, V>`: `m.get(k) -> ?V`, `m[k]` (traps if missing), `m[k] f= x`, `m[k] ??= init`

In `ioi15p6_draft.cp`, I read "`m[k]` (traps if missing)" and hesitated before writing `groups[x] += 1`, unsure if `m[k] += 1` would trap on a missing key or auto-insert `1` (`decisions.md` confirms `m[k] f= x` initializes missing keys cleanly!). One 4-word parenthetical — `m[k] f= x` (auto-inserts if missing) — completely removes that hesitation.

### 4. No RNG, `shuffle`, or Timer API on the Cheat Sheet (High for Heuristics & Randomized Algos)

In `dmopc21c5p5_draft.cp` (and any treap, randomized hashing, Miller-Rabin, or simulated annealing problem), I looked for `rand`, `shuffle`, and a monotonic clock/timer on the cheat sheet and found **none**. I had to hand-roll a splitmix64/xorshift LCG in `U64` (`dmopc21c5p5_draft.cp`) and `Int` (`dmopc21c5p5.cp`), even though `stdlib/seq.cp` tests already call `rand(lo, hi)`! Exposing `rand(lo..hi)` (or `Rng`) and `shuffle(mut xs)` on the cheat sheet directly eliminates boilerplate in randomized and heuristic CP tasks.

### 5. Missing `List.insert(i, x)` / `List.remove(i)` and `guard let` Early Return (Medium)

- **Sqrt-bucket maintenance (`ioi11p5_draft.cp`)**: Inserting/erasing inside a small sorted `List<Int>` of size `~400` is textbook in CP (sqrt-buckets, Treap-free `vector::insert`/`erase`). The cheat sheet §9 lists `.push`/`.pop`/`.clear` on `List<T>` but omits `.insert(i, x)` and `.remove(i)` (or `.erase`), forcing manual shift loops in `ioi11p5.cp`.
- **Optional early-return boilerplate (`ioi13p6_draft.cp`)**: Inside a recursive segment-tree `fn query(o: ?InnerId, ...) -> Int:`, I instinctively wrote `let id = o? ?? return 0` and immediately realized `?` is illegal in a function returning `Int` (not `?Int`). Without `let id = o else: return 0` (or `o ?? return 0`), every recursive dynamic-tree function nests an extra level of `if let id = o:`. Even if CP Lang declines adding a `guard`/`else` syntax to protect the concept budget, the cheat sheet should show the canonical pattern (`if o == none: return 0; let id = o!`, which stays flat with zero nesting).

### 6. Undocumented `@grader struct` & `SegTree.all` (Medium)

- Three of the six IOI problems (`ioi11p5`, `ioi20p1`, `ioi13p6`) are **stateful graders** (`init(...)` followed by repeated `update`/`query` calls). The cheat sheet §8 shows `@grader fn` but says nothing about file-scope `var` state vs `@grader struct`. Showing a 3-line stateful `@grader` snippet (file-scope `var` initialized inside `@grader fn init(...)` and read in `@grader fn query(...)`, or `@grader struct`) would remove the ambiguity immediately.
- `SegTree.all` and `LazySegTree.all` (O(1) whole-tree fold) already exist in `stdlib/segtree.cp`, `stdlib/lazy.cp`, and `proto/cpc`, yet are omitted from cheat sheet §11 — causing me to write `st.fold(V.first..=V.last)` in `ioi20p1_draft.cp`.

### 7. `elif` vs `else if` Parser Diagnostic (Quick Win)

Coming from C++, I accidentally typed `else if` in both `ioi15p6.cp` and `ioi20p1.cp`. `proto/cpc` rejected it with a cryptic `expected ':' (at 'if')`. Adding a 3-line special-case diagnostic (`use 'elif', not 'else if'`) costs zero language complexity and saves seconds on every contest draft.

---

## 4. Prototype (`proto/cpc`) vs Locked v2.1 Parity Failures

Running `python3 proto/cp.py build` on scratch copies of all eight blind v2.1 first drafts produced **8/8 first-compile failures** — and every single failure exposed either a real prototype bug or an unimplemented v2.1 feature already tracked in `research/language-architecture-v2.1.md`:

- **Lexer crash on labeled loops (`dmopc21c5p5_draft.cp`)**: `'improve: while true:` crashed `proto/cpc/lexer.py:111` with an unhandled Python `ValueError: substring not found` because the lexer treats every `'` as the start of a character literal `'x'`.
- **Struct field defaults unimplemented (`ioi11p5_draft.cp`, `ioi13p6_draft.cp`)**: `xs: List<Int> = []` failed with `expected end of line (at '=')`.
- **Generic structs unimplemented (`ioi20p1_draft.cp`)**: `struct MaxPos<V>:` failed with `expected ':' (at '<')`.
- **Bodyless `@grader fn` unimplemented (`ioi11p6_draft.cp`, `ioi15p6_draft.cp`)**: `@grader fn send(b: Int)` failed with `expected ':' (at None)`.
- **`V.from_idx(i)` return type mismatch (`ioi17p6_draft.cp`)**: `V.from_idx(s_raw)!` failed with `'!' on a non-optional V` (`proto/cpc` still returns `V`, whereas v2.1 returns `?V`).
- **`for mut` unimplemented (`ioi22p1_draft.cp`)**: `for mut col in cols:` failed with `expected 'in' (at 'col')`.
- **`Copy`-element borrow-checker false positive (`ioi13p6.cp`)**: Passing `in_lc[o]` (an `Int`!) into recursive `inner_update(in_lc[o], ...)` was rejected with `` `in_lc[o]` can't be passed to `inner_update`: `inner_update` also changes `in_lc` directly ``, forcing `let cur_lc = in_lc[o]; inner_update(cur_lc, ...)` at every recursive call site. This directly validates v2.1 §U4 (`Copy` reads release borrows immediately before enclosing calls) as a P0 fix for tree/arena code.
- **Container & numeric gaps hit while building runnable forms**: `Set<T>` lacked `.first_ge`/`.last_le` (forcing `SegTree`-backed ordered sets in `ioi20p1.cp`), `for mask in Subset<V>:` failed to parse (forcing manual bitmask domain `0..(1 << m_n)` in `dmopc21c5p5.cp`), `AddMax`/`SegTree` took `(l, r)` pairs instead of `l..=r` ranges, and lacking `I128`/`U64` doubled `ioi11p6.cp` from 66 LOC to 131 LOC via hand-rolled 2-limb base-`2^32` arithmetic.

---

## 5. Three Measured Mid-Contest Pivot & Debugging Drills

To measure how cleanly CP Lang handles mid-contest course corrections (not just clean-slate first drafts), I ran three realistic drills and verified every intermediate and final binary against the original C++ submission.

### Drill 1: Failing Full-Pier Greedy/DP -> 2-State Sparse DP (`ioi22p1`, Fish)

**Scenario**: Under contest time pressure, a tempting first attempt assumes every pier is either empty (`height = 0`) or full (`height = n`), collapsing each column to a single weight sum `col_sum[c]` and running a 4-state linear DP (`37 LOC`).

**Failure caught**: Differential testing against `4785724.cpp` failed **50 / 100** random cases immediately. The smallest counterexample (`n = 3, m = 4`, fish at `(1, 0, 8)`, `(0, 2, 2)`, `(2, 1, 1)`, `(1, 1, 8)`) returned `18` from the flawed full-pier DP versus exact `16`, and larger inputs missed partial-height piers that catch low fish in an adjacent column while leaving high fish in their own column open to the other neighbor.

**Pivot measurement**: Pivoting from the 37-LOC full-pier DP to the 104-LOC sparse 2-state column DP (`ioi22p1.cp`, `+96 / -29` lines) kept the `domain Col = 0..n` declaration and input loop structure intact while replacing `[Col]Int` with `[Col]List<(Int, Int)>`, prefix sums, and the `dp0`/`dp1` suffix sweep. The pivoted binary agreed **200 / 200** with `4785724.cpp`.

**Ergonomics verdict**: In v2.1 (with `for mut col in cols:`, `xs.last_le(h, by: f => f.y)`, and Tropical Optionals `?Int` with `max=`), the pivot is roughly **50 LOC** instead of 104 LOC — half the lines, because `psum` binary search and manual `if v > cmx:` boilerplate vanish. The main friction during the pivot was switching from `col_sum: [Col]Int` to sorted per-column lists without `for mut` in `proto/cpc`.

### Drill 2: `O(N^2)` Naive -> `O((N + Q) log N)` `AddMax` + `SegTree` + Binary Lifting (`ioi20p1`, Plants)

**Scenario**: Start by writing the clean, obviously correct `O(N^2)` topological extraction + `O(N)` per-query DFS on `ioi20p1` (`58 LOC`) to validate the problem model against the judge before investing in data structures.

**Verification of naive baseline**: The `O(N^2)` baseline agreed **100 / 100** with `5725518.cpp` on random inputs (`n <= 18, q = 20`).

**In-place upgrade & benchmark**: Upgrading in place to `ioi20p1.cp` (`AddMax<V>` storing `-r[i]`, `SegTree<V, Int>` ordered ready sets, max-height `SegTree<V, Int>` for nearest-lower neighbors, and `[L, V]Int` binary lifting) preserved the `domain V = 0..n`, input reading, `height: [V]Int`, and `can_reach` query loop signature verbatim. On a medium stress test (`N = 1500, K = 400, Q = 1500`, `x != y`):

- Naive `O(N^2)` CP Lang binary: **217.1 ms**
- Upgraded `O((N + Q) log N)` CP Lang binary (`ioi20p1.cp`): **5.1 ms** (**42.3× speedup**, exact output match with both naive and reference)
- Original hand-tuned C++ (`5725518.cpp`): **4.5 ms** (our CP Lang port runs within **1.13×** of the original C++, even using binary search over `AddMax` instead of a custom `max_right`!)

**Ergonomics verdict**: Upgrading an `O(N^2)` prototype to `O((N + Q) log N)` was smooth because `domain V` kept every array and tree aligned on the same index type. Two things slowed the upgrade down: lacking `Set.first_ge`/`last_le` in `proto/cpc` (forcing three extra `SegTree`s) and manually wrapping cyclic intervals (`l <= r` vs `l..n-1` + `0..r`) in both `add_cyc` and `query_max_cyc`.

### Drill 3: Authentic Subtle Edge-Case WA Diagnosis (`ioi15p6`, Towns)

**Scenario**: While running our differential harness on `ioi15p6.cp`, an unscripted, realistic contest WA surfaced on test `it = 5` (`n = 42`, returning `64` instead of `-64`). Rather than inventing an artificial typo, I used this authentic bug as Drill 3.

**Root cause**: In the stack-based Boyer-Moore branch-majority loop over `hub_nodes`, a mismatch pushes a new single-element bucket `(v, size = 1)`, while a match with the current candidate `cand` must increment **`cand`'s own bucket**:

```cp
// Buggy (incremented the most recently pushed mismatch bucket!):
elif same_branch(cand, v):
    lead += 1
    bsz[bsz.len - 1] += 1

// Fixed (2-line change tracking `cand_bi`):
elif same_branch(cand, v):
    lead += 1
    bsz[cand_bi] += 1
```

**Measured frequency & counterexample**: Across 300 random trees, the buggy version failed only **3 / 300** cases (a 1% silent failure rate that passes every sample and every small hand-drawn tree!). The smallest failing tree found had `n = 27` leaves (`it = 110`, returning `3107` instead of `-3107`). Applying the 2-line fix brought agreement to **300 / 300** with `5723344.cpp`.

**Ergonomics verdict**: CP Lang's bounds-checked domains prevented memory corruption, but two parallel `List<Int>` arrays (`buckets` and `bsz`) made it easy to index `bsz.len - 1` instead of `cand_bi`. Packing `(rep, size)` into a single `List<Bucket>` with `for mut` (blocked in `proto/cpc` only by missing struct defaults/ergonomics) and using v2.1's `cpl stress` with a constructive tree generator (`std.gen.tree`) makes 1%-rate WAs trivial to catch and shrink.

---

## 6. Differential Verification Summary & LOC Measurements

All eight runnable ports live in `proto/ports/blind_critic_4/` alongside the frozen blind first drafts (`*_draft.cp`) and the self-contained differential harness `proto/ports/blind_critic_4/check.py` (which compiles thin C++ stdio drivers around the seven IOI grader submissions in a temp directory so tracked files are never modified).

Command run:

```text
python3 proto/ports/blind_critic_4/check.py --iters 200
```

Observed output (exit code `0`):

```text
OK ioi11p5: 200 tests agree with 3469626.cpp
OK ioi22p1: 200 tests agree with 4785724.cpp
OK ioi20p1: 200 tests agree with 5725518.cpp
OK ioi17p6: 200 tests agree with 5659304.cpp
OK ioi13p6: 200 tests agree with 4688844.cpp
OK ioi15p6: 200 tests agree with 5723344.cpp
OK ioi11p6: 200 tests agree with 3735613.cpp
OK dmopc21c5p5: 12 exact Subset-DP cases (n=1..12, >= ref) + 18 heuristic cases (valid perms, score ratio 0.9983 vs 4223999.cpp)
```

One bonus differential find: on `ioi11p5` (`n = 1`), our CP Lang port returned `1` cleanly for every update while the original accepted C++ submission `3469626.cpp` segfaulted (assuming `n >= 2`).

### Dual LOC Comparison (Non-Comment, Non-Blank Lines)

- **`ioi11p5`**: `3469626.cpp` 210 raw / 124 stripped -> v2.1 draft **96** (0.46× raw, 0.77× stripped) -> `proto/cpc` runnable **122**.
- **`ioi22p1`**: `4785724.cpp` 116 raw / 90 stripped -> v2.1 draft **50** (0.43× raw, 0.56× stripped) -> `proto/cpc` runnable **83**.
- **`ioi20p1`**: `5725518.cpp` 250 raw / ~153 stripped (ex-`ST1`) -> v2.1 draft **116** (0.46× raw, 0.76× stripped) -> `proto/cpc` runnable **190**.
- **`ioi17p6`**: `5659304.cpp` 318 raw / ~221 stripped (ex-allocator/`ST1`) -> v2.1 draft **101** (0.32× raw, 0.46× stripped) -> `proto/cpc` runnable **101**.
- **`ioi13p6`**: `4688844.cpp` 124 raw / 98 stripped -> v2.1 draft **83** (0.67× raw, 0.85× stripped) -> `proto/cpc` runnable **119**.
- **`ioi15p6`**: `5723344.cpp` 174 raw / 153 stripped -> v2.1 draft **80** (0.46× raw, 0.52× stripped) -> `proto/cpc` runnable **142**.
- **`ioi11p6`**: `3735613.cpp` 96 raw / 92 stripped -> v2.1 draft **66** (0.69× raw, 0.72× stripped) -> `proto/cpc` runnable **131** (inflated 2× solely by missing `I128`/`@memo` in `proto/cpc`).
- **`dmopc21c5p5`**: `4223999.cpp` 183 raw / 158 stripped -> v2.1 draft **90** (0.49× raw, 0.57× stripped) -> `proto/cpc` runnable **120**.
- **Totals**: **1,471 raw** / **~1,089 library-stripped** C++ LOC -> **682 v2.1 draft LOC** (**53.6% raw reduction**, **37.4% library-stripped reduction**) -> **1,008 `proto/cpc` runnable LOC** (**31.5% raw reduction**).

---

## 7. Concrete Recommendations

### A. One-Page Cheat-Sheet Edits (`docs/one-page-cheat-sheet.md` — Zero Language Growth)

1. **Pin `Action` signature and `LazySegTree` arity in §11**: Replace `Action(id, compose, apply, over:)` with `Action(id, (new_f, old_f) => ..., (f, val, span) => ..., over: op)` and clarify `LazySegTree<D, T>(act, a)` infers the tag type `F` from `act`.
2. **State `first_ge` / `last_le` return types explicitly in §10**: Write "`xs.first_ge(x)` / `xs.last_le(x)` -> `?Int`/`?D` index on sequences (`by:` supported), `?K` key on `Set`/`SortedBag`, `?(K, V)` pair on `Map`".
3. **Clarify `Map` compound assignment in §9**: Change `m[k] f= x` to `m[k] f= x` (auto-inserts from identity/operand if missing; `m[k]` alone traps).
4. **Add `.all` to `SegTree` / `LazySegTree` in §11**: Note `st.all` for O(1) whole-domain fold so contestants don't write `st.fold(D.first..=D.last)`.
5. **Show a 2-line Tropical Optional DP relaxation in §2**: Add `var dp: [V]?Int = none; dp[v] min= dp[u] + w` and note `max(a, b)` promotes `(Int, ?Int) -> Int` (or clarify the exact rule).
6. **Complete `Subset<D>` verbs in §10**: Add `.empty`, `.full`, `for mask in Subset<D>:` (all subsets), and `for v in mask:` (members of one subset).
7. **Clarify `@grader` stateful files and `@memo` extern calls in §8**: Add a 3-line stateful grader note (top-level `var` initialized in `@grader fn init(...)` and read in `@grader fn query(...)`) and explicitly state whether `@memo` may wrap `@grader fn` calls.
8. **Clarify `for mut` patterns and `not in` in §5**: State whether `for (i, mut x) in xs.items:` and `x not in s` are valid, and show the flat optional early-return idiom (`if o == none: return 0; let id = o!`).

### B. v2.1 Ergonomics & Stdlib Refinements (Ranked)

1. **P0 — Interior mutability on `Comb<P>()`**: Make `Comb<P>()` auto-grow via interior mutability (like `@memo`) so `let comb = Comb<998_244_353>()` works with immutable `self`, passes freely to helper functions without `mut`, and can be captured inside closures and `@memo` functions. Align `stdlib/mod.cp` with the zero-arg `Comb<P>()` spelling.
2. **P0 — Fix `proto/cpc` P0 parity blockers exposed by the 8 drafts**: Fix the unhandled lexer crash on `'label:` (`proto/cpc/lexer.py`), implement v2.1 `Copy`-element borrow release (`ioi13p6` recursive arena calls), add `Set.first_ge`/`last_le`, add `for mut` + `for mask in Subset<D>:`, align `V.from_idx -> ?V`, and add `I128`/`U64` (cutting `ioi11p6` in half).
3. **P1 — Expose `rand` / `shuffle` on the cheat sheet and in `std`**: Randomized algorithms and heuristics (`dmopc21c5p5`, treaps, anti-hack hashing) need a first-class deterministic PRNG (`rand(lo..hi)`, `shuffle(mut xs)`), which already partially exists in `stdlib` tests.
4. **P1 — Add `List.insert(i, x)` and `List.remove(i)`**: Essential for sqrt-bucket decomposition (`ioi11p5`) and small dynamic arrays; 2-line wrappers over C++ `vector::insert` / `vector::erase`.
5. **P1 — Helpful parser diagnostic for `else if`**: Detect `else` followed by `if` in the parser and emit `use 'elif', not 'else if'` instead of `expected ':' (at 'if')`.

---

## 8. Untested / Out-of-Scope Areas

To keep claims strictly grounded:

- **`proto/cpi` interpreter & Rust `crates/`**: Not modified and not run against the eight new ports (only `proto/cpc` -> `g++ -std=c++17 -O2` was tested).
- **Max-N (200,000+) judge RSS and worst-case TLE stress**: Differential testing ran 200 random small-to-medium cases per problem (300 on `ioi15p6`, plus `N = 1500, Q = 1500` timing on `ioi20p1`); full max-N (`10^5`–`10^6`) peak-RSS and TLE sweeps were not run in this audit.
- **Real IOI grader ABI compilation (`@grader fn` / `@grader struct`)**: Verified via standalone stdio drivers against the original C++ functions; actual C++ header/ABI linking for `@grader` remains untested until `proto/cpc` implements bodyless `@grader fn`.
- **Query-count enforcement on `ioi15p6`**: Our port tracks `queries` internally with symmetric caching and Boyer-Moore bucket compression matching `5723344.cpp`, but `check.py` compared final return values rather than asserting `queries <= (7 * n + 1) / 2` on every test.
