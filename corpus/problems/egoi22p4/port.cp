// Source: EGOI 2022 P4 (Tourists), https://oj.uz/problem/view/EGOI22_tourists
// limits: 4000 ms, 256 MB
// Problem: a tree of n <= 2e5 cities; m <= 2e5 tourists start in cities a_i with opinion 0.
//   q <= 2e5 queries: "t f g c" moves tourists f..g to city c, each losing its path length;
//   "e c d" adds d <= 1e9 to everyone now in city c; "q v" prints tourist v's opinion.
// Solution: keep each city's running event total. Tourists with consecutive numbers that
//   arrived in one city at one moment form a run (std `IntervalMap`) remembering the city and
//   its total at arrival; a tourist's opinion is what it settled on leaving earlier cities plus
//   the current city's total since arrival. A move repaints f..g, and each run it overwrites
//   settles its gain there less the distance to c into a range-add `DualFenwick`. Repainting
//   is amortized O(1) runs per move: O((m + q) log m + n log n).
// Bounds: n, m, q: 1..=2e5; start: 1..=n; roads.u, roads.v: 1..=n; f, g, v: 1..=m; c: 1..=n;
//   d: 1..=1e9
// original: corpus/submissions/5199667.cpp

use std.fenwick.{DualFenwick}

let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain City = 1..=n
domain Tourist = 1..=m
let start: [Tourist]City = read()
let roads: [n - 1](u: City, v: City) = read()
let tree = Rooted(Graph.undirected(roads), root: City.first!)

var events: [City]Int = 0  // everything city c's events gave so far
var stay = IntervalMap.from([Tourist] t => (city: start[t], since: 0))
var settled = DualFenwick<Tourist, Int>()

for _ in 0..q:
    let op: Char = read()
    if op == 't':
        let (f, g, c): (Tourist, Tourist, City) = read()
        let gain = (r: Range<Tourist>, s: (city: City, since: Int)) =>
            settled.add(r, events[s.city] - s.since - tree.dist(s.city, c))
        stay.assign(f..=g, (city: c, since: events[c]), on_old: gain)
    elif op == 'e':
        let c: City = read()
        let d: 1..=1e9 = read()
        events[c] += d
    else:
        let v: Tourist = read()
        let s = stay.get(v)
        print(settled.get(v) + events[s.city] - s.since)
