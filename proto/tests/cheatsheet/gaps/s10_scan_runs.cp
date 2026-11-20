let n: 1..=10 = read()
domain I = 0..n
var a: [I]Int = read()
print(scan(a, +), scan(a, max), runs(a).len)
