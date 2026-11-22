let n: 3..=100 = read()
let a: [0..n]1..=100 = read()
let odds = a.count(x => x % 2 == 1)
print(odds)
