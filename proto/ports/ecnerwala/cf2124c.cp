@!multitest
let n: Int = read()
let b: [0..n]Int = read()
var res = 1
for (x, y) in b.adjacent if y % x != 0:
    res = lcm(res, x / gcd(x, y))
print(res)
