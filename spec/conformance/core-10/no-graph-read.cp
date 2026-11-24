// expect: E0203
// rule: 10.2.5
// input: "3 2\n0 1\n1 2\n"
domain V = 0..read<1..=10>()
let m = read<0..=10>()
let g = Graph.read(m) // <- here
print(g.len)
