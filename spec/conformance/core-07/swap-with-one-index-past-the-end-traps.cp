// expect: R0101
// rule: 07.1.5
// `swap` checks both indices: one in range does not excuse the other, which is a list index out of range like any other.
var xs = [1, 2, 3]
let j = [5][0]
xs.swap(0, j) // <- here
print(xs)
