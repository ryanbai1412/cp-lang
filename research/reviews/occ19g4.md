Score: pending
Dream: pending

Re-port for headroom: the previous port ran two CDQ passes over the radius
order and re-sorted every half by u at every level, over a Fenwick on
compressed v. This one ranks once and runs a single recursion that splits
three presorted u orders stably, so no level sorts, and v keeps its own
domain 0..=2 * 60_000 + 1 (the clamped square edges fit it, so nothing is
compressed). `--max` under `cpl test` went from 747 ms to about 545 ms
(1000 ms limit); natively 0.64 s to 0.40 s. Output matches the brute force
on 30 stress cases and six N <= 6000 seeds, and the previous port on `--max`.
