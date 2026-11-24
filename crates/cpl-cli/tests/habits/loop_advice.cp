// expect: E0302
// rule: 07.7.10
// has: "each piece is a slice"
// Each run is a slice, not a (value, count) pair, and neither is any piece a fn yields.
let xs = [1, 1, 2]
for (x, n) in runs(xs): print(x, n) // <- here
