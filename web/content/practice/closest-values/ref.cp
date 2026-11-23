let n: 2..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted()
print([b[i] - b[i - 1] for i in 1..n].min()!)
