// Codeforces 2229 A: ceiling of half the array span (max - min).
use std.math.{ceil_div}

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(ceil_div(max(a)! - min(a)!, 2))
