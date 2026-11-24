// expect: ok, output "5 9 7"
// rule: 10.2.2
// input: "3\n5\n7 9\n"
// A triangle input is read as `[Pair<V>]T`: pairs follow the domain's walk order.
domain V = 0..read<2..=100>()
let d = read<[Pair<V>]Int>()
print(d[Pair(V(0), V(1))], d[Pair(V(2), V(1))], d[Pair(V(0), V(2))])
