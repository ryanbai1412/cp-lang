# ecnerwala Corpus Wave #7 (`119 -> 139` Ports + Interactive & Float-Eps Gate)

Wave #7 adds lightweight **interactive interactor support** and **epsilon-tolerant floating-point diffing** to `proto/ports/ecnerwala/gate.sh` and expands `proto/ports/ecnerwala/` by **20 more submissions** (`119 -> 139` total), starting with newly unlocked `CF/2255/F` (`e1495d3`), `CF/2180/G`, and four interactive problems (`CF/2178/E`, `CF/2159/A`, `CF/2159/F`, `CF/2152/E`), then continuing in decreasing contest-recency order.

## Summary

- **Harness upgrades (`proto/ports/ecnerwala/gate.sh` + `interactors/_lib.py`)**:
  - **Interactive mode**: when `interactors/<name>.py` exists, `gate.sh` runs both the original C++ binary and the ASan/UBSan emitted C++17 binary live over stdin/stdout pipes against the interactor across `samples/<name>.*.case` and 40 seeded random cases, records the exact interactive input/output transcript of the sanitizer run, and replays that transcript through `cpl run` to verify interpreter parity.
  - **Epsilon float diffing**: when `gen/<name>.eps` exists (or `FLOAT_EPS` is set in the environment), `gate.sh` compares whitespace-separated tokens with relative/absolute tolerance `abs(a - b) <= eps * max(1.0, abs(a), abs(b))` for finite numeric tokens and exact match for non-numeric tokens.
- **New ports**: **20** (`cf2255f`, `cf2180g`, `cf2178e`, `cf2159a`, `cf2159f`, `cf2152e`, `cf2157b`, `cf2157c`, `cf2157d`, `cf2157e`, `cf2157f`, `cf2152a`, `cf2152b`, `cf2152c`, `cf2152d`, `cf2152f`, `cf2152h1`, `cf2152h2`, `cf2150b`, `cf2150d`).
- **Gate verification**: **20/20 pass** `proto/ports/ecnerwala/gate.sh NAME 40` (`cpl check`, `cpl build` < 64 KB, ASan/UBSan C++17 + `cpl run` interpreter + original C++ verification across all samples plus 40 seeded random cases each).
- **Friction lint (`python3 scripts/friction_lint.py` on all 20 new ports)**: **0 `wa_trap`**, **0 `real_gap`**, **0 `stale_port`** (0.00 workarounds/file).
- **Emitted C++17 size**: max is **65,372 B (63.8 KB)** (`cf2255f`, under the 65,536 B judge cap; next largest is `cf2150d` at **16,704 B / 16.3 KB**).
- **Conciseness**: **585 CP Lang LOC vs 3,293 C++ LOC** (**82.2% raw LOC reduction**; excluding `cf2255f`'s 1,949-line C++ polynomial template, **565 CP LOC vs 1,344 C++ LOC**, a **58.0% library-stripped reduction**).

## Ported Submissions

1. `cf2255f` (`CF/2255/F`) — 20 CP LOC vs 1,949 C++ LOC, 65,372 B emitted, 1 sample + 40 random PASS.
2. `cf2180g` (`CF/2180/G`) — 42 CP LOC vs 179 C++ LOC, 13,318 B emitted, 1 sample + 40 random PASS.
3. `cf2178e` (`CF/2178/E`, interactive) — 28 CP LOC vs 54 C++ LOC, 4,310 B emitted, 1 sample + 40 random PASS.
4. `cf2159a` (`CF/2159/A`, interactive) — 23 CP LOC vs 42 C++ LOC, 4,437 B emitted, 1 sample + 40 random PASS.
5. `cf2159f` (`CF/2159/F`, interactive) — 49 CP LOC vs 96 C++ LOC, 10,997 B emitted, 1 sample + 40 random PASS.
6. `cf2152e` (`CF/2152/E`, interactive) — 27 CP LOC vs 53 C++ LOC, 4,963 B emitted, 1 sample + 40 random PASS.
7. `cf2157b` (`CF/2157/B`) — 11 CP LOC vs 26 C++ LOC, 3,331 B emitted, 1 sample + 40 random PASS.
8. `cf2157c` (`CF/2157/C`) — 20 CP LOC vs 37 C++ LOC, 4,980 B emitted, 1 sample + 40 random PASS.
9. `cf2157d` (`CF/2157/D`) — 11 CP LOC vs 24 C++ LOC, 3,212 B emitted, 1 sample + 40 random PASS.
10. `cf2157e` (`CF/2157/E`) — 14 CP LOC vs 29 C++ LOC, 4,360 B emitted, 1 sample + 40 random PASS.
11. `cf2157f` (`CF/2157/F`) — 53 CP LOC vs 88 C++ LOC, 7,612 B emitted, 2 samples + 40 random PASS.
12. `cf2152a` (`CF/2152/A`) — 6 CP LOC vs 18 C++ LOC, 3,858 B emitted, 1 sample + 40 random PASS.
13. `cf2152b` (`CF/2152/B`) — 5 CP LOC vs 27 C++ LOC, 3,050 B emitted, 1 sample + 40 random PASS.
14. `cf2152c` (`CF/2152/C`) — 17 CP LOC vs 32 C++ LOC, 4,374 B emitted, 1 sample + 40 random PASS.
15. `cf2152d` (`CF/2152/D`) — 12 CP LOC vs 29 C++ LOC, 4,502 B emitted, 1 sample + 40 random PASS.
16. `cf2152f` (`CF/2152/F`) — 67 CP LOC vs 112 C++ LOC, 7,794 B emitted, 1 sample + 40 random PASS.
17. `cf2152h1` (`CF/2152/H1`) — 25 CP LOC vs 39 C++ LOC, 5,414 B emitted, 1 sample + 40 random PASS.
18. `cf2152h2` (`CF/2152/H2`) — 90 CP LOC vs 121 C++ LOC, 16,446 B emitted, 1 sample + 40 random PASS.
19. `cf2150b` (`CF/2150/B`) — 22 CP LOC vs 156 C++ LOC, 8,466 B emitted, 1 sample + 40 random PASS.
20. `cf2150d` (`CF/2150/D`) — 43 CP LOC vs 182 C++ LOC, 16,704 B emitted, 1 sample + 40 random PASS.

## Genuine Blockers & Minimal Repros

### 1. `cpl run` interpreter buffers stdin to EOF (blocks live interactive debugging)

Emitted C++17 handles interactive problems cleanly (`flush()` lowers to `fflush(stdout)` and responds immediately over pipes), but `cpl run` in `crates/cpl-cli/src/main.rs` calls `read_to_string(&mut input)` on stdin before invoking the interpreter. Any live interactor connected to `cpl run` deadlocks waiting for the first query.

Minimal repro:

```cp
let n: Int = read()
print("?", n)
flush()
let r: Int = read()
print("!", r)
flush()
```

Sending `2\n` to a pipe without closing stdin yields `? 2` immediately on the compiled C++ binary, whereas `cpl run` hangs until EOF. Streaming stdin lazily inside the interpreter (reading tokens on demand and flushing stdout on `flush()`) would unlock live interactive `cpl run` and remove the transcript-replay fallback in `gate.sh`.

### 2. Tight `lib.math.poly` emitted-size headroom (`CF/2255/F`)

`e1495d3` brought `CF/2255/F` under the 65,536 B judge cap: using `prod_linear` with `Comb<MOD>` emits **65,372 B** (164 B of headroom). Using `prod_polys([[Z(1), a[i]] for i in 0..n])` with `Comb<MOD>` still emits **66,697 B** (1,161 B over the cap). Stronger dead-function pruning in `cpl build` or further `lib.math.poly` size trimming is still needed so multi-helper FPS/NTT submissions (`CF/2159/E`, `CF/2201/E`, `CF/2187/D`) have comfortable headroom under 64 KB.

### 3. `List<T>` binary search ergonomics (`CF/2152/H2`)

`deltas.lower_bound(t)` fails with `error[E0203]: a List<Int> has no method lower_bound` (`lower_bound` currently exists only on `Fenwick`). Writing `bsearch_first(0..n, i => deltas[i] >= t) ?? n` works cleanly and passes friction lint, though `List<T>.lower_bound(x)` / `upper_bound(x)` on sorted `List`s would match standard CP expectations.

## Notes on Remaining Unported Submissions

- `CF/2180/G` turned out to be a modular-arithmetic (`Mod<1000000007>`) problem rather than a floating-point problem, so `cf2180g` passed exact diff; `gate.sh`'s epsilon float comparator was independently verified via unit checks and `FLOAT_EPS=1e-6`.
- `CF/2150/A` (`std::set<int>`), `CF/2150/C` (lazy segment tree), `CF/2150/E1..E2` (interactive), `CF/2152/G` (heavy-light / segment tree), and `CF/2159/E` (heavy FPS) remain queued for subsequent waves once `BTreeSet`/`BTreeMap` and further `poly` size headroom land.
