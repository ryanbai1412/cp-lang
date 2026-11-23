let n: 1..=100 = read()
let a: [0..n]Int = read()
print((0..n).max(by: i => a[i])!)
