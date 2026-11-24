# coci20c6p3 frictions

## Dream

The committed `dream.cp` groups words by their sorted letters and folds the knapsack.

## Counter has no walk over its counts

`counts(words.map(w => w.sorted()))` builds a `Counter`, but a `Counter` has no `values` and
cannot be walked, so the classes are counted in a `Map<Str, Int>`.
