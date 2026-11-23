const N = 100_000
let sv = Sieve(N)
let pre = Prefix([1..=N] k => if sv.is_prime(k): 1 else: 0)
@!multitest
let (l, r): (1..=1e5, 1..=1e5) = read()
print(pre.fold(l..=r))
