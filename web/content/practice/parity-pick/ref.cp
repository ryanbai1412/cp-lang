@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let total = a.fold(+)
print(yes(a.any(x => x % 2 == total % 2)))
