// expect: E0606
// rule: 10.3.8
// Walking a `HashMap` has no defined order, so it can't print; the error names `h.keys.sorted()`.
var h: HashMap<Int, Int> = HashMap()
h[5] = 1
print(h) // <- here
