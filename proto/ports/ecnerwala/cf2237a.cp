// Codeforces 2237 A: sum of prefix minima of the array.
use std.seq.{scan}

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(sum(scan(a, min)))
