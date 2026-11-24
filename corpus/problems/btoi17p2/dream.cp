// BOI 2017 "Toll": roads only join block i to block i + 1 (blocks of k places), so the cheapest
// path a -> b is the min-plus product of the hop tables between their blocks.
let (k, n, m, o): (Int, Int, Int, Int) = read()
domain Slot = 0..k
domain Gap = 0..max(ceil_div(n, k) - 1, 1)
type Hops = [Slot, Slot]?Int
let roads: [m](a: Int, b: Int, t: Int) = read()

fn chain(x: Hops, y: Hops) -> Hops =
    [Slot, Slot] (i, j) => Slot.filter_map(s => x[i, s].and_then(p => y[s, j].map(q => p + q))).min()

var hop: [Gap]Hops = None
for r in roads:
    let (g, i, j) = (Gap(r.a / k), Slot(r.a % k), Slot(r.b % k))
    hop[g][i, j] = min(hop[g][i, j] ?? r.t, r.t)
let path = SegTree(Monoid([Slot, Slot] (i, j) => if i == j: Some(0) else: None, chain), hop)

for (a, b) in read<[o](a: Int, b: Int)>():
    let (ga, gb) = (a / k, b / k)
    let cost = if ga < gb: path.fold(Gap(ga)..=Gap(gb - 1))[Slot(a % k), Slot(b % k)]
        elif a == b: Some(0)
        else: None
    print(cost ?? -1)
