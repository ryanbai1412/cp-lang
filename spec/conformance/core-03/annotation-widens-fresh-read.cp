// expect: ok, output "3003"
// rule: 03.1.8
// input: "3 1 1 1"
let n = read<1..=5>()
var a: [n]Int = read<[n]0..=5>()
for i in a.keys: a[i] += 1000
print(a.sum())
