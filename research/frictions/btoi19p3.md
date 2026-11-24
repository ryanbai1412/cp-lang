# btoi19p3 frictions

## Dream

```cp
let shop_under = tree.fold_up(v => if v in shops: Some(far[v]) else: None, min)
let fate = offline(asks, by: a => asks[a].from, along: tree.preorder, path: Depth, ...)
```

## Subtree minimum and the depth-indexed path by hand

Not a gap for the subtree minimum (corrected in review): std `Rooted.up(op, finish:)`
(std/tree.cp) is the fold over subtrees, and `min.opt()` the `?Int` monoid. Run (b244fe76c) on a
five-node tree with `let under = t.up(min.opt(), finish: (s, v) => min.opt().op(s, shop[v]))`
under `cpl run` prints `5 5 7 5 9`, the nearest shop below each node; `t.lower(V(1), V(3))`
prints `3`. The port's reverse preorder sweep with `min=` is bookkeeping the language already
removes. The "path to the exit as a table over depth" trick for offline
path queries in preorder is also written out by hand with a `SegTree<Depth, ?Int>`.
