// expect: ok, output "1 1 2\n1 -1\n2 3 3 3 2\n1 2 4\n1 1 -1 3 1 3\n-1 3 -1 -1 -1\n0 2 4"
// An `[V]T` receiver of a `self: [D]T` fn keeps `D = V`: each result is an `[V]T`.
domain V = 0..3
let av: [V]Int = [V] v => [3, 1, 2][v.ord]
let le = av.lis_ends()
print(le[V(0)], le[V(1)], le[V(2)])
let ns = av.next_smaller()
print(if let j = ns[V(0)]: j.ord else: -1, if let j = ns[V(1)]: j.ord else: -1)
domain W = 0..5
let ys = [4, 1, 3, 2, 5]
let aw: [W]Int = [W] w => ys[w.ord]
let mw = aw.ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 6)
print(W.all.map(w => if let s = mw[w]: s.len else: -1))
let zs = [3, 1, 4, 1, 5]
let az: [W]Int = [W] w => zs[w.ord]
print(az.lis().map(w => w.ord))
let cz = az.cartesian_tree()
print(cz.root!.ord, W.all.map(w => if let p = cz.parent[w]: p.ord else: -1))
let ps = az.next_same()
print(W.all.map(w => if let p = ps[w]: p.ord else: -1))
print(az.record_highs().map(w => w.ord).collect())
