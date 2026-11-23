// expect: ok, output "2 4"
// rule: 01.6.1
// input: "3"
let n = read<Int>()
var c: [n + 1]Int = 0
c[n] += 2
print(c.sum(), c.len)
