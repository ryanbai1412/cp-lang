// cmt21r2ztr: find a_1..a_n with median(a_i, a_j, a_k) = x for every query. Each (vertex, x)
// that some query names is a pin with two variables, "a_v < x" and "a_v > x". A median of x
// lets at most one of the three be below and at most one above; along one vertex's pins in
// increasing x, "above the last" and "below the next" are exactly one of the two.
enum Side: Below, Above
let (n, m) = read<(1..=100_000, 1..=100_000)>()
domain V = 1..=n
let qs = read<[m](i: V, j: V, k: V, x: 1..=1_000_000_000)>()
var marks: [V]SortedSet<Int> = [V] v => SortedSet()
for q in qs:
    for v in [q.i, q.j, q.k]:
        marks[v].add(q.x)
var pins: List<(v: V, x: Int)> = []
var pin_of: HashMap<(V, Int), Int> = HashMap()
for v in V:
    for x in marks[v].each():
        pin_of[(v, x)] = pins.len
        pins.push((v, x))
domain P = 0..pins.len
fn on(p: Int, s: Side) = Lit((P(p), s), true)
var sat = TwoSat<(P, Side)>()
for q in qs:
    let ps = [q.i, q.j, q.k].map(v => pin_of[(v, q.x)])
    for s in [Below, Above]:
        sat.at_most_one(ps.map(p => on(p, s)).collect())
for p in 0..pins.len:
    sat.add_or(on(p, Below).flip, on(p, Above).flip)
    if p + 1 < pins.len and pins[p + 1].v == pins[p].v:
        sat.add_xor(on(p, Above), on(p + 1, Below))
if let value = sat.solve():
    print("YES")
    var a: [V]Int = 1
    for (p, pin) in pins.items:
        if value[(P(p), Above)]:
            a[pin.v] max= pin.x + 1
        elif not value[(P(p), Below)]:
            a[pin.v] max= pin.x
    print(a)
else:
    print("NO")
