// expect: ok, output "6\n0"
// rule: 04.2.18 04.8.8
domain V = 1..=4
let a = [V] v => Int(v) * 2
var pre: [V.cuts]Int = 0
print(a[3])
print(pre[0])
