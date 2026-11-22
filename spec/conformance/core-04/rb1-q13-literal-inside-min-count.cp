// expect: ok, output "0"
// input: "5\n"
// rule: 04.4.25
domain V = 1..=read<1..=200000>()
var a: [V]Int = 0
print(a[1])
