let n: 1..=100000 = read()
let a: [0..n]Int = read()
fn cost(x: Int) -> Int = sum(abs(v - x) for v in a)
var lo = -1_000_000_000
var hi = 1_000_000_000
while hi - lo > 2:
    let m1 = lo + (hi - lo) / 3
    let m2 = hi - (hi - lo) / 3
    if cost(m1) < cost(m2):
        hi = m2
    else:
        lo = m1
print((lo..=hi).map(x => cost(x)).min()!)
