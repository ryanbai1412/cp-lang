// IOI 2018 Werewolf: a connected graph on cities 0..n. Each query (s, e, l, r)
// asks for a trip from s to e that starts in human form, which may only
// visit cities >= l, changes once to wolf form in some city, and from then
// on may only visit cities <= r. Print 1 or 0 for each query.
//
// Add the cities from n - 1 down to 0, hanging every part a new city touches
// under it. In that tree the cities a human reaches from s are the subtree
// of the highest ancestor of s that is still >= l. The same tree built from
// 0 upward gives the wolf side as a subtree too. A trip exists when the two
// subtrees share a city, which is a count of points in a rectangle of the
// two Euler orders, answered offline with a Fenwick tree.
//
// The original is a grader function; this port reads `n m q`, the m roads,
// then q lines `s e l r`.
// original: corpus/submissions/4727179.cpp

let (n, m, q): (Int, Int, Int) = read()
domain City = 0..n
var roads: [City]List<City> = []
for _ in 0..m:
    let (x, y): (Int, Int) = read()
    roads[City(x)].push(City(y))
    roads[City(y)].push(City(x))

// The tree of merges when the cities appear in `order`: the subtree of c is
// everything c reaches through cities that appeared no later than c.
fn merge_tree(order: List<City>):
    var parts = Dsu<City>.new()
    var newest = [City] c => c  // the latest city of each part, by its representative
    var seen: [City]Bool = false
    var kids: [City]List<(to: City)> = []
    for c in order:
        seen[c] = true
        for d in roads[c]:
            if not seen[d] or parts.same(c, d): continue
            kids[c].push((to: newest[parts.find(d)]))
            parts.unite(c, d)
            newest[parts.find(c)] = c
    Rooted(c => kids[c], order.last!)

let human = merge_tree([c for c in City.rev])
let wolf = merge_tree(City.all)

type Trip = (s: Int, e: Int, l: Int, r: Int)
domain Q = 0..q
var trips: List<Trip> = []
for _ in Q:
    let (s, e, l, r): (Int, Int, Int, Int) = read()
    trips.push((s:, e:, l:, r:))

// Sweep the human Euler order; `asks` are (position, query, sign): the
// number of cities before `position` whose wolf position is in the query's
// wolf range, added or subtracted.
var ans: [Q]Int = 0
var asks: List<(at: Int, q: Q, sign: Int)> = []
var wolf_range: [Q]Range<wolf.Pre> = [Q] t => wolf.sub(wolf.root)
for t in Q:
    let (s, e, l, r) = trips[t.to_idx]
    if s < l or e > r: continue
    let from = human.sub(human.lift_while(City(s), c => c.to_idx >= l)!)
    wolf_range[t] = wolf.sub(wolf.lift_while(City(e), c => c.to_idx <= r)!)
    asks.push((at: from.lo.ord, q: t, sign: -1))
    asks.push((at: from.hi.ord, q: t, sign: 1))

let euler = [v for v in human.at]
var placed = Fenwick<wolf.Pre, Int>.new()
var next = 0
for a in asks.sorted(by: a => a.at):
    while next < a.at:
        placed.add(wolf.pos(euler[next]), 1)
        next += 1
    ans[a.q] += a.sign * placed.sum(wolf_range[a.q])

for t in Q: print(Int(ans[t] > 0))
