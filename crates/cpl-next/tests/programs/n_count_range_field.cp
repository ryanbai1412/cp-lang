// expect: ok, output "1 2 3\n6\n2 2"
// rule: R03.1.4
// A field whose cells range up to its domain's count holds a body's `0..=V.count` array, and
// each struct argument gives the field that domain's count.
struct Depths<V: Domain>:
    dep: [V]0..=V.count
    sz: [V]1..=V.count
fn make<V: Domain>(root: V) -> Depths<V>:
    var d: [V]0..=V.count = 0
    for v in V: d[v] = Int(v) + 1
    let s: [V]1..=V.count = V.count
    Depths(dep: d, sz: s)
domain V = 0..3
let x = make(V(0))
print(x.dep[V(0)], x.dep[V(1)], x.dep[V(2)])
print(x.dep[V(0)] + x.dep[V(1)] + x.dep[V(2)])
domain W = 0..2
var y = make(W(0))
y.dep[W(0)] = 2
print(y.dep[W(0)], y.sz[W(1)])
