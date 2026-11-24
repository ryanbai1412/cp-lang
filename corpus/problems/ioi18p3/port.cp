// Source: IOI 2018 P3 (Werewolf), https://dmoj.ca/problem/ioi18p3
// Problem: a connected graph, n <= 2e5 cities and m <= 4e5 roads. For each of q <= 2e5 trips
//   (s, e, l, r) print 1 if one can walk from s to e visiting only cities >= l in human form,
//   then turn into a wolf once and visit only cities <= r, else 0.
// Solution: add cities from the largest down, hanging the components each joins beneath it
//   (a Kruskal reconstruction tree): the cities a human reaches from s are the subtree of the
//   highest ancestor of s still >= l. The mirror tree (smallest first) does the same for the
//   wolf from e. A trip works when the two subtrees share a city, a point in a rectangle of
//   (human preorder, wolf preorder): one offline sweep with a Fenwick tree, O((n + m + q) log n).
// Bounds: n: 2..=2e5; m: 1..=4e5; q: 1..=2e5; roads.u, roads.v, trips.s, trips.e, trips.l,
//   trips.r: 0..n
// original: corpus/submissions/4727179.cpp (grader interface; orig.cpp adds a stdin main)

let (n, m, q): (2..=2e5, 1..=4e5, 1..=2e5) = read()
domain V = 0..n
domain E = 0..m
domain Trip = 0..q
let roads: [E](u: V, v: V) = read()
let trips: [Trip](s: V, e: V, l: V, r: V) = read()

// a human may use a road while both ends are >= l, a wolf while both are <= r
let (domain H, human) = kruskal_tree(roads, w: e => -Int(min(roads[e].u, roads[e].v)))
let (domain W, wolf) = kruskal_tree(roads, w: e => Int(max(roads[e].u, roads[e].v)))
let (ht, wt) = (human.tree, wolf.tree)
var city: [H]?V = None
for v in V: city[human.leaf[v]] = v

// cities inside the wolf subtree among the first k nodes of the human preorder, at both ends
// of every trip's human subtree
domain Step = 0..=H.count  // the cuts of the human preorder
var asks: [Step]List<(trip: Trip, sign: Int)> = []
var wolf_range: [Trip]Range<wt.Pre> = [Trip] _ => wt.sub(wolf.leaf[V.first!])
for i in Trip:
    let t = trips[i]
    let h = ht.sub(human.reach(t.s, -Int(t.l)))
    wolf_range[i] = wt.sub(wolf.reach(t.e, Int(t.r)))
    asks[Step(Int(h.l))].push((trip: i, sign: -1))
    asks[Step(Int(h.r))].push((trip: i, sign: 1))

var shared: [Trip]Int = 0
var seen = Fenwick<wt.Pre, Int>.new()
let horder = ht.at.collect()
for k in Step:
    for a in asks[k]: shared[a.trip] += a.sign * seen.fold(wolf_range[a.trip])
    if Int(k) < H.count:
        if let v = city[horder[Int(k)]]: seen.add(wt.pos(wolf.leaf[v]), 1)
for i in Trip: print(Int(shared[i] > 0))
