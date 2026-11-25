// expect: R0102
// input: "3\n"
// `V(x)` traps for an integer before the domain's first position (R04.2.7), not only after
// its last.
let n = read<Int>()
domain V = 1..=n
var a: [V]Int = 0
print(a[V(n)])
print(a[V(0)]) // <- here
