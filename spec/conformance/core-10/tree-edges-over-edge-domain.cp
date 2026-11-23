// expect: ok, output "2"
// rule: 10.2.3
// input: "3\n1 2\n2 3\n"
// A tree reads its n-1 edges as records over an edge domain (rebuild round 6 H-TREE-EDGES).
domain V = 1..=read<Int>()
domain E = 0..V.count - 1
let es = read<[E](u: V, v: V)>()
print(es.len)
