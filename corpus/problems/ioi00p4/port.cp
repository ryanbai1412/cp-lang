// IOI 2000 Walls: walls between towns cut the country into m regions; each
// region is listed by the towns on its border, in order round it. Club
// members live in k of the towns and may start in any region touching their
// town. Pick the meeting region that minimises the total number of walls
// crossed, and print that total and the region.
//
// Two regions are neighbours when they list the same wall, so the map of
// regions is found by remembering, per wall, the first region that named it.
// original: corpus/submissions/4682453.cpp
let (m, n, k): (2..=200, 3..=250, 1..=30) = read()
domain Region = 1..=m
domain Town = 1..=n
domain Member = 0..k
let homes: [Member]Town = read()

const FAR = 1e9
var dist = [Region, Region] (a, b) => if a == b: 0 else: FAR
var touching: [Town]List<Region> = []
var first_side: Map<(Town, Town), Region> = Map()
for r in Region:
    let len: Int = read()
    domain Side = 0..len
    let border: [Side]Town = read()
    for s in Side:
        let (a, b) = (border[s], border[Side.wrap(s.ord + 1)])
        touching[a].push(r)
        let wall = (min(a, b), max(a, b))
        if let other = first_side.get(wall):
            dist[r, other] = 1
            dist[other, r] = 1
        else:
            first_side[wall] = r

for via in Region:
    for a in Region:
        for b in Region:
            dist[a, b] min= dist[a, via] + dist[via, b]

var best = (FAR, Region.first)
for r in Region:
    let total = sum([dist[r, h] for h in touching[t]].min()! for t in homes)
    best min= (total, r)
print(best.0)
print(best.1)
