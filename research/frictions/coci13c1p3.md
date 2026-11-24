# coci13c1p3 frictions

## Dream

```cp
total += pairs_equal(corner(..r, ..c), corner(r.., c..)) + pairs_equal(corner(..r, c..), corner(r.., ..c))
```

## Counting equal pairs across two lists

`matches` (a count map of one list, summed over the other) is a small local helper; std has
`counts` but no join of two multisets. The old port no longer builds, so the port was compared
against the accepted C++ only.
