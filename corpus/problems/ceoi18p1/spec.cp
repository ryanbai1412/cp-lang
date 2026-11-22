// ceoi18p1 Cloud Computing: buy computers (cores, cost) and accept orders (cores, pay). An order
// may only use cores at least as fast as it asks, so walk everything by clock rate, fastest
// first and computers before orders at a tie: a 0/1 knapsack over the cores on hand.
let n = read<1..=2_000>()
let computers = read<[n](c: 1..=50, f: Int, v: Int)>()
let m = read<1..=2_000>()
let orders = read<[m](c: 1..=50, f: Int, v: Int)>()
let events = chain(
    computers.map(x => (f: x.f, cores: x.c, gain: -x.v)),
    orders.map(x => (f: x.f, cores: -x.c, gain: x.v)),
).sorted(by: e => (desc(e.f), desc(e.cores)))
domain K = 0..=50 * n
var best: [K]?Int = none
best[K.first!] = 0
for e in events:
    let before = best.clone()
    for k in K:
        if let to = K.at(Int(k) + e.cores):
            best[to] max= before[k].map(x => x + e.gain)
print(best.max()!)
