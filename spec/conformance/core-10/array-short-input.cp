// expect: R0501 (checked)
// rule: 10.2.3
// input: "3\n1 2\n"
domain V = 0..read<Int>()
let a: [V]Int = read() // <- here
print(a.sum())
