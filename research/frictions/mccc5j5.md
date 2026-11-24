# Friction report: mccc5j5

## Dream

```cp
let best = SparseTable(max, points)
fn most(m, k) = (k.one_bits.map(d => best.fold(block(m ^ k, d) & Problem)) + leaf).max()!
```

## The trie becomes aligned blocks

The v0.2 port hand-built a max segment tree over 2^bits leaves (a `[Node]Int` with `2o` /
`2o + 1` children) and walked it as a bitwise trie. Each sibling it took is an aligned block
of 2^d indices, so the rewrite folds that block from a std `SparseTable(max, ..)` over the
real problems. No padding leaves, no node domain, O(1) per block.

## Clipping a block to the domain

A block may run past N, so the port clips it with `min(.., n) - 1` and skips blocks that
start at or past N. A range intersection with the domain (`block & Problem`) that may be
empty would remove both conditions.

## `points` bound

The statement's bound on the points is not recorded here. The read takes `0..=2^31 - 1` from
the accepted C++, which stores them as `int` and folds max from 0.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(N = 1e6, Q = 5e5) runs in 0.23 s. Corpus tests pass.
