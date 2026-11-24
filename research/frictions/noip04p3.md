# Friction Report: noip04p3

## 1. Longest rising subsequence ending at each position
- **Where:** two O(N^2) DPs `up` and `down`.
- **What felt wrong:** "the longest strictly rising subsequence ending at i" is a std idea (LIS), but std's
  LIS returns one length, not the per-position lengths.
- **Goal it hurts:** express.
- **Ideal code:** `let up = h.lis_ending()`, `let down = h.rev.lis_ending().rev`.
- **General fix:** a per-position LIS (`lis_ending(by:)`) beside the length.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5117143 on gen.py inputs; max (N = 100) 0.004 s vs C++ 0.004 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let n = read<2..=100>()
domain I = 1..=n
let h = read<[I]130..=230>()
let up = h.lis_ending()
let down = h.rev.lis_ending().rev
print(n - max(up[i] + down[i] - 1 for i in I)!)
```
