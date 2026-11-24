// expect: R0101 (checked)
// rule: 07.2.7
// input: "5\n"
let a = [1, 2, 3]
let n = read<Int>()
print(a[1..n].len) // <- here
