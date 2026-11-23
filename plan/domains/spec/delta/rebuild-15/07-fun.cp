// CF 1996D Fun
// in: t; per test n x. out: the count of ordered positive (a, b, c) with ab + ac + bc <= n and a + b + c <= x.
@!multitest
let (n, x): (1..=1e6, 1..=1e6) = read()
var count = 0
for a in 1..=n:
    if 2 * a + 1 > n: break
    for b in 1..=n:
        if a * b + a + b > n: break
        let c = min((n - a * b) / (a + b), x - a - b)
        if c > 0: count += c
print(count)
