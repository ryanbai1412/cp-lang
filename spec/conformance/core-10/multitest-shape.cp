// expect: ok
// rule: 10.1.1 10.2.1
let base = 10
@!multitest
let n: 1..=100 = read()
let a = read<[n]Int>()
print(a.sum() + base)
