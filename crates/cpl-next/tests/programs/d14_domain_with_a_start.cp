// expect: ok, output "10 30\n1\n2\n3\n3\n-2\n-1\n0\n7 1 7\n0 2\n60"
// input: "3\n"
// A domain that starts anywhere (R04.2.15): `Int(v)` is the integer written, `v.ord` counts
// from 0 (R04.2.22), and a generic fn handed the domain sees the same start.
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for v in D: s += a[v] + Int(v) - Int(v)
    s
let n = read<Int>()
domain V = 1..=n
var a: [V]Int = 0
for v in V: a[v] = Int(v) * 10
print(a[V(1)], a[V(n)])
for v in V: print(v)
print(V.count)
domain W = -2..1
for w in W: print(Int(w))
var b: [3..=5]Int = 7
b[4] = 1
print(b[3], b[4], b[5])
print(V(1).ord, V(n).ord)
print(total(a))
