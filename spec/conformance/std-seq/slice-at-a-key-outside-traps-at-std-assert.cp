// expect: R0401
// rule: std.seq.Slice.at
// A key outside a slice's keys stops at std's `Slice.at` assert on both engines.
let xs = [3, 1, 4, 1, 5]
print(xs[1..3].at(4)) // <- here
