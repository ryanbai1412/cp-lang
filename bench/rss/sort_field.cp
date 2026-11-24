domain E = 0..250_000
let es: [E](u: Int, v: Int, w: Int) = [E] e => (u: e.ord % 500, v: e.ord / 500, w: (e.ord * 7919) % 1_000_003)
var order = E.collect()
order.sort(by: e => es[e].w)
print(es[order[0]].w, es[order[E.count - 1]].w)
