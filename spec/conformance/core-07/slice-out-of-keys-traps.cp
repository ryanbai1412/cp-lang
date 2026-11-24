// expect: R0101 (checked)
// rule: 07.1.4 07.4.5 07.5.2
// input: "0\n"
let s = [10, 11, 12, 13, 14, 15]
let i: Int = read()
print(s[2..5][i]) // <- here
