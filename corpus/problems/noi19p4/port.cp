// Source: NOI 2019 D2P1 (Jump), https://dmoj.ca/problem/noi19p4
// Problem: n <= 7e4 cities at distinct points of a w x h grid (w, h <= n) and m <= 1.5e5
//   portals; portal i at city p_i takes time t_i <= 1e4 to any city in [L_i, R_i] x [D_i, U_i].
//   Print the least time from city 1 to each of cities 2..n.
// Solution: Dijkstra over cities and portals. A portal settles at distance d, so every city in
//   its rectangle not yet reached is offered exactly d and never needs looking at again. An
//   x-segment tree keeps, per node, its cities ordered by y; a rectangle drains the y-range from
//   its O(log w) nodes, so each city leaves each of its O(log w) nodes once.
//   O(n log w log n + m log w log n + (n + m) log(n + m)).
// original: corpus/submissions/3800719.cpp
let n: 1..=70000 = read()
let m: 1..=150000 = read()
let w: 1..=70000 = read()
let h: 1..=70000 = read()
domain City = 1..=n
domain Portal = 0..m
domain X = 1..=w
let at: [City](x: X, y: 1..=h) = read()
let portal: [Portal](from: City, t: 1..=1e4, xl: X, xr: X, yl: 1..=h, yr: 1..=h) = read()
let out = portal.map(.from).buckets()

enum Node: At(c: City) | Via(p: Portal)

// x-segment tree: node k covers a block of X and keeps its live cities by (y, city)
var cell: [1..4 * w]SortedSet<(Int, City)> = [1..4 * w] _ => SortedSet<(Int, City)>.new()
fn place(k: Int, lo: Int, hi: Int, c: City):
    cell[k].add((at[c].y, c))
    if lo < hi:
        let mid = (lo + hi) / 2
        if at[c].x.ord <= mid: place(2 * k, lo, mid, c) else: place(2 * k + 1, mid + 1, hi, c)
for c in City: place(1, 0, w - 1, c)

var frontier = Settle<Node, Int>()
fn harvest(k: Int, lo: Int, hi: Int, p: Portal, d: Int):
    let r = portal[p]
    if hi < r.xl.ord or r.xr.ord < lo: return
    if r.xl.ord <= lo and hi <= r.xr.ord:
        for (_, c) in cell[k].drain(after: Some((r.yl - 1, City.last!)), while: e => e.0 <= r.yr):
            frontier.offer(At(c), d)
        return
    let mid = (lo + hi) / 2
    harvest(2 * k, lo, mid, p, d)
    harvest(2 * k + 1, mid + 1, hi, p, d)

var dist: [City]Int = 0
frontier.offer(At(City.first!), 0)
while let (u, d) = frontier.pop():
    match u:
        At(c):
            dist[c] = d
            for p in out[c]: frontier.offer(Via(p), d + portal[p].t)
        Via(p): harvest(1, 0, w - 1, p, d)
for c in City.drop(1): print(dist[c])
