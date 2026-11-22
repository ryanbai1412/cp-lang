let n = read<1..=500>()
domain V = [n]
let w = read<[V, V]0..=1e9>()
let widest = Semiring(max, min)
let d: [V, V]?Int = [V, V] (u, v) => if u != v: Some(w[u, v]) else: None
let c = d.closure(over: widest)!
print(yes((V, V).all((u, v) => u == v or c[u, v] == d[u, v])))
