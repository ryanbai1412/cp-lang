// expect: R0101 (checked)
// rule: 07.2.7
// input: "7\n"
domain V = 0..5
let a: [V]Int = [10, 20, 30, 40, 50]
let k = read<Int>()
print(a[V(1)..k].len) // <- here
