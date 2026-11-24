// expect: E0203
// has: "a HashMap has no order; use Map"
// `first_ge` needs keys in order; a `HashMap` has none, so the error names `Map`, not the arguments.
var h = HashMap<Int, Int>()
h[1] = 2
print(h.first_ge(2)) // <- here
