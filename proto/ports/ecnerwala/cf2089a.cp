@!multitest
let n: Int = read()
let sieve = Sieve.new(n)
var p = n / 2 + 1
while not sieve.is_prime(p): p += 1

var res: List<Int> = [p]
for k in p + 1..=n:
    res.push(2 * p - k)
    res.push(k)
for rem in (1..=2 * p - n - 1).rev: res.push(rem)
print(res)
