# ucup4s24f — UCup 4 Stage 24 (Wulin) F

Scanning right to left, `dp[i] = c[i] + min dp[j]` over the window
`i < j <= first_full(i)`, where `first_full(i)` is the farthest last
occurrence over all `R` residues of the prefix sums (a lazy max-heap). The
window minimum is the DP area's sliding-window optimization.

## The window minimum was a hand-written monotone deque (fixed)
- **Where:** before the rewrite, the DP loop:
  ```cp
  while let (_, idx) = dq.first, idx > first_full:
      dq.pop_front()
  dp[i] = c[i - 1] + dq.first!.0
  while let (val, _) = dq.last, dp[i] <= val:
      dq.pop()
  dq.push((dp[i], i))
  ```
- **What felt wrong:** two invariants (positions leave from the front, worse
  values leave from the back) and a tie rule written inline, as in every
  monotone-queue port.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:**
  ```cp
  window.drop_before(-first_full)
  dp[i] = c[i - 1] + window.best!
  window.push(-i, dp[i])
  ```
- **General fix:** `std.window.MonoDeque` (design/stdlib/dp.md 2.2 points
  every sliding-window DP at it). Landed; the port uses it.
- **Severity:** medium (fixed)

## `MonoDeque` only slides rightward, so a right-to-left scan negates positions
- **Where:** `window.push(-(n + 1), 0)`, `window.drop_before(-first_full)`,
  `window.push(-i, dp[i])`.
- **What felt wrong:** the window's positions must rise, so a DP that runs
  from `n` down to `1` stores `-i` and drops before `-first_full`. The
  negation is a fact about the structure, not the problem, and a missed
  minus sign silently empties or never trims the window.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:**
  ```cp
  var window = MonoDeque<Int>.new(by: min, toward: Left)
  window.drop_after(first_full)
  ```
- **General fix:** a direction for the window (containers area: an option
  on `MonoDeque.new`, or `drop_after` beside `drop_before`), so either scan
  order states its bound directly. Helps every suffix DP over a window.
- **Severity:** low

## Measurements

| | tokens | runtime | peak memory | emitted C++ |
|---|---|---|---|---|
| port before | 309 | 0.31 s | 61 MB | 21,092 B |
| port after | 274 | 0.32 s | 61 MB | 21,810 B |
| original | 506 | 0.25 s | 27 MB | |

Runtime is the best of 10 runs on one test with `n = 1e6`, `R = 50`; all
three agree. The window is not the hot part: both port versions spend the
gap to the original in the `Map` and heap, and the memory gap is their
64-bit storage.
