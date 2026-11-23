# Perf tail profile #2: root causes of the slowest ports

Base: `main` at `1240c81`, which includes `98137b4` (division checks on in every
build). This is a read-only audit. Nothing in `crates/`, `stdlib/` or
`proto/ports/` changed.

**Method:**
- **Ports:** the 25 slowest from `research/perf/corpus-p95-results.json` (the 308-port
  run).
- **Timing:** each was re-run with
  `python3 bench/corpus_p95_bench.py --ports ... --runs 3 --no-write` at HEAD,
  and again with a `cpl` built at `98137b4^`. Both sides use g++ 11.4,
  `-std=c++17 -O2`, on an 8-core Xeon.
- **Experiments:** for each candidate cause, I hand-edited the emitted C++
  (`port.cpp`) to remove only that cause, then timed the edited and original
  binaries back to back on one pinned core, using the same judge-limit input.
- **Profiles:** gprof, because `perf` is blocked on this VM.

`ioi14p6io` (9.98x before) wasn't re-measured. The harness matched 0 problems
for it under `--ports`, so 24 of the 25 were timed.

## Where the tail stands at HEAD

Over these 24 ports, the runtime ratio against the original C++ is median
4.33x, p90 9.39x, p95 35.73x and max 38.37x. Peak RSS is median 1.90x and max
9.98x. The worst five:

- `dmopc21c8p5`: **38.37x** (144 ms → 5509 ms).
- `shoi16p2`: **35.73x** (16 ms → 588 ms).
- `cco17p3`: **9.39x** (362 ms → 3397 ms), RSS 9.98x (233 MB → 2323 MB).
- `noi04p3`: **6.57x**.
- `ccc20s4`: 6.24x, but only 4 ms → 26 ms.

The corpus targets are p95 under 1.1x and max under 2.0x. They were 3.05x and
36.80x in the 308-port run. Nothing that has landed since has moved the top of
the tail.

## The division checks from 98137b4 cost nothing measurable

**Corpus:** the same 24 ports at `98137b4^` and at HEAD show per-port ratios
that move both ways, within about 10%. For example:

- `dmopc21c8p5`: 38.26x → 38.37x.
- `cco17p3`: 9.37x → 9.39x.
- `noi04p3`: 6.79x → 6.57x.
- `ioi07p3`: 3.62x → 3.50x.
- `cco21p5`: 3.36x → 3.71x.
- `btoi17p2`: 3.46x → 3.77x.

Peak RSS is the same. The medians are 4.18x and 4.33x. The p95 change
(31.80x → 35.73x) comes from `shoi16p2` being timed on a different
`cpl bench` input shape in the two runs, not from the checks.

**Microbenchmark:** 3×10⁸ iterations of `(i*7+s) / b + (i*13) % b` with a
runtime divisor, on one pinned core, alternating builds:

- `divp`: 2288/2286 ms before, 2273/2310 ms after.
- The signed `ediv`/`emod` pair: 2679/2733 ms before, 2668/2707 ms after.

`__builtin_expect(b == 0, 0)` predicts perfectly, and the range checker
already lowers `%` on proven non-negative operands to a native `%` with no
helper. **Keep the checks always on.**

## Root causes, ranked by impact on p95 and max

### 1. `Map` and `@memo` use slow associative containers (sets the max)

`Map<Int, Int>` becomes `std::map`, and `if x in cache: return cache[x]`
becomes `cache.count(x)` followed by `cache.at(x)`. That is two O(log n)
tree walks plus a heap node per insert. `@memo` becomes
`std::unordered_map<std::tuple<...>, ll, cpl::CpHash>`, even when every
argument is a small bounded integer.

- **`dmopc21c8p5` (the max):** the original uses an open-addressing table
  whose `lookup` returns `long&`. Changing only `std::map<ll, ll>` to
  `std::unordered_map<ll, ll>` in the emitted C++ takes it from **5467 ms to
  213 ms** (25.7x faster). Against the original's 144 ms, the ratio goes from
  38x to **1.48x**.
- **Same pattern:** `ioi09p4` (5.18x, `@memo` on a 4-int tuple),
  `ccc18s4` (2.93x), `dpn` (3.42x) and `ioi04p4` (`@memo` on 2-tuples).

**Fix (`cpl-codegen` + `stdlib`):**
- Lower `Map<K, V>` to a flat open-addressing hash with a single-probe
  `get_or_insert` that returns a reference.
- Fuse `x in m` followed by `m[x]` into one probe.
- Lower `@memo` to a dense table when the range checker bounds every
  argument, and to the flat hash otherwise.

### 2. Small local lists are heap-allocated on every call (second worst port)

A `List` built and dropped inside a function or loop body becomes a fresh
`std::vector` each time. Fixed literals do the same: `for d in [(0,1), ...]`
still heap-allocates per iteration (`bench/repros/perf_03` is still open).

- **`shoi16p2`:** `square()` builds `xs`, `ys` and `codes`, three 4-element
  vectors, for about 6M calls. The original uses
  `std::array<pair<int,int>, 4>`. Turning only those three into reused static
  buffers takes it from **560 ms to 52 ms** (10.8x), with identical output.
  The ratio drops from 36x to about 3.2x.
- **Same pattern:** `ioi09p6` (`around(r, c)` returns a vector per call),
  `cco17p3` (`along(xs, r)`), `dmopc21c4p2` (`factor(x)` and `divisors(x)`
  per element, 5.14x), and `ioi10p3` (closure-built `List<Range>`
  temporaries).

**Fix (`cpl-codegen`):**
- Escape analysis for local lists that never leave their scope.
- Emit `std::array` for literals of known length.
- Emit a `cpl::SmallVec<T, N>` with inline capacity, or hoist and `clear()`,
  for lists whose length is bounded.
- For helpers like `around`/`divisors`, a `yields` generator form or a
  caller-provided buffer avoids returning a vector.

### 3. Layout: fixed arrays in structs, unpacked `?Int`, 64-bit cells (RSS tail and mid-tail)

- **`[0..K]T` struct field:** this becomes a heap `std::vector`
  (`perf_04` is still open). In `cco17p3`, `Node.kid: [0..2]?Int` is a
  `std::vector<std::optional<ll>>` per trie node. Changing only that field to
  `std::array<std::optional<ll>, 2>` takes it from **3.07 s / 2378 MB to
  1.86 s / 1868 MB**, with identical output.
- **`?Int` packing:** `cpl::OptInt` niche packing exists, but none of the 24
  ports use it. `yac6p6` has `cpl::Grid<std::optional<ll>, 3>` tables, and
  gprof puts about 30% of its time in `optional::operator*`/`has_value` and
  grid indexing. `cco17p3`'s remaining gap is mostly its 16-byte
  `optional<ll>` children.
- **64-bit cells:** `coci21c1p2` keeps its DP in a `std::vector<ll>` where
  the original uses `bool dp[n][n][k+1]` (RSS 6.85x). `cco22p4` has a dense
  `[V, K]Int` memo of 32 MB against the original's 3.5 MB (RSS 9.57x).

**Fix (`cpl-codegen`):**
- `std::array<T, K>` for constant-size domain arrays, inside structs and as
  locals.
- Apply `OptInt` to every `?Int` storage location (`Grid`, struct fields,
  locals), not only to flat `[D]?Int` tables.
- Choose the cell width from the proven value range (`Bool` as 1 byte,
  `0..=2` as `int8_t`), or offer an explicit width knob.

### 4. Bounds checks on stdlib-internal indices (mid-tail)

`cpl::at` checks bounds in every build. That is right for user indexing, but
the stdlib's own internal heap indices pay for it too. The emitted
`LazySegTree` does `cpl::at(t, 2 * p)`, `cpl::at(lz, p)` and so on on every
node visit.

Making `cpl::at` unchecked (as an experiment only) gives:
- **`ioi07p3`:** 402 ms → **210 ms** (1.9x).
- **`cco21p5`:** 246 ms → 204 ms.
- **`ccc20s4`:** 23 ms → 16 ms.
- Within noise on `shoi16p2`, `cco17p3`, `cco23p2`, `noi04p3` and
  `dmopc21c8p5`.

So this is a mid-tail cause, not a max cause. `std::function` callbacks still
appear in the same trees (`ioi07p3`, `cco21p5`, `btoi17p2`, `ioi10p3`,
`perf_05` is still open), but they weren't measured separately.

**Fix (`stdlib` + `cpl-codegen`), staying zero-UB:** index stdlib internals
through `a.raw[i]` guarded by the structure's own invariant (`p < 2 * size`).
Alternatively, have the range checker drop the check when it proves the
index is in range. User indexing keeps its check.

### 5. Not codegen: the port's algorithm

- **`cco05p6` (5.77x):** the original scans windows from the top and breaks
  at the first hit, and moves a window with `push_back`. The port scans the
  whole stack keeping the last hit, then shifts in O(n). The fix belongs in
  the port.
- **`noi04p3` (6.57x):** gprof shows `encrypt()` called 2×10⁸ times (36%)
  and 4×10⁷ `push_back`s. Unchecked indexing doesn't help. It isn't yet
  root-caused against the original, so I've left it unattributed.

## Expected effect

- **Cause 1** alone takes the max from 38.37x to about 1.5x on
  `dmopc21c8p5`.
- **Cause 2** takes `shoi16p2` from 36x to about 3x.
- **Cause 3** takes `cco17p3` from 9.4x to about 5x with the array change
  alone, and further with `OptInt`.
- **After 1–3**, the new max would be in the 3–5x range (`cco17p3`,
  `shoi16p2`, `noi04p3`). Getting under 2.0x then needs cause 4 and the
  niche and width work in cause 3.
- **The p95 target** depends on the whole 308-port distribution, which this
  run didn't re-measure. 8 of the 24 tail ports name cause 1 or 2 as their
  largest single factor.

## Not checked

- Missing `std::move` on comprehension assignment and `cpl::last` copies:
  the vector temporaries seen here are function results, which are elided,
  not copies. A dedicated scan wasn't done.
- `Mod<P>` arithmetic: none of the 24 ports is `Mod`-bound.
- `ioi14p6io`: not re-measured.
- All experiments edit the emitted C++ by hand. They measure the size of each
  cause, not a compiler implementation.
