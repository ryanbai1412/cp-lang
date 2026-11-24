// COCI 2021/22 C4 P4 Parkovi: put parks on exactly k vertices of a weighted tree to minimise
// the largest distance from any vertex to its nearest park. Print that distance and the parks.
let n: 1..=2e5 = read()
let k: 1..=n = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[1..n](u: V, v: V, w: 1..=1e9)>()), root: V.first!)

// Greedy for radius r, bottom up: open a park at a vertex as soon as the uncovered vertex
// furthest below it would be out of reach from its parent.
fn parks(r: Int) -> List<V>:
    var open: List<V> = []
    var far: [V]?Int = 0      // furthest uncovered vertex at or below v
    var near: [V]?Int = None  // nearest park at or below v
    for v in t.order.rev:
        if let f = far[v], let d = near[v], f + d <= r: far[v] = None
        if let p = t.parent(v):
            if let f = far[v], f + t.w(v) > r:
                open.push(v)
                near[v] = 0
                far[v] = None
            if let f = far[v]: far[p] max= f + t.w(v)
            if let d = near[v]: near[p] min= d + t.w(v)
        elif far[v] != None:
            open.push(v)
    open

let r = (0..=2e14).first_true(r => parks(r).len <= k)!
var chosen = parks(r)
let park = V.mask(chosen)
print(r)
print(chosen + V.filter(v => not park[v]).take(k - chosen.len).collect())
