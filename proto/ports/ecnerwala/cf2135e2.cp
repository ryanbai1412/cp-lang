// Codeforces 2135 E2: Beyond the Palindrome (Hard Version).
// Count length-n arrays in {-1, 0, 1} whose adjacent-sum graph has an even
// spanning-tree weight, via inclusion-exclusion on the odd-divisor sum
// odd_div_sum(x) = 2^{v_2(x)} * sigma_1(x / 2^{v_2(x)}).
const P = 998_244_353
type M = Mod<P>

let t: 1..=10_000 = read()
let ns: [0..t](1..=20_000_000) = read()
let lim = max(n + 1 for n in ns)
let comb = Comb<P>(lim)

// Linear multiplicative sieve for odd_div_sum(x).
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

    let total = sum(comb.inv_fact(i) * comb.inv_fact(n - i) * cnt[i] for i in 0..=n)
    let two: M = 2
    print(total * comb.fact(n) + two.pow(n))
