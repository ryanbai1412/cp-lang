# Friction Backlog — Scout Batch 24: Hard Segment Trees, Lazy Propagation, Persistent/Implicit DS, Mo's & Offline Queries

Scout Batch 24 ported 10 hard, previously unported data-structure and offline-query problems (`ioi07p3`, `noi07p4`, `noip22p4`, `noi20p2`, `noi05p2`, `cco18p5`, `ioi13p3io`, `dmpg15g6`, `ceoi22p1`, `cco16p5`) statement-first under `proto/ports/scout_batch_24/`, then verified all 10 runnable `today/` implementations differentially against the original AC C++ submissions (200/200 passing cases via `proto/ports/scout_batch_24/check.py`).

Raw LOC dropped **64.7%** (2,549 C++ LOC to 901 ideal CP Lang LOC; 875 runnable `today/` LOC, **-65.7%**), and library-stripped LOC dropped **33.3%** (1,351 to 901 ideal including `@brute` specs; 875 `today/`, **-35.2%**). Equally informative are the places where CP Lang tied or *lost* to stripped C++—`noi05p2` (183 stripped C++ vs 185 ideal / 191 `today/`), `ioi07p3` (48 vs 54 / 47), and `ceoi22p1` (65 vs 93 with `@brute` / 65 `today/`)—which pinpoint exact ergonomics gaps in custom monoids/actions, `Pool<Node>` trees, offline query pipelines, and stress generation.

---

## 1. Custom `Monoid` / `Action` Boilerplate & Where `LazySegTree` Breaks

CP Lang's operation-value design (`Monoid<T>`, `Action<T, F>`, `SegTree<D, T>`, `LazySegTree<D, T, F>`) eliminates 150–260 lines of hand-rolled C++ segment-tree boilerplate in `ioi07p3`, `noi07p4`, `noip22p4`, and `cco18p5` (where statement-first dual `SegTree` local-peak tracking took 51 runnable LOC vs 153 stripped C++ LOC for Ryan's offline time-segtree + rollback-segtree solution in `4801867.cpp`). Five sharp friction points surfaced beyond basic `AddSum` / `AddMax` examples.

### 1.1 Asymmetric `std.lazy` presets, missing `ArgMax`, and missing `LazySegTree.max_right` / `min_left` (`ioi07p3`, `cco18p5`)

`stdlib/lazy.cp` ships `MaxOpt` and `AddMax`, but no `MinOpt` or `AddMin`. `stdlib/algebra.cp` ships `Max<Int>`, which returns only the value and not `(val, pos: I)` (`ArgMax`). Worse, `SegTree` has `max_right` and `min_left`, while `LazySegTree` lacks both—even though segment-tree descent with lazy pushdown is standard in IOI '07 Sails, IOI '05 Mountain, and threshold/beats problems.

In `ioi07p3.cp`, we wanted:

```cp
let left_end = st.max_right(H.first.l, x => x == none or x! > v)
let right_end = st.max_right(H.first.l, x => x == none or x! >= v)
```

Without `LazySegTree.max_right`, `today/ioi07p3.cp` dropped to an $O(\log^2 H)$ outer binary search calling `st.get(H(mid))` and re-declared `min_opt` / `add_min` by hand:

```cp
let min_opt = Monoid<?Int>(
    id: none,
    op: (a, b) => if a == none: b else: if b == none: a else: if a! < b!: a else: b
)
let add_min = Action<?Int, Int>(
    id: 0,
    compose: (f, g) => f + g,
    apply: (f, x) => if x == none: none else: x! + f
)
```

Similarly, `cco18p5.cp` had to hand-write a 6-line `ArgMax` monoid over `Peak(val: Int, pos: I)` just to know *which* index achieved the window maximum:

```cp
struct Peak(val: Int, pos: I)
const ArgMax = Monoid<?Peak>(
    id: none,
    op: (a, b) =>
        if a == none: b
        else: if b == none: a
        else: if a!.val > b!.val or (a!.val == b!.val and a!.pos.to_idx > b!.pos.to_idx): a else: b
)
```

**Recommendation (stdlib):** Add `MinOpt`, `AddMin`, `AssignMax`, `AssignMin`, `ArgMax<D, T>` / `ArgMin<D, T>`, and `LazySegTree.max_right` / `min_left` (pushing lazy tags on descent, identical signature to `SegTree`). Zero language core changes.

### 1.2 Verbose `Option`-wrapped semigroups & `Assign` actions (`noi07p4`, `noip22p4`)

Every non-identity aggregate like `Run(lcol, rcol, runs)` in `noi07p4.cp` has to be lifted to `?Run` (or given a manual `empty: Bool` flag in `today/noi07p4.cp`), and then *every* `Action.apply` must handle both `f == none` and `x == none`:

```cp
const RunMonoid = opt(Semigroup<Run>(
    (a, b) => Run(a.lcol, b.rcol, a.runs + b.runs - if a.rcol == b.lcol: 1 else: 0)
))
const PaintAction = Action<?Run, ?Int>(
    id: none,
    compose: (f, g) => if f == none: g else: f,
    apply: (f, x) => match (f, x):
        (none, _) => x
        (_, none) => none
        (c, _) => Run(c, c, 1)
)
```

For `noip22p4` (NOIP '22 Contest, historical sum of $\max(a) \cdot \max(b)$), the 5-field `Node` + 6-field `Tag` took 38 lines of `HistMonoid` + `HistAction` boilerplate. Even so, typed range-assign tags (`ta: ?U64, tb: ?U64, cab, ca, cb, c1`) cut `noip22p4` from 191 stripped C++ LOC (`5445685.cpp`'s hand-packed 15-element upper-triangular matrix with `#pragma GCC unroll 5`) down to 86 CP Lang LOC—provided wrapping `U64` arithmetic exists.

**Recommendation (stdlib):** Add `opt_action(a: Action<T, F>) -> Action<?T, F>` and `assign_action<T, V>(make: (V, T) -> T) -> Action<T, ?V>` in `stdlib/algebra.cp` so `none`-tag / `none`-node plumbing is written once.

### 1.3 Heavy value-type `Monoid<Mat>` vs in-place combine (`ioi13p3io`)

In `ioi13p3io` (IOI '13 Wombats), each segment-tree node holds a `200 x 200` `I32` Monge matrix (160 KB). `SegTree<Blk, Mat>` forces `Monoid.op` to have signature `(Mat, Mat) -> Mat` by value, allocating a fresh 160 KB buffer on every internal pull and moving a full 160 KB leaf on every `st.set(b, build_block(...))`. It also requires an identity matrix `id_mat` even though Wombats only reads `st.all` over a full non-empty domain:

```cp
// Every internal pull allocates a fresh 160 KB Mat and copies it into t[p]:
let mat_monoid = Monoid<Mat>(id: id_mat, op: (a, b) => combine_monge(a, b))
var st = SegTree<Blk, Mat>.of(mat_monoid, b => build_block(...))
```

In C++ (`4722412.cpp`), fixed-size `array<array<int, 200>, 200>` lives inline in `v[256]` with zero heap churn. In CP Lang, runtime-sized `[Col, Col]I32` is heap-backed, so value returns in `Monoid.op` trigger $O(\log N)$ heap allocations per update.

**Recommendation (stdlib + compiler):** Give `SegTree` an `update_with(i, mut leaf => ...)` method (matching `4253124.cpp`'s `set_func`) and an optional in-place `combine_into(a, b, mut dst)` constructor for heavy nodes; allow a `Semigroup`-backed `SegTree` when only non-empty range folds / `st.all` are queried.

### 1.4 Structural non-fit: Tag-free Cover-Count Segment Tree (`cco16p5`)

Klee's Measure Problem (`cco16p5`, Zombie Apocalypse) exposes a classic segment tree that is **not** a `LazySegTree`:

```cp
if self.cnt[p] > 0:
    self.cov[p] = self.ys[hi] - self.ys[lo]
elif lo + 1 == hi:
    self.cov[p] = 0
else:
    self.cov[p] = self.cov[2 * p] + self.cov[2 * p + 1]
```

Paired `+1`/`-1` updates never push down; when `cnt[p]` drops from `1` to `0`, `cov[p]` must be recovered from `cov[2*p] + cov[2*p+1]`. Because `apply(-1, node)` cannot compute the new `cov` from `node` alone, `Action<T, F>` cannot express it. Even hand-rolled, `today/cco16p5.cp` (62 LOC) was **3.3x shorter** than Ryan's 209-line boundary-geometry C++ solution (`4580695.cpp`).

**Recommendation (stdlib):** Do *not* warp `Action<T, F>` to fit tag-free trees; optionally provide a small userland `CoverTree` / `RectUnion` helper if 2D sweep-line rectangle unions recur across the corpus.

---

## 2. `Pool<Node>` + `Node.Id` for Implicit & Persistent Trees

Both `noi20p2` (dynamic implicit segment-tree merging on a tree) and `noi05p2` (implicit treap with Kadane aggregates and dual lazy tags under a 64 MB memory limit) stress-tested CP Lang's `Pool<Node>` + `Node.Id` design (`plan/14-state.md`).

### 2.1 Pool indexing verbosity and self-updating recursion (`noi05p2`, `noi20p2`)

In `noi05p2.cp`, replacing C++'s `p->val`, `p->l`, `p->r` with `pool[u].val`, `pool[u].l`, `pool[u].r` produced **185 ideal CP Lang LOC vs 183 stripped C++ LOC**—no conciseness gain over raw pointers:

```cp
mut fn apply_same(id: ?Node.Id, c: I32):
    if let u = id:
        let s = c * pool[u].sz
        pool[u].val = c
        pool[u].sum = s
        pool[u].same = c
        pool[u].rev = false
        pool[u].pref = max(0i32, s)
        pool[u].suff = max(0i32, s)
        pool[u].best = if c >= 0i32: s else: c
```

Worse, in `noi20p2.cp` and `noi05p2.cp`, recursive tree updates constantly write:

```cp
pool[a].l = merge_trees(pool[a].l, pool[b].l, lo, mid, pre_a, pre_b)
pool[u].r = merge(pool[u].r, b)
```

That single line is a **two-phase borrow & vector-reallocation trap**: reading `pool[a].l` and `pool[b].l` happens before the call, then `merge_trees` mutates `pool` (potentially reallocating `pool`'s backing vector when allocating a new node!), and only *after* `merge_trees` returns may the write target `&mut pool[a].l` be resolved! In C++, `pool[a].l = merge_trees(...)` can be **dangling-reference UB** under left-to-right assignment evaluation; in a naive borrow checker, it is rejected as overlapping mutable borrows of `pool`.

**Recommendation (language):** Guarantee in the language spec and codegen that `place_container[idx].field = rhs` evaluates `rhs` (and any index expressions) *before* resolving the mutable slot in `place_container`, so recursive `pool[u].l = f(pool[u].l)` both typechecks without temporary locals and is 100% immune to backing-vector reallocation.

### 2.2 Built-in free-list recycling & compact `I32`/`U32` storage (`noi05p2`)

`noi05p2` enforces a strict **64 MB memory limit** with up to 4,000,000 inserted elements and at most 500,000 live elements. Without recycling deleted nodes, 4M × 48 B = 192 MB → instant MLE. Every dynamic BST user currently hand-rolls a free stack:

```cp
var pool = Pool<Node>.with_capacity(500_005)
var free_ids: List<Node.Id> = []
mut fn alloc_node(v: I32) -> Node.Id:
    if not free_ids.is_empty:
        let id = free_ids.pop()!
        pool[id] = nd
        id
    else:
        pool.alloc(nd)
```

Even with recycling, storing default 64-bit `Int` in all 11 `Node` fields makes `sizeof(Node) == 96` bytes (48 MB for the pool alone, risking MLE with I/O buffers), whereas `I32`/`U32` + 4-byte `?Node.Id` keeps `Node` at **44 bytes (22 MB)**.

**Recommendation (stdlib):** Add `pool.free(id)` to `Pool<Node>` (internal free-list reuse, with debug-mode generation check to catch use-after-free) and guarantee `sizeof(Node.Id) == 4` and `sizeof(?Node.Id) == 4` (niche-packed null sentinel).

---

## 3. Offline Query Sorting, Domain Clamping & Rollback Mo's

Five problems (`noip22p4`, `cco18p5`, `dmpg15g6`, `ceoi22p1`, `cco16p5`) exercised offline query pipelines, coordinate compression, and windowed domain indexing.

### 3.1 Missing `dedup()`, `lower_bound(x)`, and `sort(..., by:)` (`cco16p5`, `ceoi22p1`, `dmpg15g6`)

`cco16p5.cp` repeated 14 lines of sort-dedup-binary-search coordinate compression, returning raw `Int` instead of a typed compressed domain:

```cp
sort(mut ys)
var uniq: List<Int> = []
for y in ys:
    if uniq.is_empty or uniq.last! != y:
        uniq.push(y)

fn y_id(y: Int) -> Int:
    var (lo, hi) = (0, uniq.len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if uniq[mid] < y: lo = mid + 1
        else: hi = mid
    lo
```

In `ceoi22p1` and `dmpg15g6`, `proto/cpc` lacks `sort(mut order, by: qi => qs[qi].t)` with a key closure, forcing manual tuple packing and `.to_idx` round-trips:

```cp
let order = sorted([(qs[qi].t, qi.to_idx) for qi in Q])
for pair in order:
    let qi = Q(pair.1)
```

Meanwhile, `cco18p5.cp` had to drop out of domain `I` into `.to_idx` + `I.nth(...)` four times just to clamp a radius-`k - 1` neighbor window around `p: I`:

```cp
let pi = p.to_idx
if pi + 1 >= n: return none
let hi = min(n - 1, pi + k - 1)
val_st.fold(I.nth(pi + 1)..=I.nth(hi))
```

**Recommendation (stdlib + prototype):** Add `list.dedup()`, `list.lower_bound(x)`, `sort(mut a, by: f)` / `sorted(a, by: f)` (preserving domain element types), `group_by<D, Q>(qs, key)` for offline sweeps, and clamped domain-slice helpers (`I.clamp(lo_idx..=hi_idx)` or `p.window(rad)`).

### 3.2 Nested `undo` scopes for Rollback Mo's (`dmpg15g6`)

`dmpg15g6.cp` demonstrated that `Journaled<[Flav]Int>` + `undo` (`plan/14-state.md`) makes Rollback Mo's extraordinarily clean—*provided* `undo` can nest and snapshot small copyable value structs (`Top2`) alongside a `Journaled` array:

```cp
undo freq, top:                  // outer scope: resets whole block in O(touched), never O(M)
    var cur_r = border - 1
    for qi in bucket:
        while cur_r < r.to_idx:
            cur_r += 1
            add(I.nth(cur_r))
        undo freq, top:          // inner scope: rolls back left tail per query
            for p in l.to_idx..border:
                add(I.nth(p))
            ans[qi] = top.ok(r.to_idx - l.to_idx + 1)
```

Without `undo`, `today/dmpg15g6.cp` required 25 extra lines of manual `log_f`, `log_v`, `outer_mk`, `inner_mk`, and four saved scalar registers (`sf1, sc1, sf2, sc2`). Also note the algorithmic comparison: our statement-first Rollback Mo's (`83 LOC` ideal) was cleaner than Ryan's Misra-Gries Heavy-Hitter SegTree (`4253124.cpp`, `101 LOC` stripped), while `today/dmpg15g6.cp` (97 LOC) nearly tied it only because of the manual rollback log.

---

## 4. Non-Typeable Input & Stress Constraints

`cpl stress` aims to synthesize valid test cases directly from `read()` declarations. Five problems in Batch 24 expose structural preconditions that types alone cannot capture—and one caught a **real UB crash in an AC C++ submission**:

1. **Permutations & parity (`noip22p4`, `ceoi22p1`)**: `noip22p4` requires `a` and `b` to be permutations of `1..=n`; `ceoi22p1` requires `n % 2 == 0` AND `deck` to be a permutation of `1..=n`. Typing `deck: [Pos]Card` generates duplicate cards, breaking `pos_of[card]` bijections.
2. **Strict tree ancestor-descendant pairs (`noi20p2`)**: Each path constraint `(u, v)` must satisfy `u != v` and `tree.is_anc(u, v)` on the *tree just read* (violating this triggers `assert(depth[u] < depth[v])` in `5655455.cpp`).
3. **Stateful, variable-length command streams (`noi05p2`)**: `INSERT posi tot c_1..c_tot` reads its own payload length `tot` mid-command, and every command's `(posi, tot)` must stay within the live sequence length `cur_len`, with `cur_len >= 1` for `MAX-SUM` and `tot >= 1` for `INSERT`. When our differential harness initially generated `INSERT posi 0`, the original AC C++ submission `7249409.cpp` **segfaulted** inside `buildSeq(0)` (`st.back()` on an empty vector!), while our CP Lang port returned `none` safely.
4. **Distinct coordinates & per-variant caps (`cco16p5`, `ioi13p3io`)**: `cco16p5` expects distinct `(r, c)` zombie coordinates; `ioi13p3io` caps update events (`changeH`/`changeV <= 500`, asserted in `4722412.cpp`) separately from total events (`<= 200,500`).

**Recommendation (language + `cpl stress`):**
- Make `Perm<D>` a first-class readable and generatable type (bijection `D -> D`, O(1) `.inv`, permutation-preserving shrinker).
- Keep `gen` blocks concise and composable for dependent/stateful inputs (tree-ancestor generators, stateful command-stream generators tracking `cur_len`, distinct point sets), and treat `where` clauses on `read()` as filters/assertions rather than trying to invert arbitrary predicates.

---

## 5. Multi-Feature Collisions Discovered

End-to-end porting exposed six concrete multi-feature collisions:

1. **Dependent `domain Pre` inside `Rooted<V>` vs value field access (`noi20p2`)**: `stdlib/tree.cp` declares `domain Pre = 0..V.K` inside `pub struct Rooted<V>` and indexes `tree.at` by `Pre`, yet `for pre_pos in tree.Pre.rev:` failed in `proto/cpc` with `Rooted<V> has no field or method Pre`. We had to collect `tree.sub(V.first)` into a `List<V>` and walk it backwards. **Fix:** Expose associated domains on struct instances (`tree.Pre`) AND add `tree.postorder()` / `tree.preorder()` to `stdlib/tree.cp` so tree DP never touches `Pre`.
2. **`SegTree` × heavy value-type `Monoid<Mat>` (`ioi13p3io`)**: Value-returning `Monoid.op: (T, T) -> T` triggers $O(\log N)$ 160 KB heap allocations per update for `Mat([Col, Col]I32)`. Needs `update_with` / `combine_into` for heavy nodes.
3. **`Pool<Node>` × lexical borrowing on self-updating recursion (`noi20p2`, `noi05p2`)**: `pool[a].l = merge_trees(pool[a].l, pool[b].l, ...)` simultaneously targets `pool[a].l` on the LHS while passing `mut pool` into the RHS. Requires guaranteed RHS-before-LHS-place evaluation so it compiles cleanly and never dangles on pool resize.
4. **Typed `@tag(...)` `enum Cmd` × variable-length payloads (`noi05p2`)**: `@tag("1") ChangeH(p: Row, q: HCol, w: I32)` works cleanly for fixed-arity commands (`ioi13p3io`, `noi07p4`), but cannot parse `INSERT posi tot c_1..c_tot` where `tot` sizes the list inside the same variant.
5. **`Journaled` + `undo` × multi-variable atomic rollback (`dmpg15g6`)**: Rollback Mo's needs a single `undo freq, top:` scope to checkpoint both a `Journaled<[Flav]Int>` array and a small value struct `Top2`, AND to nest (outer per-block reset in $O(\text{touched})$, inner per-query reset).
6. **Missing narrow/wrapping scalars (`U64`, `I32`, `U32`) × advanced DS (`noip22p4`, `noi05p2`)**: Missing `U64` (wrapping mod $2^{64}$) prevents `today/noip22p4.cp` from handling full $N = 2.5 \cdot 10^5$ inputs where historical sums intentionally wrap mod $2^{64}$, and missing `I32`/`U32` struct fields double treap node memory in `noi05p2` under a 64 MB limit.

---

## 6. Prioritized Recommendations (Small-Core Discipline)

### Do Add (High Leverage, Minimal or Zero Core Complexity)
1. **`stdlib/lazy.cp` & `stdlib/algebra.cp`**: Add `LazySegTree.max_right` / `min_left`, `MinOpt`, `AddMin`, `AssignMin`, `AssignMax`, `ArgMax<D, T>` / `ArgMin<D, T>`, and `opt_action` / `assign_action` combinators.
2. **`stdlib/seq.cp` & `stdlib/tree.cp`**: Add `list.dedup()`, `list.lower_bound(x)`, `sort(mut a, by: f)` / `sorted(a, by: f)`, `group_by`, clamped domain windows, and `Rooted.preorder()` / `Rooted.postorder()` (plus fix `tree.Pre` access).
3. **`Pool<Node>` spec (`plan/14-state.md`)**: Add `pool.free(id)` free-list recycling, 4-byte niche-packed `?Node.Id`, and guaranteed RHS-before-LHS-place evaluation for `pool[u].field = f(...)`.
4. **Scalar & stress completeness**: Ship wrapping `U64`, compact `I32`/`U32` struct fields, and first-class readable/generatable `Perm<D>` in the Rust compiler.
5. **`Journaled` + `undo` spec**: Support nested `undo` scopes over both `Journaled` containers and copyable `var` structs (`undo freq, top:`).

### Do *Not* Add to the Language Core
- Do **not** distort `Action<T, F>` to fit tag-free Cover-Count segment trees (`cco16p5` is 62 lines hand-rolled and cleanly served by an optional userland `CoverTree`).
- Do **not** invent a mini-language for stateful variable-length command parsers (`noi05p2`): fixed-arity `@tag(...)` enums cover ~90% of query problems (`ioi13p3io`, `noi07p4`), and a plain `if op == "INSERT":` loop is clear for variable-length commands.
- Do **not** add stored mutable references to mimic C++ `Node*` treaps: guaranteed RHS-before-LHS evaluation on `pool[u].l = f(pool[u].l)` + `pool.free(id)` removes the friction while preserving full memory safety.
