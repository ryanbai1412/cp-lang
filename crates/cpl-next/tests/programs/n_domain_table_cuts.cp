// expect: ok, output "0 0\n1 0\n2 5\n3 0\n30"
// rule: R02.8.3 R04.2.17
// A table over a type that implements `Domain` walks its positions, as one over a domain does.
domain V = 1..=3
var p: [V.cuts]Int = 0
p[V.cuts(2)] = 5
for c in V.cuts:
    print(Int(c), p[c])
let q = [V.cuts] c => Int(c) * 10
print(q[V.cuts(3)])
