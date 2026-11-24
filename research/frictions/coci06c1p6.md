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

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
use std.strings.{Hash2D}
domain Row = 1..=2
domain Col = 1..=2
let g: [Row, Col]Char = [Row, Col] (r, c) => '0'
let turned = g.rotated(2)
let h = Hash2D<Row, Col>.pair(g, turned)
print(1)
```

```text
error[E0203]: a `[Row, Col]Char` has no method `rotated`
  --> probe.cp:5:16
5 | let turned = g.rotated(2)
  |                ^^^^^^^
```

**Status:** open (neither `rotated` nor `Hash2D.pair` exists).
