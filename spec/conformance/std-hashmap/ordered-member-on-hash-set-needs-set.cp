// expect: E0203
// has: "a HashSet has no order; use Set"
// `pop_first` takes the smallest element; a `HashSet` keeps none in order, so the error names `Set`.
var h = HashSet<Int>()
h.insert(3)
print(h.pop_first()) // <- here
