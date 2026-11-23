// CF 2065E Skibidus and Rizz. The best balance is max - min of the prefix (#big - #small).
// Climb to k with the more frequent letter, oscillate just below k, then spend the
// rest of the rarer letter going down; possible exactly when nb - ns <= k <= nb.
@!multitest
let n, m: 0..=2e5 = read()
let k: 1..=n + m = read()
let (big, small) = if n >= m: ("0", "1") else: ("1", "0")
let nb = max(n, m)
let ns = min(n, m)
if k > nb or k < nb - ns:
    print(-1)
    return
print(big.repeat(k) + (small + big).repeat(nb - k) + small.repeat(ns - (nb - k)))
