// expect: ok, output "1\n3\n0"
// rule: 10.1.1
// input: "3\n1\n1\n2\n1 2\n0\n"
@!multitest
let n: 0..=10 = read()
domain V = 0..n
let a = read<[V]Int>()
print(a.sum())
