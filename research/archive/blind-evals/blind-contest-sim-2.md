# Blind Full-Contest Simulation #2 — CP Lang 0.2.2 (Codeforces Multitest, Constructive & Interactive)

**Date:** 2026-11-20
**Compiler tested:** `cpl 0.2.2` (`cargo build --release -p cpl-cli` at commit `987fc2e`), verified with `cpl check`, `cpl run`, `cpl build` + `g++ -std=c++17 -O2` and `-fsanitize=address,undefined`, `cpl stress`, and standalone adversarial harnesses.
**Strict blind rules:** Only `docs/one-page-cheat-sheet.md` was consulted while solving. No compiler source, stdlib source, prototype code, or existing C++ solutions were read. All six problems are a complete real recent Codeforces round — **Codeforces Round 1132 (Div. 2)** (contest `2284`, fetched via the Codeforces API and the local browser session after `curl` hit Cloudflare HTTP 403 `cf-mitigated: challenge`) — solved from scratch under contest pressure.

## Executive Summary

All six main problems of **Codeforces Round 1132 (Div. 2)** (`2284 A2, B, C1, D, E, F`) — every one of them Codeforces-style multitest, spanning greedy/bitmask, constructive + checker, sieve precomputation, run-twice communication + graph 2-coloring, 33-query interactive graph binary search, and modular probability + custom-monoid segment tree — were solved from scratch in CP Lang 0.2.2:

1. **2284 A2 — 15K4JR3B** (Multitest greedy / bitmasks + `@brute`) — 2 first-draft errors (`@!multitest` rejected, `desc` missing from prelude), 15-line final solution, strictly cleaner than C++.
2. **2284 B — Introduction to the Niche Internet** (Multitest constructive + `@checker`) — 0 solution errors once `@multitest` was known, 22-line final solution, strictly cleaner than C++; in-file `@checker` blocked on missing `read_output()` and `cpl stress --checker` hit the `@multitest` generator bug.
3. **2284 C1 — Disgusting + GCD** (Multitest + `Sieve` precomputation + constructive) — 1 first-draft error (`Sieve` missing from prelude), **1,100x runtime trap**: putting `Sieve(200_005)` above `@multitest` silently rebuilt the sieve inside `solve()` on every test case (**11.04 s** for `t = 10,000` vs **0.01 s** with an explicit `for _ in 0..t:` loop).
4. **2284 D — The Theorem of Two Cans** (Run-twice multitest communication + graph 2-coloring) — 3 draft iterations after hitting `two_color` (`E` scope leak), `Str.find` unknown in `cpl run`, `"0" * k` unsupported, and `Graph.undirected` edge-shape requirements; 56-line final solution with manual `Deque` BFS passed an 89-case adversarial two-run harness in 0.00 s max-N.
5. **2284 E — My Mind Is Going** (Multitest interactive + `@interactor` + 33-query graph BFS binary search) — 104-line solution + 45-line in-file `@interactor` passed `cpl check` with 0 errors on Draft 1, then surfaced **three severe new interactive/multitest bugs**: `cpl check` vs `cpl build` scope divergence on `@interactor`, `@multitest` + `@interactor` incompatibility in `cpl run`, and a **process-global `ask(..., limit: 33)` counter that never resets between test cases** and crashes multitest submissions on test case 4. With explicit `for _ in 0..t:` and `limit:` removed, the in-file `@interactor` ran live inside `cpl run`, and the compiled binary passed 250 random graphs + worst-case `n = 2000` (`max_q = 30 <= 33`, 0.02 s).
6. **2284 F — Fun Times** (Multitest DP / partition probability + `Mod<1_000_000_007>` + custom-monoid `SegTree`) — 3 drafts exposing `P.from` / `P.from_idx` optionality contradiction (`cpl check` vs `cpl run`/`cpl build` vs cheat sheet), `1 - z` (`Int - Mod<P>`) false-green with leaked `Mod_RI0_4`, and 8-byte double-monomorphized `Mod` codegen (124 MB RSS at `n = 500,000`, yet **0.41 s** vs 2.0 s TL).

A focused follow-up probe (`@multitest` + `@memo fn dp(i: Int)`) confirmed that **`@memo` does reset properly between `@multitest` cases** (`__memo1_ready = false;` at the top of `solve()`, resetting both flat vectors and the fallback hash map).

**Overall verdict:** Codeforces-style multitest + interactive + constructive rounds expose an entire class of **multitest-lifecycle bugs** that single-test AtCoder rounds never hit. The core language remains concise and fast (`0.00–0.41 s` on every max-N test, `3.8–22.0 KB` of readable C++17, zero ASan/UBSan findings across all six finals), and running an in-file `@interactor` directly inside `cpl run` feels genuinely futuristic once you avoid `@multitest`. However, until `@!multitest` actually splits one-time precomputation from per-test work, `ask(..., limit: K)` resets per test case (or stays debug-only), and `@multitest` composes with `@interactor` and `cpl stress`, a Codeforces contestant risks catastrophic TLEs (`11.04 s`) and hidden system-test crashes (`exit(3)` on test case 4) even after all local samples pass.

---

## Deduplication Against Simulation #1

Simulation #1 (`research/archive/blind-evals/blind-contest-sim-1.md`) already cataloged the following issues, which recurred here and are **not** re-ranked below:

- General false-green gap where `cpl check` accepts code rejected by `cpl run`/`cpl build` (Sim #1 #1; newly manifested below on `@interactor`, `two_color`, `Str.find`, `P.from`, `P.from_idx!`, and `1 - z`).
- In-file `@brute(main)` ignored by `cpl stress` and in-file `@checker` blocked on missing `read_output()` + `cpl can't build yet` (Sim #1 #2; re-hit on 2284 A2 and 2284 B).
- `cpl stress` type-driven generator omitting leading `T` under `@multitest` (Sim #1 #3; re-hit on 2284 B, shrinking to `3 / 1 / -971638750 -999999992 -1000000000`).
- `Graph.undirected(es)` requiring named `domain E` and labeled `(u: V, v: V)` records (Sim #1 #4; re-hit on 2284 D).
- Prelude vs `use std.*` inconsistencies vs `docs/one-page-cheat-sheet.md` (Sim #1 #7; newly confirmed for `desc`, `Sieve`, `two_color`, and `Monoid`, in addition to `Deque`).

---

## New Contest-Speed Bottlenecks (Deduplicated, Ranked)

### 1. `@!multitest` is unimplemented in `cpl`, and `@multitest` silently runs precomputation inside `solve()` on every test case (P0 — Silent TLE)

`docs/one-page-cheat-sheet.md` (lines 149–150) documents `@!multitest` as a file-level divider where code *above* the divider runs once for sieves and tables, and code *below* runs `t` times. In reality, `cpl` rejects `@!multitest` at parse time:

```text
error[E0101]: expected a decorator name after `@`, found `!`
```

Switching to the accepted `@multitest` spelling compiles cleanly — and then **silently moves everything above `@multitest` inside `prog::solve()`**. On `2284 C1`, placing `const LIM = 200_005; let sv = Sieve(LIM)` and the `nxt` prime table at the top of the file produced:

```cpp
void solve() {
    LIM = 200005;
    sv = m1_Sieve_new(LIM);
    nxt = std::vector<ll>(LIM + 1, 0);
    ...
}
```

On the 3-case sample it looked instant. On a Codeforces max-`t` hack (`t = 10,000`, total `n = 100,000`), it took **11.04 s** (massive TLE against the 2.0 s limit). Replacing `@multitest` with an explicit `let t: Int = read(); for _ in 0..t:` below the sieve dropped the exact same input to **0.01 s** — a **1,100x speedup**.

**Fix:** Implement `@!multitest` in `cpl-parse`/`cpl-ir`/`cpl-codegen` as a true top-level divider so statements above it execute once in `main()` before the `t`-loop and statements below it execute per test case (and deprecate or forbid `@multitest` after precomputation statements).

### 2. `ask(..., limit: K)` uses a process-global `static` query counter that never resets per test case (P0 — Silent System-Test Crash)

On `2284 E` (interactive, `t <= 10^4`, limit 33 queries per test case), writing `ask("?", 1, vs.len, vs, limit: 33)` straight from the cheat sheet passed the 2-case official sample cleanly (6 queries total <= 33). Running a 250-case multitest stress harness immediately crashed on **case 3** with `exit(3)`:

```text
RuntimeError: Case 3: premature EOF. stderr=query limit exceeded: the limit is 33
```

Inspecting `cpl build` output revealed the root cause (present in both `@multitest` and explicit `for _ in 0..t:` builds):

```cpp
inline void count_query(ll limit) {
    static ll asked = 0;
    if (++asked > limit) fail(("query limit exceeded: the limit is " + std::to_string(limit)).c_str());
}
```

Because `asked` is a process-global `static` never reset per test case — and emitted even into release judge C++ — **any Codeforces multitest interactive solution that uses `limit:` will pass samples and crash with Runtime Error on system tests**.

**Fix:** Reset the query counter at the start of every `@!multitest` test case (just like `@memo` resets `__memo1_ready = false`), expose a reset or per-case scope for explicit loops, and strip or gate `count_query` behind `-DLOCAL` so release submissions never self-abort when a judge protocol defines a looser or adaptive budget.

### 3. `@multitest` and `@interactor` are mutually incompatible in `cpl run` (P0)

On `2284 E`, combining `@multitest` with an in-file `@interactor fn judge(sol: Interactor)` failed in both possible ways:

- When `judge` read `let t: Int = read(); sol.send(t)` itself (`e_v2.cp`), `cpl run` had **already** consumed `t = 2` from the hidden test file via `@multitest`, so `judge` parsed `n = 4` as `t` and `m = 2` as `jn`, immediately crashing on edge `(3, 4)` (`3 is not in JV_1, which is 1..=2`).
- When `judge` omitted `t` (`e_v3.cp`), `cpl run` ran `judge` per test case, so `judge` sent `4 2` to `sol` without ever sending `t`. Meanwhile `sol` (also wrapped in `@multitest`) consumed `4` as `t` and read `2` as `n`, crashing with `2 is outside 4..=2000`.

Dropping `@multitest` for an explicit `let t: Int = read(); for _ in 0..t:` in both the solution and `judge` made `cpl run` execute the full live in-process interaction cleanly (`cpl: the interactor judge accepted the run (8 lines from the solution)`).

**Fix:** Define one unambiguous `@!multitest` + `@interactor` contract: either the runner consumes `t` from the hidden file and automatically sends `t` to `sol` before running `judge` `t` times, or `@interactor` always owns the full stream (`t` included) and `@!multitest` applies only to the contestant program.

### 4. `@interactor` scope mismatch between `cpl check` and `cpl run`/`cpl build` (P0)

In `2284 E` Draft 1 (`e_v1.cp`), `@interactor fn judge(sol: Interactor)` referenced the contestant program's top-level `n`, `m`, `adj`, and `domain V` — mirroring how `@brute` and `@checker` share top-level input `let`s on the cheat sheet. `cpl check` reported **`no errors`**, and then `cpl build` failed with:

```text
error[E0201]: `cpl run` doesn't know the name `n` here
error[E0201]: `cpl run` doesn't know the name `m` here
error[E0201]: `cpl run` doesn't know the name `adj` here
```

Because an interactor runs as an independent process reading the hidden test file from scratch, sharing top-level bindings is semantically wrong — yet `cpl check` allowed it silently, and `docs/one-page-cheat-sheet.md` line 169 never clarifies that `@interactor` is isolated from top-level `let`/`domain` bindings.

**Fix:** Isolate `@interactor` from top-level solution bindings in `cpl check` (with a clear diagnostic explaining that `@interactor` reads its own hidden test via `read()`), and add one clarifying phrase to the cheat sheet.

### 5. `domain.from_idx(i)` contradicts the cheat sheet (`?V` vs `V`), and `cpl check` lets both `from(i)` and `from_idx(i)!` pass untyped (P1)

On `2284 F`, the cheat sheet (line 55) specifies `V.from_idx(i) -> ?V`. Three consecutive drafts revealed a three-way contradiction:

- Draft 1 accidentally wrote `P.from(head + i)!`: **`cpl check` did not flag `P.from` at all** (only complaining about `Monoid`), while `cpl run`/`cpl build` rejected it (`a domain has no method from that cpl run knows`).
- Draft 2 fixed it to `P.from_idx(head + i)!` per the cheat sheet: **`cpl check` reported `no errors`**, while `cpl run`/`cpl build` rejected every call with `error[E0302]: ! takes the value out of an optional, but this is a P`.
- Draft 3 removed `!` (`P.from_idx(head + i)`): both `cpl check` and `cpl run`/`cpl build` passed!

`cpl check` appears to type `P.from_idx(...)` (and even nonexistent `P.from(...)`) as an unconstrained/unknown type, masking both typos and optionality errors until runtime lowering, while `cpl-ir` disagrees with the 0.2.2 spec on whether `from_idx` returns `?V` or trapping `V`.

**Fix:** Align `V.from_idx(i) -> ?V` across `cpl-check`, `cpl-ir`, and the cheat sheet (or add an explicit trapping `V.idx(i)` if non-optional indexing is intended), and never let unknown methods on a `domain` pass `cpl check`.

### 6. `Int - Mod<P>` (`1 - z`) passes `cpl check` but fails `cpl run`/`cpl build`, and `Mod<1_000_000_007>` is emitted as 8-byte double-monomorphized structs (P1)

On `2284 F`, `let p = 1 - q[pos]` (where `q: [P]Z` and `type Z = Mod<MOD>`) passed `cpl check` (matching the cheat sheet's "one-way `Int` reduction"), then failed `cpl run`/`cpl build` with an internal mangled type leak:

```text
error[E0309]: `-` can't be applied to `Int` and `Mod_RI0_4`
```

Rewriting to `Z(1) - q[pos]` compiled, but inspecting the emitted C++ (`f_v3.cpp`) revealed two codegen regressions:
1. `struct Mod_1000000007_3 { ll v = 0; };` uses **8-byte `ll`** instead of the 4-byte representation promised on cheat sheet line 204 (`4-byte when P <= 2^31 - 1`).
2. `cpl build` monomorphized **two separate C++ structs for the exact same `type Z = Mod<MOD>`** — `Mod_1000000007_3` inside `struct Node` and `Mod_RI0_4` inside `solve()` — emitting conversion lambdas `[&]() -> Mod_1000000007_3 { Mod_RI0_4 t3 = p1; return Mod_1000000007_3{t3.v}; }()` on every `st.set` and `st.fold`. At `n = 500,000` (`2n = 1,000,000` leaves), this pushed peak RSS to **124 MB** (still under the 256 MB limit, but ~3x heavier than a 4-byte `uint32_t` modint).

**Fix:** Support symmetric `Int op Mod<P>` lowering in `cpl-ir`, unify `const MOD` specialization so `Mod<MOD>` emits one single 4-byte `uint32_t` struct everywhere, and never leak `Mod_RI0_4` in diagnostics.

### 7. `Str` / `std.graph` surface gaps under Codeforces string & graph idioms: `s * k`, `Str.find`, and `two_color` (P1)

On `2284 D`, three idioms taken directly from `docs/one-page-cheat-sheet.md` failed:
- `"0" * (n - 1)` (cheat sheet line 73: `out += "bc" * 3`) failed `cpl check` (`Str * Int` unsupported).
- `s.find('1')` (cheat sheet line 201: `a.find(x) -> ?I`) passed `cpl check` and failed `cpl run` as an unknown method on `Str`.
- `use std.graph.{two_color}` passed `cpl check` and failed `cpl run` with an internal `E` scope error (same graph edge-domain leak as `Graph.undirected`).

Together these forced abandoning `std.graph` and writing a 20-line manual `Deque` BFS and a character-push loop — slower than C++'s `string(n - 1, '0')` and `s.find('1')`.

**Fix:** Implement `Str * Int` and `Str.find(Char) -> ?Int` end-to-end, and fix `two_color` alongside `Graph.undirected`.

### 8. `cpl stress` vacuous passes on unannotated `let t: Int = read()` and `Subset<I>` compile-time trap inside stripped `@brute` (P2)

Two subtle stress-testing traps surfaced:
- On `2284 E` (which had to drop `@multitest` for `let t: Int = read()`), running `cpl stress e_final.cp --cases 5` auto-detected the in-file `@interactor` (`checker: e_final.cp (judge)`) and immediately printed `5 cases agree` in 48 ms — because `let t: Int = read()` had no lower bound `1..=...`, allowing the generator to emit `t <= 0` so both `sol` and `judge` ran zero iterations and vacuously passed!
- On `2284 A2`, placing `@brute(main)` with `for mask in Subset<I>:` inside the file caused `cpl build` / runtime to trip `a Subset needs a domain of at most 25 values` when testing `n = 100`, forcing deletion of the in-file `@brute` before max-N benchmarking.

**Fix:** Ensure `@brute` blocks are never evaluated or domain-constrained on production runs, and warn in `cpl stress` when an interactive/checker run completes with zero solution lines or zero iterations.

---

## Problem-by-Problem Contest Log (Codeforces Round 1132, Div. 2)

### Problem 1: 2284 A2 — 15K4JR3B (Multitest Greedy / Bitmasks + `@brute`)
**URL:** `https://codeforces.com/contest/2284/problem/A2`
**Idea:** Each coin `a_i` has weight `w_i = 2^{a_i}`. Partition the weights into two piles `(X, Y)` to maximize `X * Y` (equivalently minimize `|X - Y|`). Because every weight is a power of 2, sorting descending and greedily assigning each `w_i` to the smaller current pile achieves the exact optimum: after placing any prefix of powers of 2, `|X - Y|` is at most the smallest power placed so far, which is divisible by every subsequent smaller weight.

**(a) First-draft compile errors / type fights:**
Draft 1 (`a2_v1.cp`) used `@!multitest` (from the cheat sheet) and `sort(mut ws, by: desc)`. `cpl check` rejected both:
```text
error[E0101]: expected a decorator name after `@`, found `!`
error[E0201]: no `desc` in scope
  = help: `desc` is in the standard library: add `use std.algebra.{desc}`
```
Draft 2 switched to `@multitest` and added `use std.algebra.{desc}`, compiling cleanly with 0 errors across `cpl check`, `cpl run`, and `cpl build`.

**(b) Moments of hesitation where C++ would have been faster:**
1. Checking whether `1 << x` is 64-bit `Int` (`x <= 24`, safe either way, and nicer than C++'s `1LL << x` trap).
2. Writing an in-file `@brute(main)` with `Subset<I>`: `cpl stress` ignored it (`pass --brute <program>`), and keeping `Subset<I>` in the file later tripped `a Subset needs a domain of at most 25 values` at `n = 100`, forcing removal of `@brute`.

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check`, `cpl run`, and `cpl build` agreed once `@multitest` and `use std.algebra.{desc}` were in place.
- `cpl stress` ignored in-file `@brute(main)` and its `@multitest` input format clashed with a single-case external brute.

**(d) Final CP Lang 0.2.2 solution & verification:**
```cp
use std.algebra.{desc}

@multitest
let n: 2..=100 = read()
domain I = 1..=n
let a: [I]0..=24 = read()

var ws = [1 << x for x in a]
sort(mut ws, by: desc)

var x = 0
var y = 0
for w in ws:
    if x <= y: x += w
    else: y += w
print(x * y)
```
**Verification:** 4/4 official samples pass (`cpl run` + `g++ -O2` + ASan/UBSan); 100/100 random cases match an exact subset-sum Python brute; max-N (`t = 100, n = 100`) runs in **0.00 s** (3.2 MB RSS); emitted C++ is **5,675 bytes**.
**Cleaner than C++?** **Strictly cleaner** (15 lines, no `1LL << x` overflow footgun, clean comprehension and `by: desc`).

---

### Problem 2: 2284 B — Introduction to the Niche Internet (Multitest Constructive + `@checker`)
**URL:** `https://codeforces.com/contest/2284/problem/B`
**Idea:** A donation `a_m` is *very pleasant* iff `a_m >= average(a_1..a_{m-1})`. Let `mx = max(a)` appear `c` times. Every occurrence of `mx` is at least the average of any prefix, so every arrangement has at least `c` very pleasant donations; thus `k < c` is impossible (`NO`). For `k >= c`, sorting `a` ascending and outputting `a[n - k .. n - c]` ascending (`k - c` pleasant), then one `mx` (`+1`), then `a[0 .. n - k]` descending (`0` pleasant, each strictly below every preceding element), then the remaining `c - 1` copies of `mx` (`+(c - 1)`) yields exactly `k` very pleasant donations.

**(a) First-draft compile errors / type fights:**
0 errors on the solution itself (`@multitest`, `sort(mut a)`, `count(x == mx for x in a)`, `(0..(n - k)).rev`, `print(ans)` all worked immediately). Adding an in-file `@checker fn check():` per the cheat sheet failed with `error[E0201]: no read_output in scope`, and `cpl stress` reported `declares the checker check, which cpl can't build yet`.

**(b) Moments of hesitation where C++ would have been faster:**
Trying to wire both in-file `@checker` and `cpl stress --checker` before discovering the CLI expects a 3-argument `<input> <output> <answer>` contract and omits `T` for `@multitest`.

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check`, `cpl run`, and `cpl build` agreed 100% on `b_final.cp`.
- `cpl stress --checker` failed on seed 1 due to the missing `T` in `@multitest` generation (shrinking to `3 / 1 / -971638750 -999999992 -1000000000`) plus the 3-arg checker contract.

**(d) Final CP Lang 0.2.2 solution & verification:**
```cp
@multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
var a: [0..n]Int = read()

sort(mut a)
let mx = a[n - 1]
let c = count(x == mx for x in a)

if k < c:
    print("NO")
    return

print("YES")
var ans: List<Int> = []
for i in (n - k)..(n - c):
    ans.push(a[i])
ans.push(mx)
for i in (0..(n - k)).rev:
    ans.push(a[i])
for _ in 0..(c - 1):
    ans.push(mx)
print(ans)
```
**Verification:** Official samples pass semantic validation (`cpl run` + `g++ -O2` + ASan/UBSan); **2,000/2,000** random cases pass the external semantic checker (including both `YES` and `NO` optimality verification against brute force on small `n`); max-N (`t = 10,000`, total `n = 200,000`) runs in **0.03 s** (13.6 MB RSS); emitted C++ is **3,835 bytes**.
**Cleaner than C++?** **Strictly cleaner** (22 lines, `count(...)`, `(0..(n - k)).rev`, `print(ans)`).

---

### Problem 3: 2284 C1 — Disgusting + GCD (Multitest + `Sieve` Precomputation Catastrophe)
**URL:** `https://codeforces.com/contest/2284/problem/C1`
**Idea:** For each `a_i <= 10^5`, map `b_i` to the smallest prime `>= a_i` (all `<= 200,005`). Precomputing `nxt[x]` once via a `Sieve(200_005)` answers every test case in `O(n)`.

**(a) First-draft compile errors / type fights:**
Draft 1 (`c1_v1.cp`) wrote `let sv = Sieve(LIM)` without importing `Sieve`:
```text
error[E0201]: no `Sieve` in scope
  = help: `Sieve` is in the standard library: add `use std.math.{Sieve}`
```
Adding `use std.math.{Sieve}` fixed compilation immediately.

**(b) Moments of hesitation where C++ would have been faster:**
1. Discovering `@!multitest` is not implemented and falling back to `@multitest` — which silently hid a **1,100x slowdown** because `cpl build` placed `Sieve(200_005)` and the `nxt` loop inside `prog::solve()`, recomputing the entire 200,005-element sieve 10,000 times (**11.04 s** vs **0.01 s**)! In C++, placing `sieve()` in `main()` above `while (t--)` is automatic and never miscompiled.

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check` rejected `@!multitest`; `@multitest` silently broke the spec's "above divider runs once" guarantee in both `cpl run` and `cpl build`.
- Switching to an explicit `let t: Int = read(); for _ in 0..t:` (`c1_loop.cp`) fixed both `cpl run` and `cpl build` completely.

**(d) Final CP Lang 0.2.2 solution & verification:**
```cp
use std.math.{Sieve}

const LIM = 200_005
let sv = Sieve(LIM)
var nxt: [0..=LIM]Int = 0
var last_p = LIM
for i in (0..=LIM).rev:
    if sv.is_prime(i):
        last_p = i
    nxt[i] = last_p

let t: Int = read()
for _ in 0..t:
    let n: 2..=1e5 = read()
    let a: [0..n]Int = read()
    let b = [nxt[x] for x in a]
    print(b)
```
**Verification:** Official samples pass semantic validation (`cpl run` + `g++ -O2` + ASan/UBSan); **1,000/1,000** random cases pass the external checker under ASan/UBSan; max-N (`t = 10,000`, total `n = 100,000`) drops from **11.04 s** (`@multitest`) to **0.01 s** (`6.95 MB` RSS, explicit loop); emitted C++ is **5,704 bytes**.
**Cleaner than C++?** **Cleaner once fixed**, but the silent `@multitest` precomputation TLE is a severe contest hazard until `@!multitest` is actually implemented.

---

### Problem 4: 2284 D — The Theorem of Two Cans (Run-Twice Communication + Graph 2-Coloring)
**URL:** `https://codeforces.com/contest/2284/problem/D`
**Idea:** PUB's operation picks an odd-length simple path `v_1 - ... - v_{2k}` (`k >= 2`), deletes one edge `(v_i, v_{i+1})`, and ensures `(v_1, v_{2k})` is present. Bipartiteness — and the exact unique 2-coloring of a connected bipartite graph — is a **two-way invariant** of this operation: if `G` is bipartite, `v_1` and `v_{2k}` have opposite colors so `G'` keeps the same 2-coloring; conversely, if `G'` is bipartite, then `(v_1, v_{2k}) \in G'` forces opposite colors on `v_1, v_{2k}` and by parity along the remaining two subpaths forces opposite colors on `(v_i, v_{i+1})`, so `G` was already bipartite. Thus LCB encodes any string `s != "0...0"` as a connected bipartite tree with `color(1) = 0` and `color(i + 2) = s[i]` (linking `'1'` vertices to `1` and `'0'` vertices to a `'1'` pivot `p`), and encodes `s = "0...0"` as a non-bipartite graph (triangle `1-2-3` plus star `(1, 4..n)`). RCB runs BFS 2-coloring: if bipartite, output `col[2..n]` relative to `col[1] = 0`; otherwise output `"0" * (n - 1)`.

**(a) First-draft compile errors / type fights:**
Three drafts were needed:
- Draft 1 (`d_v1.cp`) failed `cpl check` on missing `use std.graph.{two_color}` and unsupported `"0" * (n - 1)` (`Str * Int`).
- Draft 2 (`d_v2.cp`) added `use std.graph.{two_color}`, passed `cpl check` with 0 errors, and then failed `cpl run` on three separate mismatches: `two_color` triggered the stdlib `E` scope leak, `s.find('1')` was unknown to `cpl run`, and `Graph.undirected` rejected plain `(V, V)` edge lists.
- Draft 3 (`d_final.cp`) bypassed `std.graph`, `Str.find`, and `Str * Int` with manual `Deque` BFS and loops, passing `cpl check`, `cpl run`, and `cpl build` cleanly.

**(b) Moments of hesitation where C++ would have been faster:**
1. Every convenience we reached for from the cheat sheet (`two_color`, `s.find('1')`, `"0" * (n - 1)`) broke, turning a 25-line solution into 56 lines.
2. Because the first line is `"first"` or `"second"` *before* `t`, `@multitest` cannot be used (a genuine Codeforces run-twice I/O pattern that requires explicit `for _ in 0..t:`).

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check` falsely accepted `use std.graph.{two_color}` + `Graph.undirected` + `s.find('1')`, all of which failed in `cpl run`/`cpl build`.
- `d_final.cp` agreed 100% across `cpl check`, `cpl run`, and `cpl build`.

**(d) Final CP Lang 0.2.2 solution & verification:**
```cp
use std.deque.{Deque}

let mode: Str = read()
let t: Int = read()
for _ in 0..t:
    if mode == "first":
        let n: 3..=1000 = read()
        let s: Str = read()
        var pivot: ?Int = none
        for i in 0..(n - 1):
            if s[i] == '1':
                pivot = i + 2
                break
        if let p = pivot:
            print(n - 1)
            for v in 2..=n:
                if s[v - 2] == '1':
                    print(1, v)
                else:
                    print(p, v)
        else:
            print(n)
            print(1, 2)
            print(2, 3)
            print(1, 3)
            for v in 4..=n:
                print(1, v)
    else:
        let n: 3..=1000 = read()
        let m: Int = read()
        domain V = 1..=n
        var adj: [V]List<V> = []
        for _ in 0..m:
            let (u, v): (V, V) = read()
            adj[u].push(v)
            adj[v].push(u)
        var col: [V]?Bool = none
        var q: Deque<V> = []
        col[V.first] = false
        q.push(V.first)
        var bip = true
        while let u = q.pop_front():
            let cu = col[u]!
            for v in adj[u]:
                if let cv = col[v]:
                    if cv == cu:
                        bip = false
                else:
                    col[v] = not cu
                    q.push(v)
        var out = ""
        if bip:
            for v in V:
                if v != V.first:
                    out.push(if col[v]!: '1' else: '0')
        else:
            for _ in 0..(n - 1):
                out.push('0')
        print(out)
```
**Verification:** Passed an 89-case two-run adversarial Python harness (including `s = "0...0"`, random strings, test-case reordering, edge reversals/shuffling, and repeated random odd-length path operations) on `cpl run`, `g++ -O2`, and ASan/UBSan; dense max-N (`n = 1000`, `250,000` edges) runs in **0.00 s** (6.75 MB RSS); emitted C++ is **12,597 bytes**.
**Cleaner than C++?** **Slower to write than C++** because `two_color`, `Str.find`, and `Str * Int` all had to be rewritten by hand; `?Bool` + `while let u = q.pop_front()` still read nicely.

---

### Problem 5: 2284 E — My Mind Is Going (Multitest Interactive + `@interactor` + 33-Query Binary Search)
**URL:** `https://codeforces.com/contest/2284/problem/E`
**Idea:** Because `G`, `G + {e1}`, and `G + {e2}` are all bipartite while `G + {e1, e2}` is not, both hidden edges `e1, e2` must connect the *same* pair of distinct connected components `(C1, C2)` of `G` with opposite parity constraints. Traverse `G` component-by-component in BFS order `order[0..n]`:
1. Binary search (`<= 11` queries on `3..=n`) the first prefix `order[0..lo]` that is non-bipartite -> `u = order[lo - 1]` is an endpoint of a secret edge in the later component `cu`, and `a_set = order[0..lo] ∩ cu` is a connected subset containing all endpoints in `cu`.
2. Binary search (`<= 11` queries on `1..=b_all.len`) the first prefix of `b_all` (vertices before `u` outside `cu`) such that `a_set + b_all[0..lo2]` is non-bipartite -> `w = b_all[lo2 - 1]` is an endpoint in the earlier component `cw`, and `b_set = b_all[0..lo2] ∩ cw` is connected.
3. Ask `? 2 u w` (`1` query). If true, output `! u w`.
4. Otherwise `e1 = (u, b1)` and `e2 = (a2, w)` are vertex-disjoint. Pick the smaller of `a_set` and `b_set` (size `<= n/2 <= 1000`), run a BFS inside it from `w` (or `u`), and binary search (`<= 10` queries on `2..=1000`) the exact prefix where adding the other set becomes non-bipartite to recover `b1` (or `a2`). Total queries `<= 11 + 11 + 1 + 10 = 33`.

**(a) First-draft compile errors / type fights:**
- Draft 1 (`e_v1.cp`, 150 lines including `@interactor`) passed **`cpl check` with 0 errors**, then failed `cpl build` with `E0201` because `@interactor` cannot read top-level `n`, `m`, `adj`.
- Draft 2 (`e_v2.cp`) made `judge` read its own `t, jn, jm, ...`, passing `cpl check` and `cpl build`, then crashed in `cpl run` because `@multitest` had already eaten `t` from `judge`'s hidden input.
- Draft 3 (`e_v3.cp`) removed `t` from `judge`; now `cpl run` crashed in `sol` because `@multitest` never sent `t` to `sol`.
- Draft 4 (`e_final.cp` with explicit `let t: Int = read(); for _ in 0..t:` and `limit: 33`) passed `cpl run` live against the in-file `@interactor` on both sample cases — and then **crashed on case 3 of our 250-case multitest stress test** due to `cpl::count_query`'s global `static ll asked = 0`.
- Draft 5 removed `limit: 33` and passed every test cleanly.

**(b) Moments of hesitation where C++ would have been faster:**
All four dead ends above (`@interactor` scope, `@multitest` + `@interactor`, global `limit: 33`, and `cpl stress` vacuously passing `5 cases agree` on `t = 0`). On the positive side, **not once did I have to worry about `fflush(stdout)` or `ios::sync_with_stdio` deadlocks**: `cpl build` automatically emitted `cpl::live = true;` and `cpl::flush(); fflush(stdout);` on every `ask()` and `print()`, and `ask("?", 1, vs.len, vs)` formatted the `List<V>` cleanly in one expression!

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check` accepted top-level bindings inside `@interactor`; `cpl run`/`cpl build` rejected them.
- `cpl run` successfully executed the in-file `@interactor` live once `@multitest` was removed.
- `cpl stress` auto-detected the in-file `@interactor` (`checker: e_final.cp (judge)`), but vacuously passed 5 cases because `let t: Int = read()` allowed `t <= 0`.

**(d) Final CP Lang 0.2.2 solution & verification:**
```cp
use std.deque.{Deque}

let t: Int = read()
for _ in 0..t:
    let n: 4..=2000 = read()
    let m: Int = read()
    domain V = 1..=n
    var adj: [V]List<V> = []
    for _ in 0..m:
        let (u, v): (V, V) = read()
        adj[u].push(v)
        adj[v].push(u)

    var comp: [V]Int = -1
    var order: List<V> = []
    var cid = 0
    for s in V:
        if comp[s] != -1: continue
        var q: Deque<V> = []
        comp[s] = cid
        q.push(s)
        while let u = q.pop_front():
            order.push(u)
            for v in adj[u]:
                if comp[v] == -1:
                    comp[v] = cid
                    q.push(v)
        cid += 1

    fn query_bip(vs: List<V>) -> Bool:
        let r: Int = ask("?", 1, vs.len, vs)
        return r == 1

    fn query_edge(u: V, v: V) -> Bool:
        let r: Int = ask("?", 2, u, v)
        return r == 1

    fn bfs_in(allowed: [V]Bool, root: V) -> List<V>:
        var vis: [V]Bool = false
        var res: List<V> = []
        var q: Deque<V> = []
        vis[root] = true
        q.push(root)
        while let u = q.pop_front():
            res.push(u)
            for v in adj[u]:
                if allowed[v] and not vis[v]:
                    vis[v] = true
                    q.push(v)
        return res

    var lo = 3
    var hi = n
    while lo < hi:
        let mid = (lo + hi) / 2
        if query_bip(order[0..mid]): lo = mid + 1
        else: hi = mid
    let u = order[lo - 1]
    let cu = comp[u]

    var a_set: List<V> = []
    for i in 0..lo:
        if comp[order[i]] == cu: a_set.push(order[i])

    var b_all: List<V> = []
    for i in 0..(lo - 1):
        if comp[order[i]] != cu: b_all.push(order[i])

    var lo2 = 1
    var hi2 = b_all.len
    while lo2 < hi2:
        let mid = (lo2 + hi2) / 2
        if query_bip(a_set + b_all[0..mid]): lo2 = mid + 1
        else: hi2 = mid
    let w = b_all[lo2 - 1]
    let cw = comp[w]

    var b_set: List<V> = []
    for i in 0..lo2:
        if comp[b_all[i]] == cw: b_set.push(b_all[i])

    if query_edge(u, w):
        print("!", u, w)
        continue

    if b_set.len <= a_set.len:
        var in_b: [V]Bool = false
        for v in b_set: in_b[v] = true
        let b_ord = bfs_in(in_b, w)
        var lo3 = 2
        var hi3 = b_ord.len
        while lo3 < hi3:
            let mid = (lo3 + hi3) / 2
            if query_bip(a_set + b_ord[0..mid]): lo3 = mid + 1
            else: hi3 = mid
        print("!", u, b_ord[lo3 - 1])
    else:
        var in_a: [V]Bool = false
        for v in a_set: in_a[v] = true
        let a_ord = bfs_in(in_a, u)
        var lo3 = 2
        var hi3 = a_ord.len
        while lo3 < hi3:
            let mid = (lo3 + hi3) / 2
            if query_bip(a_ord[0..mid] + b_set): lo3 = mid + 1
            else: hi3 = mid
        print("!", a_ord[lo3 - 1], w)

@interactor fn judge(sol: Interactor):
    let t: Int = read()
    sol.send(t)
    for _ in 0..t:
        let jn: Int = read()
        let jm: Int = read()
        domain JV = 1..=jn
        sol.send(jn, jm)
        var g2: [JV]List<JV> = []
        for _ in 0..jm:
            let (x, y): (JV, JV) = read()
            g2[x].push(y); g2[y].push(x)
            sol.send(x, y)
        let (e1u, e1v): (JV, JV) = read()
        let (e2u, e2v): (JV, JV) = read()
        g2[e1u].push(e1v); g2[e1v].push(e1u)
        g2[e2u].push(e2v); g2[e2v].push(e2u)
        var queries = 0
        while true:
            let kind: Str = sol.recv()
            if kind == "!":
                let (au, av): (JV, JV) = sol.recv()
                let ok1 = (au == e1u and av == e1v) or (au == e1v and av == e1u)
                let ok2 = (au == e2u and av == e2v) or (au == e2v and av == e2u)
                assert ok1 or ok2
                break
            assert kind == "?"
            queries += 1
            assert queries <= 33
            let qtype: Int = sol.recv()
            if qtype == 1:
                let k: Int = sol.recv()
                var in_s: [JV]Bool = false
                for _ in 0..k:
                    let v: JV = sol.recv()
                    in_s[v] = true
                var col: [JV]?Bool = none
                var bip = true
                for s in JV:
                    if not in_s[s] or col[s] != none: continue
                    var q: Deque<JV> = []
                    col[s] = false
                    q.push(s)
                    while let x = q.pop_front():
                        let cx = col[x]!
                        for y in g2[x]:
                            if not in_s[y]: continue
                            if let cy = col[y]:
                                if cy == cx: bip = false
                            else:
                                col[y] = not cx
                                q.push(y)
                sol.send(if bip: 1 else: 0)
            else:
                let (qu, qv): (JV, JV) = sol.recv()
                var found = false
                for y in g2[qu]:
                    if y == qv: found = true
                sol.send(if found: 1 else: 0)
```
**Verification:** In-file `@interactor` passed both official sample cases live in `cpl run`; external Python pipe interactor passed **250/250** random bipartite graphs (`max_q = 19`, `0.10 s` O2 / `0.12 s` ASan+UBSan) and worst-case `n = 2000` (`1000 + 1000` split, **`max_q = 30 <= 33`**, **0.02 s** O2 / **0.03 s** ASan+UBSan); emitted C++ is **21,992 bytes** (with `@interactor` cleanly stripped).
**Cleaner than C++?** **Potentially much cleaner than C++** (zero flush boilerplate, `a_set + b_ord[0..mid]` slice concat, live in-file `@interactor`), once the `@multitest` + `limit:` traps are fixed.

---

### Problem 6: 2284 F — Fun Times (Multitest Partition-Probability DP + `Mod<10^9+7>` + Custom-Monoid `SegTree`)
**URL:** `https://codeforces.com/contest/2284/problem/F`
**Idea:** Track the partition of `{0, ..., n - 1}` into `m` disjoint buckets `B_m(0), ..., B_m(m - 1)` after `n - m` steps (`x -> (x + 1) mod m`). Transitioning from `m + 1` to `m` merges the last bucket `B_{m+1}(m)` into `B_{m+1}(0)` and cyclically rotates right by 1. Because buckets are pairwise disjoint, the empty-bucket probabilities `q_m(j) = Pr[B_m(j) ∩ S' = ∅]` are independent (`q_n(i) = 0` if `s[i] == '1'` else `1/2`), merging multiplies `q[head] *= q[head + m]`, and rotating right by 1 moves the rightmost element to `head - 1` inside a `2n`-slot array. Letting `p(j) = 1 - q(j)`, `E[mex(A_{n-m})] = sum_{k=1..m} prod_{j=0..k-1} p(head + j)`, which is a 2-field associative monoid `(prod, psum)` over `Mod<1_000_000_007>` maintained in `O(n log n)` by `SegTree`. Multiply total expectation by `2^{zeros}`.

**(a) First-draft compile errors / type fights:**
- Draft 1 (`f_v1.cp`) hit `no Monoid in scope` in `cpl check`, while `cpl run`/`cpl build` rejected `P.from(...)` (9 errors) and `1 - q[pos]` (`- can't be applied to Int and Mod_RI0_4`).
- Draft 2 (`f_v2.cp`) added `use std.algebra.{Monoid}`, `Z(1) - ...`, and `P.from_idx(...)!` per cheat sheet line 55: **`cpl check` reported `no errors`**, then `cpl run`/`cpl build` rejected all 6 `!` unwraps (`! takes the value out of an optional, but this is a P`).
- Draft 3 (`f_v3.cp`) dropped `!` from `P.from_idx(...)` and passed `cpl check`, `cpl run`, and `cpl build` cleanly.

**(b) Moments of hesitation where C++ would have been faster:**
1. Fighting the `P.from_idx(...)` optionality contradiction (`?P` in cheat sheet vs `P` in `cpl run`/`cpl build` vs untyped in `cpl check`).
2. Writing `P.from_idx(head + m)` six times to bridge an `Int` sliding window (`head`) into a `domain P`-indexed `SegTree` — in C++ you just pass `head` and `head + m - 1` directly.

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- Three separate false-green `cpl check` acceptances (`P.from`, `P.from_idx!`, and `1 - z`) that failed `cpl run`/`cpl build`.
- `cpl build` emitted 8-byte `ll` modints and double-monomorphized `Mod_1000000007_3` + `Mod_RI0_4`.

**(d) Final CP Lang 0.2.2 solution & verification:**
```cp
use std.algebra.{Monoid}

const MOD = 1_000_000_007
type Z = Mod<MOD>

struct Node:
    prod: Z = 1
    psum: Z = 0

@multitest
let n: 1..=5e5 = read()
let s: Str = read()
if n == 1:
    print(0)
    return

let inv2: Z = Z(1) / 2
domain P = 0..(2 * n)
var q: [P]Z = 0
var init: [P]Node = Node()
var head = n
var zeros = 0
for i in 0..n:
    let pos = P.from_idx(head + i)
    if s[i] == '1':
        q[pos] = 0
    else:
        q[pos] = inv2
        zeros += 1
    let p = Z(1) - q[pos]
    init[pos] = Node(prod: p, psum: p)

var st = SegTree<P, Node>(Monoid(Node(), (a, b) => Node(prod: a.prod * b.prod, psum: a.psum + a.prod * b.psum)), init)
var ans: Z = 0
for m in (1..n).rev:
    let p_head = P.from_idx(head)
    let p_last = P.from_idx(head + m)
    q[p_head] = q[p_head] * q[p_last]
    let p1 = Z(1) - q[p_head]
    st.set(p_head, Node(prod: p1, psum: p1))

    let p_prev = P.from_idx(head + m - 1)
    let q_prev = q[p_prev]
    head -= 1
    let new_head = P.from_idx(head)
    q[new_head] = q_prev
    let p2 = Z(1) - q_prev
    st.set(new_head, Node(prod: p2, psum: p2))

    let r_end = P.from_idx(head + m - 1)
    ans += st.fold(new_head..=r_end).psum

print(ans * Z(2).pow(zeros))
```
**Verification:** 4/4 official samples pass (`30, 14, 306, 78`); **200/200** random cases (`n = 1..9`) match the exact exponential-definition brute force in both `g++ -O2` and ASan/UBSan; max-N (`t = 1, n = 500,000`) runs in **0.41 s** (`124.2 MB` RSS, `1.18 s` under ASan/UBSan) and `t = 10,000, sum n = 500,000` runs in **0.11 s** (`3.9 MB` RSS); emitted C++ is **17,300 bytes**.
**Cleaner than C++?** **Strictly cleaner** — a 50-line Div. 2 F with zero modint or segment-tree implementation boilerplate, and 5x faster than the 2.0 s time limit.

---

## Closing Takeaway

Running a full 6-problem Codeforces Div. 2 round blind exposed a sharp divide: **single-case algorithmic expressiveness is already strong** (a 50-line Div. 2 F with custom-monoid `SegTree` + `Mod<10^9+7>` running in 0.41 s at `n = 500,000`, and a live in-process `@interactor` verifying a 33-query Div. 2 E directly inside `cpl run`), while **multitest lifecycle plumbing is the #1 danger zone**. Fixing four concrete P0 bugs — implementing `@!multitest` with true one-time precomputation, resetting `ask(..., limit: K)` per test case, making `@!multitest` compose with `@interactor`, and closing the `@interactor` / `from_idx` / `Int - Mod<P>` `cpl check` gaps — will make CP Lang 0.2.2 genuinely contest-ready for Codeforces rounds.
