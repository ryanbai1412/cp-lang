// expect: ok, output "16\n20"
// rule: 10.1.1
// input: "2\n3\n1 2 3\n2\n5 5\n"
let base = 10
@!multitest
let n: 1..=100 = read()
let a = read<[n]Int>()
print(a.sum() + base)
