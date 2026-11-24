// ioi00p4 Walls: regions sharing a wall are one crossing apart; meet where the members'
// crossings, each from the nearest region at their town, sum least.
let (m, n, k): (2..=200, 3..=250, 1..=30) = read()
domain Region = 1..=m
domain Town = 1..=n
let homes: [k]Town = read()

var hops: [Region, Region]?Int = None
var touching: [Town]List<Region> = []
var first_side: Map<(Town, Town), Region> = Map()
for r in Region:
    let len: Int = read()
    let border: [len]Town = read()
    for (a, b) in border.cyclic_pairs():
        touching[a].push(r)
        if let other = first_side.get((min(a, b), max(a, b))):
            hops[r, other] = 1
            hops[other, r] = 1
        else:
            first_side[(min(a, b), max(a, b))] = r
assert floyd_warshall(mut hops)

let total: [Region]Int = [Region] r => sum(touching[t].map(h => hops[r, h]!).min()! for t in homes)
let meet = total.argmin()!
print(total[meet])
print(meet)
