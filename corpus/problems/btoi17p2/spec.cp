// BOI 2017 "Toll": roads only join layer i to layer i+1 (layers of k nodes);
// cheapest path a -> b is the min-plus product of the hop matrices between.
// No two places share more than one street, so m <= k n.
let (k, n, m, q) = read<(1..=5, 1..=5e4, 0..=2.5e5, 1..=1e4)>()
domain Node = [n]
domain Layer = [n.ceil_div(k)]
domain Slot = [k]
type Hop = [Slot, Slot]?Int
let place = (v: Node) => (layer: Layer(Int(v) / k), slot: Slot(Int(v) % k))

// Min-plus product; `None` is "no road" (`min` skips it, `+` keeps it).
fn then(x: Hop, y: Hop) -> Hop:
    var z: Hop = None
    for i in Slot:
        for s in Slot:
            for j in Slot: z[i, j] min= x[i, s] + y[s, j]
    z
let hops = Monoid(id: [Slot, Slot] (i, j) => if i == j: Some(0) else: None, op: then)

var hop: [Layer]Hop = None
for (u, v, w) in read<[m](u: Node, v: Node, w: 1..=1e4)>():
    hop[place(u).layer][place(u).slot, place(v).slot] = w
let path = SegTree(hops, hop)

for (a, b) in read<[q](a: Node, b: Node)>():
    let (pa, pb) = (place(a), place(b))
    print(if pa.layer < pb.layer: path.fold(pa.layer..pb.layer)[pa.slot, pb.slot] ?? -1 else: -1)
