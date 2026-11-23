# ecnerwala Corpus Wave #8 (`139 -> 159`)

Wave #8 sweeps back through the newly unblocked submissions from Waves #4–#6 (interactive, `std.rand.Rng`, ordered `Map` / `Set`, epsilon float DP, and `Mod<P>.inv()`) and continues in decreasing recency order across `https://github.com/ecnerwala/contest-programming` to add **20 new verified ports** (`139 -> 159` total, crossing 150+).

## Summary

- **New ports**: 20 (`139 -> 159` total in `proto/ports/ecnerwala/`):
  - **6 sweep-backs** from Waves #4–#6: `cf2222e` (interactive), `cf2219c` (float tree DP), `agc077c` (ordered `Map` complement-graph BFS), `cf2201d` (ordered `Map` + `SegTree`), `cf2183g` (interactive), `cf2178g` (`std.rand.Rng` + Fenwick XOR hash).
  - **14 new submissions** in decreasing recency order: `cf2150a`, `cf2150c` (ordered `Map` DP), `cf2150e1`, `cf2150e2` (interactive + `std.rand.Rng.shuffle`), `cf2147a`, `cf2147b`, `cf2147c`, `cf2147d`, `cf2147e`, `cf2147g` (`Mod<998244353>.inv()`), `cf2138a`, `cf2138b`, `cf2138c1`, `cf2138d`.
- **Gate verification**: **20/20 passed** `proto/ports/ecnerwala/gate.sh NAME 40` (samples + 40 seeded random cases, ASan/UBSan C++17 + `cpl run` interpreter + differential match against ecnerwala's original C++; live interactive pipes + `cpl run` transcript replay for the 4 interactive ports `cf2222e`, `cf2183g`, `cf2150e1`, `cf2150e2`; epsilon float comparator `1e-6` for `cf2219c`).
- **Friction lint**: **0 findings** across all 20 ports (`0 wa_trap`, `0 real_gap`, `0 stale_port`).
- **Emitted C++17 size**: max **17.6 KB** (`cf2201d`, 18,004 B), well under the 64 KB (65,536 B) judge cap.
- **LOC**: **649 CP LOC** vs **1,787 raw C++ LOC** (**-63.7%** raw) and **1,072 library-stripped C++ LOC** (**-39.5%** stripped).

## Ports

| Port | Upstream | Category / Features Exercised | CP LOC | Orig LOC (Raw / Stripped) | Emitted C++17 | Gate (`40`) |
| --- | --- | --- | ---: | ---: | ---: | --- |
| `cf2222e` | `CF/2222/E` | Interactive (`flush()` + interactor) | 36 | 99 / 97 | 5,278 B (5.2 KB) | PASS (1 + 40) |
| `cf2219c` | `CF/2219/C` | Tree DP, `F64`, `gen/cf2219c.eps` (`1e-6`) | 35 | 50 / 48 | 6,502 B (6.3 KB) | PASS (1 + 40) |
| `agc077c` | `AGC/077/C` | Ordered `Map` (`first_ge`/`first_gt`), BFS | 47 | 82 / 80 | 12,823 B (12.5 KB) | PASS (1 + 40) |
| `cf2201d` | `CF/2201/D` | Ordered `Map` + `SegTree<V, SegNode>` | 51 | 383 / 93 | 18,004 B (17.6 KB) | PASS (1 + 40) |
| `cf2183g` | `CF/2183/G` | Interactive (`flush()` + interactor) | 41 | 68 / 66 | 8,812 B (8.6 KB) | PASS (1 + 40) |
| `cf2178g` | `CF/2178/G` | `std.rand.Rng` (`next()`), `U64` Fenwick | 36 | 183 / 33 | 6,160 B (6.0 KB) | PASS (1 + 40) |
| `cf2150a` | `CF/2150/A` | Ordered `Map` membership (`in`) | 18 | 32 / 30 | 4,460 B (4.4 KB) | PASS (1 + 40) |
| `cf2150c` | `CF/2150/C` | Ordered `Map` (`first_gt`, `last_le`) | 33 | 49 / 47 | 9,600 B (9.4 KB) | PASS (1 + 40) |
| `cf2150e1` | `CF/2150/E1` | Interactive + `std.rand.Rng` (`shuffle`) | 52 | 91 / 89 | 10,004 B (9.8 KB) | PASS (1 + 40) |
| `cf2150e2` | `CF/2150/E2` | Interactive + `std.rand.Rng` (`shuffle`) | 61 | 93 / 91 | 11,055 B (10.8 KB) | PASS (1 + 40) |
| `cf2147a` | `CF/2147/A` | Case analysis | 8 | 21 / 19 | 2,844 B (2.8 KB) | PASS (1 + 40) |
| `cf2147b` | `CF/2147/B` | Constructive | 7 | 21 / 19 | 3,019 B (2.9 KB) | PASS (1 + 40) |
| `cf2147c` | `CF/2147/C` | Bitmask DP | 12 | 24 / 22 | 3,353 B (3.3 KB) | PASS (1 + 40) |
| `cf2147d` | `CF/2147/D` | Frequency `Map`, greedy sort | 20 | 30 / 28 | 4,886 B (4.8 KB) | PASS (1 + 40) |
| `cf2147e` | `CF/2147/E` | Bitwise greedy + `bsearch_first` | 32 | 47 / 45 | 7,083 B (6.9 KB) | PASS (1 + 40) |
| `cf2147g` | `CF/2147/G` | Prime factorization + `Mod<998244353>` (`inv()`) | 41 | 191 / 67 | 15,005 B (14.7 KB) | PASS (1 + 40) |
| `cf2138a` | `CF/2138/A` | Constructive reverse trace | 17 | 31 / 29 | 3,785 B (3.7 KB) | PASS (1 + 40) |
| `cf2138b` | `CF/2138/B` | Monotonic stacks + prefix min | 23 | 45 / 43 | 6,051 B (5.9 KB) | PASS (1 + 40) |
| `cf2138c1` | `CF/2138/C1` | Tree depths + subset-sum DP (`[0..m]Bool`) | 42 | 65 / 63 | 6,042 B (5.9 KB) | PASS (1 + 40) |
| `cf2138d` | `CF/2138/D` | Combinatorics + `Mod<1_000_000_007>` | 37 | 182 / 63 | 13,413 B (13.1 KB) | PASS (1 + 40) |
| **Total** | | | **649** | **1,787 / 1,072** | **max 17.6 KB** | **20/20 PASS** |

## Friction & Compiler/Stdlib Findings (with Minimal Repros)

1. **`SegTree<D, T>.fold(..)` open full-domain range rejected (`E0501`)** (hit in `cf2201d`):
   Calling `seg.fold(..)` over a named domain `domain V = 0..n` fails with:
   ```text
   error[E0501]: `cpl run` and `cpl build` can't handle a range with neither end yet
   ```
   Minimal repro:
   ```cp
   domain V = 0..n
   var seg = SegTree<V, Int>(Monoid(0, (a, b) => a + b))
   let total = seg.fold(..) // E0501; must write seg.fold(V(0)..)
   ```
   A full-domain `..` range over `V` has known bounds `V(0)..V(n)` and should lower directly to querying the entire tree.

2. **`SegTree<0..n, T>` inline domain argument triggers internal `D` / `ord` / `to_idx` checker leak** (hit in `cf2201d`):
   Writing `SegTree<0..n, Int>` instead of introducing `domain V = 0..n` fails with leaky internal errors (`a Int has no member ord`, `cpl run doesn't know the name D here`). Either allow inline `0..n` as a domain type argument or emit a clear single diagnostic directing to `domain V = 0..n`.

3. **Ordered `Map<K, V>` / `Set<K>` lack `.first()` / `.last()` (or `.min_key()` / `.max_key()`)** (hit in `cf2201d`, `agc077c`, `cf2150c`):
   Getting the minimum or maximum key of a non-empty ordered `Map` or `Set` currently requires passing an explicit lower/upper sentinel bound (`m.first_ge(0)!.0`, `m.last_le(n)!.0`). Adding `first(self)` / `last(self)` (returning `?(K, V)` on `Map` and `?K` on `Set`) avoids sentinel boilerplate and removes a class of off-by-one/negative-key bugs when keys can be arbitrary `Int`s.

4. **Remaining unported backlog for Wave #9**:
   - `AGC/078/A` (now unblocked by ordered `Set`/`Map`, deferred only by the 20-port wave cap).
   - `CF/2201/E`, `CF/2187/D`, `CF/2159/E` (`lib.math.poly` / NTT size headroom near 64 KB).
   - `CF/2150/F`, `CF/2138/C2` (needs `Bitset` > 64-bit dynamic shift/or for `N <= 2e5` subset sum), and `CF/2138/E*` onward.
