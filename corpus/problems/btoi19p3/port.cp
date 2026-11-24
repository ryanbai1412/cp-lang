// Source: Baltic OI 2019 Day 1 P3 (Alpine Valley), https://dmoj.ca/problem/btoi19p3
// limits: 3000 ms, 256 MB
// Problem: a weighted tree on n <= 1e5 villages, s of them with shops, and the exit E. Each of
//   q <= 1e5 queries closes road I and names village R: print "escaped" if R still reaches E,
//   else the distance to the nearest reachable shop, or "oo" if none.
// Solution: root at E; closing a road cuts off the subtree of its lower end, so R escapes unless
//   it lies inside. Otherwise the best walk climbs to a turn t between R and the cut and drops to
//   the nearest shop under t: far[R] + min(shop_under[t] - 2 far[t]). Queries are answered
//   offline in preorder, where a table over depth holds the path to E, so the minimum is one
//   SegTree range fold: O((n + q) log n).
// original: corpus/submissions/4456164.cpp
// Bounds: n: 2..=1e5; s, exit, shops, asks.from: 1..=n; q: 1..=1e5; asks.closed: 1..=n - 1
let n: 2..=1e5 = read()
let s: 1..=n = read()
let q: 1..=1e5 = read()
domain V = 1..=n
domain Road = 1..n
domain Ask = 1..=q
let exit: V = read()
type Link = (u: V, v: V, w: 1..=1e9)

fn valley() -> [Road]Link:
    [Road] r => (V(rand(1, r.label)), V(r.label + 1), rand(1, 1e9))

let roads: [Road]Link = read(gen: valley)
let shops: [s]V = read()  // distinct in the statement; nothing here says so
let asks: [Ask](closed: Road, from: V) = read()

// Hang the valley from the exit. Closing a road cuts off the subtree under
// its lower end: villages outside it escape, villages inside can only use
// shops inside. Such a walk climbs from the village to some turn t, no higher
// than the cut, then drops to the nearest shop under t:
//   far[from] + (far[nearest shop under t] - 2 far[t])
// with far[] measured from the exit.
let tree = Rooted(Graph.undirected(roads), exit)
let far: [V]Int = [V] v => tree.dist(exit, v)
var shop_under: [V]?Int = None  // far[] of the nearest shop in the subtree
for v in shops: shop_under[v] = far[v]
for v in tree.at.rev:
    if let p = tree.parent(v):
        if let d = shop_under[v]: shop_under[p] min= d

var asked_at: [V]List<Ask> = []
for a in Ask: asked_at[asks[a].from].push(a)

// Walking the villages in preorder, the last village seen at each depth is an
// ancestor of the current one. So a table over depth holds the whole path to
// the exit, and "best turn between the cut and here" is one range minimum.
domain Depth = 0..n
var turn: [Depth]?Int = None
var best_turn = SegTree<Depth, ?Int>.of(min, d => turn[d])
var escaped: [Ask]Bool = false
var nearest: [Ask]?Int = None
for v in tree.at:
    let here = Depth(tree.depth(v))
    turn[here] = None
    if let d = shop_under[v]: turn[here] = d - 2 * far[v]
    best_turn.refresh(here)
    for a in asked_at[v]:
        let (u, w, _) = roads[asks[a].closed]
        let cut = if tree.parent(u) == w: u else: w
        if tree.is_anc(cut, v):
            if let t = best_turn.fold(Depth(tree.depth(cut))..=here): nearest[a] = far[v] + t
        else:
            escaped[a] = true

for a in Ask:
    if escaped[a]: print("escaped")
    elif let d = nearest[a]: print(d)
    else: print("oo")
