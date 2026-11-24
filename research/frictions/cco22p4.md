# cco22p4 — Bi-ing Lottery Treekets (CCO 2022 P4)

Port: `corpus/problems/cco22p4/port.cp`. The old port hand-rolled modular power, factorial
tables, `-1` child sentinels, a `-1` memo table and a recursive size/sum pass. The rewrite reads
children as `?Node` (`Node.at`), counts with `Comb<Mod<1e9+7>>`, keeps `spare` as an additive
memoized fn (`1 - here[u] + spare of the children`) and memoizes `fill` over `(Node, Arrive)`.

## Frictions

- **A memo keyed by a domain value needs an `Int` wrapper.** Arrival counts past `spare(u)` are
  zero, and they can exceed `n`, so `ways(u, k: Int)` guards and converts to `Arrive(k)` before the
  `@memo(array)` `fill`; one fn would do if the memo took the guard.

## Dream

```cp
@memo(array)
fn ways(u: Node, k: Arrive) -> Z:
    if k > spare(u): return Z(0)
    match (left[u], right[u]):
        (Some(l), Some(r)) => ...
        (Some(c), None) | (None, Some(c)) => comb.arrange(a + h, h) * ways(c, a + h - Int(a == spare(u)))
        (None, None) => Z(1)
```

## Measurements

Max case (N = K = 4000, deep tree, balls mostly near the root): port 0.04 s / 81 MB, accepted
C++ 0.06 s / 129 MB, same output.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let n: 1..=5 = read()
domain Arrive = 0..=n
@memo(array)
fn ways(k: Arrive) -> Int = k.ord
print(ways(Arrive(n + 1)))
```

```text
error[R0102]: 4 is not in `Arrive`, which is 0..=3
  --> probe.cp:5:12
5 | print(ways(Arrive(n + 1)))
  |            ^^^^^^^^^^^^^
  = note: locals: n = 3
  = help: the value is outside the domain: check it with `if x in 0..=3:` first, or give the variable the domain's range
```

**Status:** open: forming the key out of the domain traps, so the guard has to come before `Arrive(k)`.
