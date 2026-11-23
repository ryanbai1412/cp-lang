// COI 2006 Policija: a connected road map. Each query closes one road or
// one town and asks whether the police can still drive from town a to b.
let (n, m): (2..=1e5, 1..=5e5) = read()
domain Town = 1..=n
domain Road = 0..m
let map = Graph<Town, Road>.undirected(read())

// In a depth-first tree every other road joins a town to one of its
// ancestors. low[v]: the earliest town such a road reaches from v's subtree.
let tree = Rooted(map, Town.first)
var low = [Town] v => tree.pos(v)
for v in tree.at.rev:
    for (w, _) in map.out(v):
        if tree.parent(v) != w: low[v] min= tree.pos(w)
    if let p = tree.parent(v): low[p] min= low[v]

// With town c closed, the part of c's subtree that x is cut off in, if any.
fn stranded(c: Town, x: Town) -> ?Town:
    if not tree.is_anc(c, x): return None
    let child = tree.step(c, x, 1)?
    if low[child] < tree.pos(c): None else: child

fn answer(ok: Bool): print(if ok: "yes" else: "no")

let q: 1..=3e5 = read()
for _ in 0..q:
    let (kind, a, b): (1..=2, Town, Town) = read()
    if kind == 1:
        let (u, v): (Town, Town) = read()
        let (top, bottom) = if tree.is_anc(u, v): (u, v) else: (v, u)
        // closing a road matters only if it is a bridge with a and b on opposite sides
        answer(low[bottom] <= tree.pos(top) or tree.is_anc(bottom, a) == tree.is_anc(bottom, b))
    else:
        let c: Town = read()
        answer(stranded(c, a) == stranded(c, b))
