// expect: ok, output "-1 -1 0 6 -3 4"
// rule: 07.9.6
// A range's own `min()`, `max()` and `count()` come from its ends: an empty range has no
// `min`/`max`, and the inclusive and exclusive ends give the same answers a walk would.
let e = 5..5
print(e.min() ?? -1, e.max() ?? -1, e.count(), (2..7).max()!, (-3..=0).min()!, (-3..=0).count())
