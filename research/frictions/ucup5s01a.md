# Friction: `ucup5s01a`

## 64-bit `Mod<P>` arithmetic and per-step 2D DP table reallocation (1.7x slowdown)

- **Severity:** medium
- **Goal it hurts:** performance
- **Measured:** `144.53 ms` vs `86.50 ms` (`1.67x`) on `gen size=200` (`research/scoreboard.json`).
- **Cause (layout + codegen):** `Mod<998_244_353>` stores `int64_t` and calls 64-bit `cpl::modp` in the $O(m(n+m)^2)$ inner DP loop, and `var next: [0..size, 0..size]Z = 0` allocates and frees a fresh 2D table on each of the $m$ outer iterations.
- **Fix:** narrow `Mod<P>` to `uint32_t` with branchless add/sub when $P < 2^{31}$, and lower `dp = next` / buffer reuse without reallocating.

## Small polynomial products have no cheap stdlib form

- **Goal it hurts:** express, performance
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s01a.cp:21`
- **Actual:** `proto/ports/ecnerwala/ucup5s01a.cp:21-27`
- **General fix:** a `Poly<Z>` value with `*` and a `prod` over a generator that picks schoolbook multiplication below an NTT threshold. `std.poly.prod_linear` computes the same product but pulls in the NTT: it grew the emitted C++ from 15,957 to 26,085 bytes for 500 factors, so the port keeps a comprehension that multiplies by `(d + x)` one factor at a time.

## Pull-form DP layers need out-of-range reads as zero

- **Goal it hurts:** express, hard to get wrong
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s01a.cp:28-35`
- **Actual:** `proto/ports/ecnerwala/ucup5s01a.cp:28-45`
- **General fix:** a total read on tables (`dp.get(i, j)` returning the element's zero, or `?Z` with `?? 0`) so a new layer is one table comprehension over the pull recurrence. Its leaf case merges with `others - (i - 1)`, which the push form spells as two separate updates. A pull over the full `Count x Count` square also costs about 3x the triangle `i + j <= n + z` that the push loop visits, so this needs the comprehension to accept a filtered domain, too.

## `deg` is a count of parents, built by hand

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s01a.cp:20`
- **Actual:** `proto/ports/ecnerwala/ucup5s01a.cp:18-19`
- **General fix:** a counting table over a domain (`parent.counts(in: V)` or `[V] v => parent.count(v)` lowered to one pass) so occurrence counts are a declaration, not a `var` with a loop.
