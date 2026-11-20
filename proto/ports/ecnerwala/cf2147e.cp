@!multitest
let (n, q): (Int, Int) = read()
var a: [0..n]Int = read()
var costs: List<Int> = []
var pops: List<Int> = []
var cur_cost = 0
var v = 0
for x in a: v |= x
costs.push(0)
pops.push(popcount(v))
for b in 0..31:
    if ((v >> b) & 1) != 0: continue
    for x in (0..=b).rev:
        if ((v >> x) & 1) != 0: continue
        var cheapest = (1 << x) + 1
        var idx = 0
        for i in 0..n:
            let cnd = (1 << x) - (a[i] & ((1 << x) - 1))
            if cnd < cheapest:
                cheapest = cnd
                idx = i
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
