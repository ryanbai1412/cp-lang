// Source: IOI 2000 P4 (Walls), https://dmoj.ca/problem/ioi00p4
// Problem: a plane graph of N <= 250 towns and walls; each of the M <= 200 regions is given by
//   the towns round its border (the outer region last). L <= 30 club members live in given
//   towns and may start in any region touching their town. Print the least total of walls
//   crossed to meet in one region, and such a region.
// Solution: two regions are adjacent when their borders share a wall (two consecutive
//   towns), found by remembering the first region naming each wall. Floyd-Warshall on the
//   regions, then a region's total sums, per member, the distance to the nearest region at
//   the member's town. O(M^3).
// original: corpus/submissions/4682453.cpp
// Bounds: m: 2..=200; n: 3..=250; k: 1..=30; homes: 1..=n; border: 1..=n
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

var best = (FAR, Region.first!)
for r in Region:
    let total = sum([dist[r, h] for h in touching[t]].min()! for t in homes)
    best min= (total, r)
print(best.0)
print(best.1)
