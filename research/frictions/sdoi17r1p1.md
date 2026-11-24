# sdoi17r1p1 — Number Table

## Quotient blocks are a hand `while` with `r = min(n / (n / l), ...)`
- **Where:** `port.cp`, the per-query loop.
- **What felt wrong:** "T ranges over blocks where n / T and m / T are constant" is a standard
  number-theory iteration, written as index arithmetic every time.
- **General fix:** a std `quotient_blocks(n, m)` yielding `(range, n / T, m / T)`.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Severity:** low.

## Mobius exponent picked by two `if`s
- **Where:** `if mu[k / d] == 1: ... if mu[k / d] == -1: ...`.
- **What felt wrong:** `fib[d] ** mu` with a negative exponent would be one expression; `Mod ** -1` is not the inverse today.
- **General fix:** `Mod ** Int` accepting negative exponents (inverse power).
- **Severity:** low.

## Dream

```cp
let asks = read<[read<1..=1000>()](n: 1..=1e6, m: 1..=1e6)>()
let top = max(max(a.n, a.m) for a in asks)!
let mu = Sieve(top).mobius_table()
let fib = fibonacci<Z>(top + 1)
var g = List.filled(top + 1, Z.one)
for d in 1..=top: for k in (d..=top).step(d): g[k] *= fib[d] ** mu[k / d]
let upto = prefix_fold(*, g)
for a in asks:
    print(product((upto[r.end] / upto[r.start]) ** (qn * qm) for (r, qn, qm) in quotient_blocks(a.n, a.m)))
```
