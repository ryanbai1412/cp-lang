# Friction report: noi06p1

## Dream

```cp
let reach = Prefix<User, User, Int>(read symmetric [User, User]0..=500)
fn best(u: HeapNode, flags: [Depth]Bool) -> [0..=u.leaves]?Int
```

## Changes

The v0.2 port filled a bottom-up `dp[0..2n, 0..2n]?Int` over states packed as
`(cnt << d) | par`, with hand row prefix sums and shifts to find each leaf's other half. The
rewrite recurses top-down: `best(u, depth, flags)` returns the cost per count of B under `u`,
a router tries both flags and keeps the counts that agree, and leaves charge through a 2-D
std `Prefix` fold over `users(v, depth)`, the leaf range of a heap node. Users are a domain;
counts and flags are no longer packed together.

## Remaining friction

- No reader for a symmetric matrix given as its upper triangle: the port stores `i < j` and
  folds over `flow[min(i, j), max(i, j)]`.
- Heap nodes stay `Int`: a domain for an implicit complete binary tree (children, depth, leaf
  range) would remove the shifts in `users` and the leaf test.
- Ancestor flags are a bit mask read with `.bit(d)`.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass; the
max case runs in 0.15 s.
