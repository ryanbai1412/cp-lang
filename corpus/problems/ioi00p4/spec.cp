// ioi00p4 Walls: regions sharing a wall are one crossing apart; meet where the members'
// crossings, each from the nearest region at their town, sum least.
let (m, n, k): (2..=200, 3..=250, 1..=30) = read()
domain Region = 1..=m
domain Town = 1..=n
domain Member = 0..k
let homes: [Member]Town = read()

// The first region naming a wall waits for the second.
var hops: [Region, Region]?Int = None
var touching: [Town]List<Region> = []
var first_side: Map<(Town, Town), Region> = Map()
for r in Region:
    let len: Int = read()
    let border: [len]Town = read()
    for (a, b) in border.cyclic_pairs():
        touching[a].push(r)
        let wall = (min(a, b), max(a, b))
        if let other = first_side.get(wall):
            for (x, y) in [(r, other), (other, r)]: hops[x, y] = 1
        else:
            first_side[wall] = r
assert floyd_warshall(mut hops)

let total: [Region]Int = [Region] r => sum(touching[t].map(h => hops[r, h]!).min()! for t in homes)
let meet = total.argmin()!
print(total[meet])
print(meet)
