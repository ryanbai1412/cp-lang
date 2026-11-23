const N = 100_000
let sv = Sieve(N)
let pre = Prefix([1..=N] k => if sv.is_prime(k): 1 else: 0)
let q: 1..=2e5 = read()
for _ in 0..q:
    let (l, r): (1..=1e5, 1..=1e5) = read()
    print(pre.fold(l..=r))
