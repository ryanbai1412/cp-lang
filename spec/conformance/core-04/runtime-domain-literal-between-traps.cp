// expect: R0101 at 3 (checked)
// rule: 04.4.25
// input: "2\n"
domain V = 1..=read<1..=5>()
var a: [V]Int = 0
print(a[4])
