// expect: ok, output "4 4 9 0 7\n1"
// rule: R01.6.1
// A single element value fills every slot of an array of arrays: each row is a filled row, also
// over a library domain, and each row is its own (writing one cell leaves the others).
domain V = 0..3
let t: [Pair<V>][V]Int = 4
var u: [V][V]Int = 0
u[V(1)][V(2)] = 9
var w: [V][V][V]Int = 7
var b: [V][V]Bool = false
print(t[V(0), V(1)][V(2)], t[V(0), V(2)][V(0)], u[V(1)][V(2)], u[V(0)][V(2)], w[V(2)][V(0)][V(1)])
print(if b[V(0)][V(0)]: 0 else: 1)
