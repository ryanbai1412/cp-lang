// cp: 0.1 0f8d77b+dirty
// COI 2006 Policija: a connected road map. Each query closes one road or
// one town and asks whether the police can still drive from town a to b.
let n: 2..=1e5 = read()
let m: n - 1..=5e5 = read()
domain Town = 1..=n
domain Road = 0..m

fn connected() -> [Road](u: Town, v: Town):     // generator: a random tree, then extra roads
    [Road] r => (if r.idx < n - 1: (Town(rand(1, r.idx + 1)), Town(r.idx + 2)) else: (Town(rand(1, n)), Town(rand(1, n))))

let roads: [Road](u: Town, v: Town) = read(gen: connected)
let map = Graph.undirected(roads)

// In a depth-first tree every other road joins a town to one of its
// ancestors. low[v]: the earliest town such a road reaches from v's subtree.
let tree = Rooted(map, Town.first)
var low: [Town]Int = [Town] v => tree.pos(v)
for v in tree.preorder.rev:
    for w in map[v]:
        if tree.parent(v) != w: low[v] min= tree.pos(w)
    if let p = tree.parent(v): low[p] min= low[v]

// With town c closed, the part of c's subtree that x is cut off in, if any.
fn stranded(c: Town, x: Town) -> ?Town:
    if not tree.is_anc(c, x): return none
    let child = tree.step(c, x, 1)?
    if low[child] < tree.pos(c): none else: child

let q: 1..=3e5 = read()
for _ in 0..q:
    let kind: 1..=2 = read()
    let a: Town = read()
    let b: Town = read()
    if kind == 1:
        let u: Town = read()
        let v: Town = read()
        let (top, bottom) = if tree.is_anc(u, v): (u, v) else: (v, u)
        // closing a road matters only if it is a bridge with a and b on opposite sides
        let ok = low[bottom] <= tree.pos(top) or tree.is_anc(bottom, a) == tree.is_anc(bottom, b)
        print(if ok: "yes" else: "no")
    else:
        let c: Town = read() where c != a and c != b
        print(if stranded(c, a) == stranded(c, b): "yes" else: "no")
