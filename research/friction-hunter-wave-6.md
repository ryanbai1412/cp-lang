# Friction Hunter — Wave 6 (30 hard ports, post-Syntax-batch-4)

Base: `main` at `919ac75`. Read-only audit; every finding was checked on HEAD
with `target/release/cpl run`, and the two WA traps also through `cpl build` +
`g++ -std=c++17 -fsanitize=address,undefined`. None of these 30 ports appear
in waves 1–5.

Labels: **[WA-TRAP]** compiles and silently gives a wrong answer;
**[REAL GAP]** no clean form on HEAD; **[STALE PORT]** a clean form exists on
HEAD; **[REJECTED]** one-off or algorithm-specific, not a language gap.

## Ports (30)

- **Geometry / sweeps:** `cco16p2`, `dpz` (convex hull trick),
  `scout_batch_34/today/dmopc20c7p2` (perimeter walk on lattice),
  `scout_batch_24/today/cco16p5` (union-of-squares area sweep),
  `scout_batch_52/noi08p5` (2D Fenwick on a folded plane).
- **Strings / automata:** `scout_batch_10/sac22cc3p5` (trie),
  `scout_batch_22/ccc01s5` and `scout_batch_22/cco19p6` (Post-correspondence
  overhangs), `scout_batch_16/coci21c6p5` (bracket repair),
  `scout_batch_23/today/coci21c1p2` (game DP over a string).
- **Heavy data structures:** `scout_batch_18/today/ioi05p2`,
  `scout_batch_24/today/noi07p4`, `scout_batch_24/today/cco18p5`,
  `scout_batch_24/today/ioi07p3`, `scout_batch_34/today/dmopc21c7p4`,
  `scout_batch_18/today/btoi17p2` (min-plus matrix segtree),
  `scout_batch_39/today/sac22cc4p4` (monotone deques),
  `scout_batch_45/noimps22p5` (`SortedBag`).
- **Flows / matching:** `scout_batch_15/coci25c3p3` (bipartite peel matching),
  `scout_batch_15/sdoi17r1p4` (fractional matching), `scout_batch_5/today/noi06p4`
  (closure max-flow), `scout_batch_52/noi19p3` (simulated cost flow).
- **Trees / bitmask DP:** `scout_batch_18/today/ccc22s5`,
  `scout_batch_51/today/cco03p4`.
- **Interactive / constructive:** `scout_batch_12/cco02p3`,
  `scout_batch_19/today/ioi14p3`, `scout_batch_19/today/ioi16p5`,
  `scout_batch_52/ioi17p4`, `scout_batch_47/cco07p5`, `scout_batch_52/noi02p2`.

## Averages

- **Real gaps:** 13 across the 30 ports, so `real_gaps / 30` = **0.43**
  (wave 5: 0.57).
- **Awkward lines:** 288 across the 30 ports, so `total_awkward_lines / 30` =
  **9.60** (wave 5: 10.97). About 85% of them already have a clean form on HEAD.
- **WA traps:** **2 confirmed**, both in the toolchain rather than in a port's
  output (below).
- **Cumulative, 180 ports:** 112 real gaps and 1123 awkward lines, so
  **0.62** real gaps and **6.24** awkward lines per problem.

Per-port awkward lines / real gaps: cco16p2 7/0, dpz 2/1, dmopc20c7p2 9/0,
sac22cc3p5 8/2, coci21c6p5 7/0, ccc22s5 16/1, ioi05p2 20/0, ccc01s5 3/1,
cco19p6 6/1, coci21c1p2 12/0, noi07p4 12/0, btoi17p2 14/1, cco16p5 25/1,
cco18p5 12/1, ioi07p3 10/0, dmopc21c7p4 18/0, sac22cc4p4 14/0, noimps22p5 1/0,
noi08p5 2/0, coci25c3p3 11/1, sdoi17r1p4 14/0, noi06p4 6/1, cco03p4 9/0,
noi19p3 8/1, cco02p3 7/0, ioi14p3 16/0, ioi16p5 12/0, ioi17p4 3/0, cco07p5 1/0,
noi02p2 3/1.

## [WA-TRAP] findings (jump the queue, per Audit #11)

### 1. Reversed ranges in range queries are silently empty

The slice-bounds trap (`500ce0e`) covers `List` slices only. Range queries on
data structures still accept a reversed range and quietly do nothing:

```cp
domain P = 0..5
var st = SegTree<P, Int>(+, [P] p => 1)
print(st.fold(P(3)..=P(1)))        // prints 0, no error
var lz = LazySegTree<P, Int, Int>(AddMin, [P] p => 0)
lz.apply(P(3)..=P(1), 7)           // silently a no-op
var f = Fenwick<P, Int>([P] _ => 1)
print(f.sum(P(3)..=P(1)))          // prints 0
```

`cpl run` and the emitted C++ (under ASan/UBSan) agree on `0`, so the
differential fuzzer cannot see it either. This is exactly the bug class that
`noi07p4` has to dodge by hand on a cyclic array: `phys(i)` can land after
`phys(j)`, and the port splits into `l..=Pos.last` + `Pos.first..=r`. A port
that forgets the split gets a silent WA, not a trap.

**Fix:** apply the slice rule to every `Range<D>` handed to `fold`, `apply`,
`sum`, `max_right`/`min_left` and `Prefix*.sum`: `l..r` needs `l <= r` and
`l..=r` needs `l <= r + 1` (so the empty `x..=x-1` stays legal), otherwise trap
with the same message as slices. `for p in l..=r` keeps today's "empty loop"
meaning. Add a `cpl run` vs C++17 differential test per structure.

### 2. `D.first` on an empty domain traps in `cpl run` but not in `cpl build`

```cp
let n = 0
domain G = 0..n
let st = SegTree<G, Int>(+)
print(st.fold(G.first..=G.last))
```

`cpl run` stops with R0001 "`G` has no position 0: it is empty"; the emitted
C++ prints `0`. A stress run under `cpl run` and the submitted binary disagree.
`btoi17p2` pads its domain (`num_gaps = if layers > 1: layers - 1 else: 1`)
to avoid this. **Fix:** emit the same empty-domain check in C++ (one compare,
hoisted out of loops) and add the case to the differential corpus.

## [REAL GAP] findings

### New (not in waves 1–5 or `friction-queue.md`)

1. **No `Str.starts_with` / `ends_with` / `find`.** `ccc01s5` and `cco19p6`
   both define
   `fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head`.
   `s.starts_with("ab")` and `s.find('S')` fail on HEAD (E0203). `cco02p3`
   maps a direction letter with four `if facing == 'E'` lines for lack of
   `find`.
2. **No `Char.lower` / `upper`.** `sac22cc3p5` writes
   `if c.is_upper: Letter.nth(c - 'A') else: Letter(c)`; `c.lower` fails.
3. **No `Trie` in `std.strings`.** `sac22cc3p5` builds one by hand as a
   `List<Node>` with `[Letter]?Int` children, re-deriving the new node's id as
   `trie.len - 1` twice. Trie insert/walk recurs across the string corpus
   (Aho–Corasick ports build on the same node arena).
4. **Empty domains can't host a structure** (the flip side of WA trap 2):
   there is no clean way to say "this `SegTree` may be empty", so ports invent
   a dummy element.

### Recurring (already known; counted again)

- **From-end indexing:** `hull[hull.len - 2]` (`dpz`, wave 2).
- **Domain label crossing:** `init_s[u.to_idx]` on a 1-based `V` (`ccc22s5`),
  and `Node(s.label - 1)` / `Node(n + u.label - 1)` packing two domains into
  one `FlowGraph` node domain (`noi06p4`). Sub-domain / sum-domain embedding is
  still the largest unowned gap.
- **Position-aware `Action.apply`** (wave 5 #4): `cco16p5` hand-rolls a
  14-line covered-length segment tree because a cover-count action needs each
  segment's real width (`ys[hi] - ys[lo]`), not just `len`.
- **`D.window(lo, hi)`** (wave 5 #7): `cco18p5` writes
  `I.nth(max(0, pi - k + 1))..p` and `I.nth(min(n - 1, pi + k - 1))`.
- **Worklist over a growing list** (`coci25c3p3`, `var head = 0; while head < q.len`).
- **Fieldless enums as an index** (`friction-queue.md` P5/W19): `noi19p3`
  encodes 4 states as `Int` codes in a `match s: 0 => .. 1 => ..` and indexes
  `bags: [0..4, 0..3]`. `[S, S]Int` with `enum S` still fails with
  "`S` is a type, and a value is needed here".
- **Guard / loop-bound refinement** (wave 2): `noi02p2` writes `(p + 1)!`
  inside `for p in (..wild).rev`, where `p + 1` is always in range.

### Correction to wave 5

`Range.clamp(D)` already exists: `noi08p5` uses `(x1..=x2).clamp(Grid)` and
`(-10..=1).clamp(V)` prints `-3..2`. Wave 5's "domain clamp" item shrinks to
the scalar `D.clamp(x)` and `D.window(lo, hi)`; `V.clamp(9)` still fails.

## [STALE PORT] patterns (clean form verified on HEAD)

- **Hand-rolled DSU** with `par: [V]Int = -1` and `V(p[cur])` (`ioi14p3`,
  16 lines): `Dsu<V>` has `find`, `unite`, `size`, `same`.
- **Tagged unions as parallel lists** (`ioi05p2`: `tags/ca/cb/cd` padded with
  dummy `0`s; `dmopc21c7p4`: `q_kind/q_arg`): a payload `enum Op` in a
  `List<Op>` with `match` runs on HEAD.
- **Empty-flag monoids** (`noi07p4` `Run.empty`, `cco18p5` `Peak.has` with
  `idx: -1`): `Monoid<?T>` works, and `cco18p5`'s tie-break is plain tuple
  order, so it is `SegTree<I, ?(Int, I)>` with a `max` op.
- **Hand `?Int` helpers** (`ccc22s5` `opt_add`/`opt_min`, 11 lines):
  `min(a, b)` skips `none` and `a + b` propagates it.
- **`Int` bitmasks** (`cco03p4` `dp[Mask(s | b)]`, `(need & s) == need`;
  `ioi16p5` `(a / (1 << bit)) % 2`): `[Subset<V>]Int`, `s.with(v)`, `v in s`,
  `need <= s` and `a.bit(k)` all work.
- **Hand binary searches** over a sorted list or a segtree prefix (`ioi05p2`
  ×2, `cco16p5`, `ioi07p3`, `dmopc21c7p4`, `sdoi17r1p4`): `compress`,
  `bsearch_first`/`bsearch_last`, `max_right`.
- **Monotone deques as index arrays + 4 cursors** (`sac22cc4p4`, 14 lines):
  `Deque<Int>`.
- **`1e9` as flow infinity** (`noi06p4`): `FlowGraph.add(u, v, none)`.
- **Manual fixed-point printing** (`sdoi17r1p4`, 10 lines): `fixed(num, den, 6)`.
- **Flat memo with `-1` sentinel and `Bool` as `0/1`** (`coci21c1p2`):
  `@memo` or `[0..=n, 0..=n, 0..=k]?Bool`.
- **Map relax** `if d < (dist.get(t) ?? Int.MAX): dist[t] = d` (`cco19p6`):
  `if dist[t] min= d:` works on `Map`.
- **`if dp[v] < dp[u] + 1: dp[v] = ..; nxt[v] = u`** (`coci25c3p3`):
  `if dp[v] max= dp[u] + 1: nxt[v] = u`.
- **Fold of an empty prefix guarded by hand**
  (`dmopc21c7p4` `if k <= 0: Node.empty() else: st.fold(..)`):
  `st.fold(S.first..S.first)` already returns the identity.
- **Read-and-push / fill-by-push loops** (`ioi07p3`, `ioi16p5`, `coci21c6p5`),
  **manual swap** (`coci21c6p5`), **`for idx in 0..xs.len: let a = xs[idx]`**
  (`ioi16p5` ×3), **dedupe by hand** (`ioi05p2`, `cco16p5`).
- **Redundant prelude imports** in 12 of the 30 files (`std.heap`,
  `std.graph`, `std.tree`, `std.segtree`, `std.algebra`, `std.lazy`,
  `std.flow`, `std.fenwick`, `std.geometry`). `next_permutation` is the one
  import that is genuinely needed (`std.seq`, not in the prelude).

## [REJECTED]

- A convex-hull-trick / Li Chao structure for `dpz`: algorithm, not a gap.
- `for`-`else` for `noi19p3`'s `ok` flag: a new control-flow form for one port.
- A cyclic `Range` type for `noi07p4`: WA trap 1 makes the wrap-around split
  safe; a special range kind is not worth the concept budget.
- `List.push` returning the new index: `xs.len - 1` is fine once `Trie` exists.

## Proposed Syntax Batch #6 / #7 additions

Ordered by priority. Wave 5's eight items still stand (with the clamp
correction above).

1. **[WA-TRAP] Range-query bounds:** one shared `Range<D>` check (the slice
   rule) on every structure query, plus differential tests.
2. **[WA-TRAP] Empty-domain parity:** emitted C++ traps on `D.first`/`D.last`
   of an empty domain like `cpl run` does; add the case to the differential
   corpus.
3. **`std.strings` prefix API:** `Str.starts_with`, `ends_with`,
   `find(c | s) -> ?Int`, and `Char.lower` / `upper`. Userland, no core change.
4. **`Trie<D>` in `std.strings`:** `insert(word) -> Node`,
   `child(node, c) -> ?Node`, `root`; terminal flags and counters live in user
   arrays indexed by `Node`, so one type serves trie, Aho–Corasick and
   prefix-count ports.
5. **`Monoid.opt(op)`:** builds the `?T` monoid from a semigroup op
   (`none` is the identity), retiring the hand-written empty-flag structs in
   `noi07p4` and `cco18p5`. Three lines in `std.algebra`.
6. **Fieldless enums as `Index`:** promote the queued P5/W19 item; it is the
   cleanest fix for `Int` state codes in flow/DP ports.

## Proposed `friction_lint.py` rules

- `par: [V]Int = -1` / `while p[cur] >= 0` → use `Dsu<V>`.
- Struct fields named `empty: Bool` / `has: Bool` inside a `Monoid<..>` →
  use `Monoid<?T>`.
- Two or more `xs.push(0)` dummies in sibling branches (parallel tag lists) →
  payload `enum`.
- `1e9` / `1e12` / `1e18` as a `FlowGraph.add` capacity → `none`.
- `List.filled((` products used as a flat memo with `-1` → `@memo` or a
  multi-dim domain array.
- `& (1 << `, `| (1 << ` or `(x / (1 << k)) % 2` → `x.bit(k)` / `Subset<D>`.
- A user `fn starts_with` / `fn ends_with` → `Str.starts_with` (after item 3).
