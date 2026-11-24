# cco22p2 friction

## A market's two stops are a hop across domains

- **Where:** `port.cp`, `room[Stop(i.ord)]` and `room[Stop(i.ord + 1)]` for market `i`.
- **What felt wrong:** market `i` sits between stops `i` and `i + 1`: the markets are the inner
  cuts of `Stop`. There is no domain for the inner cuts of another domain, so `Market` is its
  own domain and each read crosses over by ordinal.
- **Goal it hurts:** express, types carry the proofs (`Stop(i.ord + 1)` is checked at run time).
- **Ideal code:** `domain Market = Stop.inner_cuts`, then `room[i.l]`, `room[i.r]`.
- **Severity:** low.

## The spare-umbrella stack is hand-kept

- **Where:** `port.cp`, `spare` with `total` and each entry's `below`.
- **What felt wrong:** the structure is a stack of (market, count) runs that supports "take k
  from the top" and "keep only the bottom `cap` units". Both loops are written out; std has no
  run-length stack with a capped prefix.
- **Goal it hurts:** express (the two loops are most of the program).
- **Ideal code:** `spare.take_top(need)` yielding `(market, count)` runs and `spare.keep(right)`.
- **Severity:** low.

## Dream

```cp
domain Market = Stop.inner_cuts
...
        var left = min(people[i], room[i.l] - came)
        let right = min(room[i.r], people[i] - left)
...
            for (j, take) in spare.take_top(need):
                plan[j].buy += take
                plan[j].right -= take
                shifted.add(j.r..i.l, take)
...
        spare.keep(right)
```

## Measurements

30 generated cases agree with the old port and 30 with the accepted C++. The `--max` input
(n = 1e6, everyone dry) matches the accepted C++: port 0.12 s, accepted C++ 0.26 s.
