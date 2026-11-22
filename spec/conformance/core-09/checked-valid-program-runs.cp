// expect: ok, output "6"
// rule: 09.2.1
// input: "3\n1 2 3\n"
let n = read<Int>()
let xs = read<[n]Int>()
var s = 0
for x in xs: s += x
print(s)
