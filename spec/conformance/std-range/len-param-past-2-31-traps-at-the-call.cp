// expect: R0201
// rule: 03.1.10
// input: "2147483648\n"
let n = read<Int>()
print("ab".repeat(n).len) // <- here
