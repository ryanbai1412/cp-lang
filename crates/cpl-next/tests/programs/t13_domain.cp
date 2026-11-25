// expect: R0102
// input: "5\n"
// A domain with a run-time size, used by a generic fn that never declared it.
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for v in D: s += a[v]
    s
let n: Int = read()
domain V = 0..n
var a: [V]Int = 0
for v in V: a[v] = Int(v) * 2
a[V(1)] = 7
print(total(a))
print(V.count)
a[V(n)] = 1  // <- here
