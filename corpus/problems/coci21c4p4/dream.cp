// coci21c4p4 Autobus... Centres: open k centres so every vertex is within the smallest possible
// distance of one. Binary search the distance; for a given limit, open centres greedily from the
// leaves up, only where a waiting vertex would otherwise fall out of range.
let (n, k) = read<(1..=200_000, 1..=200_000)>()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V, w: 1..=1_000_000_000)>())
let t = Rooted(g, root: V.first!)

fn centres(limit: Int) -> List<V>:
    var open: List<V> = []
    // far: the farthest vertex below v still waiting for a centre; near: the nearest centre.
    fn visit(v: V) -> (far: ?Int, near: ?Int):
        var far = 0
        var near: ?Int = none
        for c in t.children(v):
            let sub = visit(c)
            near min= sub.near.map(d => d + t.w(c))
            if let f = sub.far:
                if f + t.w(c) > limit:
                    open.push(c)
                    near min= t.w(c)
                else:
                    far max= f + t.w(c)
        if far + (near ?? (limit + 1)) <= limit:
            return (none, near)
        (far, near)
    if visit(t.root).far != none:
        open.push(t.root)
    open

let best = (0..=200_000_000_000_000).first_true(x => centres(x).len <= k)!
var chosen = centres(best)
var used: [V]Bool = false
for v in chosen:
    used[v] = true
for v in V:
    if chosen.len < k and not used[v]:
        chosen.push(v)
print(best)
print(chosen)
