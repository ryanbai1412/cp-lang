When an algorithm mixes vertex numbers, item positions, prefix boundaries, and ordinary counts inside plain `int`, every lookup depends on remembering which kind of integer is in which variable. CP Lang 0.2.2 gives you both **structural range arrays** (`[0..n]T`, `[0..=k]T`) for fast integer-indexed tables and **nominal domains** (`domain V = 1..=n`) when you want the type checker to keep index spaces strictly separate.

## From integers to nominal domains

A `domain` declaration freezes an integer range into a nominal index type:

```cp CP Lang 0.2.2 · fragment
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
```

Two rules make this pay off in contests:

1. **Nominal identity.** `domain Row = 1..=n` and `domain Col = 1..=n` are distinct types even when `n` is identical. Indexing `g[c, r]` instead of `g[r, c]` fails at compile time.
2. **Valid by construction.** Every `v: V` lies inside `1..=n`, so `a[v]` needs no runtime bounds check in release C++17. Stepping `v + k` or `v - k` returns `?V` (`none` if the step leaves the domain), and `v.to_idx` converts to `0..V.K` when you need a zero-based integer.

When you only need a scratch array indexed by `Int`—such as a frequency counter—write `[0..=k]Int` directly and reset it with `cnt.fill(0)`:

```example range_array
```

## Boundaries: `.l` and `.r`

Prefix sums, difference arrays, and DP transitions live on the `n + 1` gaps around `n` elements. Every ordered domain `V` has a companion boundary space `V.bounds`:

- `v.l` is the gap immediately left of `v`; `v.r` is the gap immediately right of `v`.
- `v.l(k)` and `v.r(k)` jump to the `k`-element window boundary counting `v` (`v.l == v.l(1)`, `v.r == v.r(1)`).
- `b.left -> ?V` and `b.right -> ?V` step from a boundary back to the adjacent element.

```example bounds
```

Notice how `pre[v.r] = pre[v.l] + a[v]` never writes `i - 1`, never guesses whether to allocate `n` or `n + 1`, and prints the total from `pre[V.last.r]`. Answering `q` inclusive range-sum queries and sliding over every length-`k` window both use the same boundary table (both run in real WASM):

```cp CP Lang · range-sums.cp
let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()

var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]

for _ in 0..q:
    let l: V = read()
    let r: V = read()
    print(pre[r.r] - pre[l.l])
```

```cp CP Lang · best-window.cp
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain V = 1..=n
let a: [V]Int = read()

var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]

var best = pre[V(k).r] - pre[V(k).l(k)]
for v in V(k)..=V.last:
    best max= pre[v.r] - pre[v.l(k)]
print(best)
```

## Stepping safely with `?T`

Because `v + 1` and `v - 1` return `?V`, boundary checks are enforced by the type system rather than remembered by hand. The same `?T` type replaces `-1` and `1e18` sentinels across the language:

```example optional
```

Use `??` for a fallback, `if let` to bind a present value, and `opt!` when an earlier check already guarantees presence. [Chapter 5](course-dp.html) dives deep into Tropical Optionals (`min=`, `max=`, `??=`) and `@memo` DP.

## Practice

```exercise domains-why-reject
```

```exercise domains-predict-bounds
```

```exercise domains-range-formula
```

```exercise domains-window-edge
```
