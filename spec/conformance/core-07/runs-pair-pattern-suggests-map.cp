// expect: E0302
// rule: 07.7.10
// has: "each run is a slice"
// has: ".map(r => (r.first!, r.len))"
// Each run is a slice, not a (value, count) pair; the fix maps it to one.
let xs = [1, 1, 2]
for (x, n) in runs(xs): print(x, n) // <- here
