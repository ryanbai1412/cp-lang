// Primes by the sieve, smallest prime factors, and factorisations.
let n: 2..=1000000 = read()
var spf: [0..=n]Int = 0
var primes: List<Int> = []
for i in 2..=n:
    if spf[i] == 0:
        spf[i] = i
        primes.push(i)
    for p in primes:
        if p > spf[i] or i * p > n:
            break
        spf[i * p] = p
print(primes.len)
print(primes)
let q: Int = read()
for k in 0..q:
    let x0: 2..=1000000 = read()
    var x: Int = x0
    var parts: List<Int> = []
    while x > 1:
        parts.push(spf[x])
        x /= spf[x]
    print(parts)
var is_prime: [0..=n]Bool = true
is_prime[0] = false
is_prime[1] = false
for i in 2..=n:
    if is_prime[i]:
        for j in i * i..=n by i:
            is_prime[j] = false
print(count(i for i in 0..=n if is_prime[i]))
