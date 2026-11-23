// cmt21r2ztr: find a_1..a_n with median(a_i, a_j, a_k) = x for every query. Each (vertex, x)
// that some query names is a pin with two variables, "a_v < x" and "a_v > x". A median of x
// lets at most one of the three be below and at most one above; along one vertex's pins in
// increasing x, "above the last" and "below the next" are exactly one of the two.
enum Side: Below | Above
let (n, m) = read<(1..=100_000, 1..=100_000)>()
domain V = 1..=n
let qs = read<[m](i: V, j: V, k: V, x: 1..=1_000_000_000)>()
let (domain P, pins) = compress(qs.flat_map(q => [q.i, q.j, q.k].map(v => (v:, x: q.x))))
fn on(p: P, s: Side) = Lit((p, s), true)
var sat = TwoSat<(P, Side)>()
for q in qs:
    let ps = [q.i, q.j, q.k].map(v => pins.at((v:, x: q.x)))
    for s in Side:
        sat.at_most_one(ps.map(p => on(p, s)).collect())
for p in P:
    sat.add_or(on(p, Below).flip, on(p, Above).flip)
for (p, q) in P.adjacent if pins.get(p).v == pins.get(q).v:
    sat.add_xor(on(p, Above), on(q, Below))
if let value = sat.solve():
    print("YES")
    var a: [V]Int = 1
    for p in P:
        let pin = pins.get(p)
        if value[(p, Above)]:
            a[pin.v] max= pin.x + 1
        elif not value[(p, Below)]:
            a[pin.v] max= pin.x
    print(a)
else:
    print("NO")
