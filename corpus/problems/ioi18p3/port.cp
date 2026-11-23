// IOI '18 P3 - Werewolf (30pt): n cities joined by m roads. A trip goes from
// s to e; it starts in human form, which may only visit cities >= l, and
// ends in wolf form, which may only visit cities <= r, changing once in a
// city that is allowed in both. For each of q trips print 1 if it can be
// made, else 0.
// Add the cities from the largest down, hanging the tops of the components
// each one joins beneath it: in that tree the cities a human can reach from
// s are the subtree of the highest ancestor of s that is still >= l. The
// mirror tree (smallest city first) does the same for the wolf walking back
// from e. A trip works when the two subtrees share a city, i.e. when some
// city falls in a rectangle of (human preorder, wolf preorder) positions:
// one sweep over the human order with a Fenwick tree over the wolf order.
// original: corpus/submissions/4727179.cpp (the grader interface; check.py
// wraps it in the stdin/stdout main of the DMOJ version)

let (n, m, q): (2..=2e5, 1..=4e5, 1..=2e5) = read()
domain V = 0..n
domain E = 0..m
domain Trip = 0..q
let roads: [E](u: V, v: V) = read()
let trips: [Trip](s: V, e: V, l: V, r: V) = read()
let g = Graph<V, E>.undirected(roads)

// The tree made by adding cities in `order`, linking only to cities added before.
// WISH TR12: `Dsu<V>.unite_under(a, b)` (b becomes the top) or a ready
// `kruskal_tree(order, adj)`; std.dsu picks the top by size, so the find is
// rewritten here.
fn reach_tree(order: List<V>, earlier: (V, V) -> Bool):
    var top: [V]V = [V] v => v
    var boss: [V]?V = None
    fn find(v: V) -> V:
        var x = v
        while top[x] != x:
            top[x] = top[top[x]]
            x = top[x]
        x
    for v in order:
        for a in g.out(v):
            if not earlier(a.to, v): continue
            let c = find(a.to)
            if c != v:
                top[c] = v
                boss[c] = v
    Rooted.from_parents(boss)

let down: List<V> = [v for v in V.rev]
let up: List<V> = [v for v in V]
let human = reach_tree(down, (a, v) => a > v)
let wolf = reach_tree(up, (a, v) => a < v)

// cities inside the wolf range among the first k of the human order, for
// both ends of every trip's human range
domain Cut = 0..=n
var asks: [Cut]List<(trip: Trip, sign: Int)> = [Cut] _ => []
var wolf_range: [Trip]Range<wolf.Pre> = [Trip] _ => wolf.sub(V.first)
for i in Trip:
    let trip = trips[i]
    let h = human.sub(human.lift_while(trip.s, c => c >= trip.l)!)
    wolf_range[i] = wolf.sub(wolf.lift_while(trip.e, c => c <= trip.r)!)
    asks[Cut(h.start.ord)].push((trip: i, sign: -1))
    asks[Cut(h.end.ord)].push((trip: i, sign: 1))

let horder = [v for v in human.at]
var shared: [Trip]Int = 0
var seen = Fenwick<wolf.Pre, Int>.new()
for k in Cut:
    for a in asks[k]: shared[a.trip] += a.sign * seen.fold(wolf_range[a.trip])
    if k.ord < n: seen.add(wolf.pos(horder[k.ord]), 1)
for i in Trip: print(if shared[i] > 0: 1 else: 0)
