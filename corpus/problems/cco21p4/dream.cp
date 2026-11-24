// CCO 2021 P4 Travelling Merchant: the least starting money per city to travel forever, or -1.
// Peel dead cities backwards; among the roads left, the one needing the most money is safe to
// take forever, so it settles its start at r. Then peel again.
let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain Road = 0..m
let roads = read<[Road](u: V, v: V, r: 0..=1e9, p: 0..=1e9)>()
let g = Graph.directed(roads)
let into = g.transpose()
var left = [V] u => g.deg(u)
var gone: [Road]Bool = false
var ans: [V]?Int = None
var dead = V.filter(u => left[u] == 0).collect()

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
