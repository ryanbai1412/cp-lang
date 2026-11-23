let n: 1..=100 = read()
domain V = 0..n
let a: [V]Int = read()
let j = V.max(by: i => a[i])!
print(j)
