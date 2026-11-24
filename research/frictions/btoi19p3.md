# btoi19p3 frictions

## Dream

```cp
let shop_under = tree.fold_up(v => if v in shops: Some(far[v]) else: None, min)
let fate = offline(asks, by: a => asks[a].from, along: tree.preorder, path: Depth, ...)
```

## Subtree minimum and the depth-indexed path by hand

std's `Rooted` has no fold over subtrees, so the nearest shop below each village is a reverse
preorder sweep with `min=`. The "path to the exit as a table over depth" trick for offline
path queries in preorder is also written out by hand with a `SegTree<Depth, ?Int>`.
