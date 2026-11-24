// expect: R0101
// rule: 07.4.17
// `nth(k)` on a slice counts from the slice's start, so position 5 of a two-element slice traps on both engines.
let xs = [1, 2, 3, 4, 5]
print(xs[1..3].nth(5)) // <- here
