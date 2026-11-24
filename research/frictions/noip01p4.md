# Friction Report: noip01p4

## 1. A fn as a matrix, and a closure that keeps its weights unwrapped
- **Where:** `([Airport, Airport] (u, v) => Some(price(u, v))).closure()!` and `trip[...]!`.
- **What felt wrong:** a complete graph must be wrapped in `Some` per entry and unwrapped per
  lookup; `[Airport, Airport] price` does not parse (`error[E0103]: expected ) to close the
  parenthesis, found a name`).
- **Goal it hurts:** express.
- **Ideal code:** `let trip = ([Airport, Airport] price).closure()`.
- **General fix:** a named fn is an array builder argument (`[D] f` as `[D] x => f(x)`), and
  `closure` over a total `[V, V]W` returns `[V, V]W`.
- **Severity:** low.

## 2. `closure` over `?F64` is 20x the C++ Floyd-Warshall
- **Where:** `([Airport, Airport] (u, v) => Some(price(u, v))).closure()!` on 400 airports.
- **What felt wrong:** the std `floyd_warshall` folds `?W` through `Monoid.of(over.add)` per entry; the C++ inner
  loop is a plain `min(d[i][j], d[i][k] + d[k][j])` over doubles and vectorizes.
- **Goal it hurts:** fast (R10).
- **General fix:** a total-matrix fast path (no `?W` when every entry is present) with the semiring ops inlined;
  repro sent to Core.
- **Severity:** medium.

## Measurements
Stress: 30/30 agree with AC 5086731 on gen.py inputs (size 20); `gen.py --max` is n = 100. The frozen v0.2 port no longer checks (`unreachable`), so the AC is the oracle. Max input (n = 100): port 0.58 s, C++ 0.03 s, same output.

## Dream
The ungated ideal (does not check today; the gaps above are the difference):

```cp
let n: 1..=100 = read()
let air: 1..=1000 = read()
domain City = 1..=n
let (from, to): (City, City) = read()
let city = read<[City](a: Pt, b: Pt, c: Pt, rail: 1..=1000)>()

domain Corner = 0..4
type Airport = (City, Corner)
let corner = [Airport] ((c, k)) => do:
    let (a, b, c3, _) = city[c]
    let (p, q, r) = [(a, b, c3), (b, c3, a), (c3, a, b)].find(((p, q, r)) => (q - p).dot(r - p) == 0)!
    [a, b, c3, q + r - p][k.ord]

fn price(u: Airport, v: Airport) -> F64 =
    F64(if u.0 == v.0: city[u.0].rail else: air) * corner[u].dist(corner[v])
let trip = ([Airport, Airport] price).closure()
print(min(trip[(from, i), (to, j)] for i in Corner for j in Corner), precision: 2)
```
