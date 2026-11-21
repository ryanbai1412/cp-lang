# Friction: `ucup5s01a`

## 64-bit `Mod<P>` arithmetic and per-step 2D DP table reallocation (1.7x slowdown)

- **Severity:** medium
- **Goal it hurts:** performance
- **Measured:** `144.53 ms` vs `86.50 ms` (`1.67x`) on `gen size=200` (`research/scoreboard.json`).
- **Cause (layout + codegen):** `Mod<998_244_353>` stores `int64_t` and calls 64-bit `cpl::modp` in the $O(m(n+m)^2)$ inner DP loop, and `var next: [0..size, 0..size]Z = 0` allocates and frees a fresh 2D table on each of the $m$ outer iterations.
- **Fix:** narrow `Mod<P>` to `uint32_t` with branchless add/sub when $P < 2^{31}$, and lower `dp = next` / buffer reuse without reallocating.
