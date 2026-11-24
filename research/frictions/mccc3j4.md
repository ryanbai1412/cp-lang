# Friction report: mccc3j4

## Dream

```cp
let (n, t): (1..=2000, 1..=2000) = read()
domain Player = 1..=n
let first: [Player]Player = read()
var queue: [Player]Deque<Player> = [Player] p => Deque.of([first[p]])
var last = first
repeat t:
    let hits = [(to: x, from: p) for p in Player if let x = queue[p].pop_front()]
    for h in hits: last[h.from] = h.to; queue[h.to].push(h.from)
print(last)
```

## No filter-map comprehension over an optional

The round's hits are built with an explicit `List` and `push` inside `if let`. A
comprehension clause `if let x = ...` (or `Player.filter_map(p => ...)` that owns a mutable
borrow of `queue`) would make the round one expression, with `last` updated from it.

## `for _ in 0..t` for a plain repeat

`repeat t:` exists (`repeat n: print(1)` runs on a fresh `cpl`); the port owes it in place of
the unused `_` loop variable.

## Verification

30 + 10 random cases agree with the v0.2 port and 30 with the accepted C++
(`corpus/submissions/4223440.cpp`); the max case (N = T = 2000) matches the C++.
