# coci06c1p6 frictions

## Dream

```cp
let turned = g.rotated(2)
let h = Hash2D<Row, Col>.pair(g, turned)
```

## Shared bases

Two `Hash2D`s compare only with the same bases, so the port draws them once and passes `bases:`;
a grid turn (`rot180`) is a local `flip` on each domain. The old port no longer builds (`span`
takes cuts), so it was compared only against the accepted C++.
