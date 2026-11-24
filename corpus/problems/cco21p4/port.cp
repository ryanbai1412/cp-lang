// Source: CCO 2021 P4 (Travelling Merchant), https://dmoj.ca/problem/cco21p4
// Problem: n, m <= 2e5; one-way road i from a to b needs at least r_i money and then pays
//   p_i >= 0. For every starting city print the least starting money that lets the merchant
//   travel forever, or -1.
// Solution: a city with no usable outgoing road is dead: peel dead cities backwards, offering
//   each road u -> v into a dead v the price max(r, ans[v] - p) for u. Among the remaining
//   roads the one with the largest r is affordable forever from its start once taken (every
//   road left is cheaper and money never drops), so it offers its start r; then the road is
//   gone and the peeling goes on. Each road is dropped once: O((n + m) log m).
// original: corpus/submissions/6419583.cpp
let (n, m): (1..=2e5, 1..=2e5) = read()
domain City = 1..=n
domain Road = 0..m
let roads: [Road](u: City, v: City, r: 0..=1e9, p: 0..=1e9) = read()
let g = Graph.directed(roads)
let into = g.transpose()

var left = [City] u => g.deg(u)
var gone: [Road]Bool = false
var ans: [City]?Int = None
var dead: List<City> = City.filter(u => left[u] == 0).collect()

fn drop(e: Road, price: ?Int):
    if gone[e]: return
    gone[e] = true
    let u = roads[e].u
    ans[u] min= price
    left[u] -= 1
    if left[u] == 0: dead.push(u)

fn peel():
    while let v = dead.pop():
        for a in into.out(v): drop(a.e, ans[v].map(x => max(a.r, x - a.p)))

peel()
for e in Road.sort(by: e => desc(roads[e].r)):
    drop(e, Some(roads[e].r))
    peel()
print(ans, none: -1)
