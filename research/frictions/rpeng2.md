# Friction report: rpeng2

There is no dream.cp; the ideal is below. The port is the interval DP on the ring. `span[i, len]` keeps the least and largest value of the chain of len + 1 vertices from i, and `join` combines two spans by the edge's operation, taking the four products for `x`.

## Dream

```cp
for len in Len.drop(1):
    for i in V:
        let parts = [join(ring[j].op, span[i, Len(left)], span[j, Len(len.ord - left - 1)])
                     for left in 0..len.ord let j = i.cyclic(left + 1)]
        span[i, len] = (lo: parts.map(.lo).min()!, hi: parts.map(.hi).max()!)
```

## A cyclic successor is label arithmetic inside a one-element loop
- **Where:** line 28: `for j in [V.nth((i.ord + left + 1) % n)]`.
- **What felt wrong:** two gaps in one line:
  - "the vertex left + 1 steps round the ring from i" is spelled with ordinals, `% n` and `nth`;
  - the result is bound with a loop over a one-element list, because a comprehension has no `let`.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `let j = i.cyclic(left + 1)` clause.
- **General fix:**
  - a std `cyclic(k)` on domain values, wrapping modulo the domain. Owner: stdlib.
  - a `let` clause in comprehensions. Owner: core/spec.
- **Severity:** low.

## Measurements

- **Tokens:** port 387, accepted C++ 792 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 2 MB, accepted C++ 0.01 s / 3 MB, with the same output.
- **Emitted C++:** 19323 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
