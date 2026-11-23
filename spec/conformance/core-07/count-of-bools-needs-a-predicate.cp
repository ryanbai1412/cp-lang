// expect: E0305 at 2
// rule: 07.9.4
// has: "count(x => x)"
// `count()` counts every element, so on bools it would count the falses too; the error names `count(x => x)` and `len` (acceptance P0 count-of-bools-counts-all).
let b = [true, false, true, false]
print(b.count())
