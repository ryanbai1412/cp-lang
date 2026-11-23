// 1955G GCD on a grid: largest divisor of gcd(corners) with a monotone path of multiples.
use std.math.{divisors}

@!multitest
let n: 1..=100 = read()
let m: 1..=100 = read()
let g = read<[n, m]1..=1e6>()

fn passable(d: Int) -> Bool:
    var r: [n, m]Bool = false
    for i in 0..n, j in 0..m:
        if g[i, j] % d != 0: continue
        r[i, j] = (i == 0 and j == 0) or (i > 0 and r[i - 1, j]) or (j > 0 and r[i, j - 1])
    return r[n - 1, m - 1]

let ds = divisors(gcd(g[0, 0], g[n - 1, m - 1]))
print(ds.rev.find(passable)!)
