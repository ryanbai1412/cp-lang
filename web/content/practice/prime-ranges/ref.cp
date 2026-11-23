use std.math.{Sieve}
const N = 100_000
let sv = Sieve(N)
var pre: [0..=N]Int = 0
for k in 1..=N:
    pre[k] = pre[k - 1] + (if sv.is_prime(k): 1 else: 0)
let q: 1..=2e5 = read()
for _ in 0..q:
    let (l, r): (1..=1e5, 1..=1e5) = read()
    print(pre[r] - pre[l - 1])
