@!multitest
let n: Int = read()
let b: List<Int> = [read::<Int>() for _ in 0..n]
var res = 1
for i in 0..(n - 1):
    if b[i + 1] % b[i] != 0:
        res = lcm(res, b[i] / gcd(b[i], b[i + 1]))
print(res)
