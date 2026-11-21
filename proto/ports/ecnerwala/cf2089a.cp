@!multitest
let n: Int = read()
var primes: List<Int> = []
var pfac: [0..=n]Int = 0
for x in 2..=n:
    if pfac[x] == 0:
        primes.push(x)
        pfac[x] = x
    for pr in primes:
        if pr * x > n:
            break
        pfac[x * pr] = pr
        if pr == pfac[x]:
            break
var p = n / 2 + 1
while pfac[p] != p:
    p += 1
var res: List<Int> = [p]
for k in p + 1..=n:
    res.push(2 * p - k)
    res.push(k)
var rem = 2 * p - n - 1
while rem >= 1:
    res.push(rem)
    rem -= 1
print(res)
