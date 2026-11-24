# Friction Report: noip06p1

## 1. Ring offsets go through `Bead.wrap(i.ord + k)`
- **Where:** three `Bead.wrap(i.ord + k)` calls in `best`.
- **What felt wrong:** moving k beads round a ring is one idea, spelled as position-to-int-and-back.
- **Goal it hurts:** express.
- **Ideal code:** `i + k` on a cyclic domain (`domain Bead = ring 0..n`).
- **General fix:** a cyclic domain whose `+ Int` wraps.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and AC 5165686 on gen.py inputs; max (N = 100) 0.012 s vs C++ 0.020 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let n = read<4..=100>()
domain Bead = ring 0..n
let mark = read<[Bead]1..=1000>()
@memo
fn best(i: Bead, len: 1..=n) -> Int =
    max(best(i, k) + best(i + k, len - k) + mark[i] * mark[i + k] * mark[i + len] for k in 1..len) ?? 0
print(max(best(i, n) for i in Bead)!)
```
