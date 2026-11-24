# Friction Report: noi09p5

## 1. Two-pipe state as a 2-D table
- **Where:** `var dp: [0..=n, 0..=n]Z = 0`, replacing a flat `List` with stride `n + 1`.
- **What felt wrong:** nothing left; the old manual stride and `% MOD` are gone (`Mod<1024523>`).
- **Goal it hurts:** none.
- **Ideal code:** as written.
- **General fix:** a lint for `dp[i * (n + 1) + j]` suggesting a 2-D table.
- **Severity:** none.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = m = 500) 0.63 s vs C++ 0.48 s, same output.
