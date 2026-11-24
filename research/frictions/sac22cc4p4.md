# sac22cc4p4 — subarrays with B <= max - min <= T

## Two `Window`s for one window's range
- **Where:** `port.cp`, `top` and `bottom` pushed and popped together.
- **What felt wrong:** the window's spread is one fact kept in two structures; `Window((min, max))` over a product op would say it once.
- **General fix:** a `Window` over a pair of selective ops (or a `spread` window), so the two pointer loop moves one structure.
- **Goal it hurts:** express.
- **Severity:** low.

## `B - 1` is below the type of `limit` when `B = 0`
- **Where:** `if lo_gap == 0: 0 else: within(lo_gap - 1)`.
- **What felt wrong:** "count(T) - count(B - 1)" needs a guard because `within` takes `0..=2e9`.
- **Severity:** low.

## Dream

```cp
let n = read<1..=1e6>()
let (b, t) = read<(0..=2e9, 0..=2e9)>()
let a = read<[n]-1e9..=1e9>()
fn within(limit: Int) -> Int:
    var w = Window((min, max))
    var count = 0
    for (l, r) in a.two_pointers(keep: (w, r) => w.all!.1 - w.all!.0 <= limit): count += r - l + 1
    count
print(within(t) - within(b - 1))
```
