// expect: pass
// rule: 07.9.4
// output: 2 2 2 3 4
// Only `count()` is refused on bools: a predicate, a generator of tests, or `count()` on any other element type counts as written.
let b = [true, false, true, false]
print(b.count(x => x), count(x for x in b), b.count(x => not x), [1, 2, 3].count(), b.len)
