let n: 1..=2000 = read()
let a: [0..n]Int = read()
print(a.pairs().count(p => p.0 + p.1 == 0))
