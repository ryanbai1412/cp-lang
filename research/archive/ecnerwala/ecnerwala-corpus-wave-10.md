# ecnerwala Corpus Wave #10 (`179 -> 200` ports — crossed 200+)

Ported **21 submissions** from `ecnerwala/contest-programming` in decreasing recency order immediately after `cf2127b` (`CF/2127/C` through `CF/2101/B`), bringing `proto/ports/ecnerwala/` to **200 total ports**. All 21 pass `proto/ports/ecnerwala/gate.sh NAME 40` (samples + 40 seeded random cases across original C++, ASan/UBSan emitted C++17, and `cpl run`), emit under 64 KB (**max 20.2 KB**, `cf2127h` at 20,701 B), and report **0 `wa_trap`, 0 `real_gap`, 0 `stale_port`** under `scripts/friction_lint.py`.

Aggregate conciseness across Wave #10: **710 CP LOC** vs **2,657 raw C++ LOC (-73.3%)** and **1,165 library-stripped C++ LOC (-39.1%)**. Notably, `cf2127e` uses `Rooted(adj, V(0))` (`tree.tin`, `tree.lca`, `tree.at` preorder) in 32 CP LOC vs 347 C++ LOC (-90.8%), and `cf2127h` exercises `Set.pop_first()`/`Set.pop_last()` (`929c022`) alongside `Map<(Int, Int), List<Int>>` (`if k in mp: mp[k]` single-lookup + salted hash from `63d3ab6`).

## Ported Submissions (21/21 passing `gate.sh NAME 40`)

- `cf2127c` (`CF/2127/C`): 9 CP / 36 raw / 34 stripped, 4,674 B — greedy interval pairing.
- `cf2127d` (`CF/2127/D`): 36 CP / 170 raw / 51 stripped, 8,148 B — caterpillar tree counting (`Mod<1_000_000_007>`).
- `cf2127e` (`CF/2127/E`): 32 CP / 347 raw / 53 stripped, 16,339 B — colored tree LCA via `Rooted` (`tin`, `lca`, `at`, `par`).
- `cf2127f` (`CF/2127/F`): 39 CP / 165 raw / 41 stripped, 12,874 B — inclusion-exclusion combinatorics (`Mod.inv()`).
- `cf2127h` (`CF/2127/H`): 90 CP / 166 raw / 160 stripped, 20,701 B — series-parallel graph reduction (`Set.pop_first`/`pop_last`, `Map<(Int, Int), List<Int>>`, `Deque`).
- `cf2124a` (`CF/2124/A`): 10 CP / 30 raw / 28 stripped, 3,683 B — adjacent inversion check.
- `cf2124b` (`CF/2124/B`): 4 CP / 16 raw / 14 stripped, 3,215 B — prefix-min closed form.
- `cf2124c` (`CF/2124/C`): 8 CP / 22 raw / 20 stripped, 6,003 B — adjacent `gcd`/`lcm` divisibility.
- `cf2124d` (`CF/2124/D`): 32 CP / 49 raw / 47 stripped, 5,264 B — threshold palindrome check.
- `cf2124e` (`CF/2124/E`): 31 CP / 57 raw / 55 stripped, 5,132 B — 1-or-2 balanced split constructive.
- `cf2124f1` (`CF/2124/F1`): 40 CP / 173 raw / 54 stripped, 9,814 B — restricted circular-shift DP (`Mod<998_244_353>`).
- `cf2124g` (`CF/2124/G`): 37 CP / 76 raw / 72 stripped, 7,578 B — prefix-min monotonic suffix gain.
- `cf2124h` (`CF/2124/H`): 22 CP / 43 raw / 41 stripped, 4,419 B — packed triangular O(N^2) DP.
- `cf2124i` (`CF/2124/I`): 47 CP / 71 raw / 69 stripped, 7,053 B — tree-linked-list permutation reconstruction.
- `cf2115a` (`CF/2115/A`): 54 CP / 254 raw / 65 stripped, 12,295 B — Mobius inversion + `std.rand.Rng`.
- `cf2115b` (`CF/2115/B`): 48 CP / 80 raw / 78 stripped, 8,119 B — reverse DAG topological propagation.
- `cf2115c` (`CF/2115/C`): 43 CP / 234 raw / 80 stripped, 9,559 B — 2D `F64` probability DP (`gen/cf2115c.eps` = `1e-6`).
- `cf2115d` (`CF/2115/D`): 26 CP / 49 raw / 47 stripped, 6,652 B — 60-bit XOR linear basis (`bit_width`).
- `cf2115e` (`CF/2115/E`): 62 CP / 247 raw / 91 stripped, 11,174 B — DAG unbounded knapsack + modulo best-ratio DP.
- `cf2101a` (`CF/2101/A`): 14 CP / 187 raw / 29 stripped, 5,094 B — 2D spiral MEX grid (`[0..n, 0..n]Int`).
- `cf2101b` (`CF/2101/B`): 26 CP / 185 raw / 36 stripped, 8,093 B — inversion-parity Fenwick tree (`Fenwick<V, Int>`, `a.swap`).

## Findings and Minimal Repros

### 1. `Set`/`Map` `.first` vs `List` `.last()` property-vs-method diagnostic (`E0203`)
On `List<T>`, reading the last element is a method call (`xs.last()`), while on `Set<T>`/`Map<K, V>` (`929c022`), `.first` and `.last` are zero-arg properties (`s.first`, `s.last`) and `.pop_first()`/`.pop_last()` are methods. Calling `s.first()` produces an unhelpful `E0203` without mentioning the property:

```cp
var s: Set<Int> = Set()
s.insert(1)
let x = s.first()! // error[E0203]: a `Set<Int>` has no method `first` that `cpl run` knows
```

Adding a diagnostic hint (`use s.first without ()`) whenever an `E0203` method call matches a property on the receiver type (or aligning `List.last` / `Set.first`) would remove this cold-writer paper cut.

### 2. Small fixed-size value matrices inside `Map` values (`cf2127h`)
In `cf2127h`, each series-parallel edge carries a 3x3 integer DP matrix (`std::array<std::array<int, 3>, 3>` in C++). Because counted arrays with fixed bounds are heap/table types rather than inline value arrays in structs, storing them in `Map<(Int, Int), ...>` required `List.filled(9, 0)` (9-element flat `List<Int>` per map value). Supporting compact fixed-size value arrays (e.g. `[0..3, 0..3]Int` as a struct field or map value lowered to `std::array`) would eliminate heap allocations and manual `i * 3 + j` indexing.

### 3. Skipped files in recency window
- **Empty 8-line templates** (no solution body): `CF/2127/G2`, `CF/2120/A`–`CF/2120/G`.
- **Deferred to next sweep**: `CF/2127/G1` (interactive permutation query with `mt19937`/`std::shuffle`) and `CF/2124/F2` (349-line lazy segtree large-N version of `CF/2124/F1`).
