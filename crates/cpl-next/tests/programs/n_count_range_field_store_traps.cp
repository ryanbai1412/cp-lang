// expect: R0201
// rule: R03.1.4
// A store into a `0..=V.count` field checks the count of the struct's own domain.
struct Depths<V: Domain>:
    dep: [V]0..=V.count
fn make<V: Domain>(root: V) -> Depths<V>:
    var d: [V]0..=V.count = 0
    Depths(dep: d)
domain V = 0..3
var x = make(V(0))
x.dep[V(1)] = 3
print(x.dep[V(1)])
x.dep[V(1)] = 4  // <- here
print(x.dep[V(1)])
