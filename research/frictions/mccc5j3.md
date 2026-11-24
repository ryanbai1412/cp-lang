# Friction report: mccc5j3

## Dream

```cp
let at = a.inverse_map()
print(I.count(j => at.get(m - b[j]) is Some(i) and abs(i - j) >= k))
```

## Distinct values: one position each

The v0.2 port kept a `Map<Int, List<Int>>` of positions and binary-searched it. Since the
a_i are distinct, a value has exactly one position, and the rewrite keeps a `Map<Int, I>` and
tests `|i - j| >= K` directly, which is O(n). Building that map is a loop: an `inverse_map()`
on a table of distinct values would make it one expression.

## `if let` inside a predicate

`good` is `if let i = at.get(..): .. else: false`. An `is Some(i) and ...` pattern test would
make it a boolean expression.

## Position distance

`abs(Int(i) - Int(j))` converts both positions to `Int`. A `dist(i, j)` on a domain would skip
the conversions.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(N = 2e5, M = 1e18) runs in 0.03 s. Corpus tests pass.
