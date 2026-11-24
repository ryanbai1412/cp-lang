# Friction report: noip21p1

There is no dream.cp; the ideal is below. The port sieves survivors up to `LIMIT = 10000100` in a `Bitset<X>`. Each still-alive number with a digit 7 kills its multiples, and a query is `contains` plus `next`.

## Dream

```cp
domain X = 1..=10000100
var alive = Bitset<X>.full()
for x in X if x in alive and 7 in digits(x.label):
    for y in X.multiples(of: x): alive.remove(y)
for x in read<[t]1..=1e7>(): print(if X(x) in alive: alive.next(X(x + 1))! else: -1)
```

## Multiples of a domain value go through labels
- **Where:** line 22: `for m in (x.label..=LIMIT).step(x.label): alive.remove(X(m))`.
- **What felt wrong:** "every multiple of x in X" leaves the domain for an `Int` range, repeats `LIMIT`, and converts back with `X(m)` on every step.
- **Goal it hurts:** express.
- **Ideal code:** `for y in X.multiples(of: x)`.
- **General fix:** a std `multiples(of:)` on integer domains, which any sieve-shaped port would use. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 133, accepted C++ 793 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.33 s / 4 MB, accepted C++ 0.23 s / 17 MB, with the same output.
- **Emitted C++:** 15731 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
