let n: 1..=10 = read()
domain I = 0..n
var a: [I]Int = read()
print(sliding_min(a, 2), sliding_max(a, 2))
