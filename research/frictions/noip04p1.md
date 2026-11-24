# Friction Report: noip04p1

## 1. Early exit from a simulation needs a flag
- **Where:** `var short: ?Month` with `break`.
- **What felt wrong:** "the first month that goes negative, else the final money" is a fold with an early
  result; the port keeps a mutable optional beside the loop.
- **Goal it hurts:** express.
- **Ideal code:** a `fn` with `return` per outcome, or `Month.find` over a running scan.
- **General fix:** a `scan`/`try_fold` that stops with a value (`Month.try_fold(state, f) -> Result`).
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5117097 on gen.py inputs; max 0.007 s vs C++ 0.004 s (process start), same output.

## Dream
The ungated ideal (does not check today):

```cp
domain Month = 1..=12
let budget = read<[Month]0..=350>()
var (cash, banked) = (0, 0)
for month in Month:
    cash += 300 - budget[month]
    if cash < 0: return print(-month)
    banked += cash / 100 * 100
    cash %= 100
print(cash + banked * 6 / 5)
```
