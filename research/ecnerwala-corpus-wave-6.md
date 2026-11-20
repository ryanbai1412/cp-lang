# ecnerwala Corpus Wave #6 (`99 -> 119` Ports — Crossed 100+)

Wave #6 expands `proto/ports/ecnerwala/` by **20 recent submissions** in decreasing contest-recency order (`99 -> 119` total, crossing the 100-port milestone), rebased onto `origin/main` (including `98137b4`).

## Summary

- **New ports**: **20** (`cf2180c`, `cf2180d`, `cf2180e`, `cf2180h1`, `cf2180h2`, `cf2178a`, `cf2178b`, `cf2178c`, `cf2178d`, `cf2178f`, `cf2165a`, `cf2165b`, `cf2165c`, `cf2165d`, `cf2165e`, `cf2159b`, `cf2159c`, `cf2159d1`, `cf2159d2`, `cf2157a`).
- **Gate verification**: **20/20 pass** `proto/ports/ecnerwala/gate.sh NAME 40` (`cpl check`, `cpl build` < 64 KB, ASan/UBSan C++17 + `cpl run` interpreter + original C++ differential match across all upstream samples plus 40 seeded random cases each).
- **Friction lint (`python3 scripts/friction_lint.py` on all 20 new ports)**: **0 `wa_trap`**, **0 `real_gap`**, **0 `stale_port`** (0.00 workarounds/file).
- **Emitted C++17 size**: max is **13,032 B (12.7 KB)** (`cf2159c`, well under the 65,536 B judge cap).
- **Conciseness**: **466 CP Lang LOC vs 1,302 C++ LOC** (**64.2% raw LOC reduction**).

## Ported Submissions (Decreasing Recency Order)

1. `cf2180c` (`CF/2180/C`) — 18 CP LOC vs 37 C++ LOC, 5,672 B emitted, 1 sample + 40 random PASS.
2. `cf2180d` (`CF/2180/D`) — 22 CP LOC vs 32 C++ LOC, 3,582 B emitted, 1 sample + 40 random PASS.
3. `cf2180e` (`CF/2180/E`) — 11 CP LOC vs 24 C++ LOC, 3,770 B emitted, 1 sample + 40 random PASS.
4. `cf2180h1` (`CF/2180/H1`) — 25 CP LOC vs 43 C++ LOC, 7,094 B emitted, 1 sample + 40 random PASS.
5. `cf2180h2` (`CF/2180/H2`) — 22 CP LOC vs 31 C++ LOC, 6,502 B emitted, 1 sample + 40 random PASS.
6. `cf2178a` (`CF/2178/A`) — 6 CP LOC vs 16 C++ LOC, 2,777 B emitted, 1 sample + 40 random PASS.
7. `cf2178b` (`CF/2178/B`) — 14 CP LOC vs 23 C++ LOC, 3,162 B emitted, 1 sample + 40 random PASS.
8. `cf2178c` (`CF/2178/C`) — 9 CP LOC vs 23 C++ LOC, 3,032 B emitted, 1 sample + 40 random PASS.
9. `cf2178d` (`CF/2178/D`) — 28 CP LOC vs 48 C++ LOC, 5,570 B emitted, 1 sample + 40 random PASS.
10. `cf2178f` (`CF/2178/F`) — 41 CP LOC vs 167 C++ LOC, 10,434 B emitted, 1 sample + 40 random PASS.
11. `cf2165a` (`CF/2165/A`) — 7 CP LOC vs 18 C++ LOC, 3,969 B emitted, 1 sample + 40 random PASS.
12. `cf2165b` (`CF/2165/B`) — 22 CP LOC vs 154 C++ LOC, 6,986 B emitted, 1 sample + 40 random PASS.
13. `cf2165c` (`CF/2165/C`) — 31 CP LOC vs 49 C++ LOC, 6,894 B emitted, 1 sample + 40 random PASS.
14. `cf2165d` (`CF/2165/D`) — 27 CP LOC vs 49 C++ LOC, 5,487 B emitted, 1 sample + 40 random PASS.
15. `cf2165e` (`CF/2165/E`) — 43 CP LOC vs 70 C++ LOC, 6,472 B emitted, 1 sample + 40 random PASS.
16. `cf2159b` (`CF/2159/B`) — 40 CP LOC vs 217 C++ LOC, 7,433 B emitted, 1 sample + 40 random PASS.
17. `cf2159c` (`CF/2159/C`) — 37 CP LOC vs 181 C++ LOC, 13,032 B emitted, 1 sample + 40 random PASS.
18. `cf2159d1` (`CF/2159/D1`) — 22 CP LOC vs 42 C++ LOC, 7,198 B emitted, 1 sample + 40 random PASS.
19. `cf2159d2` (`CF/2159/D2`) — 32 CP LOC vs 57 C++ LOC, 8,709 B emitted, 1 sample + 40 random PASS.
20. `cf2157a` (`CF/2157/A`) — 9 CP LOC vs 21 C++ LOC, 2,976 B emitted, 1 sample + 40 random PASS.

## Genuine Blockers & Minimal Repros

### 1. `Mod<P>.inv()` method signature bug (`error[E0303]`)

Calling `.inv()` on a `Mod<P>` value fails with an extra-parameter error (`x` missing), while free `inv(x)` is not in scope (`E0201`). Users must write `1 / x` instead.

Minimal repro:

```cp
type Z = Mod<998244353>
let x: Z = 3
print(x.inv())
```

Command: `cpl check repro_mod_inv.cp`
Observed error: `error[E0303]: x.inv takes 1 argument (x), but x is missing`.

### 2. Interactive gate harness missing (`CF/2178/E`, `CF/2159/A`, `CF/2159/F`)

Three more submissions in this window (`CF/2178/E`, `CF/2159/A`, `CF/2159/F`) are interactive problems using `std::flush`. `flush()` already typechecks in CP Lang, so the remaining blocker is adding an interactor mode to `proto/ports/ecnerwala/gate.sh`.

### 3. `lib.math.poly` / polynomial codegen size & floating-point tolerance (`CF/2159/E`, `CF/2180/G`)

`CF/2159/E` relies on polynomial/NTT helpers (blocked by the > 64 KB emitted-size issue logged in Waves #3–#5 for `lib.math.poly`). `CF/2180/G` outputs floating-point probabilities (`double`), where `gate.sh` currently performs exact string diff rather than epsilon-tolerant float comparison.

### 4. Seeded PRNG / `mt19937_64` helper (`CF/2178/G`)

`CF/2178/G` uses `std::mt19937_64` for randomized 64-bit hashes. While prelude provides 2-argument `rand(lo, hi)`, a dedicated deterministic/seeded PRNG type (`Rng` / `splitmix64`) in `stdlib/` is still missing for reproducible hashing and stress-test parity.

## Unfinished Upstream Drafts & Deferred Heavy Submissions (Through `CF/2157/A`)

- **Blocked by interactive interactor harness (Blocker #2)**: `CF/2178/E`, `CF/2159/A`, `CF/2159/F`.
- **Blocked by poly emitted size / float gate tolerance (Blocker #3)**: `CF/2159/E`, `CF/2180/G`.
- **Blocked by seeded 64-bit PRNG (Blocker #4)**: `CF/2178/G`.
- **Deferred larger multi-hundred-line submissions for Wave #7**: `CF/2180/F1`, `CF/2180/F2`, `CF/2178/H`, `CF/2178/I`, `CF/2165/F`, plus `CF/2157/B..F` and `CF/2152/*`.
