# coci13c1p3 frictions

## Dream

```cp
total += pairs_equal(corner(..r, ..c), corner(r.., c..)) + pairs_equal(corner(..r, c..), corner(r.., ..c))
```

## Counting equal pairs across two lists

`matches` (a count map of one list, summed over the other) is a small local helper; std has
`counts` but no join of two multisets. The old port no longer builds, so the port was compared
against the accepted C++ only.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
print(pairs_equal([1, 2, 2], [2, 3]))
```

```text
error[E0201]: no `pairs_equal` in scope, and not in the stdlib (yet)
  --> probe.cp:1:7
1 | print(pairs_equal([1, 2, 2], [2, 3]))
  |       ^^^^^^^^^^^
  = help: declare it first: `let pairs_equal = todo`
```

**Status:** open.
