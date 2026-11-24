// expect: R0101
// rule: 07.4.17
// `at` on a slice takes a source key inside the slice; key 4 of `xs[1..3]` is outside it and traps on both engines.
let xs = [1, 2, 3, 4, 5]
print(xs[1..3].at(4)) // <- here
