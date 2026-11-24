// Structs that hold a lent value point to it (R05.4.4): a user struct
// built from a parameter, and the std graph views.
struct H: xs: List<Int>

fn wrap(xs: List<Int>) -> lends H = H(xs)
fn total(h: H) -> Int = h.xs.sum()
fn pass(h: H) -> lends H = h

let n: Int = read()
domain V = 0..n
domain E = 0..n
let a = [i for i in 0..n]
let h = wrap(a)
var c = a.clone()
c.push(1)
let w = if n > 2: wrap(a) else: wrap(c)
print(total(h), total(pass(w)), yes(h == wrap(a)), c.len)
print(h)
let es: [E](u: V, v: V) = [(u: V(e.ord), v: V((e.ord + 1) % n)) for e in E]
let g = Graph.undirected(es)
var s = 0
for _ in 0..3:
    s += wrap(a).xs.len
    s += g.only(keep: v => v.ord != 1).deg(V(0))
    s += only(g, keep: v => v.ord != 2).deg(V(1))
    let (domain C, comp) = components(complement(g))
    s += C.count
    if let d = dijkstra(g.weighted(a => 5), from: [(V(0), 0)])[V(2)]: s += d
print(s)
