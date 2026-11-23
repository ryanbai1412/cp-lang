// expect: ok, output "3000000000 3000000000"
// rule: 03.1.7
// input: "1000000000 1000000000 1000000000\n"
domain V = 0..3
let a = read<[V]1..=1e9>()
print(Fenwick(a).fold(V(0)..=V(2)), a.sum())
