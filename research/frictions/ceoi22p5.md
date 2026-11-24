# ceoi22p5 frictions

## Dream

```cp
var line = SegTree<Rank, Crowd>(Monoid(Crowd.empty, join))
```

## Empty crowd as an identity

`Crowd` has no value that is a true identity for `join` (its `left` and `right` maxima would
need minus infinity), so `join` tests `count == 0` on both sides first.
