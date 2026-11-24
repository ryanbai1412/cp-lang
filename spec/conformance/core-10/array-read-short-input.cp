// expect: R0501 (checked)
// rule: 10.2.3
// input: "3\n1 2\n"
domain V = 0..read<1..=10>()
let a = read<[V]Int>() // <- here
print(a.sum())
