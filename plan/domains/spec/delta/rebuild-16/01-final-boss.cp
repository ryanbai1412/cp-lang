// Codeforces 1985F Final Boss: fewest turns to deal h damage with n attacks on cooldowns.
// In: t; per test `h n`, then a_1..a_n, then c_1..c_n. Out: one integer per test.
@!multitest
let h, n: 1..=2e5 = read()
domain A = 0..n
let a = read<[A]1..=2e5>()
let c = read<[A]1..=2e5>()

// Attack i fires on turns 1, 1 + c, 1 + 2c, ...; capping each term at h keeps the sum small.
fn dealt(turns: Int) -> Int = sum(min(a[i] * ((turns - 1) / c[i] + 1), h) for i in A)

print((1..=4e10).first_true(t => dealt(t) >= h)!)
