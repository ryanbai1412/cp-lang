// expect: ok, output "6"
// rule: 10.2.1
// input: "3\n1 2 3\n"
let n: 1..=10 = read()
let a = read<[n]Int>()
print(a.sum())
