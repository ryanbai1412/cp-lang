@!multitest
let (n, q): (Int, Int) = read()
var a: [0..n]Int = read()

var v = 0
for x in a: v |= x
var cur_cost = 0
var costs = [0]
var pops = [popcount(v)]

for b in 0..31 if not v.bit(b):
    for x in (0..=b).rev if not v.bit(x):
        let mask = (1 << x) - 1
        let idx = min_by(0..n, i => (1 << x) - (a[i] & mask))!
        let cheapest = (1 << x) - (a[idx] & mask)
        a[idx] += cheapest
        cur_cost += cheapest
        v = 0
        for val in a: v |= val
    costs.push(cur_cost)
    pops.push(popcount(v))

for _ in 0..q:
    let b: Int = read()
    let pos = costs.first_ge(b + 1) ?? costs.len
    print(pops[pos - 1])
