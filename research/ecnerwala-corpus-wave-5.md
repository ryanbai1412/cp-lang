# ecnerwala Corpus Wave #5 (`79 -> 99` Ports)

Wave #5 expands `proto/ports/ecnerwala/` by **20 recent submissions** in decreasing recency order (`79 -> 99` total), rebased onto `origin/main` (including Batch #5 `a7e575b`).

## Summary

- **New ports**: **20** (`ucup4s25e`, `cf2207e1`, `agc077a`, `cf2201a1`, `cf2201a2`, `cf2201b`, `cf2201c`, `cf2201f1`, `cf2187a`, `cf2187b`, `cf2187c`, `cf2187e`, `cf2187g`, `cf2183a`, `cf2183b`, `cf2183c`, `cf2183d1`, `cf2183d2`, `cf2180a`, `cf2180b`).
- **Gate verification**: **20/20 pass** `proto/ports/ecnerwala/gate.sh NAME 40` (`cpl check`, `cpl build` < 64 KB, ASan/UBSan C++17 + `cpl run` interpreter + original C++ differential match across all upstream samples plus 40 seeded random cases each).
- **Friction lint (`python3 scripts/friction_lint.py` on all 20 new ports)**: **0 `wa_trap`**, **0 `real_gap`**, **0 `stale_port`** (0.00 workarounds/file).
- **Emitted C++17 size**: max is **13,943 B (13.6 KB)** (`agc077a`, well under the 65,536 B judge cap).
- **Conciseness**: **593 CP Lang LOC vs 1,142 C++ LOC** (**48.1% raw LOC reduction**).

## Ported Submissions (Decreasing Recency Order)

1. `ucup4s25e` (`ACM/UCup4/Stage25-Zagreb/E`) — 71 CP LOC vs 172 C++ LOC, 12,685 B emitted, 40 random PASS (upstream has no `.sample.in`).
2. `cf2207e1` (`CF/2207/E1`) — 28 CP LOC vs 52 C++ LOC, 5,016 B emitted, 1 sample + 40 random PASS.
3. `agc077a` (`AtCoder/AGC/077/A`) — 86 CP LOC vs 164 C++ LOC, 13,943 B emitted, 1 sample + 40 random PASS.
4. `cf2201a1` (`CF/2201/A1`) — 23 CP LOC vs 38 C++ LOC, 6,415 B emitted, 1 sample + 40 random PASS.
5. `cf2201a2` (`CF/2201/A2`) — 23 CP LOC vs 38 C++ LOC, 6,720 B emitted, 1 sample + 40 random PASS.
6. `cf2201b` (`CF/2201/B`) — 27 CP LOC vs 41 C++ LOC, 4,329 B emitted, 1 sample + 40 random PASS.
7. `cf2201c` (`CF/2201/C`) — 27 CP LOC vs 155 C++ LOC, 6,939 B emitted, 1 sample + 40 random PASS.
8. `cf2201f1` (`CF/2201/F1`) — 40 CP LOC vs 47 C++ LOC, 11,052 B emitted, 1 sample + 40 random PASS.
9. `cf2187a` (`CF/2187/A`) — 15 CP LOC vs 26 C++ LOC, 5,238 B emitted, 1 sample + 40 random PASS.
10. `cf2187b` (`CF/2187/B`) — 41 CP LOC vs 60 C++ LOC, 10,968 B emitted, 1 sample + 40 random PASS.
11. `cf2187c` (`CF/2187/C`) — 31 CP LOC vs 46 C++ LOC, 6,229 B emitted, 1 sample + 40 random PASS.
12. `cf2187e` (`CF/2187/E`) — 24 CP LOC vs 39 C++ LOC, 6,086 B emitted, 1 sample + 40 random PASS.
13. `cf2187g` (`CF/2187/G`) — 29 CP LOC vs 30 C++ LOC, 7,101 B emitted, 1 sample + 40 random PASS.
14. `cf2183a` (`CF/2183/A`) — 5 CP LOC vs 24 C++ LOC, 2,672 B emitted, 1 sample + 40 random PASS.
15. `cf2183b` (`CF/2183/B`) — 9 CP LOC vs 20 C++ LOC, 3,173 B emitted, 1 sample + 40 random PASS.
16. `cf2183c` (`CF/2183/C`) — 9 CP LOC vs 26 C++ LOC, 3,481 B emitted, 1 sample + 40 random PASS.
17. `cf2183d1` (`CF/2183/D1`) — 27 CP LOC vs 41 C++ LOC, 6,512 B emitted, 1 sample + 40 random PASS.
18. `cf2183d2` (`CF/2183/D2`) — 63 CP LOC vs 89 C++ LOC, 11,513 B emitted, 1 sample + 40 random PASS.
19. `cf2180a` (`CF/2180/A`) — 5 CP LOC vs 15 C++ LOC, 4,414 B emitted, 1 sample + 40 random PASS.
20. `cf2180b` (`CF/2180/B`) — 10 CP LOC vs 19 C++ LOC, 2,981 B emitted, 1 sample + 40 random PASS.

## Genuine Blockers & Minimal Repros

### 1. Missing ordered set / ordered map (`BTreeSet<T>` / `BTreeMap<K, V>`)

Wave #5 encountered three more recent submissions requiring ordered `std::set` / `std::map` queries (`lower_bound`, predecessor/successor traversal, and arbitrary-key erase): `AGC/077/C` (`std::set` predecessor/successor), `AGC/078/A` (`std::map` ordered iteration + `lower_bound`), and `CF/2201/D` (`std::set` maintaining contiguous equal-value intervals alongside a segment tree).

Minimal repro:

```cp
var s: BTreeSet<Int> = BTreeSet()
s.insert(1)
```

Command: `cpl check repro_btreeset.cp`
Observed error: `error[E0204]: no type or domain named BTreeSet` and `error[E0201]: no BTreeSet in scope`.

Why it matters: Ordered set/map is now the single highest-frequency stdlib gap blocking recent ecnerwala submissions (`CF/2222/E`, `CF/2219/C`, `AGC/077/C`, `AGC/078/A`, `CF/2201/D`).

### 2. Missing stdlib PRNG / randomized hashing helper (`std.rand` / `mt19937_64` equivalent)

`CF/2201/F1` originally used `std::mt19937_64` to generate random 64-bit hashes for Zobrist/multiset hashing. While `U64` and `I128` both typecheck and run cleanly in CP Lang, `stdlib/` has no PRNG or hash-mixing module (`std.rand`), so `cf2201f1.cp` had to write an explicit double modular polynomial mixer (`Mod<998244353>` + `Mod<1_000_000_007>`) instead of sampling random 64-bit keys. A small `std.rand` (`Rng` / `splitmix64`) module would remove boilerplate on every randomized-hash and treap problem.

### 3. `lib.math.poly` FPS/NTT emitted size still exceeds 64 KB (`70,416 B > 65,536 B`)

Re-verified on `a7e575b`: `CF/2255/F` still emits **70,416 bytes** (`4,880 B` over the 65,536 B judge cap) when importing `compose_trunc`, `convolve`, `pow_trunc`, and `prod_linear` from `lib.math.poly`. This also continues to block the heavy NTT/polynomial submissions encountered in Wave #5 (`CF/2201/E`, `CF/2187/D`).

### 4. Interactive gate harness missing (`CF/2183/G`)

`flush()` already typechecks in CP Lang (`cpl check` passes on `print(1); flush()`), so the language side is ready, but `proto/ports/ecnerwala/gate.sh` only supports batch stdin/stdout diffing and has no interactor driver for interactive problems (`CF/2222/E`, `CF/2183/G`).

## Unfinished Upstream Drafts & Deferred Submissions (Through `CF/2180/B`)

- **Unfinished/empty upstream drafts (skipped)**: `AGC/077/B`, `AGC/077/E`, `AGC/078/B`, `CF/2201/G`, `CF/2183/I1`, `CF/2183/I2`.
- **Blocked by missing ordered set/map (Blocker #1)**: `AGC/077/C`, `AGC/078/A`, `CF/2201/D`.
- **Blocked by NTT/poly emitted size > 64 KB (Blocker #3)**: `CF/2255/F`, `CF/2201/E`, `CF/2187/D`.
- **Blocked by interactive interactor harness (Blocker #4)**: `CF/2183/G`.
- **Deferred larger/heavier submissions for Wave #6**: `AGC/077/D`, `AGC/077/F`, `AGC/078/C`, `CF/2187/F1`, `CF/2187/F2`, `CF/2183/E`, `CF/2183/F`, `CF/2183/H`, `CF/2180/C`.
