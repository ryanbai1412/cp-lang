// Codeforces 2165 A: merging adjacent cyclic elements costs the sum of adjacent
// maxima around the ring minus the global maximum (the last merge's winner).
use std.seq.{cyclic_pairs}

@!multitest
let n: Int = read()
let a: [0..n]Int = read()
print(sum(max(x, y) for (x, y) in cyclic_pairs(a)) - a.max()!)
