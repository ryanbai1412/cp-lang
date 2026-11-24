# Friction report: mmcc14p3

## Dream

```cp
let t = Rooted(Graph<V, E>.undirected(read()), V.first!)
fn outposts(radius) = t.fold_up((far: 0, near: None), merge).count_placed
print((0..n).first_true(r => outposts(r) <= k)!)
```

## Optional minimum

`near` is the nearest outpost below, if any. Taking its minimum with a new distance is
`Some(if let x = near: min(x, d + 1) else: d + 1)`. The v0.2 port used `near ?? n` as an
infinity instead. A `min` that treats `None` as absent (`near min= Some(d + 1)` with
"missing" semantics) would be shorter than both.

## A bottom-up fold with a side counter

`outposts` walks `t.at.rev` with a `var used` beside the two per-vertex tables. A tree
`fold_up` whose state carries the count would drop the side `var`.

## Header bound

The old header said n <= 5000. The typed reads (and the accepted C++'s `maxn = 2e5 + 5`) are
2e5, so the header now says so.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). The max case
(n = 2e5) runs in 0.04 s. Corpus tests pass.
