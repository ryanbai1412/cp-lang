// Source: NOI 2019 D2P1 (Jump), https://dmoj.ca/problem/noi19p4
// Problem: n <= 7e4 cities at distinct points of a w x h grid (w, h <= n) and m <= 1.5e5
//   portals; portal i at city p_i takes time t_i <= 1e4 to any city in [L_i, R_i] x [D_i, U_i].
//   Print the least time from city 1 to each of cities 2..n.
// Solution: Dijkstra over cities and portals. A portal settles at distance d, so every city in
//   its rectangle not yet reached is offered exactly d and never needs looking at again: the
//   cities live in a `PointSet`, and a portal drains its rectangle from it.
//   O((n + m) log^2 n + (n + m) log(n + m)).
// original: corpus/submissions/3800719.cpp
use std.geometry.{PointSet}

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

var unreached = PointSet.of([Pt(Int(at[c].x), at[c].y) for c in City])
var frontier = Settle<Node, Int>()
var dist: [City]Int = 0
frontier.offer(At(City.first!), 0)
while let (u, d) = frontier.pop():
    match u:
        At(c):
            dist[c] = d
            for p in out[c]: frontier.offer(Via(p), d + portal[p].t)
        Via(p):
            let r = portal[p]
            for i in unreached.drain(Int(r.xl)..Int(r.xr) + 1, r.yl..r.yr + 1):
                frontier.offer(At(City.nth(i)), d)
for c in City.drop(1): print(dist[c])
