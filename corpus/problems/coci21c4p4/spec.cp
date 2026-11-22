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
        var near: ?Int = None
        for c in t.children(v):
            let sub = visit(c)
            let w = t.up_arc(c)!.w
            near min= sub.near.map(d => d + w)
            if let f = sub.far:
                if f + w > limit:
                    open.push(c)
                    near min= w
                else:
                    far max= f + w
        if near.map(d => far + d <= limit) ?? false:
            return (None, near)
        (far, near)
    if visit(t.root).far != None:
        open.push(t.root)
    open

let best = (0..=200_000_000_000_000).first_true(x => centres(x).len <= k)!
let chosen = centres(best)
let picked = V.mask(chosen)
let spare = V.filter(v => not picked[v]).take(k - chosen.len)
print(best)
print(chain(chosen, spare))
