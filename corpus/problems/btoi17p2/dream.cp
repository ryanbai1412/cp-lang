// BOI 2017 "Toll": roads only join layer i to layer i+1 (layers of k nodes);
// cheapest path a -> b is the min-plus product of the hop matrices between.
let (k, n, m, q) = read<(1..=5, 1..=5e4, 0..=1e4, 1..=1e4)>()
domain Node = [n]
domain Layer = [n.ceil_div(k)]
domain Slot = [k]
let place = (v: Node) => (layer: Layer(Int(v) / k), slot: Slot(Int(v) % k))

var hop = [Layer] _ => Mat<Slot, Slot>(over: Semiring(min, +))
for (u, v, w) in read<[m](u: Node, v: Node, w: 1..=1e4)>():
    hop[place(u).layer][place(u).slot, place(v).slot] = Some(w)
let path = SegTree(*, hop)

for (a, b) in read<[q](a: Node, b: Node)>():
    let (pa, pb) = (place(a), place(b))
    print(if pa.layer < pb.layer: path.fold(pa.layer..pb.layer)[pa.slot, pb.slot]
          else: (a == b).then(0), None: -1)
