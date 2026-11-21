type M = Mod<998_244_353>

let t: 1..=10_000 = read()
let ns: [0..t](1..=20_000_000) = read()

var lim = 1
for n in ns: lim max= n + 1

let comb = Comb<998_244_353>(lim)

// Multiplicative sieve for odd_div_sum(x) = sum of divisors d | x with x / d odd
// (= 2^{v_2(x)} * sigma_1(x / 2^{v_2(x)})).
var primes = List<Int>()
var pfac: [0..=lim]Int = 0
var odd_div_sum: [0..=lim]Int = 0
odd_div_sum[1] = 1

fn extend(i: Int, p: Int):
    var s = odd_div_sum[i]
    var j = i
    while j <= lim / p:
        j *= p
        pfac[j] = p
        s *= p
        if p > 2: s += odd_div_sum[i]
        odd_div_sum[j] = s

for i in 2..=lim:
    if pfac[i] == 0:
        primes.push(i)
        extend(1, i)
    for p in primes:
        if p == pfac[i] or i > lim / p: break
        extend(i, p)

for n in ns:
    var cnt: [0..=n]M = 0
    for i in 1..=n + 1:
        var v = M(odd_div_sum[i])
        if i % 2 == 1:
            v -= 1
        elif n % 2 == 1 and i % 4 == 2:
            v -= 2
        if (i + n) % 2 == 1:
            if i - 1 <= n: cnt[(n + i - 1) / 2] += 2 * v
            if i + 1 <= n: cnt[(n + i + 1) / 2] += 2 * v
        else:
            if i <= n: cnt[(n + i) / 2] -= 4 * v

    var ans: M = 0
    for i in 0..=n:
        ans += comb.inv_fact(i) * comb.inv_fact(n - i) * cnt[i]
    ans = ans * comb.fact(n) + M(2).pow(n)
    print(ans)
