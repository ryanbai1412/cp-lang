// CCO 2021 "Bread First Search": add the fewest edges so that 1, 2, .., n
// is a BFS order from 1. Layers are consecutive blocks. A block starting at i
// pays one edge per vertex in it whose earliest neighbour is not before i, and
// the next block may start at cut c only if no vertex from c on has an earliest
// neighbour before i (it would have been found sooner).
let (n, m): (2..=2e5, 1..=2e5) = read()
domain V = 1..=n
var earliest: [V]?V = None
for (a, b) in read<[m](V, V)>():
    let (lo, hi) = minmax(a, b)
    earliest[hi] min= Some(lo)
var found_by: [V]List<V> = []
for v in V:
    if let e = earliest[v]: found_by[e].push(v)
var later: [V.cuts]?V = None
for v in V.rev: later[v.l] = min(later[v.r], earliest[v])

// best[c]: fewest new edges for the vertices from cut c on, when a block starts at c.
let add_min = Action<?Int, Int>(0, (f, g) => f + g, (f, x, _) => x.map(y => y + f), over: Monoid<?Int>(None, min))
var best = SegTree(add_min.over, [V.cuts] c => if c == V.cuts.last!: Some(0) else: None, act: add_min)
let end = V.cuts.last!
for i in V.drop(1).rev:
    for v in found_by[i]: best.apply(v.r..=end, 1)
    if earliest[i] == None: best.apply(i.r..=end, 1)
    let next = V.cuts((Int(i.r)..=n).first_true(k => later[V.cuts(k)].is_none_or(e => e >= i))!)
    best.set(i.l, best.fold(max(next, i.r)..=end))
print(best.get(V(2).l)!)
