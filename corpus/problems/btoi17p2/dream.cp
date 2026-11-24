// BOI 2017 "Toll": roads only join block i to block i + 1 (blocks of k places), so the cheapest
// path a -> b is the min-plus product of the hop matrices between their blocks.
let (k, n, m, o): (Int, Int, Int, Int) = read()
domain Slot = 0..k
domain Gap = 0..max(ceil_div(n, k) - 1, 1)
let roads: [m](a: Int, b: Int, t: Int) = read()

var hop: [Gap, Slot, Slot]?Int = None
for r in roads:
    let (g, i, j) = (Gap(r.a / k), Slot(r.a % k), Slot(r.b % k))
    hop[g, i, j] = min(hop[g, i, j] ?? r.t, r.t)
let path = SegTree(Mat<?Int>.product(k, over: Semiring(min, +)), [Gap] g => Mat([Slot, Slot] (i, j) => hop[g, i, j], over: Semiring(min, +)))

for (a, b) in read<[o](a: Int, b: Int)>():
    let (ga, gb) = (a / k, b / k)
    let cost = if ga < gb: path.fold(Gap(ga)..=Gap(gb - 1)).get(a % k, b % k)
        elif a == b: Some(0)
        else: None
    print(cost ?? -1)
