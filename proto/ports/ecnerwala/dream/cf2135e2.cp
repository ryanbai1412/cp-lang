// Count length-n arrays in {-1, 0, 1} whose adjacent-sum graph has an even
// spanning-tree weight, via inclusion-exclusion on the odd-divisor sum
// odd_div_sum(x) = 2^{v_2(x)} * sigma_1(x / 2^{v_2(x)}).
// Precompute Comb and the linear multiplicative sieve once across all tests.
type M = Mod<998_244_353>

let t: 1..=1e4 = read()
let ns: [0..t](1..=2e7) = read()
let lim = ns.max() + 1
let comb = Comb<M>(lim)

// Linear multiplicative sieve for odd_div_sum(x).
let odd_div_sum = Sieve(lim).multiplicative<I32>(
    at_prime_pow: (p, e, prev) => if p == 2: prev * 2 else: prev * p + 1,
)

for n in ns:
    var cnt: [0..=n]M = 0
    for i in 1..=n + 1:
        var v = M(odd_div_sum[i])
        if i % 2 == 1: v -= 1
        elif n % 2 == 1 and i % 4 == 2: v -= 2
        if (i + n) % 2 == 1:
            if i - 1 <= n: cnt[(n + i - 1) / 2] += 2 * v
            if i + 1 <= n: cnt[(n + i + 1) / 2] += 2 * v
        else:
            if i <= n: cnt[(n + i) / 2] -= 4 * v

    let ans = comb.fact(n) * sum(comb.inv_fact(i) * comb.inv_fact(n - i) * cnt[i] for i in 0..=n) + M(2).pow(n)
    print(ans)
