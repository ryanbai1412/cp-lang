// expect: E0602
// rule: 05.6.2 05.7.1 05.7.6
domain V = 1..=4
fn window(xs: [V]Int, l: V) -> lends [V]Int = xs[l..]
var a = [V] v => Int(v)
let w = window(a, V(3))
a[V(4)] = 0 // <- here
print(w.sum())
