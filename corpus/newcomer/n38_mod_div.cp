let (n, k): (1..=200000, 1..=1000000) = read()
let a: [0..n]1..=1000000000 = read()
var s: Mod<998244353> = 0
for x in a:
    s += Mod<998244353>(x) / Mod<998244353>(k)  // expect: loop_invariant_mod_div
print(s)
