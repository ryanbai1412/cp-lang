# coci20c6p3 frictions

## Dream

The committed `dream.cp` groups words by their sorted letters and folds the knapsack.

## Counter has no walk over its counts

`counts(words.map(w => w.sorted()))` builds a `Counter`, but a `Counter` has no `values` and
cannot be walked, so the classes are counted in a `Map<Str, Int>`.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
let words: [3]Str = read()
for size in words.tally(by: w => w.sorted()).values: print(size)
```

```text
2
1
```

**Status:** not a gap: `Counter` has `values`, so the classes are `words.tally(by: w => w.sorted()).values` (the dream's line 10); the port's `Map` tally is owed.
