let n: 2..=100 = read()
domain V = 1..=n
let a: [V]Int = read()
print(count(x < y for (x, y) in a.adjacent))
