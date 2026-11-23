let n: 1..=100 = read()
let a: [0..n]Int = read()
print(a.sorted(by: x => x % 7))
